//! End-to-end smoke test for the v0.1 done line (PRD §7).
//!
//! Verifies the full library pipeline (lex → parse → emit → shim → cargo)
//! produces a runnable executable whose stdout matches the PRD §7 done line.
//! This is the smallest possible proof that R6's emit + shim produce valid
//! Rust that cargo + rustc actually accept.
//!
//! R8 (testing harness) will own systematic e2e coverage; this single test
//! exists as a smoke test from R6 forward so any regression in emit or shim
//! shape that would break the hello-world fails fast.
//!
//! Requires `cargo` on PATH (always true in any environment running our
//! own `cargo test`). Cold-cache invocation typically completes in 1–3
//! seconds; warm-cache subsequent runs reuse the shim's `target/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use marain_core::emit::emit;
use marain_core::lexer::lex;
use marain_core::parser::parse;
use marain_core::shim::write_shim;
use marain_core::source::SourceMap;

/// Workspace-root scratch dir; gitignored. Resolved from the crate's
/// `CARGO_MANIFEST_DIR` so it's stable regardless of where tests are run from.
fn project_scratch_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has at least 2 ancestors")
        .join(".scratch")
}

/// RAII tempdir guard. Mirrors the helper used by shim.rs's own tests; copied
/// here rather than re-exported because tempdir scaffolding is a test-only
/// concern that does not belong on the public crate surface.
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
        let dir = project_scratch_dir().join(format!("e2e-{label}-{pid}-{nanos}-{n}"));
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
fn hello_world_done_line_runs_end_to_end() {
    // 1. Compile the Marain source.
    let src = "dic \"salve, munde\".\n";
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("hello.lat"), src.to_string());
    let tokens = lex(map.get(id)).expect("lex");
    let module = parse(&tokens).expect("parse");
    let rust_main = emit(&module).expect("emit");

    // Sanity check on the emitted shape before we hand it to cargo.
    assert!(
        rust_main.contains("println!(\"{}\", \"salve, munde\");"),
        "unexpected emitted Rust:\n{rust_main}",
    );

    // 2. Write the shim to a fresh tempdir.
    let tmp = TempDir::new("hello");
    let shim_dir = tmp.path().join("hello");
    write_shim(&shim_dir, "hello", &rust_main).expect("write_shim");

    assert!(shim_dir.join("Cargo.toml").exists());
    assert!(shim_dir.join("src/main.rs").exists());

    // 3. Invoke cargo run on the shim and capture stdout.
    //
    // CARGO_TARGET_DIR is unset for the spawned cargo so it uses the shim's
    // own target/, not the parent test runner's target dir (which would race
    // with the test's own build of marain-core).
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--manifest-path"])
        .arg(shim_dir.join("Cargo.toml"))
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("cargo invocation failed (is cargo on PATH?)");

    // 4. Assert success + expected stdout.
    assert!(
        output.status.success(),
        "cargo run failed (exit {:?}):\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert_eq!(stdout.trim_end(), "salve, munde");
}
