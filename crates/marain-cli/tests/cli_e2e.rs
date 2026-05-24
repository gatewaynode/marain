//! Binary-level end-to-end tests for the `marain` binary.
//!
//! Spawn the actual `marain` binary via `env!("CARGO_BIN_EXE_marain")`
//! (cargo's standard pattern for integration-testing a binary in the
//! same crate). Each test isolates its `$XDG_STATE_HOME` to a scratch
//! subdir so the user's real `~/.local/state` isn't touched and two
//! parallel tests can't race over the same shim directory.
//!
//! The env-isolation happens via `Command::env` (subprocess environment,
//! not our own), so the workspace `unsafe_code = "forbid"` lint is
//! satisfied without any `unsafe` blocks.
//!
//! These tests cover the user-facing contract from PRD §6 + §7:
//! `marain build` writes the shim and prints its path; `marain run`
//! prints the user program's stdout; bad source produces a diagnostic
//! and exits 1; `--help` / `--version` succeed; clap argument errors
//! exit 2.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const BIN: &str = env!("CARGO_BIN_EXE_marain");

fn project_scratch_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has 2+ ancestors")
        .join(".scratch")
}

/// RAII scaffolding: per-test scratch dir holding the source file and an
/// isolated `XDG_STATE_HOME`. Cleans up on drop.
struct TestEnv {
    dir: PathBuf,
}

impl TestEnv {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let dir = project_scratch_dir().join(format!("cli-{label}-{pid}-{nanos}-{n}"));
        fs::create_dir_all(dir.join("state")).expect("create scratch dir");
        Self { dir }
    }

    fn write_source(&self, name: &str, contents: &str) -> PathBuf {
        let p = self.dir.join(name);
        fs::write(&p, contents).expect("write source");
        p
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .env("XDG_STATE_HOME", self.dir.join("state"))
            // Don't let an outer CARGO_TARGET_DIR redirect the shim's
            // `cargo run` (would race with the parent test runner's
            // own target/ if both share a dir).
            .env_remove("CARGO_TARGET_DIR")
            .output()
            .expect("spawn marain")
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn assert_status_success(out: &Output) {
    assert!(
        out.status.success(),
        "expected success, got exit {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[test]
fn build_hello_world_prints_shim_path_to_stdout() {
    let env = TestEnv::new("build_hello");
    let src = env.write_source("hello.lat", "dic \"salve, munde\".\n");
    let out = env.run(&["build", src.to_str().expect("utf-8 path")]);

    assert_status_success(&out);
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    let printed = stdout.trim_end();
    assert!(
        printed.contains("/state/marain/builds/hello-"),
        "expected shim path under isolated XDG state, got: {printed}",
    );
    // Confirm the printed path actually exists and contains the shim.
    let shim_dir = PathBuf::from(printed);
    assert!(shim_dir.join("Cargo.toml").exists());
    assert!(shim_dir.join("src").join("main.rs").exists());
}

#[test]
fn run_hello_world_prints_salve_munde_to_stdout() {
    let env = TestEnv::new("run_hello");
    let src = env.write_source("hello.lat", "dic \"salve, munde\".\n");
    let out = env.run(&["run", src.to_str().expect("utf-8 path")]);

    assert_status_success(&out);
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert_eq!(stdout.trim_end(), "salve, munde");
}

#[test]
fn build_bad_source_prints_diagnostic_and_exits_one() {
    let env = TestEnv::new("build_bad");
    let src = env.write_source("bad.lat", "?\n");
    let out = env.run(&["build", src.to_str().expect("utf-8 path")]);

    assert_eq!(
        out.status.code(),
        Some(1),
        "expected exit 1, got {:?}",
        out.status.code()
    );
    let stderr = String::from_utf8(out.stderr).expect("utf-8 stderr");
    assert!(
        stderr.contains("bad.lat:1:1: error:") && stderr.contains("unexpected character"),
        "unexpected stderr:\n{stderr}",
    );
    // Source-mappable errors go through Diagnostic::render and DO NOT carry
    // the `marain:` prefix (only driver-layer errors do).
    assert!(
        !stderr.contains("marain: error:"),
        "source diagnostic should not be prefixed `marain:`, got:\n{stderr}",
    );
}

#[test]
fn run_with_unescapable_keyword_surfaces_emit_error() {
    let env = TestEnv::new("run_emit_err");
    let src = env.write_source("oops.lat", "sit ^self est 5.\n");
    let out = env.run(&["build", src.to_str().expect("utf-8 path")]);

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).expect("utf-8 stderr");
    assert!(
        stderr.contains("oops.lat:") && stderr.contains("self"),
        "expected emit-error diagnostic mentioning `self`, got:\n{stderr}",
    );
}

#[test]
fn help_succeeds_and_lists_subcommands() {
    let env = TestEnv::new("help");
    let out = env.run(&["--help"]);
    assert_status_success(&out);
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("run"));
}

#[test]
fn version_prints_crate_version() {
    let env = TestEnv::new("version");
    let out = env.run(&["--version"]);
    assert_status_success(&out);
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    assert!(stdout.contains("marain"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_subcommand_exits_two() {
    // Clap's convention is exit code 2 for argument-parsing errors,
    // distinct from our driver-error exit code 1.
    let env = TestEnv::new("bad_sub");
    let out = env.run(&["frobnicate"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn missing_path_arg_exits_two() {
    let env = TestEnv::new("missing_path");
    let out = env.run(&["build"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn no_subcommand_exits_two() {
    let env = TestEnv::new("no_sub");
    let out = env.run(&[]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn missing_source_file_exits_one_with_io_error() {
    let env = TestEnv::new("missing_src");
    let nowhere = env.dir.join("does-not-exist.lat");
    let out = env.run(&["build", nowhere.to_str().expect("utf-8 path")]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).expect("utf-8 stderr");
    // Driver-layer error → `marain: error: <context>: <io-error>` shape.
    assert!(
        stderr.contains("marain: error:"),
        "expected `marain:` prefix on driver-layer error, got:\n{stderr}",
    );
}
