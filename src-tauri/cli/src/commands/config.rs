use crate::utils::env_paths::EnvPaths;
use anyhow::{anyhow, Result};
use colored::*;
use std::collections::HashMap;

/// List all loaded configurations
pub async fn list(format: String) -> Result<()> {
    // Load environment paths
    let env_paths = EnvPaths::load()?;

    // Load all configurations using ConfigurationLoader
    let configs =
        schema_manager::configuration::ConfigurationLoader::load_configurations_from_directory(
            &env_paths.configuration_path,
        )
        .await
        .map_err(|e| anyhow!("Failed to load configurations: {}", e))?;

    // Convert to a HashMap for display
    let mut config_map = HashMap::new();
    for config in configs.iter() {
        config_map.insert(config.id().to_string(), config.get_all_values().clone());
    }

    match format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&config_map)?;
            println!("{}", json_output);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&config_map)?;
            println!("{}", yaml_output);
        }
        "text" => {
            print_configs_text(&config_map);
        }
        _ => {
            print_configs_text(&config_map);
        }
    }

    Ok(())
}

/// Get a specific configuration section
pub async fn get(section: String, format: String) -> Result<()> {
    // Load environment paths
    let env_paths = EnvPaths::load()?;

    // Load all configurations using ConfigurationLoader
    let configs =
        schema_manager::configuration::ConfigurationLoader::load_configurations_from_directory(
            &env_paths.configuration_path,
        )
        .await
        .map_err(|e| anyhow!("Failed to load configurations: {}", e))?;

    // Convert to a HashMap for navigation
    let mut config_map = HashMap::new();
    for config in configs.iter() {
        config_map.insert(
            config.id().to_string(),
            serde_yaml::to_value(config.get_all_values())?,
        );
    }

    // Parse the section path (e.g., "system.database.path")
    let parts: Vec<&str> = section.split('.').collect();

    // Navigate through the configuration structure
    let value = navigate_config_path(&config_map, &parts)?;

    match format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&value)?;
            println!("{}", json_output);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&value)?;
            println!("{}", yaml_output);
        }
        "text" => {
            print_config_value(&section, &value);
        }
        _ => {
            print_config_value(&section, &value);
        }
    }

    Ok(())
}

/// Navigate through the configuration structure to find a specific value
fn navigate_config_path(
    configs: &HashMap<String, serde_yaml::Value>,
    path: &[&str],
) -> Result<serde_yaml::Value> {
    if path.is_empty() {
        return Err(anyhow!("Empty configuration path"));
    }

    // Get the top-level section
    let section = path[0];
    let mut current_value = configs
        .get(section)
        .ok_or_else(|| anyhow!("Configuration section '{}' not found", section))?
        .clone();

    // Navigate through the rest of the path
    for (i, &key) in path.iter().enumerate().skip(1) {
        match current_value {
            serde_yaml::Value::Mapping(ref map) => {
                current_value = map
                    .get(serde_yaml::Value::String(key.to_string()))
                    .ok_or_else(|| {
                        let partial_path = path[..=i].join(".");
                        anyhow!("Configuration key '{}' not found", partial_path)
                    })?
                    .clone();
            }
            _ => {
                let partial_path = path[..i].join(".");
                return Err(anyhow!(
                    "Cannot navigate further from '{}': not a mapping",
                    partial_path
                ));
            }
        }
    }

    Ok(current_value)
}

/// Print configurations in a formatted text output
fn print_configs_text(configs: &HashMap<String, HashMap<String, serde_yaml::Value>>) {
    println!("{}", "=== Marain Configuration ===".bold());
    println!();

    if configs.is_empty() {
        println!("{}", "No configurations loaded".yellow());
        return;
    }

    for (name, values) in configs {
        println!("{}", format!("[{}]", name).cyan().bold());
        for (key, value) in values {
            print!("  {}: ", key.cyan());
            print_yaml_value(value, 1);
        }
        println!();
    }

    println!("{}", format!("Total sections: {}", configs.len()).green());
}

/// Print a specific configuration value
fn print_config_value(path: &str, value: &serde_yaml::Value) {
    println!("{}", "=== Configuration Value ===".bold());
    println!();
    println!("{}: {}", "Path".bold(), path.cyan());
    println!("{}: {}", "Type".bold(), value_type_name(value).yellow());
    println!();
    println!("{}:", "Value".bold());
    print_yaml_value(value, 0);
}

/// Get a human-readable name for a YAML value type
fn value_type_name(value: &serde_yaml::Value) -> &str {
    match value {
        serde_yaml::Value::Null => "null",
        serde_yaml::Value::Bool(_) => "boolean",
        serde_yaml::Value::Number(_) => "number",
        serde_yaml::Value::String(_) => "string",
        serde_yaml::Value::Sequence(_) => "array",
        serde_yaml::Value::Mapping(_) => "object",
        serde_yaml::Value::Tagged(_) => "tagged",
    }
}

/// Recursively print a YAML value with indentation
fn print_yaml_value(value: &serde_yaml::Value, indent_level: usize) {
    let indent = "  ".repeat(indent_level);

    match value {
        serde_yaml::Value::Null => {
            println!("{}null", indent);
        }
        serde_yaml::Value::Bool(b) => {
            println!("{}{}", indent, b.to_string().blue());
        }
        serde_yaml::Value::Number(n) => {
            println!("{}{}", indent, n.to_string().magenta());
        }
        serde_yaml::Value::String(s) => {
            // Check if it looks like a path or URL
            if s.contains('/') || s.contains('\\') {
                println!("{}{}", indent, s.green());
            } else {
                println!("{}{}", indent, s.yellow());
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                println!("{}- ", indent);
                print_yaml_value(item, indent_level + 1);
            }
        }
        serde_yaml::Value::Mapping(map) => {
            for (key, val) in map {
                if let serde_yaml::Value::String(key_str) = key {
                    print!("{}{}: ", indent, key_str.cyan());

                    // Print simple values on the same line
                    match val {
                        serde_yaml::Value::Null
                        | serde_yaml::Value::Bool(_)
                        | serde_yaml::Value::Number(_)
                        | serde_yaml::Value::String(_) => {
                            print_yaml_value(val, 0);
                        }
                        _ => {
                            println!();
                            print_yaml_value(val, indent_level + 1);
                        }
                    }
                } else {
                    println!("{}{:?}:", indent, key);
                    print_yaml_value(val, indent_level + 1);
                }
            }
        }
        serde_yaml::Value::Tagged(tagged) => {
            println!("{}!{} ", indent, tagged.tag);
            print_yaml_value(&tagged.value, indent_level + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigate_config_path() {
        let mut configs = HashMap::new();
        configs.insert(
            "system".to_string(),
            serde_yaml::from_str(
                r#"
                database:
                    path: "/data/test.db"
                    max_connections: 10
                api:
                    port: 3030
            "#,
            )
            .unwrap(),
        );

        // Test valid paths
        let result = navigate_config_path(&configs, &["system", "database", "path"]).unwrap();
        assert_eq!(
            result,
            serde_yaml::Value::String("/data/test.db".to_string())
        );

        let result = navigate_config_path(&configs, &["system", "api", "port"]).unwrap();
        assert_eq!(result, serde_yaml::Value::Number(3030.into()));

        // Test invalid paths
        assert!(navigate_config_path(&configs, &["invalid"]).is_err());
        assert!(navigate_config_path(&configs, &["system", "invalid"]).is_err());
        assert!(navigate_config_path(&configs, &[]).is_err());
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(value_type_name(&serde_yaml::Value::Null), "null");
        assert_eq!(value_type_name(&serde_yaml::Value::Bool(true)), "boolean");
        assert_eq!(
            value_type_name(&serde_yaml::Value::Number(42.into())),
            "number"
        );
        assert_eq!(
            value_type_name(&serde_yaml::Value::String("test".to_string())),
            "string"
        );
        assert_eq!(
            value_type_name(&serde_yaml::Value::Sequence(vec![])),
            "array"
        );
        assert_eq!(
            value_type_name(&serde_yaml::Value::Mapping(serde_yaml::Mapping::new())),
            "object"
        );
    }
}
