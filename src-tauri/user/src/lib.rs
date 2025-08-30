pub mod auth;
pub mod database;
pub mod error;
pub mod secure_log;

#[cfg(test)]
mod test_secure_log_restart;

use std::sync::Arc;
use tracing::info;

use auth::AuthBackend;
pub use auth::{SessionConfig, SqlxSessionStore};
use database::UserDatabase;

/// User management system with authentication
pub struct UserManager {
    database: Arc<UserDatabase>,
    auth_backend: Arc<AuthBackend>,
    session_store: SqlxSessionStore,
    session_config: SessionConfig,
}

impl UserManager {
    /// Create a new user manager with the provided configuration
    pub async fn new(
        db_config: database::UserDatabaseConfig,
        session_config: SessionConfig,
    ) -> error::Result<Self> {
        info!("Initializing user management system with authentication");

        // Initialize database
        let database = Arc::new(database::UserDatabase::new(db_config).await?);

        // Get secure logger from database (already an Arc)
        let secure_logger = database.get_logger();

        // Initialize authentication backend
        let auth_backend = Arc::new(AuthBackend::new(database.clone(), secure_logger));

        // Initialize session store
        let session_store = SqlxSessionStore::new(database.pool().clone()).await?;

        info!("User management system initialized successfully");

        Ok(Self {
            database,
            auth_backend,
            session_store,
            session_config,
        })
    }

    /// Create a new user manager with default configuration
    pub async fn new_default() -> error::Result<Self> {
        Self::new(database::UserDatabaseConfig::default(), SessionConfig::new()?).await
    }

    /// Get a reference to the database
    pub fn database(&self) -> &UserDatabase {
        &self.database
    }

    /// Get a reference to the authentication backend
    pub fn auth_backend(&self) -> &AuthBackend {
        &self.auth_backend
    }

    /// Get a reference to the session store
    pub fn session_store(&self) -> &SqlxSessionStore {
        &self.session_store
    }

    /// Get the session configuration
    pub fn session_config(&self) -> &SessionConfig {
        &self.session_config
    }

    /// Verify system integrity
    pub async fn verify_integrity(&self) -> error::Result<bool> {
        self.database.verify_integrity().await
    }

    /// Clean up expired sessions and tokens
    pub async fn cleanup_expired(&self) -> error::Result<()> {
        // Clean up expired sessions
        self.session_store.cleanup_expired().await?;

        // Clean up expired magic link tokens
        auth::magic_link::MagicLinkManager::cleanup_expired_tokens(&self.database).await?;

        info!("Cleaned up expired sessions and tokens");
        Ok(())
    }
}

// Re-export commonly used types
pub use database::UserDatabaseConfig;
pub use error::{Result as UserResult, UserError};
pub use secure_log::{SecureLogConfig, SecureLogEntry, SecureLogger};

// Re-export authentication types from auth module
pub use auth::{AuthState, AuthenticatedUser, AuthenticationMethod, Credentials};

// Re-export types from auth::types
pub use auth::types::{
    AuthResponse, MagicLinkRequest, MagicLinkVerificationRequest, PassKeyLoginChallenge,
    PassKeyLoginRequest, PassKeyVerificationRequest, SessionData,
};

// Re-export session types
pub use auth::session::SessionManager;
pub use auth::store::SameSiteConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_user_manager_creation() {
        // Ensure .env file is loaded for tests
        dotenvy::dotenv().ok();

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

        let manager = UserManager::new(config, SessionConfig::new().unwrap())
            .await
            .unwrap();


        // Verify integrity
        assert!(manager.verify_integrity().await.unwrap());
    }
}
