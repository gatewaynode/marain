//! Cargo shim generator.
//!
//! Given a target directory and an emitted Rust `main.rs` body, [`write_shim`]
//! produces a self-contained cargo project at the target path:
//!
//! ```text
//! <target>/
//!   Cargo.toml
//!   src/main.rs
//! ```
//!
//! Writes are atomic in the sense that a successful return guarantees both
//! files are in place; a failure leaves either the old shim intact or no
//! shim at all (per [`ARCHITECTURE.md`] §3.1). Atomicity is achieved by
//! staging into `<parent>/.staging-<basename>` and then `fs::rename` over
//! the target.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Render a minimal `Cargo.toml` for a generated shim.
///
/// Single `[package]` table, edition 2024, plus an *empty* `[workspace]`
/// table. The empty workspace table opts the shim out of any enclosing
/// cargo workspace it may sit inside (e.g. the project-local test scratch
/// dir under `.scratch/`, or any user workspace if the shim ever co-locates
/// with source). Without it, cargo would walk up from the shim's manifest,
/// find an outer `[workspace]` Cargo.toml, and reject the shim as a
/// non-member.
///
/// No `[[bin]]` table — cargo auto-discovers `src/main.rs`.
pub fn render_cargo_toml(name: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n"
    )
}

/// Atomically write a shim at `target_dir`.
///
/// Replaces any existing shim at the target path. If `target_dir.parent()`
/// does not yet exist, it is created.
pub fn write_shim(target_dir: &Path, name: &str, rust_main: &str) -> Result<(), ShimError> {
    let staging = staging_path_for(target_dir);

    // Discard any leftover staging from a prior crashed invocation.
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| ShimError::RemoveDir {
            path: staging.clone(),
            source: e,
        })?;
    }

    // Ensure the target's parent exists (creates the XDG builds/ dir on
    // first ever invocation).
    if let Some(parent) = target_dir.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| ShimError::CreateDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    write_into_staging(&staging, name, rust_main)?;

    // Replace existing target if present, then move staging into place.
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).map_err(|e| ShimError::RemoveDir {
            path: target_dir.to_path_buf(),
            source: e,
        })?;
    }
    fs::rename(&staging, target_dir).map_err(|e| ShimError::Rename {
        from: staging,
        to: target_dir.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

fn write_into_staging(staging: &Path, name: &str, rust_main: &str) -> Result<(), ShimError> {
    let staging_src = staging.join("src");
    fs::create_dir_all(&staging_src).map_err(|e| ShimError::CreateDir {
        path: staging_src.clone(),
        source: e,
    })?;

    let cargo_path = staging.join("Cargo.toml");
    fs::write(&cargo_path, render_cargo_toml(name).as_bytes()).map_err(|e| {
        ShimError::WriteFile {
            path: cargo_path,
            source: e,
        }
    })?;

    let main_path = staging_src.join("main.rs");
    fs::write(&main_path, rust_main.as_bytes()).map_err(|e| ShimError::WriteFile {
        path: main_path,
        source: e,
    })?;

    Ok(())
}

fn staging_path_for(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let basename = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("shim");
    parent.join(format!(".staging-{basename}"))
}

#[derive(Debug)]
pub enum ShimError {
    CreateDir {
        path: PathBuf,
        source: io::Error,
    },
    WriteFile {
        path: PathBuf,
        source: io::Error,
    },
    RemoveDir {
        path: PathBuf,
        source: io::Error,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
}

impl ShimError {
    fn inner(&self) -> &io::Error {
        match self {
            Self::CreateDir { source, .. }
            | Self::WriteFile { source, .. }
            | Self::RemoveDir { source, .. }
            | Self::Rename { source, .. } => source,
        }
    }
}

impl fmt::Display for ShimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateDir { path, source } => write!(
                f,
                "failed to create directory `{}`: {source}",
                path.display(),
            ),
            Self::WriteFile { path, source } => {
                write!(f, "failed to write file `{}`: {source}", path.display())
            }
            Self::RemoveDir { path, source } => write!(
                f,
                "failed to remove directory `{}`: {source}",
                path.display(),
            ),
            Self::Rename { from, to, source } => write!(
                f,
                "failed to rename `{}` to `{}`: {source}",
                from.display(),
                to.display(),
            ),
        }
    }
}

impl std::error::Error for ShimError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.inner())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    /// Workspace-root scratch dir; gitignored. Resolved from the crate's
    /// `CARGO_MANIFEST_DIR` so it's stable regardless of where tests are run
    /// from. Surviving debris (after a hard crash) is inspectable and a
    /// `rm -rf .scratch` from project root cleans it up.
    fn project_scratch_dir() -> PathBuf {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .and_then(|p| p.parent())
            .expect("CARGO_MANIFEST_DIR has at least 2 ancestors")
            .join(".scratch")
    }

    /// RAII guard for a unique scratch subdir. Cleans up on drop, so tests
    /// don't leave debris regardless of panic.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let pid = std::process::id();
            let dir = project_scratch_dir().join(format!("shim-test-{label}-{pid}-{nanos}-{n}"));
            fs::create_dir_all(&dir).expect("scratch dir create");
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn render_cargo_toml_contains_package_section() {
        let toml = render_cargo_toml("hello");
        assert!(toml.contains("[package]"));
        assert!(toml.contains("name = \"hello\""));
        assert!(toml.contains("version = \"0.1.0\""));
        assert!(toml.contains("edition = \"2024\""));
    }

    #[test]
    fn render_cargo_toml_has_empty_workspace_table() {
        // The empty `[workspace]` opts the shim out of any enclosing cargo
        // workspace; without it, cargo would treat the shim as a workspace
        // member candidate of whatever workspace it sits inside.
        let toml = render_cargo_toml("hello");
        assert!(toml.contains("[workspace]"));
    }

    #[test]
    fn render_cargo_toml_omits_bin_table() {
        // src/main.rs is auto-discovered; no [[bin]] needed.
        let toml = render_cargo_toml("hello");
        assert!(!toml.contains("[[bin]]"));
    }

    #[test]
    fn write_shim_creates_cargo_toml_and_main_rs() {
        let tmp = TempDir::new("write_creates");
        let target = tmp.path().join("hello-abc");
        write_shim(&target, "hello", "fn main() {}\n").expect("write ok");

        let cargo = fs::read_to_string(target.join("Cargo.toml")).expect("read cargo");
        assert!(cargo.contains("name = \"hello\""));

        let main = fs::read_to_string(target.join("src/main.rs")).expect("read main");
        assert_eq!(main, "fn main() {}\n");
    }

    #[test]
    fn write_shim_creates_src_subdirectory() {
        let tmp = TempDir::new("creates_src");
        let target = tmp.path().join("hello-abc");
        write_shim(&target, "hello", "fn main() {}\n").expect("write ok");
        assert!(target.join("src").is_dir());
    }

    #[test]
    fn write_shim_overwrites_existing_target() {
        let tmp = TempDir::new("overwrite");
        let target = tmp.path().join("hello-abc");
        write_shim(&target, "hello", "fn main() { println!(\"a\"); }\n").expect("first ok");
        write_shim(&target, "hello", "fn main() { println!(\"b\"); }\n").expect("second ok");

        let main = fs::read_to_string(target.join("src/main.rs")).expect("read main");
        assert!(main.contains("println!(\"b\");"));
        assert!(!main.contains("println!(\"a\");"));
    }

    #[test]
    fn write_shim_cleans_up_leftover_staging() {
        let tmp = TempDir::new("staging_cleanup");
        let target = tmp.path().join("hello-abc");
        let staging = staging_path_for(&target);

        // Simulate a crashed prior invocation: leftover staging dir with
        // garbage content.
        fs::create_dir_all(&staging).expect("create staging");
        fs::write(staging.join("garbage.txt"), b"junk").expect("write garbage");

        write_shim(&target, "hello", "fn main() {}\n").expect("write should clean and proceed");

        // After success, staging no longer exists; target has fresh files.
        assert!(!staging.exists(), "staging should be cleaned up");
        assert!(target.join("Cargo.toml").exists());
        assert!(target.join("src/main.rs").exists());
    }

    #[test]
    fn write_shim_creates_missing_parent_dir() {
        let tmp = TempDir::new("missing_parent");
        // Target two levels deep; intermediate dir doesn't exist yet.
        let target = tmp.path().join("builds").join("hello-abc");
        write_shim(&target, "hello", "fn main() {}\n").expect("write ok");
        assert!(target.join("Cargo.toml").exists());
    }

    #[test]
    fn shim_error_display_includes_path() {
        let err = ShimError::WriteFile {
            path: PathBuf::from("/tmp/foo/Cargo.toml"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };
        let msg = err.to_string();
        assert!(msg.contains("/tmp/foo/Cargo.toml"));
        assert!(msg.contains("denied"));
    }

    #[test]
    fn shim_error_source_chains_to_io() {
        use std::error::Error;
        let err = ShimError::WriteFile {
            path: PathBuf::from("/tmp/foo"),
            source: io::Error::other("boom"),
        };
        let source = err.source().expect("source present");
        assert!(source.to_string().contains("boom"));
    }

    #[test]
    fn shim_error_rename_display_mentions_both_paths() {
        let err = ShimError::Rename {
            from: PathBuf::from("/tmp/a"),
            to: PathBuf::from("/tmp/b"),
            source: io::Error::other("nope"),
        };
        let msg = err.to_string();
        assert!(msg.contains("/tmp/a"));
        assert!(msg.contains("/tmp/b"));
    }

    #[test]
    fn staging_path_is_sibling_of_target() {
        let target = PathBuf::from("/tmp/marain/builds/hello-abc");
        let staging = staging_path_for(&target);
        assert_eq!(
            staging,
            PathBuf::from("/tmp/marain/builds/.staging-hello-abc")
        );
    }

    #[test]
    fn staging_path_handles_relative_target_with_no_directory() {
        // `PathBuf::from("hello-abc").parent()` returns `Some("")`, not None;
        // joining onto an empty path yields the bare basename. Either way the
        // path resolves relative to the cwd.
        let target = PathBuf::from("hello-abc");
        let staging = staging_path_for(&target);
        assert_eq!(staging, PathBuf::from(".staging-hello-abc"));
    }
}
