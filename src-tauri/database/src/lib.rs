use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::Path;
use tracing::{debug, info};

pub mod error;
pub mod init;
pub mod storage;

pub use error::{DatabaseError, Result};

// Re-export entity and schema types from the entities crate
pub use entities::{Entity, EntityDefinition, GenericEntity, SchemaLoader};
// Re-export field types from the fields crate
pub use fields::{Field, FieldType};

// Re-export initialization functions for convenience
pub use init::{initialize_database, initialize_default, DatabaseConfig};

/// Database connection pool
#[derive(Debug)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_path: &str) -> Result<Self> {
        // Ensure the data directory exists
        if let Some(parent) = Path::new(database_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Connecting to database at: {}", database_path);

        // For SQLite, we need to ensure the proper connection string format
        let connection_string =
            if database_path.starts_with("sqlite:") || database_path.starts_with(":memory:") {
                database_path.to_string()
            } else {
                // SQLite connection strings need to be in the format sqlite://path
                // For absolute paths, use sqlite:///path
                // For relative paths, use sqlite://path
                if database_path.starts_with("/") {
                    format!("sqlite://{}", database_path)
                } else {
                    format!("sqlite:{}", database_path)
                }
            };

        debug!("Using connection string: {}", connection_string);

        let pool = SqlitePool::connect(&connection_string).await?;

        debug!("Database connection established");

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Get a clone of the connection pool
    pub fn get_pool(&self) -> Pool<Sqlite> {
        self.pool.clone()
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");
        // TODO: Add migrations when needed
        // sqlx::migrate!("./migrations")
        //     .run(&self.pool)
        //     .await?;
        info!("Database migrations completed");
        Ok(())
    }

    /// Check if a table exists
    pub async fn table_exists(&self, table_name: &str) -> Result<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM sqlite_master
            WHERE type='table' AND name=?
        "#;

        let result: (i32,) = sqlx::query_as(query)
            .bind(table_name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0 > 0)
    }

    /// Execute raw SQL (for table creation, etc.)
    pub async fn execute_raw(&self, sql: &str) -> Result<()> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static _INIT: Once = Once::new();

    fn get_test_db_path() -> String {
        // Get the project root by navigating up from CARGO_MANIFEST_DIR
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let database_dir = std::path::Path::new(manifest_dir); // src-tauri/database
        let src_tauri = database_dir.parent().unwrap(); // src-tauri
        let project_root = src_tauri.parent().unwrap(); // project root

        // Use /data/test_databases/ at project root
        let test_db_dir = project_root.join("data").join("test_databases");
        std::fs::create_dir_all(&test_db_dir).unwrap();

        // Create a unique test database name using process ID and thread ID
        let test_id = format!("{}_{:?}", std::process::id(), std::thread::current().id());
        let db_path = test_db_dir.join(format!("test_{}.db", test_id));

        // Clean up any existing test database with this name
        let _ = std::fs::remove_file(&db_path);

        db_path.to_string_lossy().to_string()
    }

    async fn create_test_db() -> Database {
        let db_path = get_test_db_path();

        // Remove existing test database if it exists
        let _ = std::fs::remove_file(&db_path);

        // Create the database file first (SQLite requires this)
        std::fs::File::create(&db_path).unwrap();

        let db = Database::new(&db_path).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_database_connection() {
        let db = create_test_db().await;
        assert!(db.pool().acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_table_exists() {
        let db = create_test_db().await;

        // Create a test table
        db.execute_raw("CREATE TABLE test_table (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();

        assert!(db.table_exists("test_table").await.unwrap());
        assert!(!db.table_exists("non_existent_table").await.unwrap());
    }
}
