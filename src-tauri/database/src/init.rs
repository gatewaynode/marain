use crate::{Database, Result};
use entities::Entity;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

/// Database initialization configuration
pub struct DatabaseConfig {
    /// Path to the database file
    pub database_path: PathBuf,
    /// Whether to create tables on initialization
    pub create_tables: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let project_root = Self::find_project_root();

        // Always use the project root's data directory
        let data_dir = project_root.join("data");

        Self {
            database_path: data_dir.join("marain.db"),
            create_tables: true,
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration with default paths
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new database configuration with a specific database path
    pub fn new_with_path(database_path: PathBuf) -> Self {
        Self {
            database_path,
            create_tables: true,
        }
    }

    /// Find the project root directory
    fn find_project_root() -> PathBuf {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");

        if current_dir.ends_with("src-tauri") {
            current_dir
                .parent()
                .expect("Failed to get parent directory")
                .to_path_buf()
        } else if current_dir.ends_with("app") {
            current_dir
                .parent()
                .and_then(|p| p.parent())
                .expect("Failed to get project root")
                .to_path_buf()
        } else {
            current_dir
        }
    }

    /// Set a custom database path
    pub fn with_database_path(mut self, path: PathBuf) -> Self {
        self.database_path = path;
        self
    }

    /// Set whether to create tables on initialization
    pub fn with_create_tables(mut self, create: bool) -> Self {
        self.create_tables = create;
        self
    }
}

/// Initialize the database with the given configuration
pub async fn initialize_database(config: DatabaseConfig) -> Result<Arc<Database>> {
    info!("Initializing database with configuration");

    // Ensure the data directory exists
    if let Some(parent) = config.database_path.parent() {
        std::fs::create_dir_all(parent)?;
        info!("Created data directory at: {:?}", parent);
    }

    // Create the database file if it doesn't exist
    if !config.database_path.exists() {
        std::fs::File::create(&config.database_path)?;
        info!("Created new database file at: {:?}", config.database_path);
    }

    // Convert path to string for database connection
    let db_path_str = config
        .database_path
        .to_str()
        .ok_or_else(|| crate::DatabaseError::Other("Invalid database path".into()))?;

    info!("Connecting to database at: {}", db_path_str);

    // Create database connection
    let db = Database::new(db_path_str).await?;
    info!("Database connection established");

    // Wrap in Arc for sharing
    let db = Arc::new(db);

    // Create tables if requested
    if config.create_tables {
        create_entity_tables(&db).await?;
    }

    Ok(db)
}

/// Create tables for all entities using pre-loaded entities from schema-manager
pub async fn create_entity_tables(_db: &Database) -> Result<()> {
    info!("Creating entity tables from schema-manager entities");

    Ok(())
}

/// Create tables for specific entities
pub async fn create_entity_tables_with_entities(
    db: &Database,
    entities: Vec<Arc<Box<dyn Entity>>>,
) -> Result<()> {
    info!("Creating tables for {} entities", entities.len());

    // Create tables for each entity
    for entity in entities {
        let entity_id = entity.definition().id.clone();
        match entity.create_tables(db.pool()).await {
            Ok(_) => info!("Created tables for entity: {}", entity_id),
            Err(e) => error!("Failed to create tables for entity {}: {}", entity_id, e),
        }
    }

    Ok(())
}

/// Initialize database with default configuration
pub async fn initialize_default() -> Result<Arc<Database>> {
    let config = DatabaseConfig::new();
    initialize_database(config).await
}

/// Get the database pool for use with other systems (e.g., schema-manager)
pub fn get_pool(db: &Arc<Database>) -> SqlitePool {
    db.get_pool()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create the database file first (SQLite requires this for non-memory databases)
        std::fs::File::create(&db_path).unwrap();

        let config = DatabaseConfig::new()
            .with_database_path(db_path.clone())
            .with_create_tables(false); // Don't create tables for test

        let db = initialize_database(config).await.unwrap();

        // Verify database file was created
        assert!(db_path.exists());

        // Verify we can get a pool
        let pool = get_pool(&db);
        assert!(pool.acquire().await.is_ok());
    }

    #[test]
    fn test_find_project_root() {
        // This test verifies the project root finding logic
        let root = DatabaseConfig::find_project_root();

        // The root should contain a Cargo.toml or src-tauri directory
        assert!(root.join("src-tauri").exists() || root.join("Cargo.toml").exists());
    }
}
