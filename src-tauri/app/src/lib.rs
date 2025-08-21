mod logging;

use schema_manager::{get_configuration, config_access};
use database::{Database, DatabaseConfig, initialize_database};
use std::sync::Arc;
use std::path::PathBuf;
use tauri::State;

/// Environment paths configuration
#[derive(Debug, Clone)]
pub struct EnvPaths {
    pub data_path: PathBuf,
    pub static_path: PathBuf,
    pub entity_schema_path: PathBuf,
    pub configuration_path: PathBuf,
}

impl EnvPaths {
    /// Load paths from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenv::dotenv().ok();
        
        // Get the project root
        let project_root = Self::find_project_root();
        
        // Load paths from environment variables with defaults
        let data_path = std::env::var("DATA_PATH")
            .unwrap_or_else(|_| "./data".to_string());
        let static_path = std::env::var("STATIC_PATH")
            .unwrap_or_else(|_| "./static".to_string());
        let entity_schema_path = std::env::var("ENTITY_SCHEMA_PATH")
            .unwrap_or_else(|_| "./schemas".to_string());
        let configuration_path = std::env::var("CONFIGURATION_PATH")
            .unwrap_or_else(|_| "./config".to_string());
        
        // Convert relative paths to absolute paths based on project root
        let resolve_path = |path: &str| -> PathBuf {
            if path.starts_with("./") {
                project_root.join(&path[2..])
            } else if path.starts_with('/') {
                PathBuf::from(path)
            } else {
                project_root.join(path)
            }
        };
        
        Ok(Self {
            data_path: resolve_path(&data_path),
            static_path: resolve_path(&static_path),
            entity_schema_path: resolve_path(&entity_schema_path),
            configuration_path: resolve_path(&configuration_path),
        })
    }
    
    /// Find the project root directory
    fn find_project_root() -> PathBuf {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        
        if current_dir.ends_with("src-tauri") {
            current_dir.parent().expect("Failed to get parent directory").to_path_buf()
        } else if current_dir.ends_with("app") {
            current_dir.parent()
                .and_then(|p| p.parent())
                .expect("Failed to get project root")
                .to_path_buf()
        } else {
            current_dir
        }
    }
}

/// Application state that holds the database connection and environment paths
pub struct AppState {
    pub db: Arc<Database>,
    pub env_paths: EnvPaths,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    tracing::debug!("Greet command called with name: {}", name);
    
    // Example of accessing the configuration using the new system
    if let Some(app_name) = config_access::get_system_string("app.name") {
        tracing::debug!("Current app name from config: {}", app_name);
    }
    
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_app_config() -> Result<String, String> {
    // Get the system configuration using the new system
    if let Some(config) = get_configuration("system") {
        let values = config.get_all_values();
        serde_json::to_string(&values).map_err(|e| e.to_string())
    } else {
        Err("System configuration not found".to_string())
    }
}

#[tauri::command]
async fn create_snippet(
    state: State<'_, AppState>,
    title: String,
    body: String,
) -> Result<String, String> {
    use database::storage::EntityStorage;
    use serde_json::json;
    use std::collections::HashMap;
    
    tracing::info!("Creating snippet with title: {}", title);
    
    let storage = EntityStorage::new(&state.db, "snippet");
    
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), json!(title));
    fields.insert("body".to_string(), json!(body));
    fields.insert("status".to_string(), json!("draft"));
    
    match storage.create(fields).await {
        Ok(id) => {
            tracing::info!("Created snippet with id: {}", id);
            Ok(id)
        }
        Err(e) => {
            tracing::error!("Failed to create snippet: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_snippet(
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    use database::storage::EntityStorage;
    
    tracing::info!("Getting snippet with id: {}", id);
    
    let storage = EntityStorage::new(&state.db, "snippet");
    
    match storage.get(&id).await {
        Ok(Some(item)) => {
            tracing::info!("Found snippet: {:?}", item.id);
            serde_json::to_string(&item).map_err(|e| e.to_string())
        }
        Ok(None) => {
            tracing::warn!("Snippet not found: {}", id);
            Err("Snippet not found".to_string())
        }
        Err(e) => {
            tracing::error!("Failed to get snippet: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn list_snippets(state: State<'_, AppState>) -> Result<String, String> {
    use database::storage::EntityStorage;
    
    tracing::info!("Listing all snippets");
    
    let storage = EntityStorage::new(&state.db, "snippet");
    
    match storage.list(Some(100), None).await {
        Ok(items) => {
            tracing::info!("Found {} snippets", items.len());
            serde_json::to_string(&items).map_err(|e| e.to_string())
        }
        Err(e) => {
            tracing::error!("Failed to list snippets: {}", e);
            Err(e.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load environment paths first
    let env_paths = EnvPaths::from_env()
        .expect("Failed to load environment paths");
    
    tracing::info!("Environment paths loaded:");
    tracing::info!("  Data path: {:?}", env_paths.data_path);
    tracing::info!("  Static path: {:?}", env_paths.static_path);
    tracing::info!("  Entity schema path: {:?}", env_paths.entity_schema_path);
    tracing::info!("  Configuration path: {:?}", env_paths.configuration_path);
    
    // Initialize logging system with environment paths
    let _guard = logging::init_logging(&env_paths)
        .expect("Failed to initialize logging system");
    
    tracing::info!("=== Marain CMS starting up ===");
    tracing::info!("Initializing Tauri application");
    
    // Initialize runtime for async operations
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    // Clone paths for async block
    let env_paths_clone = env_paths.clone();
    
    // Initialize database and schema-manager using the new architecture
    let db = runtime.block_on(async {
        tracing::info!("Initializing application systems");
        
        // First, initialize schema-manager to load all schemas
        tracing::info!("Initializing schema-manager system");
        
        // Set environment variables for schema-manager to use
        std::env::set_var("ENTITY_SCHEMA_PATH", env_paths_clone.entity_schema_path.to_str().unwrap());
        std::env::set_var("CONFIGURATION_PATH", env_paths_clone.configuration_path.to_str().unwrap());
        
        if let Err(e) = schema_manager::init(None).await {
            tracing::error!("Failed to initialize schema-manager system: {}", e);
            panic!("Cannot start application without schema-manager: {}", e);
        }
        
        // Get loaded entities from schema-manager
        let entities = schema_manager::get_entities_for_database();
        tracing::info!("Schema-manager loaded {} entities", entities.len());
        
        // Get the database file path from configuration
        let db_file = schema_manager::config_access::get_system_string("database.connections.sqlite.file")
            .unwrap_or_else(|| "content/marain.db".to_string());
        
        // Create database configuration with environment paths
        let database_path = env_paths_clone.data_path.join(db_file);
        let db_config = DatabaseConfig::new_with_path(database_path);
        
        // Initialize database without creating tables yet
        match initialize_database(db_config).await {
            Ok(db) => {
                tracing::info!("Database connection established");
                
                // Now create tables using entities from schema-manager
                if !entities.is_empty() {
                    if let Err(e) = database::init::create_entity_tables_with_entities(&db, entities).await {
                        tracing::error!("Failed to create entity tables: {}", e);
                    } else {
                        tracing::info!("Entity tables created successfully");
                    }
                }
                
                // Get the SQLite pool for schema-manager's watcher (for hot-reload actions)
                let db_pool = database::init::get_pool(&db);
                
                // Re-initialize schema-manager with database pool for hot-reload actions
                schema_manager::set_database_pool(db_pool);
                tracing::info!("Schema-manager configured with database pool for hot-reload");
                
                db
            }
            Err(e) => {
                tracing::error!("Failed to initialize database: {}", e);
                panic!("Cannot start application without database: {}", e);
            }
        }
    });
    
    // Clone the database for the API server
    let api_db = db.clone();
    
    // Start the API server in a background task
    runtime.spawn(async move {
        tracing::info!("Starting API server on port 3030");
        if let Err(e) = api::start_server(api_db).await {
            tracing::error!("API server error: {}", e);
        }
    });
    
    // Create the application state
    let app_state = AppState { db, env_paths };
    
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_config,
            create_snippet,
            get_snippet,
            list_snippets
        ])
        .setup(|app| {
            tracing::info!("Application setup complete");
            tracing::info!("App name: {}", app.package_info().name);
            tracing::info!("App version: {}", app.package_info().version);
            
            // Log the loaded configuration using the new system
            if let Some(env) = config_access::get_system_string("app.environment") {
                tracing::info!("Configuration loaded - Environment: {}", env);
            }
            if let Some(debug_mode) = config_access::get_system_bool("app.debug") {
                tracing::info!("Debug mode: {}", debug_mode);
            }
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    
    tracing::info!("Starting Tauri application event loop");
    
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::Exit => {
            tracing::info!("Exit event received");
            logging::log_shutdown();
        }
        tauri::RunEvent::ExitRequested { .. } => {
            tracing::info!("Exit requested event received");
            // Don't prevent exit - allow graceful shutdown
            // Shutdown logging will be handled by the Exit event
        }
        tauri::RunEvent::WindowEvent { label, event, .. } => {
            tracing::trace!("Window event for {}: {:?}", label, event);
        }
        _ => {}
    });
}
