pub mod action_executor;
pub mod action_generator;
pub mod config_access;
pub mod configuration;
pub mod diff_engine;
pub mod version_tracker;
mod watcher;

use configuration::{Configuration, ConfigurationLoader};
use entities::{Entity, SchemaLoader};
use once_cell::sync::{Lazy, OnceCell};
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

// Global configuration definitions loaded from config files - similar to entity definitions
// Type alias for complex configuration type
pub type ConfigurationList = Vec<Arc<Box<dyn Configuration>>>;
pub type EntityList = Vec<Arc<Box<dyn Entity>>>;

pub static CONFIGURATION_DEFINITIONS: Lazy<RwLock<ConfigurationList>> =
    Lazy::new(|| RwLock::new(Vec::new()));

// Global entity definitions loaded from schemas - stores Arc-wrapped entities for sharing
pub static ENTITY_DEFINITIONS: Lazy<RwLock<EntityList>> = Lazy::new(|| RwLock::new(Vec::new()));

// File state cache for diff detection
pub static FILE_STATE_CACHE: Lazy<RwLock<HashMap<String, Value>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// Database pool for hot-reload actions
static DATABASE_POOL: OnceCell<sqlx::SqlitePool> = OnceCell::new();

/// Initialize the schema-manager system
pub async fn init(db_pool: Option<sqlx::SqlitePool>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing schema-manager system");

    // Load initial configurations
    load_configurations().await?;

    // Load initial entity schemas with file count check
    load_schemas_with_count_check().await?;

    // Start watching for changes
    watcher::start_watching(db_pool).await?;

    info!("Schema-manager system initialized");
    Ok(())
}

/// Load configurations using the trait-based system
pub async fn load_configurations() -> Result<(), Box<dyn std::error::Error>> {
    // Determine environment (default to dev)
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    let config_path = get_config_directory()?;
    info!(
        "Loading configurations from: {:?} (environment: {})",
        config_path, env
    );

    if !config_path.exists() {
        info!("Config directory does not exist, skipping configuration loading");
        return Ok(());
    }

    // Load configurations using ConfigurationLoader
    let mut loaded_configs =
        ConfigurationLoader::load_configurations_from_directory(&config_path).await?;

    // Filter configurations based on environment
    // For system config, load the environment-specific one
    loaded_configs.retain(|config| {
        if config.id() == "system" {
            // For system config, check if it matches the environment
            // The file should be named config.system.{env}.yaml
            true // We'll handle this in the loader
        } else {
            // Other configs are loaded regardless of environment
            true
        }
    });

    let configs: ConfigurationList = loaded_configs.into_iter().map(Arc::new).collect();
    info!("Loaded {} configuration modules", configs.len());

    // Update the global configuration definitions
    let mut config_defs = CONFIGURATION_DEFINITIONS.write().unwrap();
    *config_defs = configs;

    info!("Configuration modules loaded and cached");
    Ok(())
}

/// Get the configuration directory path
fn get_config_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // First try to get from environment variable
    if let Ok(config_path) = std::env::var("CONFIGURATION_PATH") {
        return Ok(PathBuf::from(config_path));
    }

    // Fallback to default behavior
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

/// Get a configuration by ID from the new trait-based system
pub fn get_configuration(id: &str) -> Option<Arc<Box<dyn Configuration>>> {
    let configs = CONFIGURATION_DEFINITIONS.read().unwrap();
    configs.iter().find(|c| c.id() == id).cloned()
}

/// Get all configurations
pub fn get_all_configurations() -> ConfigurationList {
    CONFIGURATION_DEFINITIONS.read().unwrap().clone()
}

/// Get a configuration value using the new system
/// Path format: "configuration_id.key" or "configuration_id.nested.key"
pub fn get_configuration_value(path: &str) -> Option<serde_yaml::Value> {
    let parts: Vec<&str> = path.splitn(2, '.').collect();
    if parts.is_empty() {
        return None;
    }

    let config_id = parts[0];
    let config = get_configuration(config_id)?;

    if parts.len() == 1 {
        // Return all values for this configuration
        let all_values = config.get_all_values();
        Some(serde_yaml::to_value(all_values).ok()?)
    } else {
        // Return specific value
        let key = parts[1];
        config.get_value(key).cloned()
    }
}

/// Load entity schemas with file count check
/// This ensures schemas are loaded even on first run or after migration
pub async fn load_schemas_with_count_check() -> Result<(), Box<dyn std::error::Error>> {
    let schemas_path = get_schemas_path()?;
    info!("Loading schemas from: {:?}", schemas_path);

    if !schemas_path.exists() {
        info!("Schemas directory does not exist, skipping schema loading");
        return Ok(());
    }

    // Count YAML schema files in the directory
    let file_count = count_schema_files(&schemas_path)?;

    // Get current entity count
    let current_entity_count = {
        let entities = ENTITY_DEFINITIONS.read().unwrap();
        entities.len()
    };

    debug!(
        "Schema file count: {}, Current entity count: {}",
        file_count, current_entity_count
    );

    // Load schemas if counts don't match or no entities loaded
    if file_count != current_entity_count || current_entity_count == 0 {
        info!("Entity count mismatch or no entities loaded. Loading schemas...");
        load_schemas().await?;
    } else {
        debug!("Entity count matches file count, skipping reload");
    }

    Ok(())
}

/// Count schema files in the directory
fn count_schema_files(path: &std::path::Path) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(extension) = path.extension() {
            if extension == "yaml" || extension == "yml" {
                if let Some(stem) = path.file_stem() {
                    let filename = stem.to_string_lossy();
                    if filename.ends_with(".schema") {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Load entity schemas from the schemas directory
pub async fn load_schemas() -> Result<(), Box<dyn std::error::Error>> {
    let schemas_path = get_schemas_path()?;
    info!("Loading schemas from: {:?}", schemas_path);

    if !schemas_path.exists() {
        info!("Schemas directory does not exist, skipping schema loading");
        return Ok(());
    }

    // Load entities using SchemaLoader and wrap in Arc for sharing
    let loaded_entities = SchemaLoader::load_entities_from_directory(&schemas_path).await?;
    let entities: EntityList = loaded_entities.into_iter().map(Arc::new).collect();
    info!("Loaded {} entity schemas", entities.len());

    // Also update the file state cache for diff detection
    let mut cache = HashMap::new();
    for entry in std::fs::read_dir(&schemas_path)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(extension) = path.extension() {
            if extension == "yaml" || extension == "yml" {
                if let Some(stem) = path.file_stem() {
                    let filename = stem.to_string_lossy();
                    if filename.ends_with(".schema") {
                        let content = std::fs::read_to_string(&path)?;
                        let schema: Value = serde_yaml::from_str(&content)?;
                        cache.insert(path.to_string_lossy().to_string(), schema);
                    }
                }
            }
        }
    }

    // Update the global entity definitions
    let mut entity_defs = ENTITY_DEFINITIONS.write().unwrap();
    *entity_defs = entities;

    // Update the file state cache
    let mut file_cache = FILE_STATE_CACHE.write().unwrap();
    *file_cache = cache;

    info!("Entity schemas loaded and cached");
    Ok(())
}

/// Get the schemas directory path
fn get_schemas_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // First try to get from environment variable
    if let Ok(schema_path) = std::env::var("ENTITY_SCHEMA_PATH") {
        return Ok(PathBuf::from(schema_path));
    }

    // Fallback to default behavior
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

/// Reload configuration (called by the watcher)
pub(crate) fn reload_config() {
    // Reload trait-based configurations
    tokio::spawn(async {
        match load_configurations().await {
            Ok(_) => info!("Configuration modules reloaded successfully"),
            Err(e) => error!("Failed to reload configuration modules: {}", e),
        }
    });
}

/// Reload schemas (called by the watcher)
pub(crate) async fn reload_schemas() {
    match load_schemas().await {
        Ok(_) => info!("Schemas reloaded successfully"),
        Err(e) => error!("Failed to reload schemas: {}", e),
    }
}

/// Get a clone of the current entity definitions
pub fn get_entity_definitions() -> EntityList {
    ENTITY_DEFINITIONS.read().unwrap().clone()
}

/// Get entity definitions for database initialization
/// This provides the loaded entities to the database module
pub fn get_entities_for_database() -> EntityList {
    let entities = ENTITY_DEFINITIONS.read().unwrap();
    entities.clone()
}

/// Check if entities are loaded
pub fn has_entities() -> bool {
    let entities = ENTITY_DEFINITIONS.read().unwrap();
    !entities.is_empty()
}

/// Set the database pool for hot-reload actions
pub fn set_database_pool(pool: sqlx::SqlitePool) {
    if DATABASE_POOL.set(pool).is_err() {
        error!("Database pool already set");
    }
}

/// Get the database pool
pub fn get_database_pool() -> Option<&'static sqlx::SqlitePool> {
    DATABASE_POOL.get()
}
