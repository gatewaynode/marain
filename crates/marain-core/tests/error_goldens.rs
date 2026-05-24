//! Golden-text-diff tripwire for diagnostic rendering.
//!
//! For each `*.lat` fixture under `tests/fixtures/errors/`, runs
//! `lex → parse → emit` and asserts the pipeline produces an error; the
//! rendered diagnostic (`<basename>.lat:line:col: error: msg`) is then
//! compared to `<name>.expected.txt`. Set `MARAIN_UPDATE_GOLDENS=1` to
//! regenerate.
//!
//! The fixture path is rebased to the bare filename when loading into the
//! [`SourceMap`] so the rendered diagnostic is stable across machines
//! regardless of where the repo lives on disk.
//!
//! Trailing newlines in the golden file are tolerated (compared with
//! `trim_end`), so editors that auto-add a trailing newline don't flake
//! the test.

use std::fs;
use std::path::{Path, PathBuf};

use marain_core::emit::emit;
use marain_core::error::MarainError;
use marain_core::lexer::lex;
use marain_core::parser::parse;
use marain_core::source::SourceMap;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("errors")
}

fn is_update_mode() -> bool {
    std::env::var_os("MARAIN_UPDATE_GOLDENS")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
}

fn collect_lat_fixtures(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read error fixtures dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("lat"))
        .collect();
    out.sort();
    out
}

fn render_pipeline_error(basename: &str, text: &str) -> String {
    // Use the basename as the synthetic path so the rendered diagnostic
    // is stable: `<basename>:1:5: error: ...`.
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from(basename), text.to_string());

    let result: Result<String, MarainError> = (|| {
        let tokens = lex(map.get(id))?;
        let module = parse(&tokens)?;
        let rust = emit(&module)?;
        Ok(rust)
    })();

    match result {
        Ok(s) => panic!("expected error from {basename}, but pipeline succeeded:\n{s}"),
        Err(e) => e.to_diagnostic().render(&map),
    }
}

#[test]
fn error_golden_tripwire() {
    let dir = fixtures_dir();
    let fixtures = collect_lat_fixtures(&dir);
    assert!(
        !fixtures.is_empty(),
        "no .lat error fixtures in {}",
        dir.display()
    );

    let update = is_update_mode();
    let mut failures: Vec<String> = Vec::new();
    let mut updated: Vec<String> = Vec::new();

    for lat_path in fixtures {
        let basename = lat_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let text = fs::read_to_string(&lat_path).expect("read lat");
        let expected_path = lat_path.with_extension("expected.txt");
        let actual = render_pipeline_error(&basename, &text);

        if update {
            // Newline-terminate so editors don't fight the file on save.
            let on_disk = if actual.ends_with('\n') {
                actual.clone()
            } else {
                format!("{actual}\n")
            };
            fs::write(&expected_path, on_disk).expect("write expected");
            updated.push(basename);
            continue;
        }

        let expected_raw = match fs::read_to_string(&expected_path) {
            Ok(s) => s,
            Err(_) => {
                failures.push(format!(
                    "MISSING golden for {basename}: {} (re-run with MARAIN_UPDATE_GOLDENS=1)",
                    expected_path.display(),
                ));
                continue;
            }
        };

        let expected = expected_raw.trim_end();
        let actual_trim = actual.trim_end();

        if actual_trim != expected {
            failures.push(format!(
                "MISMATCH for {basename}\n--- expected ---\n{expected}\n--- actual ---\n{actual_trim}",
            ));
        }
    }

    if update {
        eprintln!(
            "MARAIN_UPDATE_GOLDENS=1: wrote {} error-golden file(s): {}",
            updated.len(),
            updated.join(", ")
        );
        return;
    }

    if !failures.is_empty() {
        panic!(
            "{} error fixture(s) drifted; re-run with MARAIN_UPDATE_GOLDENS=1:\n\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}
