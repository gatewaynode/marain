//! XDG-spec path resolution and the FNV-1a 8-hex hash used to derive each
//! shim project's identity.
//!
//! Hand-rolled per PRD §9 self-supporting: no `dirs` crate, no `xdg` crate,
//! no [`std::hash::DefaultHasher`] (process-stable but Rust-version-fragile,
//! so its output cannot be persisted to disk and reproduced).
//!
//! All env-var-reading functions delegate to a pure
//! [`xdg_state_home_from`] helper so tests can exercise resolution logic
//! without touching the process environment.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const FNV_OFFSET_BASIS_32: u32 = 0x811c_9dc5;
const FNV_PRIME_32: u32 = 0x0100_0193;

/// 8-char hex digest of `bytes` via FNV-1a 32-bit.
///
/// Stable across processes and Rust versions. Used to disambiguate shim
/// directories for two source files with the same basename in different
/// directories; collision probability at the number of shims a single user
/// accumulates is negligible.
pub fn fnv1a_8hex(bytes: &[u8]) -> String {
    let mut hash = FNV_OFFSET_BASIS_32;
    for &b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME_32);
    }
    format!("{hash:08x}")
}

/// XDG state home per the XDG Base Directory Specification.
///
/// Reads `$XDG_STATE_HOME` and `$HOME` from the process environment. See
/// [`xdg_state_home_from`] for the pure resolution logic.
pub fn xdg_state_home() -> PathBuf {
    let state = std::env::var_os("XDG_STATE_HOME");
    let home = std::env::var_os("HOME");
    xdg_state_home_from(state.as_deref(), home.as_deref())
}

/// Pure XDG state-home resolution. Returns `state_var` if set to an absolute
/// path, else `<home_var>/.local/state`, else `.` as a last-resort fallback.
///
/// The XDG spec explicitly requires absolute paths; non-absolute values
/// of `$XDG_STATE_HOME` must be ignored (and we do).
pub fn xdg_state_home_from(state_var: Option<&OsStr>, home_var: Option<&OsStr>) -> PathBuf {
    if let Some(s) = state_var {
        let path = PathBuf::from(s);
        if path.is_absolute() {
            return path;
        }
    }
    if let Some(h) = home_var {
        return PathBuf::from(h).join(".local").join("state");
    }
    PathBuf::from(".")
}

/// Root directory under which all per-source shim projects live.
pub fn marain_builds_dir() -> PathBuf {
    xdg_state_home().join("marain").join("builds")
}

/// Compute the shim directory for `source` per ARCHITECTURE.md §3.2:
/// `<XDG_STATE_HOME>/marain/builds/<basename>-<8hex-hash>`. The hash is
/// over the source's canonical absolute path so `hello.lat` in two
/// different directories never collide.
///
/// Requires `source` to exist on disk (uses `canonicalize`).
pub fn shim_dir_for(source: &Path) -> std::io::Result<PathBuf> {
    let canonical = source.canonicalize()?;
    let basename = stem_or_default(&canonical);
    let hash = fnv1a_8hex(canonical.to_string_lossy().as_bytes());
    Ok(marain_builds_dir().join(format!("{basename}-{hash}")))
}

/// Cargo-project name for the shim, derived from the source basename
/// (`hello.lat` → `"hello"`). Falls back to `"main"` for pathological inputs.
pub fn shim_name_for(source: &Path) -> String {
    stem_or_default(source)
}

fn stem_or_default(p: &Path) -> String {
    p.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn project_scratch_dir() -> PathBuf {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .and_then(|p| p.parent())
            .expect("CARGO_MANIFEST_DIR has at least 2 ancestors")
            .join(".scratch")
    }

    /// RAII guard for a unique scratch subdir containing one source file.
    struct TempSource {
        dir: PathBuf,
        path: PathBuf,
    }

    impl TempSource {
        fn new(label: &str, filename: &str, contents: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let pid = std::process::id();
            let dir = project_scratch_dir().join(format!("paths-{label}-{pid}-{nanos}-{n}"));
            fs::create_dir_all(&dir).expect("scratch dir create");
            let path = dir.join(filename);
            fs::write(&path, contents).expect("write source");
            Self { dir, path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempSource {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    // ---- FNV-1a known vectors ----

    #[test]
    fn fnv1a_empty_is_offset_basis() {
        assert_eq!(fnv1a_8hex(b""), format!("{FNV_OFFSET_BASIS_32:08x}"));
    }

    #[test]
    fn fnv1a_known_vector_a() {
        // From the FNV reference test vectors.
        assert_eq!(fnv1a_8hex(b"a"), "e40c292c");
    }

    #[test]
    fn fnv1a_known_vector_foobar() {
        // From the FNV reference test vectors.
        assert_eq!(fnv1a_8hex(b"foobar"), "bf9cf968");
    }

    #[test]
    fn fnv1a_output_shape_is_eight_lowercase_hex() {
        for input in ["", "x", "longer", "/abs/path/hello.lat"] {
            let h = fnv1a_8hex(input.as_bytes());
            assert_eq!(h.len(), 8, "length for {input:?}");
            assert!(
                h.chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "hex shape for {input:?}: {h}"
            );
        }
    }

    #[test]
    fn fnv1a_distinguishes_inputs() {
        assert_ne!(fnv1a_8hex(b"a"), fnv1a_8hex(b"b"));
        assert_ne!(fnv1a_8hex(b"/a/hello.lat"), fnv1a_8hex(b"/b/hello.lat"));
    }

    // ---- XDG resolution (pure; no env-var mutation in tests) ----

    #[test]
    fn xdg_uses_absolute_state_var() {
        let s = OsString::from("/var/state/x");
        let h = OsString::from("/home/u");
        let p = xdg_state_home_from(Some(&s), Some(&h));
        assert_eq!(p, PathBuf::from("/var/state/x"));
    }

    #[test]
    fn xdg_ignores_relative_state_var_falls_back_to_home() {
        let s = OsString::from("relative/path");
        let h = OsString::from("/home/u");
        let p = xdg_state_home_from(Some(&s), Some(&h));
        assert_eq!(p, PathBuf::from("/home/u/.local/state"));
    }

    #[test]
    fn xdg_uses_home_when_state_absent() {
        let h = OsString::from("/home/u");
        let p = xdg_state_home_from(None, Some(&h));
        assert_eq!(p, PathBuf::from("/home/u/.local/state"));
    }

    #[test]
    fn xdg_fallback_to_dot_when_no_env() {
        let p = xdg_state_home_from(None, None);
        assert_eq!(p, PathBuf::from("."));
    }

    // ---- shim dir composition ----

    #[test]
    fn shim_dir_for_uses_basename_and_eight_hex_suffix() {
        let f = TempSource::new("compose", "hello.lat", "dic \"hi\".");
        let d = shim_dir_for(f.path()).expect("shim_dir_for");

        // Last path component is `<basename>-<8 hex>`.
        let name = d.file_name().and_then(|s| s.to_str()).expect("basename");
        let (basename, hash) = name.rsplit_once('-').expect("contains hyphen");
        assert_eq!(basename, "hello");
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn shim_dir_for_stable_across_calls_for_same_source() {
        let f = TempSource::new("stable", "hello.lat", "dic \"hi\".");
        let d1 = shim_dir_for(f.path()).expect("ok");
        let d2 = shim_dir_for(f.path()).expect("ok");
        assert_eq!(d1, d2);
    }

    #[test]
    fn shim_dir_for_differs_for_different_paths() {
        let a = TempSource::new("diff_a", "hello.lat", "dic \"hi\".");
        let b = TempSource::new("diff_b", "hello.lat", "dic \"hi\".");
        let da = shim_dir_for(a.path()).expect("ok a");
        let db = shim_dir_for(b.path()).expect("ok b");
        // Same basename, different hash → different dir.
        assert_ne!(da, db);
        assert_eq!(
            da.file_name().and_then(|s| s.to_str()).unwrap()[..6].to_string(),
            "hello-".to_string()
        );
        assert_eq!(
            db.file_name().and_then(|s| s.to_str()).unwrap()[..6].to_string(),
            "hello-".to_string()
        );
    }

    #[test]
    fn shim_dir_for_propagates_io_error_for_missing_file() {
        let r = shim_dir_for(Path::new("/this/does/not/exist/anywhere/hello.lat"));
        assert!(r.is_err());
    }

    // ---- shim_name_for ----

    #[test]
    fn shim_name_strips_lat_extension() {
        assert_eq!(shim_name_for(Path::new("hello.lat")), "hello");
        assert_eq!(shim_name_for(Path::new("dir/world.lat")), "world");
    }

    #[test]
    fn shim_name_handles_no_extension() {
        assert_eq!(shim_name_for(Path::new("naked")), "naked");
    }

    #[test]
    fn shim_name_handles_dotted() {
        // `file_stem` strips only the last extension.
        assert_eq!(shim_name_for(Path::new("a.b.lat")), "a.b");
    }
}
