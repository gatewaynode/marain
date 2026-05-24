# Marain — TODO

## Architecture design rounds (in flight)

Driving `ARCHITECTURE.md` to completeness for Stage 1. Each round closes in conversation, then commits to its section in `ARCHITECTURE.md`.

- [x] **Round 1 — Skeleton** — workspace + crate layout + XDG on-disk paths (closed 2026-05-22)
- [x] **Round 2 — Span & source-map** — multi-file-ready `Span { start, end, file: FileId }`, eager line index, `SourceMap` registry (closed 2026-05-22)
- [x] **Round 3 — Error model** — `Diagnostic` + `Severity` + spartan renderer; per-stage enums + `MarainError` facade convention documented, materializes in Round 4 (closed 2026-05-22)
- [x] **Round 4 — Lexer** — 8-file decomposition under `lexer/`; sigils + indentation + Latin keywords + string/int/punct/bracket tokens; first `LexError` activates `MarainError` facade; 500-LOC target held without invoking pressure-release (closed 2026-05-23)
- [x] **Round 5 — Parser + AST** — recursive-descent over 5 productions (let-binding, no-punct macro call, string/int lit, var-ref); `Ident` / `SigiledIdent` wrappers with `Option<Inflection>` slot (carry-over α landed); `MarainError::Parse(ParseError)` joins facade (closed 2026-05-23)
- [x] **Round 6 — Codegen + cargo shim** — `emit.rs` (AST → Rust source, full Rust 2024 keyword escaping via `r#`); `shim.rs` (Cargo.toml + main.rs writer with atomic-write); `MarainError::Emit(EmitError)` joins facade; `ShimError` stands alone (no `Span`); v0.1 done line proven end-to-end via `tests/e2e_hello_world.rs` (closed 2026-05-23)
- [x] **Round 7 — CLI + driver** — `clap`-based arg parsing (PRD §9 amended 2026-05-23 to permit `clap`); `marain build` prints shim path to stdout; `marain run` invokes cargo with inherited stdio; `DriverError` composes `MarainError` + `ShimError` + `io::Error` + `Cargo { exit_code }`; hand-rolled XDG resolution + FNV-1a 8-hex shim identity; v0.1 done line operational at the user-facing CLI layer (closed 2026-05-23)
- [x] **Round 8 — Testing harness** — three-layer coverage: per-phase unit (in-source); fixture-walker goldens (`marain-core/tests/{emit,error}_goldens.rs` + 13 paired fixtures, `MARAIN_UPDATE_GOLDENS=1` to regenerate); behavioral e2e at library (carried from R6) and binary (`marain-cli/tests/cli_e2e.rs`, 10 tests via `env!("CARGO_BIN_EXE_marain")`). 252 tests pass total. Concern ε (test strategy) retired. (closed 2026-05-23)

## Completed

- ~~**Task 0** — Fix PRD §11 numbering inconsistency~~ (done 2026-05-17)
- ~~**Project rename**~~ Rubigo → Marain (done 2026-05-17)
- ~~**Round 1 disk wipe + workspace scaffold**~~ (done 2026-05-22)
  - Removed default `src/main.rs` package layout.
  - Created workspace `Cargo.toml` (resolver "3", `default-members = ["crates/marain-cli"]`, `workspace.lints.rust.unsafe_code = "forbid"`).
  - Pinned toolchain to 1.94.1 via `rust-toolchain.toml`.
  - Created `crates/marain-core` (lib stub) and `crates/marain-cli` (bin stub, depends on `marain-core`).
  - Quality gates pass: `cargo check`, `cargo run --` (`marain v0.1.0 (stub)`), `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all`.
- ~~**Round 2 implementation**~~ (done 2026-05-22)
  - `marain-core/src/span.rs`: `FileId(NonZeroU32)`, `Span { start, end, file }`, `join` / `len` / `is_empty`.
  - `marain-core/src/source.rs`: `SourceFile` (id + path + text + eager line_starts), `SourceMap` registry, `line_col` via binary search.
  - 18 unit tests passing; fmt + clippy -D warnings clean.
- ~~**Round 3 implementation**~~ (done 2026-05-22)
  - `marain-core/src/error.rs`: `Severity { Error, Warning }`, `Diagnostic { severity, span, message }`, `Diagnostic::error/warning/render`.
  - Renderer format: `path:line:col: severity: message`.
  - `MarainError` facade deferred to Round 4 (no empty enums); convention documented in `ARCHITECTURE.md` §5.
  - 7 unit tests passing; fmt + clippy -D warnings clean.
- ~~**Round 4 implementation**~~ (done 2026-05-23)
  - 8-file lexer under `crates/marain-core/src/lexer/`: `mod.rs` (driver), `cursor.rs`, `indent.rs`, `strings.rs`, `numbers.rs`, `idents.rs`, `keywords.rs`, `error.rs`.
  - `crates/marain-core/src/token.rs`: `Sigil`, `TokenKind` (20 variants incl. Indent/Dedent/Eof), `Token`.
  - 29 Stage-1 keywords in the table (including `DETONATIO` exception and multi-word op components).
  - `MarainError::Lex(LexError)` facade activated with `From`, `to_diagnostic`, `Display`, `std::error::Error`.
  - Indentation: spaces-only, bracket-suppressed, eager DEDENT cascade on outdent.
  - 100 unit tests passing (75 new); fmt + clippy -D warnings clean.
  - CLAUDE.md amended with the three-tier 500-LOC pressure-release rule; not invoked in Round 4.
  - PRD §10 risk row updated to reflect mitigated status.
- ~~**Round 5 implementation**~~ (done 2026-05-23)
  - `crates/marain-core/src/ast.rs`: `Module`, `Stmt::{Let, MacroCall}`, `Expr::{StringLit, IntegerLit, VarRef}`, `LetStmt`, `MacroCallStmt`, `StringLit`, `IntegerLit`, `Ident`, `SigiledIdent`, `Inflection` (empty marker). `Ident::new` / `SigiledIdent::new` constructors default `inflection: None` so Stage 1 parser sites never type the slot.
  - `crates/marain-core/src/parser/` with `mod.rs` (driver + `Parser<'tokens>` cursor), `grammar.rs` (per-production fns), `error.rs` (`ParseError` enum, 3 variants).
  - `TokenKind: Display` impl added to `token.rs` so parse-error messages render token names without leaking literal payloads.
  - `MarainError::Parse(ParseError)` joins facade with `From`, `to_diagnostic`, `Display`, `std::error::Error::source`.
  - 5 productions: let-binding (`sit ^x est <expr>.`), no-punct macro call (`dic <expr>.`), string lit, integer lit, sigiled var ref. Fail-fast: no error recovery.
  - 142 unit tests passing (42 new); fmt + clippy -D warnings clean.
  - All four R5 files comfortably under 500-LOC target; pressure-release not invoked.
  - Carry-over concern α (AST inflection slot) resolved via `Ident` / `SigiledIdent` wrappers.
- ~~**Round 8 implementation**~~ (done 2026-05-23)
  - `crates/marain-core/tests/emit_goldens.rs`: fixture-walker; loads each `tests/fixtures/*.lat`, runs lex→parse→emit, compares to `*.expected.rs`. One `#[test]` aggregates all fixtures and reports every drift in a single run. `MARAIN_UPDATE_GOLDENS=1` regenerates.
  - `crates/marain-core/tests/error_goldens.rs`: same shape for `tests/fixtures/errors/*.lat`; pipeline must error; rendered diagnostic compares to `*.expected.txt`. Fixture path rebased to bare basename so diagnostics are machine-stable.
  - 8 emit fixtures: hello-world, let-integer, let-string, let-mutable, let-then-print, all-macros (dic/queror/agmen/forma), integer-separators (1_000_000 stripping), Rust-keyword-escape (`^if` → `r#if`).
  - 5 error fixtures: unexpected-char (lex), unterminated-string (lex), missing-period (parse), unescapable-keyword (`^self` → emit error), no-sigil-in-binding (parse).
  - `crates/marain-cli/tests/cli_e2e.rs`: 10 binary-level tests via `env!("CARGO_BIN_EXE_marain")`. Per-test isolated `$XDG_STATE_HOME` via `Command::env` (no env mutation in our process — `unsafe_code = "forbid"` satisfied). Covers build/run/bad-source/emit-error/help/version/unknown-subcommand/missing-path/no-subcommand/missing-source-file. Exit-code shape (1 vs 2) asserted explicitly.
  - `#![forbid(unsafe_code)]` added to both crate roots (`marain-core/src/lib.rs`, `marain-cli/src/main.rs`) as belt-and-braces alongside the existing workspace `unsafe_code = "forbid"` lint per CLAUDE.md.
  - **Total tests: 252** (199 core unit + 1 hello-world e2e + 1 emit-goldens + 1 error-goldens + 40 cli unit + 10 cli-e2e). `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.
  - All R8 files under 500-LOC target (`cli_e2e.rs` ~210, goldens harnesses ~130 each); pressure-release not invoked.
  - Carry-over concern ε (test strategy) RETIRED.
  - **v0.1 of the v0.1 done line is now fully proven** at unit, library-e2e, binary-e2e, and golden-tripwire layers.
- ~~**Round 7 implementation**~~ (done 2026-05-23)
  - `crates/marain-cli/Cargo.toml` adds `clap = "=4.5.61"` features `["derive"]` (N-1 minor of latest 4.6.1; released 2026-03-12; 72 days before pin date; MSRV 1.74 ≤ our 1.94.1).
  - `crates/marain-cli/src/args.rs`: `#[derive(Parser)] Cli` + `#[derive(Subcommand)] Command { Build { path }, Run { path } }`. `--help` / `--version` from clap.
  - `crates/marain-cli/src/paths.rs`: hand-rolled XDG resolution (pure `xdg_state_home_from(state_var, home_var)` plus env-reading wrapper); hand-rolled FNV-1a 32-bit (`fnv1a_8hex`); `shim_dir_for` composes `$XDG_STATE_HOME/marain/builds/<basename>-<8hex-hash>` over the canonical absolute path.
  - `crates/marain-cli/src/error.rs`: `DriverError { Source { error, map }, Shim(ShimError), Io { context, source }, Cargo { exit_code } }`; constructors `from_source` / `from_io`; `From<ShimError>`; `Display` / hand-rolled `Debug` / `std::error::Error::source`; `report()` writes to stderr in the right shape per variant.
  - `crates/marain-cli/src/driver.rs`: `dispatch(cli)` / `build(source)` / `run(source)` plus private `write_shim_from_source(source, shim_dir)` seam so tests drive the pipeline without env-var mutation (workspace `unsafe_code = "forbid"` correctly blocks the alternative).
  - `crates/marain-cli/src/main.rs`: thin shim — parse args, dispatch, report any error, `process::exit(0|1)`.
  - 199 marain-core unit + 40 marain-cli unit + 1 integration test pass (40 new tests + the new binary); `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.
  - All five marain-cli files comfortably under 500-LOC target (largest: paths.rs at 277 LOC, driver.rs at 274 LOC); pressure-release not invoked.
  - Manual binary smoke test: `marain build hello.lat` prints shim path to stdout; `marain run hello.lat` prints `salve, munde`; `--help` / `--version` render via clap; `marain build bad.lat` (with `?` source) prints `bad.lat:1:1: error: unexpected character '?'` and exits 1.
  - Carry-over concern δ (hand-rolled CLI parsing) RETIRED after PRD §9 amendment.
- ~~**Round 6 implementation**~~ (done 2026-05-23)
  - `crates/marain-core/src/emit.rs`: `emit(&Module) -> Result<String, EmitError>`; uniform `println!("{}", arg)` shape avoids format-string footgun; integers emit with `i64` suffix; sigils discarded at use sites; complete Rust 2024 keyword escaping (45 escapable via `r#`, 5 unescapable → `EmitError`).
  - `crates/marain-core/src/shim.rs`: `render_cargo_toml(&str) -> String` (minimal `[package]` table, no `[workspace]`, no `[[bin]]`); `write_shim(&Path, &str, &str) -> Result<(), ShimError>` with sibling-staging + `fs::rename` atomic-write protocol; `ShimError` enum (`CreateDir`, `WriteFile`, `RemoveDir`, `Rename`) wrapping `io::Error` via `source()` chain.
  - `MarainError::Emit(EmitError)` joins facade. `ShimError` stands alone (no `Span`).
  - `crates/marain-core/tests/e2e_hello_world.rs`: integration test exercises the full pipeline (lex → parse → emit → shim → `cargo run`) and asserts stdout is `salve, munde`. The PRD §7 v0.1 done line is now automatically regression-tested.
  - 198 unit tests + 1 integration test passing (56 new unit tests + e2e); fmt + clippy -D warnings clean.
  - Both R6 files comfortably under 500-LOC target; pressure-release not invoked.
  - Carry-over concern γ (Variabile runtime injection) re-pinned for when Variabile literals enter the language.
