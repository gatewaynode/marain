//! End-to-end test for control-flow emit under `-D warnings` (R18, TODO Task 3).
//!
//! The emit goldens are string-compare only — they never invoke cargo, so a
//! `unused_parens` warning (or any other lint) in emitted Rust slips past them.
//! This test closes that gap for the warning-bearing slots (`if`/`while`
//! conditions, `let`/`return`/assignment RHS): it compiles a real control-flow
//! program with `RUSTFLAGS=-D warnings` so any lint becomes a hard build error,
//! AND runs it to assert the computed result.
//!
//! Why both assertions: a warning-clean build proves R18 removed the redundant
//! parens, but a build-only check can't catch a precedence *miscompile* that
//! still compiles. The program below relies on `*` binding tighter than `+`
//! (`summa + i*2`, not `(summa+i)*2`), so the printed value `20` is what proves
//! the minimal-paren emitter preserved Rust's precedence.
//!
//! Requires `cargo` on PATH (always true under `cargo test`). Reuses the
//! tempdir + shim pattern from `e2e_hello_world.rs`.

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

fn project_scratch_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has at least 2 ancestors")
        .join(".scratch")
}

/// RAII tempdir guard (mirrors `e2e_hello_world.rs`; test-only scaffolding that
/// doesn't belong on the public crate surface).
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
fn control_flow_compiles_warning_clean_and_computes_correct_value() {
    // Exercises every `unused_parens`-checked slot through R18 minimal-paren
    // emit: a `for` over an inclusive range, a `fit` assignment whose RHS is a
    // mixed-precedence BinOp, and an `if` whose condition is a comparison.
    //
    //   summa += i * 2  for i in 1..=4  →  2 + 4 + 6 + 8 = 20  (>10 → printed)
    let src = "\
sit @summa est 0.
pro ^i in 1..=4 :
    @summa fit @summa plus ^i per 2.
si @summa maior quam 10 :
    dic @summa.
";
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("accumulator.lat"), src.to_string());
    let tokens = lex(map.get(id)).expect("lex");
    let module = parse(&tokens).expect("parse");
    let rust_main = emit(&module).expect("emit");

    // Guard the precedence shape before handing to cargo: the RHS must be the
    // unparenthesized `summa + i * 2`, NOT `(summa + i) * 2` or paren-wrapped.
    assert!(
        rust_main.contains("summa = summa + i * 2i64;"),
        "expected minimal-paren precedence-correct RHS, got:\n{rust_main}",
    );
    assert!(
        !rust_main.contains("if ("),
        "condition slot should carry no redundant parens, got:\n{rust_main}",
    );

    let tmp = TempDir::new("ctrlflow");
    let shim_dir = tmp.path().join("accumulator");
    write_shim(&shim_dir, "accumulator", &rust_main).expect("write_shim");

    // `-D warnings` turns any lint (notably `unused_parens`) into a build error.
    // The generated program has no dependencies, so this flag only governs the
    // emitted crate. CARGO_TARGET_DIR is cleared so the shim builds in its own
    // target/ rather than racing the test runner's.
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--manifest-path"])
        .arg(shim_dir.join("Cargo.toml"))
        .env("RUSTFLAGS", "-D warnings")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("cargo invocation failed (is cargo on PATH?)");

    assert!(
        output.status.success(),
        "cargo run failed under -D warnings (exit {:?}):\n--- emitted ---\n{}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status.code(),
        rust_main,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert_eq!(
        stdout.trim_end(),
        "20",
        "precedence miscompile: expected summa + i*2 = 20",
    );
}
