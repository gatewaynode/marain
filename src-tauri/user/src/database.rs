use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::error::{Result, UserError};
use crate::secure_log::{SecureLogConfig, SecureLogger};

/// Configuration for the user database
#[derive(Debug, Clone)]
pub struct UserDatabaseConfig {
    /// Path to the database file
    pub database_path: PathBuf,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Secure log configuration
    pub secure_log_config: SecureLogConfig,
}

impl Default for UserDatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("data/user-backend/marain_user.db"),
            max_connections: 5,
            connection_timeout: 30,
            secure_log_config: SecureLogConfig::default(),
        }
    }
}

/// User database manager
pub struct UserDatabase {
    pool: Pool<Sqlite>,
    #[allow(dead_code)]
    config: UserDatabaseConfig,
    secure_logger: SecureLogger,
}

impl UserDatabase {
    /// Initialize the user database
    pub async fn new(config: UserDatabaseConfig) -> Result<Self> {
        // Ensure the directory exists
        if let Some(parent) = config.database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create database URL
        let db_url = format!("sqlite:{}", config.database_path.display());

        // Create database if it doesn't exist
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            info!(
                "Creating user database at: {}",
                config.database_path.display()
            );
            Sqlite::create_database(&db_url).await.map_err(|e| {
                UserError::Initialization(format!("Failed to create database: {}", e))
            })?;
        }

        // Create connection pool
        let pool = SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&config.database_path)
                .create_if_missing(true),
        )
        .await?;

        // Initialize secure logger
        let secure_logger = SecureLogger::new(config.secure_log_config.clone())?;

        let db = Self {
            pool,
            config,
            secure_logger,
        };

        // Run migrations
        db.run_migrations().await?;

        // Log initialization
        db.secure_logger
            .log_action(0, "database_initialized", None, None, None, true)
            .await?;

        info!("User database initialized successfully");

        Ok(db)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        info!("Running user database migrations");

        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                backup_email TEXT UNIQUE,
                phone_number TEXT UNIQUE,
                backup_phone_number TEXT UNIQUE,
                passkey TEXT,
                magic_link_token TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_login TIMESTAMP,
                is_active BOOLEAN DEFAULT 1,
                metadata TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create roles table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS roles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create permissions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS permissions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                resource TEXT NOT NULL,
                action TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(resource, action)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create user_roles junction table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_roles (
                user_id TEXT NOT NULL,
                role_id TEXT NOT NULL,
                assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                assigned_by TEXT,
                PRIMARY KEY (user_id, role_id),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create role_permissions junction table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS role_permissions (
                role_id TEXT NOT NULL,
                permission_id TEXT NOT NULL,
                granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                granted_by TEXT,
                PRIMARY KEY (role_id, permission_id),
                FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
                FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create sessions table for tower-sessions
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tower_sessions (
                id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                expiry_date INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_roles_role ON user_roles(role_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON tower_sessions(expiry_date)",
        )
        .execute(&self.pool)
        .await?;

        // Insert default roles if they don't exist
        self.create_default_roles().await?;

        info!("User database migrations completed");

        Ok(())
    }

    /// Create default roles
    async fn create_default_roles(&self) -> Result<()> {
        let default_roles = vec![
            ("admin", "Administrator", "Full system access"),
            ("editor", "Editor", "Can create and edit content"),
            ("viewer", "Viewer", "Read-only access"),
        ];

        for (name, display_name, description) in default_roles {
            let id = ulid::Ulid::new().to_string();

            // Check if role exists
            let exists =
                sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM roles WHERE name = ?)")
                    .bind(name)
                    .fetch_one(&self.pool)
                    .await?;

            if !exists {
                sqlx::query("INSERT INTO roles (id, name, description) VALUES (?, ?, ?)")
                    .bind(&id)
                    .bind(name)
                    .bind(format!("{}: {}", display_name, description))
                    .execute(&self.pool)
                    .await?;

                info!("Created default role: {}", name);

                // Log role creation
                self.secure_logger
                    .log_action(
                        0,
                        "role_created",
                        Some(id),
                        Some(serde_json::json!({
                            "name": name,
                            "description": description
                        })),
                        None,
                        true,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    /// Get the database pool for external use
    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Get the secure logger
    pub fn get_logger(&self) -> &SecureLogger {
        &self.secure_logger
    }

    /// Verify database integrity
    pub async fn verify_integrity(&self) -> Result<bool> {
        // Check if all required tables exist
        let tables = vec![
            "users",
            "roles",
            "permissions",
            "user_roles",
            "role_permissions",
            "tower_sessions",
        ];

        for table in tables {
            let exists = sqlx::query_scalar::<_, bool>(&format!(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='{}')",
                table
            ))
            .fetch_one(&self.pool)
            .await?;

            if !exists {
                warn!("Missing table: {}", table);
                return Ok(false);
            }
        }

        // Verify secure log chain
        let log_valid = self.secure_logger.verify_log_chain().await?;
        if !log_valid {
            warn!("Secure log chain verification failed");
            return Ok(false);
        }

        info!("Database integrity check passed");
        Ok(true)
    }

    /// Close the database connection
    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        info!("User database connection closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_user.db");
        let log_path = temp_dir.path().join("test_secure.log");

        let config = UserDatabaseConfig {
            database_path: db_path.clone(),
            max_connections: 5,
            connection_timeout: 30,
            secure_log_config: SecureLogConfig {
                log_path,
                max_size_mb: 10,
                max_rotations: 5,
                enable_verification: true,
            },
        };

        let db = UserDatabase::new(config).await.unwrap();

        // Verify database was created
        assert!(db_path.exists());

        // Verify integrity
        assert!(db.verify_integrity().await.unwrap());

        // Close database
        db.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_default_roles_creation() {
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

        let db = UserDatabase::new(config).await.unwrap();

        // Check that default roles were created
        let role_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM roles")
            .fetch_one(db.get_pool())
            .await
            .unwrap();

        assert_eq!(role_count, 3); // admin, editor, viewer

        db.close().await.unwrap();
    }
}
