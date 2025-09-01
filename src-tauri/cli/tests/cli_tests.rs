use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Helper function to create a mock project structure with proper configuration
fn create_mock_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create required directories
    fs::create_dir(root.join("src-tauri")).unwrap();
    fs::create_dir(root.join("schemas")).unwrap();
    fs::create_dir(root.join("config")).unwrap();
    fs::create_dir(root.join("data")).unwrap();
    fs::create_dir(root.join("static")).unwrap();

    // Create required files for project root detection
    fs::write(root.join("package.json"), "{}").unwrap();
    fs::write(root.join("src-tauri").join("Cargo.toml"), "[package]").unwrap();

    // Don't create .env file - let the CLI use defaults relative to project root
    // This avoids path resolution issues

    // Create sample config files with proper structure
    fs::write(
        root.join("config").join("config.system.dev.yaml"),
        r#"id: system
name: System Configuration
description: Test system configuration
provider: core
version: 1.0.0
values:
  database:
    path: "data/marain.db"
    max_connections: 10
  api:
    host: "127.0.0.1"
    port: 3030
"#,
    )
    .unwrap();

    fs::write(
        root.join("config").join("config.api.yaml"),
        r#"id: api
name: API Configuration
description: Test API configuration
provider: api
version: 1.0.0
values:
  cors:
    allowed_origins:
      - "http://localhost:3000"
      - "http://localhost:1420"
"#,
    )
    .unwrap();

    fs::write(
        root.join("config").join("config.content.yaml"),
        r#"id: content
name: Content Configuration
description: Test content configuration
provider: content
version: 1.0.0
values:
  cache:
    ttl: 3600
    max_size: 1000
"#,
    )
    .unwrap();

    // Create a sample schema
    fs::write(
        root.join("schemas").join("test.schema.yaml"),
        r#"name: test
fields:
  - name: title
    type: text
    required: true
"#,
    )
    .unwrap();

    temp_dir
}

/// Helper to set up environment for a test
fn setup_test_env() {
    // Clear any environment variables that might interfere
    env::remove_var("DATA_PATH");
    env::remove_var("STATIC_PATH");
    env::remove_var("ENTITY_SCHEMA_PATH");
    env::remove_var("CONFIGURATION_PATH");
    env::remove_var("ENVIRONMENT");
}

#[test]
fn test_cli_help() {
    setup_test_env();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Marain CLI"))
        .stdout(predicate::str::contains("Command line interface"));
}

#[test]
fn test_cli_version() {
    setup_test_env();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("marc"));
}

#[test]
fn test_not_in_project_root() {
    setup_test_env();
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("health")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Not in a Marain project directory",
        ))
        .stderr(predicate::str::contains("project root"));
}

#[test]
fn test_health_command_text() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("Marain System Health Check"))
        .stdout(predicate::str::contains("Overall Status"));
}

#[test]
fn test_health_command_json() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["health", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\""))
        .stdout(predicate::str::contains("\"components\""));
}

#[test]
fn test_config_list_text() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Marain Configuration"))
        .stdout(predicate::str::contains("Total sections"));
}

#[test]
fn test_config_list_json() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["config", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("}"));
}

#[test]
fn test_config_list_yaml() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["config", "list", "--format", "yaml"])
        .assert()
        .success();
}

#[test]
fn test_config_get_valid_path() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["config", "get", "system.database.path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration Value"))
        .stdout(predicate::str::contains("data/marain.db"));
}

#[test]
fn test_config_get_invalid_path() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["config", "get", "invalid.path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_config_get_json_format() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["config", "get", "system.api.port", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("3030"));
}

#[test]
fn test_verbose_flag() {
    setup_test_env();
    let project_dir = create_mock_project();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(project_dir.path())
        .args(&["--verbose", "health"])
        .assert()
        .success();
}

#[test]
fn test_subcommand_help() {
    setup_test_env();

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.args(&["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration management"));

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.args(&["health", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check system health"));
}

// Additional test to verify the CLI works from subdirectories
#[test]
fn test_cli_from_subdirectory() {
    setup_test_env();
    let project_dir = create_mock_project();

    // Create a subdirectory
    let subdir = project_dir.path().join("src-tauri").join("app");
    fs::create_dir_all(&subdir).unwrap();

    // The CLI should find the project root even when run from a subdirectory
    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(subdir)
        .arg("health")
        .assert()
        .success()
        .stdout(predicate::str::contains("Marain System Health Check"));
}

// Test that environment variables are respected when set
#[test]
fn test_custom_env_paths() {
    let project_dir = create_mock_project();
    let root = project_dir.path();

    // Create custom directories
    let custom_data = root.join("custom_data");
    let custom_config = root.join("custom_config");
    let custom_schemas = root.join("custom_schemas");

    fs::create_dir(&custom_data).unwrap();
    fs::create_dir(&custom_config).unwrap();
    fs::create_dir(&custom_schemas).unwrap();

    // Copy config files to custom location
    fs::write(
        custom_config.join("config.system.dev.yaml"),
        r#"id: system
name: System Configuration
description: Test system configuration
provider: core
version: 1.0.0
values:
  test: "custom"
"#,
    )
    .unwrap();

    // Set custom environment variables
    env::set_var("DATA_PATH", custom_data.to_str().unwrap());
    env::set_var("CONFIGURATION_PATH", custom_config.to_str().unwrap());
    env::set_var("ENTITY_SCHEMA_PATH", custom_schemas.to_str().unwrap());

    let mut cmd = Command::cargo_bin("marc").unwrap();
    cmd.current_dir(root)
        .args(&["config", "get", "system.test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("custom"));

    // Clean up
    env::remove_var("DATA_PATH");
    env::remove_var("CONFIGURATION_PATH");
    env::remove_var("ENTITY_SCHEMA_PATH");
}
