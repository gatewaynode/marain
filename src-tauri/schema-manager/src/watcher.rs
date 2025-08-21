use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::task;
use tracing::{debug, info, warn};

/// Start watching the config and schemas directories for changes
pub async fn start_watching(
    _db_pool: Option<sqlx::SqlitePool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_directory()?;
    let schemas_dir = get_schemas_directory()?;

    info!(
        "Starting file watcher for directories: {:?} and {:?}",
        config_dir, schemas_dir
    );

    // Create a channel for file events
    let (tx, rx) = channel();

    // Create a watcher with a debounce period
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    // Start watching both directories
    watcher.watch(&config_dir, RecursiveMode::NonRecursive)?;
    if schemas_dir.exists() {
        watcher.watch(&schemas_dir, RecursiveMode::NonRecursive)?;
    }

    // Spawn a task to handle file events
    task::spawn_blocking(move || {
        info!("File watcher task started");

        // Keep the watcher alive
        let _watcher = watcher;

        // Process events
        for event in rx {
            handle_file_event(event);
        }
    });

    Ok(())
}

/// Handle file system events
fn handle_file_event(event: Event) {
    debug!("File event: {:?}", event);

    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            // Check if the event is for a YAML file
            for path in &event.paths {
                if is_config_file(path) {
                    info!("Configuration file changed: {:?}", path);
                    crate::reload_config();
                } else if is_schema_file(path) {
                    info!("Schema file changed: {:?}", path);
                    handle_schema_change(path);
                }
            }
        }
        EventKind::Remove(_) => {
            for path in &event.paths {
                if is_config_file(path) {
                    warn!("Configuration file removed: {:?}", path);
                } else if is_schema_file(path) {
                    warn!("Schema file removed: {:?}", path);
                    handle_schema_removal(path);
                }
            }
        }
        _ => {
            // Ignore other events
        }
    }
}

/// Handle schema file changes
fn handle_schema_change(path: &Path) {
    use crate::action_generator::{ActionGenerator, FileType};
    use crate::diff_engine::DiffEngine;

    // Load the new schema
    let new_content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            warn!("Failed to read schema file: {}", e);
            return;
        }
    };

    let new_schema: serde_yaml::Value = match serde_yaml::from_str(&new_content) {
        Ok(schema) => schema,
        Err(e) => {
            warn!("Failed to parse schema file: {}", e);
            return;
        }
    };

    // Get the old schema from cache
    let path_str = path.to_string_lossy().to_string();
    let old_schema = {
        let cache = crate::FILE_STATE_CACHE.read().unwrap();
        cache.get(&path_str).cloned()
    };

    // If we have an old schema, generate a diff
    if let Some(old) = old_schema {
        let diff = DiffEngine::compare(&old, &new_schema);

        if diff.has_changes() {
            info!("Schema changes detected in {:?}", path);

            // Generate actions based on the diff
            let file_type = FileType::from_path(path);
            match ActionGenerator::generate_actions(file_type, &diff) {
                Ok(actions) => {
                    info!("Generated {} actions from schema changes", actions.len());

                    // If we have a database pool, execute the actions
                    if let Some(pool) = crate::get_database_pool() {
                        let pool_clone = pool.clone();
                        tokio::spawn(async move {
                            execute_schema_actions(actions, pool_clone).await;
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to generate actions from schema changes: {}", e);
                }
            }
        }
    }

    // Update the cache with the new schema
    {
        let mut cache = crate::FILE_STATE_CACHE.write().unwrap();
        cache.insert(path_str, new_schema);
    }

    // Reload schemas asynchronously
    tokio::spawn(async {
        crate::reload_schemas().await;
    });
}

/// Handle schema file removal
fn handle_schema_removal(path: &Path) {
    let path_str = path.to_string_lossy().to_string();

    // Remove from cache
    {
        let mut cache = crate::FILE_STATE_CACHE.write().unwrap();
        cache.remove(&path_str);
    }

    // Reload schemas asynchronously
    tokio::spawn(async {
        crate::reload_schemas().await;
    });
}

/// Execute actions generated from schema changes
async fn execute_schema_actions(
    actions: Vec<crate::action_generator::Action>,
    pool: sqlx::SqlitePool,
) {
    use crate::action_executor::ActionExecutor;

    let executor = ActionExecutor::new(pool);
    match executor.execute_actions(actions).await {
        Ok(report) => {
            info!(
                "Schema actions executed successfully: {} successful, {} failed",
                report.successful_actions, report.failed_actions
            );
        }
        Err(e) => {
            warn!("Failed to execute schema actions: {}", e);
        }
    }
}

/// Check if a path is a configuration file
fn is_config_file(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        if let Some(name_str) = file_name.to_str() {
            // Check for both old-style (system.*) and new-style (config.*) configuration files
            return (name_str.starts_with("system.") || name_str.starts_with("config."))
                && (name_str.ends_with(".yaml") || name_str.ends_with(".yml"));
        }
    }
    false
}

/// Check if a path is a schema file
fn is_schema_file(path: &Path) -> bool {
    if let Some(parent) = path.parent() {
        if let Some(parent_name) = parent.file_name() {
            if parent_name == "schemas" {
                if let Some(extension) = path.extension() {
                    return extension == "yaml" || extension == "yml";
                }
            }
        }
    }
    false
}

/// Get the configuration directory path
fn get_config_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Get the current working directory
    let cwd = std::env::current_dir()?;

    // Navigate to the project root if we're in src-tauri
    let project_root = if cwd.ends_with("src-tauri") || cwd.ends_with("app") {
        cwd.parent()
            .and_then(|p| {
                if p.ends_with("src-tauri") {
                    p.parent()
                } else {
                    Some(p)
                }
            })
            .ok_or("Failed to get project root")?
            .to_path_buf()
    } else {
        cwd
    };

    Ok(project_root.join("config"))
}

/// Get the schemas directory path
fn get_schemas_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // Get the current working directory
    let cwd = std::env::current_dir()?;

    // Navigate to the project root if we're in src-tauri
    let project_root = if cwd.ends_with("src-tauri") || cwd.ends_with("app") {
        cwd.parent()
            .and_then(|p| {
                if p.ends_with("src-tauri") {
                    p.parent()
                } else {
                    Some(p)
                }
            })
            .ok_or("Failed to get project root")?
            .to_path_buf()
    } else {
        cwd
    };

    Ok(project_root.join("schemas"))
}
