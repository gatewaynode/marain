use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt::Debug;

/// Configuration definition that describes a configuration module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationDefinition {
    /// Unique identifier for this configuration (e.g., "system", "api", "module_content")
    pub id: String,
    /// Human-readable name for this configuration
    pub name: String,
    /// Description of what this configuration controls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The module or crate that provides this configuration
    pub provider: String,
    /// Version of the configuration schema
    pub version: String,
    /// The actual configuration values
    pub values: HashMap<String, Value>,
}

/// Trait for configuration operations - similar to Entity trait but for configurations
pub trait Configuration: Send + Sync + Debug {
    /// Get the configuration definition
    fn definition(&self) -> &ConfigurationDefinition;

    /// Get the configuration ID
    fn id(&self) -> &str {
        &self.definition().id
    }

    /// Get the provider module name
    fn provider(&self) -> &str {
        &self.definition().provider
    }

    /// Get a configuration value by key
    fn get_value(&self, key: &str) -> Option<&Value>;

    /// Get all configuration values
    fn get_all_values(&self) -> &HashMap<String, Value>;

    /// Validate the configuration values
    fn validate(&self) -> Result<(), ConfigurationError>;

    /// Merge with another configuration (for overrides)
    fn merge(&mut self, other: &dyn Configuration) -> Result<(), ConfigurationError>;

    /// Convert to a serializable format
    fn to_yaml(&self) -> Result<String, ConfigurationError>;

    /// Apply configuration changes (hook for modules to react to config changes)
    fn apply_changes(&self) -> Result<(), ConfigurationError> {
        // Default implementation does nothing
        // Modules can override this to react to configuration changes
        Ok(())
    }
}

/// Generic configuration implementation
#[derive(Debug, Clone)]
pub struct GenericConfiguration {
    definition: ConfigurationDefinition,
}

impl GenericConfiguration {
    /// Create a new generic configuration
    pub fn new(definition: ConfigurationDefinition) -> Self {
        Self { definition }
    }

    /// Create from YAML content
    pub fn from_yaml(content: &str) -> Result<Self, ConfigurationError> {
        let definition: ConfigurationDefinition = serde_yaml::from_str(content)
            .map_err(|e| ConfigurationError::ParseError(e.to_string()))?;
        Ok(Self::new(definition))
    }

    /// Load from file path
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigurationError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigurationError::IoError(e.to_string()))?;
        Self::from_yaml(&content)
    }
}

impl Configuration for GenericConfiguration {
    fn definition(&self) -> &ConfigurationDefinition {
        &self.definition
    }

    fn get_value(&self, key: &str) -> Option<&Value> {
        self.definition.values.get(key)
    }

    fn get_all_values(&self) -> &HashMap<String, Value> {
        &self.definition.values
    }

    fn validate(&self) -> Result<(), ConfigurationError> {
        // Basic validation - ensure required fields are present
        if self.definition.id.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Configuration ID cannot be empty".to_string(),
            ));
        }
        if self.definition.provider.is_empty() {
            return Err(ConfigurationError::ValidationError(
                "Configuration provider cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn merge(&mut self, other: &dyn Configuration) -> Result<(), ConfigurationError> {
        // Only merge configurations with the same ID
        if self.id() != other.id() {
            return Err(ConfigurationError::MergeError(format!(
                "Cannot merge configurations with different IDs: {} and {}",
                self.id(),
                other.id()
            )));
        }

        // Merge values - other configuration values override self
        for (key, value) in other.get_all_values() {
            self.definition.values.insert(key.clone(), value.clone());
        }

        Ok(())
    }

    fn to_yaml(&self) -> Result<String, ConfigurationError> {
        serde_yaml::to_string(&self.definition)
            .map_err(|e| ConfigurationError::SerializationError(e.to_string()))
    }
}

/// Configuration-specific error type
#[derive(Debug)]
pub enum ConfigurationError {
    ParseError(String),
    ValidationError(String),
    MergeError(String),
    SerializationError(String),
    IoError(String),
    NotFound(String),
}

impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Configuration parse error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Configuration validation error: {}", msg),
            Self::MergeError(msg) => write!(f, "Configuration merge error: {}", msg),
            Self::SerializationError(msg) => {
                write!(f, "Configuration serialization error: {}", msg)
            }
            Self::IoError(msg) => write!(f, "Configuration I/O error: {}", msg),
            Self::NotFound(msg) => write!(f, "Configuration not found: {}", msg),
        }
    }
}

impl std::error::Error for ConfigurationError {}

/// Configuration loader - loads configurations from YAML files
pub struct ConfigurationLoader;

impl ConfigurationLoader {
    /// Load configurations from a directory
    pub async fn load_configurations_from_directory(
        dir: &std::path::Path,
    ) -> Result<Vec<Box<dyn Configuration>>, ConfigurationError> {
        let mut configurations = Vec::new();

        if !dir.exists() {
            return Ok(configurations);
        }

        // Get the current environment
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

        // Read all YAML files in the directory
        let entries =
            std::fs::read_dir(dir).map_err(|e| ConfigurationError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| ConfigurationError::IoError(e.to_string()))?;
            let path = entry.path();

            // Check if it's a YAML configuration file
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    // Check if it's a configuration file (not a schema file)
                    if let Some(stem) = path.file_stem() {
                        let filename = stem.to_string_lossy();
                        // Only load new-style configuration files (config.*)
                        if filename.starts_with("config.") {
                            // Check if this is an environment-specific file
                            let should_load = if filename.starts_with("config.system.") {
                                // For system config, only load the one matching the current environment
                                filename == format!("config.system.{}", env)
                            } else {
                                // For other configs, load all of them
                                true
                            };

                            if should_load {
                                match GenericConfiguration::from_file(&path) {
                                    Ok(config) => {
                                        if let Err(e) = config.validate() {
                                            tracing::warn!(
                                                "Configuration validation failed for {:?}: {}",
                                                path,
                                                e
                                            );
                                            continue;
                                        }
                                        configurations
                                            .push(Box::new(config) as Box<dyn Configuration>);
                                        tracing::info!("Loaded configuration from {:?}", path);
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to load configuration from {:?}: {}",
                                            path,
                                            e
                                        );
                                    }
                                }
                            } else {
                                tracing::debug!(
                                    "Skipping configuration file for different environment: {:?}",
                                    path
                                );
                            }
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Loaded {} configurations from {:?} for environment: {}",
            configurations.len(),
            dir,
            env
        );
        Ok(configurations)
    }

    /// Load a single configuration from a file
    pub async fn load_configuration(
        path: &std::path::Path,
    ) -> Result<Box<dyn Configuration>, ConfigurationError> {
        let config = GenericConfiguration::from_file(path)?;
        config.validate()?;
        Ok(Box::new(config))
    }
}

/// Helper functions for working with configuration values
pub mod helpers {
    use super::*;

    /// Extract a string value from configuration
    pub fn get_string(config: &dyn Configuration, key: &str) -> Option<String> {
        config
            .get_value(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Extract a boolean value from configuration
    pub fn get_bool(config: &dyn Configuration, key: &str) -> Option<bool> {
        config.get_value(key).and_then(|v| v.as_bool())
    }

    /// Extract an integer value from configuration
    pub fn get_i64(config: &dyn Configuration, key: &str) -> Option<i64> {
        config.get_value(key).and_then(|v| v.as_i64())
    }

    /// Extract a float value from configuration
    pub fn get_f64(config: &dyn Configuration, key: &str) -> Option<f64> {
        config.get_value(key).and_then(|v| v.as_f64())
    }

    /// Extract a nested configuration value using dot notation
    pub fn get_nested<'a>(config: &'a dyn Configuration, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config.get_value(parts[0])?;

        for part in &parts[1..] {
            current = current.get(part)?;
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_configuration() {
        let mut values = HashMap::new();
        values.insert("debug".to_string(), Value::Bool(true));
        values.insert("port".to_string(), Value::Number(8080.into()));

        let def = ConfigurationDefinition {
            id: "test".to_string(),
            name: "Test Configuration".to_string(),
            description: Some("Test configuration for unit tests".to_string()),
            provider: "test_module".to_string(),
            version: "1.0.0".to_string(),
            values,
        };

        let config = GenericConfiguration::new(def);

        assert_eq!(config.id(), "test");
        assert_eq!(config.provider(), "test_module");
        assert_eq!(config.get_value("debug"), Some(&Value::Bool(true)));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_configuration_merge() {
        let mut values1 = HashMap::new();
        values1.insert("debug".to_string(), Value::Bool(true));
        values1.insert("port".to_string(), Value::Number(8080.into()));

        let mut values2 = HashMap::new();
        values2.insert("debug".to_string(), Value::Bool(false)); // Override
        values2.insert("host".to_string(), Value::String("localhost".to_string())); // New value

        let def1 = ConfigurationDefinition {
            id: "test".to_string(),
            name: "Test Configuration".to_string(),
            description: None,
            provider: "test_module".to_string(),
            version: "1.0.0".to_string(),
            values: values1,
        };

        let def2 = ConfigurationDefinition {
            id: "test".to_string(),
            name: "Test Configuration Override".to_string(),
            description: None,
            provider: "test_module".to_string(),
            version: "1.0.0".to_string(),
            values: values2,
        };

        let mut config1 = GenericConfiguration::new(def1);
        let config2 = GenericConfiguration::new(def2);

        assert!(config1.merge(&config2).is_ok());

        // Check that values were merged correctly
        assert_eq!(config1.get_value("debug"), Some(&Value::Bool(false))); // Overridden
        assert_eq!(config1.get_value("port"), Some(&Value::Number(8080.into()))); // Kept
        assert_eq!(
            config1.get_value("host"),
            Some(&Value::String("localhost".to_string()))
        ); // Added
    }
}
