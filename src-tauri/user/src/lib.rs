pub mod database;
pub mod error;
pub mod secure_log;

use std::sync::Arc;
use tracing::info;

/// User management system
pub struct UserManager {
    database: Arc<database::UserDatabase>,
}

impl UserManager {
    /// Create a new user manager with the provided configuration
    pub async fn new(db_config: database::UserDatabaseConfig) -> error::Result<Self> {
        info!("Initializing user management system");

        let database = Arc::new(database::UserDatabase::new(db_config).await?);

        info!("User management system initialized successfully");

        Ok(Self { database })
    }

    /// Create a new user manager with default configuration
    pub async fn new_default() -> error::Result<Self> {
        Self::new(database::UserDatabaseConfig::default()).await
    }

    /// Get a reference to the database
    pub fn database(&self) -> &database::UserDatabase {
        &self.database
    }

    /// Verify system integrity
    pub async fn verify_integrity(&self) -> error::Result<bool> {
        self.database.verify_integrity().await
    }
}

// Re-export commonly used types with unique names
pub use database::UserDatabaseConfig;
pub use error::{Result as UserResult, UserError};
pub use secure_log::{SecureLogConfig, SecureLogEntry, SecureLogger};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_user_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_user.db");
        let log_path = temp_dir.path().join("test_secure.log");

        let config = UserDatabaseConfig {
            database_path: db_path,
            max_connections: 5,
            connection_timeout: 30,
            secure_log_config: SecureLogConfig {
                log_path,
                max_size_mb: 10,
                max_rotations: 5,
                enable_verification: true,
            },
        };

        let manager = UserManager::new(config).await.unwrap();

        // Verify integrity
        assert!(manager.verify_integrity().await.unwrap());
    }
}
