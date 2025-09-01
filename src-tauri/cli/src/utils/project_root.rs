use anyhow::{anyhow, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Find the Marain project root directory by looking for marker files
pub fn find_project_root() -> Result<PathBuf> {
    // Start from the current directory
    let current_dir = env::current_dir()?;

    // Check if we're already in the project root
    if is_project_root(&current_dir) {
        return Ok(current_dir);
    }

    // Try to find the project root by traversing up
    let mut path = current_dir.as_path();

    while let Some(parent) = path.parent() {
        if is_project_root(parent) {
            return Ok(parent.to_path_buf());
        }
        path = parent;
    }

    Err(anyhow!(
        "Not in a Marain project directory. Please run marc from the project root.\n\
         The project root should contain 'src-tauri/', 'schemas/', and 'config/' directories."
    ))
}

/// Check if a directory is the Marain project root
fn is_project_root(path: &Path) -> bool {
    // Check for required directories that indicate this is the project root
    let required_dirs = ["src-tauri", "schemas", "config"];
    let required_files = ["package.json", "Cargo.toml"];

    // Check if all required directories exist
    let has_dirs = required_dirs.iter().all(|dir| path.join(dir).is_dir());

    // Check if at least one required file exists
    let has_files = required_files.iter().any(|file| path.join(file).is_file());

    // Additional check: verify src-tauri contains Cargo.toml
    let has_src_tauri_cargo = path.join("src-tauri").join("Cargo.toml").is_file();

    has_dirs && has_files && has_src_tauri_cargo
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_project_root_valid() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create required directories
        fs::create_dir(root.join("src-tauri")).unwrap();
        fs::create_dir(root.join("schemas")).unwrap();
        fs::create_dir(root.join("config")).unwrap();

        // Create required files
        fs::write(root.join("package.json"), "{}").unwrap();
        fs::write(root.join("src-tauri").join("Cargo.toml"), "[package]").unwrap();

        assert!(is_project_root(root));
    }

    #[test]
    fn test_is_project_root_invalid_missing_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Only create some directories
        fs::create_dir(root.join("src-tauri")).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();

        assert!(!is_project_root(root));
    }

    #[test]
    fn test_is_project_root_invalid_missing_cargo() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create directories but no Cargo.toml in src-tauri
        fs::create_dir(root.join("src-tauri")).unwrap();
        fs::create_dir(root.join("schemas")).unwrap();
        fs::create_dir(root.join("config")).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();

        assert!(!is_project_root(root));
    }
}
