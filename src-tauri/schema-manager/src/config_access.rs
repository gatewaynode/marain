//! Helper module for accessing configurations using the new trait-based system
//! This provides a bridge between the old and new configuration systems

use crate::configuration::{Configuration, helpers};
use crate::{get_configuration, get_all_configurations, get_configuration_value};
use serde_yaml::Value;
use std::sync::Arc;

/// Configuration accessor that provides type-safe access to configuration values
pub struct ConfigAccess;

impl ConfigAccess {
    /// Get a string value from a configuration
    /// Path format: "configuration_id.key"
    pub fn get_string(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let config = get_configuration(parts[0])?;
        helpers::get_string(&**config, parts[1])
    }
    
    /// Get a boolean value from a configuration
    /// Path format: "configuration_id.key"
    pub fn get_bool(path: &str) -> Option<bool> {
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let config = get_configuration(parts[0])?;
        helpers::get_bool(&**config, parts[1])
    }
    
    /// Get an integer value from a configuration
    /// Path format: "configuration_id.key"
    pub fn get_i64(path: &str) -> Option<i64> {
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let config = get_configuration(parts[0])?;
        helpers::get_i64(&**config, parts[1])
    }
    
    /// Get a float value from a configuration
    /// Path format: "configuration_id.key"
    pub fn get_f64(path: &str) -> Option<f64> {
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let config = get_configuration(parts[0])?;
        helpers::get_f64(&**config, parts[1])
    }
    
    /// Get a nested value using dot notation
    /// Path format: "configuration_id.nested.key.path"
    pub fn get_nested_value(path: &str) -> Option<Value> {
        get_configuration_value(path)
    }
    
    /// Get all configurations for a specific provider/module
    pub fn get_configurations_by_provider(provider: &str) -> Vec<Arc<Box<dyn Configuration>>> {
        get_all_configurations()
            .into_iter()
            .filter(|c| c.provider() == provider)
            .collect()
    }
    
    /// Check if a configuration exists
    pub fn has_configuration(id: &str) -> bool {
        get_configuration(id).is_some()
    }
    
    /// Get all configuration IDs
    pub fn get_all_configuration_ids() -> Vec<String> {
        get_all_configurations()
            .iter()
            .map(|c| c.id().to_string())
            .collect()
    }
}

/// Macro for easy configuration access
/// Usage: config_get!(string, "system.app.name")
///        config_get!(bool, "system.app.debug")
#[macro_export]
macro_rules! config_get {
    (string, $path:expr) => {
        $crate::config_access::ConfigAccess::get_string($path)
    };
    (bool, $path:expr) => {
        $crate::config_access::ConfigAccess::get_bool($path)
    };
    (i64, $path:expr) => {
        $crate::config_access::ConfigAccess::get_i64($path)
    };
    (f64, $path:expr) => {
        $crate::config_access::ConfigAccess::get_f64($path)
    };
}

/// Helper to get system configuration values
/// This provides easy access to the system configuration
pub fn get_system_value(key: &str) -> Option<Value> {
    get_configuration_value(&format!("system.{}", key))
}

/// Helper to get system configuration as string
pub fn get_system_string(key: &str) -> Option<String> {
    ConfigAccess::get_string(&format!("system.{}", key))
}

/// Helper to get system configuration as bool
pub fn get_system_bool(key: &str) -> Option<bool> {
    ConfigAccess::get_bool(&format!("system.{}", key))
}

/// Helper to get system configuration as i64
pub fn get_system_i64(key: &str) -> Option<i64> {
    ConfigAccess::get_i64(&format!("system.{}", key))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_access_parsing() {
        // Test path parsing
        let path = "system.app.name";
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "system");
        assert_eq!(parts[1], "app.name");
    }
}