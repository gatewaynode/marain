use crate::utils::env_paths::{get_environment, EnvPaths};
use anyhow::Result;
use colored::*;
use serde_json::json;

/// Execute the health check command
pub async fn execute(format: String) -> Result<()> {
    let health_status = check_system_health().await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&health_status)?);
        }
        "text" => {
            print_health_status_text(&health_status);
        }
        _ => {
            print_health_status_text(&health_status);
        }
    }

    Ok(())
}

/// Check the health of various system components
async fn check_system_health() -> Result<serde_json::Value> {
    let mut status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {}
    });

    // Check database
    let db_status = check_database_health().await;
    status["components"]["database"] = db_status;

    // Check configuration
    let config_status = check_configuration_health().await;
    status["components"]["configuration"] = config_status;

    // Check schemas
    let schema_status = check_schemas_health().await;
    status["components"]["schemas"] = schema_status;

    // Check API readiness
    let api_status = check_api_health().await;
    status["components"]["api"] = api_status;

    // Determine overall status
    let components = status["components"].as_object().unwrap();
    let all_healthy = components
        .values()
        .all(|v| v["status"].as_str().unwrap_or("unknown") == "healthy");

    if !all_healthy {
        status["status"] = json!("degraded");
    }

    Ok(status)
}

/// Check database health
async fn check_database_health() -> serde_json::Value {
    let env_paths = match EnvPaths::load() {
        Ok(paths) => paths,
        Err(e) => {
            return json!({
                "status": "unhealthy",
                "message": format!("Failed to load environment paths: {}", e)
            });
        }
    };

    let db_path = env_paths.database_path();

    if db_path.exists() {
        // Try to initialize database to verify it's accessible
        let config = database::init::DatabaseConfig::new_with_path(db_path.clone())
            .with_create_tables(false);

        match database::init::initialize_database(config).await {
            Ok(_) => json!({
                "status": "healthy",
                "message": "Database file exists and is accessible",
                "path": db_path.display().to_string()
            }),
            Err(e) => json!({
                "status": "unhealthy",
                "message": format!("Database exists but cannot be accessed: {}", e),
                "path": db_path.display().to_string()
            }),
        }
    } else {
        json!({
            "status": "not_initialized",
            "message": "Database file does not exist yet",
            "path": db_path.display().to_string()
        })
    }
}

/// Check configuration health
async fn check_configuration_health() -> serde_json::Value {
    let env_paths = match EnvPaths::load() {
        Ok(paths) => paths,
        Err(e) => {
            return json!({
                "status": "unhealthy",
                "message": format!("Failed to load environment paths: {}", e)
            });
        }
    };

    let config_dir = &env_paths.configuration_path;

    if !config_dir.exists() {
        return json!({
            "status": "unhealthy",
            "message": format!("Configuration directory not found at: {}", config_dir.display())
        });
    }

    // Get current environment to determine which system config to check
    let env = get_environment();

    // Check for required configuration files based on environment
    let required_configs = vec![
        format!("config.system.{}.yaml", env),
        "config.api.yaml".to_string(),
        "config.content.yaml".to_string(),
    ];

    let mut missing_configs = Vec::new();
    let mut found_configs = Vec::new();

    for config in &required_configs {
        let config_path = config_dir.join(config);
        if config_path.exists() {
            found_configs.push(config.to_string());
        } else {
            missing_configs.push(config.to_string());
        }
    }

    if missing_configs.is_empty() {
        // Try to load configurations using ConfigurationLoader
        match schema_manager::configuration::ConfigurationLoader::load_configurations_from_directory(config_dir).await {
            Ok(configs) => {
                let config_ids: Vec<String> = configs.iter().map(|c| c.id().to_string()).collect();
                json!({
                    "status": "healthy",
                    "message": "All configuration files present and valid",
                    "found": found_configs,
                    "loaded_sections": config_ids
                })
            }
            Err(e) => {
                json!({
                    "status": "unhealthy",
                    "message": format!("Configuration files present but invalid: {}", e),
                    "found": found_configs
                })
            }
        }
    } else {
        json!({
            "status": "unhealthy",
            "message": "Missing required configuration files",
            "found": found_configs,
            "missing": missing_configs
        })
    }
}

/// Check schemas health
async fn check_schemas_health() -> serde_json::Value {
    let env_paths = match EnvPaths::load() {
        Ok(paths) => paths,
        Err(e) => {
            return json!({
                "status": "unhealthy",
                "message": format!("Failed to load environment paths: {}", e)
            });
        }
    };

    let schemas_dir = &env_paths.entity_schema_path;

    if !schemas_dir.exists() {
        return json!({
            "status": "unhealthy",
            "message": format!("Schemas directory not found at: {}", schemas_dir.display())
        });
    }

    // Count schema files
    let schema_files: Vec<_> = std::fs::read_dir(schemas_dir)
        .unwrap_or_else(|_| panic!("Failed to read schemas directory"))
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "yaml" {
                    Some(e.file_name().to_string_lossy().to_string())
                } else {
                    None
                }
            })
        })
        .collect();

    if schema_files.is_empty() {
        json!({
            "status": "warning",
            "message": "No schema files found",
            "count": 0
        })
    } else {
        json!({
            "status": "healthy",
            "message": format!("Found {} schema file(s)", schema_files.len()),
            "count": schema_files.len(),
            "schemas": schema_files
        })
    }
}

/// Check API health
async fn check_api_health() -> serde_json::Value {
    // TODO: Get API host and port from configuration dynamically
    // For now, using default values but this should be loaded from config
    let api_url = "http://localhost:3030/api/v1/health";

    // Check if API server can be reached (if running)
    match reqwest::get(api_url).await {
        Ok(response) => {
            if response.status().is_success() {
                json!({
                    "status": "healthy",
                    "message": "API server is running and responsive",
                    "endpoint": api_url.replace("/api/v1/health", "")
                })
            } else {
                json!({
                    "status": "unhealthy",
                    "message": format!("API server returned status: {}", response.status()),
                    "endpoint": api_url.replace("/api/v1/health", "")
                })
            }
        }
        Err(_) => {
            json!({
                "status": "offline",
                "message": "API server is not running or not reachable",
                "endpoint": api_url.replace("/api/v1/health", "")
            })
        }
    }
}

/// Print health status in a formatted text output
fn print_health_status_text(status: &serde_json::Value) {
    println!("{}", "=== Marain System Health Check ===".bold());
    println!();

    let overall_status = status["status"].as_str().unwrap_or("unknown");
    let status_display = match overall_status {
        "healthy" => "HEALTHY".green().bold(),
        "degraded" => "DEGRADED".yellow().bold(),
        "unhealthy" => "UNHEALTHY".red().bold(),
        _ => "UNKNOWN".white().bold(),
    };

    println!("Overall Status: {}", status_display);
    println!("Timestamp: {}", status["timestamp"].as_str().unwrap_or(""));
    println!();

    println!("{}", "Components:".bold());
    println!("{}", "─".repeat(50));

    if let Some(components) = status["components"].as_object() {
        for (name, component) in components {
            let comp_status = component["status"].as_str().unwrap_or("unknown");
            let status_icon = match comp_status {
                "healthy" => "✓".green(),
                "unhealthy" => "✗".red(),
                "warning" => "⚠".yellow(),
                "offline" | "not_initialized" => "○".white(),
                _ => "?".white(),
            };

            let status_text = match comp_status {
                "healthy" => comp_status.green(),
                "unhealthy" => comp_status.red(),
                "warning" => comp_status.yellow(),
                _ => comp_status.white(),
            };

            println!(
                "{} {} ({})",
                status_icon,
                name.to_uppercase().bold(),
                status_text
            );

            if let Some(message) = component["message"].as_str() {
                println!("  {}", message);
            }

            // Print additional details for some components
            match name.as_str() {
                "configuration" => {
                    if let Some(sections) = component["loaded_sections"].as_array() {
                        if !sections.is_empty() {
                            println!(
                                "  Loaded sections: {}",
                                sections
                                    .iter()
                                    .filter_map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                    }
                }
                "schemas" => {
                    if let Some(count) = component["count"].as_u64() {
                        if count > 0 {
                            if let Some(schemas) = component["schemas"].as_array() {
                                println!(
                                    "  Schemas: {}",
                                    schemas
                                        .iter()
                                        .filter_map(|s| s.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                );
                            }
                        }
                    }
                }
                _ => {}
            }

            println!();
        }
    }
}

// Add reqwest to dependencies for API health check
use reqwest;
