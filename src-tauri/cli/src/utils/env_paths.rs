use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Environment-based path configuration
#[derive(Debug, Clone)]
pub struct EnvPaths {
    pub data_path: PathBuf,
    #[allow(dead_code)]
    pub static_path: PathBuf,
    pub entity_schema_path: PathBuf,
    pub configuration_path: PathBuf,
}

impl EnvPaths {
    /// Load paths from environment variables with defaults
    pub fn load() -> Result<Self> {
        Self::load_with_base(None)
    }

    /// Load paths from environment variables with an optional base directory
    /// This is primarily for testing purposes
    pub fn load_with_base(base_dir: Option<PathBuf>) -> Result<Self> {
        // Determine the base directory to use
        let base = if let Some(base) = base_dir {
            base
        } else {
            // Try to load .env file if it exists in current directory
            if let Ok(env_path) = env::current_dir() {
                let env_file = env_path.join(".env");
                if env_file.exists() {
                    dotenv::from_path(&env_file).ok();
                }
            }
            env::current_dir().context("Failed to get current directory")?
        };

        Ok(Self {
            data_path: Self::get_path_from_env("DATA_PATH", "./data", &base)?,
            static_path: Self::get_path_from_env("STATIC_PATH", "./static", &base)?,
            entity_schema_path: Self::get_path_from_env("ENTITY_SCHEMA_PATH", "./schemas", &base)?,
            configuration_path: Self::get_path_from_env("CONFIGURATION_PATH", "./config", &base)?,
        })
    }

    /// Get a path from environment variable or use default
    fn get_path_from_env(var_name: &str, default: &str, base_dir: &Path) -> Result<PathBuf> {
        let path_str = env::var(var_name).unwrap_or_else(|_| default.to_string());
        let path = PathBuf::from(path_str);

        // If the path is relative, make it relative to the base directory
        if path.is_relative() {
            Ok(base_dir.join(path))
        } else {
            Ok(path)
        }
    }

    /// Get the database path
    pub fn database_path(&self) -> PathBuf {
        self.data_path.join("content").join("marain.db")
    }

    /// Get the logs directory path
    #[allow(dead_code)]
    pub fn logs_path(&self) -> PathBuf {
        self.data_path.join("logs")
    }

    /// Get the JSON cache database path
    #[allow(dead_code)]
    pub fn json_cache_path(&self) -> PathBuf {
        self.data_path
            .join("json-cache")
            .join("marain_json_cache.db")
    }

    /// Get the user database path
    #[allow(dead_code)]
    pub fn user_database_path(&self) -> PathBuf {
        self.data_path.join("user-backend").join("marain_user.db")
    }

    /// Get the secure log path
    #[allow(dead_code)]
    pub fn secure_log_path(&self) -> PathBuf {
        self.data_path.join("user-backend").join("secure.log")
    }

    /// Get the uploads directory path
    #[allow(dead_code)]
    pub fn uploads_path(&self) -> PathBuf {
        self.data_path.join("uploads")
    }

    /// Get the search index path
    #[allow(dead_code)]
    pub fn search_index_path(&self) -> PathBuf {
        self.data_path.join("search_index")
    }
}

/// Get the current environment (dev, staging, production)
pub fn get_environment() -> String {
    env::var("ENVIRONMENT")
        .unwrap_or_else(|_| env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Use a mutex to ensure tests don't interfere with each other's environment variables
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_env_paths_with_defaults() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Clear any existing env vars
        env::remove_var("DATA_PATH");
        env::remove_var("STATIC_PATH");
        env::remove_var("ENTITY_SCHEMA_PATH");
        env::remove_var("CONFIGURATION_PATH");

        let paths = EnvPaths::load().unwrap();

        // Should use defaults
        assert!(paths.data_path.ends_with("data"));
        assert!(paths.static_path.ends_with("static"));
        assert!(paths.entity_schema_path.ends_with("schemas"));
        assert!(paths.configuration_path.ends_with("config"));
    }

    #[test]
    fn test_env_paths_with_relative_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Use relative paths for environment variables
        env::set_var("DATA_PATH", "./custom_data");
        env::set_var("STATIC_PATH", "./custom_static");
        env::set_var("ENTITY_SCHEMA_PATH", "./custom_schemas");
        env::set_var("CONFIGURATION_PATH", "./custom_config");

        let paths = EnvPaths::load().unwrap();

        assert!(paths.data_path.ends_with("custom_data"));
        assert!(paths.static_path.ends_with("custom_static"));
        assert!(paths.entity_schema_path.ends_with("custom_schemas"));
        assert!(paths.configuration_path.ends_with("custom_config"));

        // Clean up
        env::remove_var("DATA_PATH");
        env::remove_var("STATIC_PATH");
        env::remove_var("ENTITY_SCHEMA_PATH");
        env::remove_var("CONFIGURATION_PATH");
    }

    #[test]
    fn test_env_paths_with_absolute_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Set absolute paths
        env::set_var("DATA_PATH", temp_path.join("custom_data").to_str().unwrap());
        env::set_var(
            "STATIC_PATH",
            temp_path.join("custom_static").to_str().unwrap(),
        );
        env::set_var(
            "ENTITY_SCHEMA_PATH",
            temp_path.join("custom_schemas").to_str().unwrap(),
        );
        env::set_var(
            "CONFIGURATION_PATH",
            temp_path.join("custom_config").to_str().unwrap(),
        );

        let paths = EnvPaths::load().unwrap();

        // When absolute paths are provided, they should be used as-is
        assert_eq!(paths.data_path, temp_path.join("custom_data"));
        assert_eq!(paths.static_path, temp_path.join("custom_static"));
        assert_eq!(paths.entity_schema_path, temp_path.join("custom_schemas"));
        assert_eq!(paths.configuration_path, temp_path.join("custom_config"));

        // Clean up
        env::remove_var("DATA_PATH");
        env::remove_var("STATIC_PATH");
        env::remove_var("ENTITY_SCHEMA_PATH");
        env::remove_var("CONFIGURATION_PATH");
    }

    #[test]
    fn test_env_paths_with_base_dir() {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Clear any existing env vars to use defaults
        env::remove_var("DATA_PATH");
        env::remove_var("STATIC_PATH");
        env::remove_var("ENTITY_SCHEMA_PATH");
        env::remove_var("CONFIGURATION_PATH");

        let paths = EnvPaths::load_with_base(Some(base_path.clone())).unwrap();

        // Should use defaults relative to base_path
        assert_eq!(paths.data_path, base_path.join("data"));
        assert_eq!(paths.static_path, base_path.join("static"));
        assert_eq!(paths.entity_schema_path, base_path.join("schemas"));
        assert_eq!(paths.configuration_path, base_path.join("config"));
    }

    #[test]
    fn test_derived_paths() {
        let _guard = ENV_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();

        // Clear env vars to use defaults
        env::remove_var("DATA_PATH");
        env::remove_var("STATIC_PATH");
        env::remove_var("ENTITY_SCHEMA_PATH");
        env::remove_var("CONFIGURATION_PATH");

        let paths = EnvPaths::load_with_base(Some(base_path.clone())).unwrap();

        // Check that the paths are correctly derived
        assert_eq!(
            paths.database_path(),
            base_path.join("data/content/marain.db")
        );
        assert_eq!(paths.logs_path(), base_path.join("data/logs"));
        assert_eq!(
            paths.json_cache_path(),
            base_path.join("data/json-cache/marain_json_cache.db")
        );
        assert_eq!(
            paths.user_database_path(),
            base_path.join("data/user-backend/marain_user.db")
        );
        assert_eq!(
            paths.secure_log_path(),
            base_path.join("data/user-backend/secure.log")
        );
        assert_eq!(paths.uploads_path(), base_path.join("data/uploads"));
        assert_eq!(
            paths.search_index_path(),
            base_path.join("data/search_index")
        );
    }

    #[test]
    fn test_get_environment() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Test default
        env::remove_var("ENVIRONMENT");
        env::remove_var("APP_ENV");
        assert_eq!(get_environment(), "dev");

        // Test ENVIRONMENT var
        env::set_var("ENVIRONMENT", "production");
        assert_eq!(get_environment(), "production");

        // Test APP_ENV as fallback
        env::remove_var("ENVIRONMENT");
        env::set_var("APP_ENV", "staging");
        assert_eq!(get_environment(), "staging");

        // Clean up
        env::remove_var("ENVIRONMENT");
        env::remove_var("APP_ENV");
    }
}
