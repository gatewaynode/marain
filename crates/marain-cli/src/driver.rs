//! End-to-end transpile pipeline and cargo invocation.
//!
//! [`dispatch`] is the entry point the binary calls after argument parsing.
//! It matches on the subcommand and routes to [`build`] (which performs the
//! transpile and writes the cargo shim) or [`run`] (which does the same and
//! then invokes `cargo run` on the shim).
//!
//! Both functions wrap every error into [`DriverError`] so the binary has
//! a single error type to report.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use marain_core::emit::emit;
use marain_core::error::MarainError;
use marain_core::lexer::lex;
use marain_core::parser::parse;
use marain_core::shim::write_shim;
use marain_core::source::SourceMap;

use crate::args::{Cli, Command as Subcommand};
use crate::error::DriverError;
use crate::paths::{shim_dir_for, shim_name_for};

/// Run the subcommand parsed from the command line.
///
/// `Build` prints the shim directory path on stdout on success;
/// `Run` forwards the user program's stdout/stderr verbatim (via inherited
/// stdio) and exits with the cargo subprocess's exit code.
pub fn dispatch(cli: Cli) -> Result<(), DriverError> {
    match cli.command {
        Subcommand::Build { path } => {
            let shim = build(&path)?;
            println!("{}", shim.display());
            Ok(())
        }
        Subcommand::Run { path } => run(&path),
    }
}

/// Transpile `source` and write the resulting cargo shim project to the
/// XDG-resolved location. Returns the shim directory path on success.
pub fn build(source: &Path) -> Result<PathBuf, DriverError> {
    let shim_dir = shim_dir_for(source).map_err(|e| {
        DriverError::from_io(format!("failed to canonicalize {}", source.display()), e)
    })?;
    write_shim_from_source(source, &shim_dir)?;
    Ok(shim_dir)
}

/// Transpile `source`, then invoke `cargo run` on the resulting shim,
/// inheriting stdio so the user sees both cargo's progress output and
/// their program's output live.
pub fn run(source: &Path) -> Result<(), DriverError> {
    let shim_dir = build(source)?;
    let manifest = shim_dir.join("Cargo.toml");

    // `--quiet` suppresses cargo's "Compiling..." lines so the user sees
    // only their program's output. `--manifest-path` points at the shim so
    // the cwd-walk-up for a workspace doesn't kick in. `CARGO_TARGET_DIR`
    // is unset so cargo uses the shim's own `target/`, not whatever the
    // outer environment may have set.
    let status = Command::new("cargo")
        .args(["run", "--quiet", "--manifest-path"])
        .arg(&manifest)
        .env_remove("CARGO_TARGET_DIR")
        .status()
        .map_err(|e| {
            DriverError::from_io(
                format!("failed to invoke cargo for {}", manifest.display()),
                e,
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(DriverError::Cargo {
            exit_code: status.code(),
        })
    }
}

/// The transpile-and-write step parameterized by an explicit `shim_dir`.
///
/// Split out from [`build`] so unit tests can drive the pipeline to a
/// tempdir without going through XDG resolution (which reads process
/// env vars and writes outside the test sandbox).
fn write_shim_from_source(source: &Path, shim_dir: &Path) -> Result<(), DriverError> {
    let text = fs::read_to_string(source)
        .map_err(|e| DriverError::from_io(format!("failed to read {}", source.display()), e))?;

    let mut map = SourceMap::new();
    let id = map.add(source.to_path_buf(), text);

    let rust_main = match transpile(&map, id) {
        Ok(s) => s,
        Err(e) => return Err(DriverError::from_source(e, map)),
    };

    let name = shim_name_for(source);
    write_shim(shim_dir, &name, &rust_main)?;
    Ok(())
}

/// Pure transpile pipeline. Kept separate so [`write_shim_from_source`]
/// can convert any single [`MarainError`] to a [`DriverError`] with the
/// owned [`SourceMap`] in one place.
fn transpile(map: &SourceMap, id: marain_core::span::FileId) -> Result<String, MarainError> {
    let tokens = lex(map.get(id))?;
    let module = parse(&tokens)?;
    let rust_main = emit(&module)?;
    Ok(rust_main)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::error::DriverError;

    fn project_scratch_dir() -> PathBuf {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .and_then(|p| p.parent())
            .expect("CARGO_MANIFEST_DIR has 2+ ancestors")
            .join(".scratch")
    }

    /// RAII guard for a unique scratch subdir containing one source file
    /// and a sibling target dir for the shim. No env mutation; tests drive
    /// [`write_shim_from_source`] directly with the explicit target path.
    struct TestSource {
        dir: PathBuf,
        source: PathBuf,
        target: PathBuf,
    }

    impl TestSource {
        fn new(label: &str, source_name: &str, source_text: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let pid = std::process::id();
            let dir = project_scratch_dir().join(format!("driver-{label}-{pid}-{nanos}-{n}"));
            fs::create_dir_all(&dir).expect("create scratch");

            let source = dir.join(source_name);
            fs::write(&source, source_text).expect("write source");

            let target = dir.join("shim-target");
            Self {
                dir,
                source,
                target,
            }
        }

        fn source(&self) -> &Path {
            &self.source
        }
        fn target(&self) -> &Path {
            &self.target
        }
    }

    impl Drop for TestSource {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn pipeline_writes_shim_for_hello_world() {
        let env = TestSource::new("hello", "hello.lat", "dic \"salve, munde\".\n");
        write_shim_from_source(env.source(), env.target()).expect("ok");

        assert!(env.target().join("Cargo.toml").exists());
        assert!(env.target().join("src").join("main.rs").exists());

        let main_rs = fs::read_to_string(env.target().join("src/main.rs")).expect("read");
        assert!(main_rs.contains("println!(\"{}\", \"salve, munde\");"));
    }

    #[test]
    fn pipeline_uses_basename_as_cargo_name() {
        let env = TestSource::new("name", "greeting.lat", "dic \"hi\".\n");
        write_shim_from_source(env.source(), env.target()).expect("ok");
        let cargo = fs::read_to_string(env.target().join("Cargo.toml")).expect("read");
        assert!(cargo.contains("name = \"greeting\""));
    }

    #[test]
    fn pipeline_propagates_lex_error_as_source_variant() {
        // `?` is not a recognized token; lexer emits `UnexpectedChar`.
        let env = TestSource::new("lex_err", "bad.lat", "?\n");
        let err = write_shim_from_source(env.source(), env.target()).expect_err("expected err");
        match err {
            DriverError::Source { error, .. } => {
                assert!(matches!(error, MarainError::Lex(_)));
            }
            other => panic!("expected Source, got {other:?}"),
        }
    }

    #[test]
    fn pipeline_propagates_parse_error_as_source_variant() {
        // Missing period after the macro arg.
        let env = TestSource::new("parse_err", "bad.lat", "dic \"hi\"\n");
        let err = write_shim_from_source(env.source(), env.target()).expect_err("expected err");
        match err {
            DriverError::Source { error, .. } => {
                assert!(matches!(error, MarainError::Parse(_)));
            }
            other => panic!("expected Source, got {other:?}"),
        }
    }

    #[test]
    fn pipeline_returns_io_error_for_missing_source_file() {
        let nowhere = project_scratch_dir().join("definitely-does-not-exist-driver-test.lat");
        let _ = fs::remove_file(&nowhere); // belt-and-braces
        let target = project_scratch_dir().join("unused-target-driver-test");
        let err = write_shim_from_source(&nowhere, &target).expect_err("expected io");
        match err {
            DriverError::Io { context, .. } => {
                assert!(context.contains("failed to read"));
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn pipeline_overwrites_existing_shim_on_second_call() {
        let env = TestSource::new("overwrite", "hello.lat", "dic \"first\".\n");
        write_shim_from_source(env.source(), env.target()).expect("first ok");

        fs::write(env.source(), "dic \"second\".\n").expect("rewrite source");
        write_shim_from_source(env.source(), env.target()).expect("second ok");

        let main_rs = fs::read_to_string(env.target().join("src/main.rs")).expect("read");
        assert!(main_rs.contains("\"second\""));
        assert!(!main_rs.contains("\"first\""));
    }

    #[test]
    fn public_build_canonicalize_failure_surfaces_as_io_error() {
        // Exercises the path-resolution arm of `build` (not reachable via
        // `write_shim_from_source` because that takes shim_dir as a param).
        // A nonexistent source file can't be canonicalized — same Io error
        // shape used for a read failure, just a different context string.
        let nowhere = project_scratch_dir().join("nowhere-for-canonicalize-test.lat");
        let _ = fs::remove_file(&nowhere);
        let err = build(&nowhere).expect_err("expected io");
        // The first thing `build` does is canonicalize; on miss, we get the
        // "failed to canonicalize" context.
        match err {
            DriverError::Io { context, .. } => {
                assert!(
                    context.contains("failed to canonicalize"),
                    "expected canonicalize context, got: {context}"
                );
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }
}
