//! Golden-text-diff tripwire for the emit pipeline.
//!
//! For each `*.lat` fixture under `tests/fixtures/` (excluding the
//! `errors/` subdirectory), runs `lex → parse → emit` and compares the
//! emitted Rust source to `<name>.expected.rs`. Set the env var
//! `MARAIN_UPDATE_GOLDENS=1` to regenerate the expected files (handy
//! after adding a fixture or accepting an intentional emit-shape change).
//!
//! Per ARCHITECTURE.md §10 / PRD §7: the user-facing contract is
//! behavioral (the v0.1 done line runs to expected stdout via
//! `tests/e2e_hello_world.rs`). Golden-text diff is a *tripwire* for
//! unintended drift in emission shape; the shape itself is not part of
//! the user contract.
//!
//! All fixtures are exercised by a single `#[test]` so a regression
//! reports every drifted fixture in one run, not one-at-a-time.

use std::fs;
use std::path::{Path, PathBuf};

use marain_core::emit::emit;
use marain_core::lexer::lex;
use marain_core::parser::parse;
use marain_core::source::SourceMap;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn is_update_mode() -> bool {
    std::env::var_os("MARAIN_UPDATE_GOLDENS")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
}

fn collect_lat_fixtures(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read fixtures dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("lat"))
        .collect();
    // Deterministic order so failure output is stable.
    out.sort();
    out
}

fn compile(source_path: &Path, text: &str) -> String {
    let mut map = SourceMap::new();
    let id = map.add(source_path.to_path_buf(), text.to_string());
    let tokens =
        lex(map.get(id)).unwrap_or_else(|e| panic!("lex failed on {}: {e}", source_path.display()));
    let module =
        parse(&tokens).unwrap_or_else(|e| panic!("parse failed on {}: {e}", source_path.display()));
    emit(&module).unwrap_or_else(|e| panic!("emit failed on {}: {e}", source_path.display()))
}

#[test]
fn emit_golden_tripwire() {
    let dir = fixtures_dir();
    let fixtures = collect_lat_fixtures(&dir);
    assert!(
        !fixtures.is_empty(),
        "no .lat fixtures found in {}",
        dir.display()
    );

    let update = is_update_mode();
    let mut failures: Vec<String> = Vec::new();
    let mut updated: Vec<String> = Vec::new();

    for lat_path in fixtures {
        let text = fs::read_to_string(&lat_path).expect("read lat");
        let expected_path = lat_path.with_extension("expected.rs");
        let actual = compile(&lat_path, &text);
        let basename = lat_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();

        if update {
            fs::write(&expected_path, &actual).expect("write expected");
            updated.push(basename);
            continue;
        }

        let expected = match fs::read_to_string(&expected_path) {
            Ok(s) => s,
            Err(_) => {
                failures.push(format!(
                    "MISSING golden for {basename}: {} (re-run with MARAIN_UPDATE_GOLDENS=1)",
                    expected_path.display(),
                ));
                continue;
            }
        };

        if actual != expected {
            failures.push(format!(
                "MISMATCH for {basename}\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
            ));
        }
    }

    if update {
        eprintln!(
            "MARAIN_UPDATE_GOLDENS=1: wrote {} golden file(s): {}",
            updated.len(),
            updated.join(", ")
        );
        return;
    }

    if !failures.is_empty() {
        panic!(
            "{} fixture(s) drifted; re-run with MARAIN_UPDATE_GOLDENS=1 to accept:\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}
