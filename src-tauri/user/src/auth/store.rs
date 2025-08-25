//! SQLx session store implementation for tower-sessions

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_sessions_sqlx_store::SqliteStore;
use tracing::{debug, error, info};

use crate::error::{Result, UserError};

/// SQLx-based session store for tower-sessions
#[derive(Debug, Clone)]
pub struct SqlxSessionStore {
    store: SqliteStore,
    pool: SqlitePool,
}

impl SqlxSessionStore {
    /// Create a new SQLx session store
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        // Create the session table if it doesn't exist
        Self::create_session_table(&pool).await?;

        // Create the SqliteStore
        let store = SqliteStore::new(pool.clone());

        info!("SQLx session store initialized");
        Ok(Self { store, pool })
    }

    /// Create the session table in the database
    async fn create_session_table(pool: &SqlitePool) -> Result<()> {
        let query = r#"
            CREATE TABLE IF NOT EXISTS tower_sessions (
                id TEXT PRIMARY KEY NOT NULL,
                data BLOB NOT NULL,
                expiry_date INTEGER NOT NULL
            )
        "#;

        sqlx::query(query).execute(pool).await.map_err(|e| {
            error!("Failed to create session table: {}", e);
            UserError::Database(e)
        })?;

        // Create index for expiry_date for efficient cleanup
        let index_query = r#"
            CREATE INDEX IF NOT EXISTS idx_tower_sessions_expiry 
            ON tower_sessions(expiry_date)
        "#;

        sqlx::query(index_query).execute(pool).await.map_err(|e| {
            error!("Failed to create session index: {}", e);
            UserError::Database(e)
        })?;

        debug!("Session table and indexes created/verified");
        Ok(())
    }

    /// Get the underlying SqliteStore
    pub fn inner(&self) -> &SqliteStore {
        &self.store
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<()> {
        // For tower-sessions-sqlx-store 0.14, we need to manually clean up expired sessions
        // by querying the database directly
        let now = chrono::Utc::now().timestamp();

        sqlx::query("DELETE FROM tower_sessions WHERE expiry_date < ?")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired sessions: {}", e);
                UserError::Configuration(format!("Session cleanup failed: {}", e))
            })?;

        info!("Expired sessions cleaned up");
        Ok(())
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session cookie name
    pub cookie_name: String,
    /// Session timeout in seconds
    pub timeout_seconds: i64,
    /// Whether to use secure cookies (HTTPS only)
    pub secure: bool,
    /// SameSite cookie attribute
    pub same_site: SameSiteConfig,
    /// HTTP only cookie (not accessible via JavaScript)
    pub http_only: bool,
    /// Session encryption key (32 bytes)
    pub secret_key: Vec<u8>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        // Generate a random 32-byte key for development
        // In production, this should be loaded from configuration
        let mut secret_key = vec![0u8; 32];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut secret_key);

        Self {
            cookie_name: "marain_session".to_string(),
            timeout_seconds: 86400, // 24 hours
            secure: false,          // Set to true in production with HTTPS
            same_site: SameSiteConfig::Lax,
            http_only: true,
            secret_key,
        }
    }
}

/// SameSite cookie configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SameSiteConfig {
    Strict,
    Lax,
    None,
}

impl From<SameSiteConfig> for tower_sessions::cookie::SameSite {
    fn from(config: SameSiteConfig) -> Self {
        match config {
            SameSiteConfig::Strict => tower_sessions::cookie::SameSite::Strict,
            SameSiteConfig::Lax => tower_sessions::cookie::SameSite::Lax,
            SameSiteConfig::None => tower_sessions::cookie::SameSite::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    async fn create_test_pool() -> SqlitePool {
        // Use a permanent test database directory at project root
        // This follows the CRITICAL-PATHS.md guidelines
        let test_dir = PathBuf::from("../../data/test_databases");
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create a unique test database file
        let test_id = ulid::Ulid::new().to_string();
        let db_path = test_dir.join(format!("test_sessions_{}.db", test_id));

        // Create the empty database file
        std::fs::File::create(&db_path).unwrap();

        let db_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&db_url).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_session_store_creation() {
        let pool = create_test_pool().await;
        let _store = SqlxSessionStore::new(pool.clone()).await.unwrap();

        // Verify table exists by querying it
        let result = sqlx::query("SELECT COUNT(*) as count FROM tower_sessions")
            .fetch_one(&pool)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_config_default() {
        let config = SessionConfig::default();

        assert_eq!(config.cookie_name, "marain_session");
        assert_eq!(config.timeout_seconds, 86400);
        assert!(!config.secure);
        assert!(config.http_only);
        assert_eq!(config.secret_key.len(), 32);
    }
}
