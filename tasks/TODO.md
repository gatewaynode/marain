# Marain — TODO

## v0.2 implementation plan

Spec gates closed 2026-05-25; decisions recorded in `tasks/questions_and_answers_1.md` (vocabulary) and `tasks/notes/v0.2_loops_final_decisions.md` (architecture: A round granularity, B type-system scope, C lowering pass, D comment syntax).

**Locked decisions driving the plan:**

- **A — Round granularity:** batched where dependencies naturally overlap (R10+R11 share state; R14+R15 are small enough to ship together).
- **B — Type-system scope:** open pass-through with a small emitter translation table (`Sermo`→`String`, `Numerus`→`i64`). Lexer doesn't gain type keywords. Generics (`<T>`) rejected in type position with `ParseError::GenericsDeferred`.
- **C — Lowering pass:** deferred. Parser produces `Ast` directly, same as v0.1. Stage 2 hook stays documented in `ARCHITECTURE.md` §7.8.
- **D — Comment syntax:** `//` line comments only in v0.2; `/*` reserved with deferred-feature error. PRD §4.12 amended, `ARCHITECTURE.md` §11 η entry marked resolved, lexicon Structural Punctuation table updated.

**Rounds (one feature per round unless batched per A):**

- [x] **Round 9 — Line comments** (closed 2026-05-25). New `crates/marain-core/src/lexer/comments.rs`; `/` byte added to `lexer/mod.rs` dispatch with two-char lookahead. `LexError::BlockCommentsDeferred` variant with explicit `use // for a line comment (PRD §4.12)` diagnostic. Indent state machine unchanged (comment-only lines are blank). Fixtures `09_line_comments` (emit) and `errors/06_block_comments_deferred` landed. +20 tests; workspace total 272. Sentrux: signal +3 (7079→7082), 0 cycles, 0 coupling, DSM `above_diagonal` stays 0. ARCHITECTURE.md §12.
- [x] **Round 10 — Block parsing + `si`** (closed 2026-05-29). Scope expanded mid-framing to fold the `si <cond> :` head in as the substrate parent (alternative was a test-only seam exposing `parse_block`); `aliter` chain stays in R11+R12. New AST: `Block { stmts, span }`, `IfStmt { cond, then_block, span }`, `Stmt::If`. New parser: `parse_block` (Indent → stmts until Dedent → Block), `parse_if`; `parse_stmt` dispatches on `Si`. Empty-block-via-comment-or-blank-line is structurally impossible to produce (lexer transparency rules); the only "empty block" failure is `ExpectedIndent` via the existing `UnexpectedToken { expected: "indented block" }` variant — no `EmptyBlock` variant ships. `emit_stmt` gains `indent_level: usize` (resolves ARCHITECTURE §8.10 forward hook); `emit_if` emits `if <cond> { ... }` with nested indent. Caveat: R10's expression set is still string/int/var-ref; `si 1 :` parses + emits as `if 1 { }` which rustc rejects — goldens are string-compare only, so fixtures don't exercise cargo. R11+R12 (Boolean literals + operator expressions) makes the produced Rust typecheck. +17 tests; workspace total 289. Sentrux: signal +7 (7082→7089), 0 cycles, 0 coupling, DSM `above_diagonal` stays 0. ARCHITECTURE.md §13.
- [x] **Round 11 + Round 12 — Operator expressions + control flow** (closed 2026-05-29). Precedence-climbing cascade (or → and → equality → comparison → additive → multiplicative → unary → primary) covering all 13 binary ops + unary `non`; multi-word phrases (`maior quam` / `minor vel par` / `divisus per` / `non aequat`) consumed greedily at parse level with descriptive `UnexpectedToken` errors on bare components. `non aequat` disambiguated via one-token lookahead (new `Parser::peek_kind_at`). Boolean literals `verum` / `falsum` as `Expr::BoolLit` atoms; `(expr)` grouping. `IfStmt.else_branch: Option<ElseBranch>` with `Block` / boxed `If` variants for terminal `aliter :` / chained `aliter si`. New `Stmt::While` / `Loop` / `Break` / `Continue`. Emit paren-wraps every BinOp/UnaryOp; chain emit walks `else_branch` recursively. No new `ParseError` variants. Test files split via `#[path = "…_tests.rs"]` per CLAUDE.md (parser/mod.rs 905→73 prod + 836 sibling; emit.rs 899→349 prod + 554 sibling). +65 tests; workspace total 354. Sentrux signal_delta -85 (7089→7005), 0 cycles, 0 coupling, DSM `above_diagonal` stays 0, `check_rules` passes. R10's `si 1 :` typecheck caveat retires. ARCHITECTURE.md §14.
- [ ] **Round 13 — Functions.** `functio <name>(<params>) dat <Type> :` declaration + body. `<Type>` accepts any PascalCase `PlainIdent`; emitter has translation table for `Sermo` / `Numerus`, passes others through (B-3). Generics rejected in type position with `ParseError::GenericsDeferred` carrying a "deferred to v0.3+" message. `redde <expr>.` emits as Rust `return <expr>;`. `emit_stmt` gains an `indent_level` parameter (ARCHITECTURE §8.10 forward hook resolved). Parameter syntax: comma-separated `<sigiled-name>: <Type>` per PRD §4.11.1. Quality gates + sentrux scan.
- [ ] **Round 14 + Round 15 — `pro` + ranges + `nihil`** (batched per A). New lexer tokens `DotDot` and `DotDotEq`. `pro <sigiled-binding> in <iterable> :` parsing; iterable is any expression so a range literal `0..10` flows through naturally. `nihil.` parses as `Stmt::Nihil` and emits as Rust `()` statement (or empty block, TBD at impl). Quality gates + sentrux scan.

**Per-round closing protocol:** `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --all` clean; sentrux MCP scan for complexity baseline; ARCHITECTURE.md round section drafted in conversation, then committed; `tasks/TODO.md` round entry checked off with a short result summary appended to "Completed"; `tasks/CONTINUITY.md` rewritten only on compact.

**Open architectural sub-decisions during rounds (not gating now):**
- R10: empty-block rule — `nihil.` required or empty-`Indent`/`Dedent` allowed? (Recommend `nihil.` required for clarity.)
- R11+R12: `else if` chain shape — parse as single `IfElse { else: Else::If(...) }` or two nested nodes the emitter assembles? (Recommend single nested shape; cleaner emit.)
- R13: function arity at signature time — typed parens are mandatory even for zero-arg (`functio foo() :`) or allow paren-less (`functio foo :`)? (Recommend mandatory parens per PRD §4.11.1.)
- R13: unit return — omit `dat` clause entirely (already PRD-committed) — confirm emit produces no `-> ()` annotation (let Rust infer).
- R14+R15: emit shape for `nihil.` — `();` statement vs empty `{}` block. (Recommend `();`.)

## Architecture design rounds (Stage 1 / v0.1 — closed)

Driving `ARCHITECTURE.md` to completeness for Stage 1. Each round closed in conversation, then committed to its section in `ARCHITECTURE.md`.

- [x] **Round 1 — Skeleton** — workspace + crate layout + XDG on-disk paths (closed 2026-05-22)
- [x] **Round 2 — Span & source-map** — multi-file-ready `Span { start, end, file: FileId }`, eager line index, `SourceMap` registry (closed 2026-05-22)
- [x] **Round 3 — Error model** — `Diagnostic` + `Severity` + spartan renderer; per-stage enums + `MarainError` facade convention documented, materializes in Round 4 (closed 2026-05-22)
- [x] **Round 4 — Lexer** — 8-file decomposition under `lexer/`; sigils + indentation + Latin keywords + string/int/punct/bracket tokens; first `LexError` activates `MarainError` facade; 500-LOC target held without invoking pressure-release (closed 2026-05-23)
- [x] **Round 5 — Parser + AST** — recursive-descent over 5 productions (let-binding, no-punct macro call, string/int lit, var-ref); `Ident` / `SigiledIdent` wrappers with `Option<Inflection>` slot (carry-over α landed); `MarainError::Parse(ParseError)` joins facade (closed 2026-05-23)
- [x] **Round 6 — Codegen + cargo shim** — `emit.rs` (AST → Rust source, full Rust 2024 keyword escaping via `r#`); `shim.rs` (Cargo.toml + main.rs writer with atomic-write); `MarainError::Emit(EmitError)` joins facade; `ShimError` stands alone (no `Span`); v0.1 done line proven end-to-end via `tests/e2e_hello_world.rs` (closed 2026-05-23)
- [x] **Round 7 — CLI + driver** — `clap`-based arg parsing (PRD §9 amended 2026-05-23 to permit `clap`); `marain build` prints shim path to stdout; `marain run` invokes cargo with inherited stdio; `DriverError` composes `MarainError` + `ShimError` + `io::Error` + `Cargo { exit_code }`; hand-rolled XDG resolution + FNV-1a 8-hex shim identity; v0.1 done line operational at the user-facing CLI layer (closed 2026-05-23)
- [x] **Round 8 — Testing harness** — three-layer coverage: per-phase unit (in-source); fixture-walker goldens (`marain-core/tests/{emit,error}_goldens.rs` + 13 paired fixtures, `MARAIN_UPDATE_GOLDENS=1` to regenerate); behavioral e2e at library (carried from R6) and binary (`marain-cli/tests/cli_e2e.rs`, 10 tests via `env!("CARGO_BIN_EXE_marain")`). 252 tests pass total. Concern ε (test strategy) retired. (closed 2026-05-23)

## Completed

- ~~**Round 11 + Round 12 implementation**~~ (done 2026-05-29)
  - **AST.** New `Expr::BoolLit(BoolLit)`, `Expr::BinOp(BinOpExpr)` over `BinOp { Plus, Minus, Per, DivisusPer, Modulo, Aequat, NonAequat, MinorQuam, MaiorQuam, MinorVelPar, MaiorVelPar, Et, Vel }`, `Expr::UnaryOp(UnaryOpExpr)` over `UnaryOp { Non }`. New `Stmt::While(WhileStmt)`, `Stmt::Loop(LoopStmt)`, `Stmt::Break(BreakStmt)`, `Stmt::Continue(ContinueStmt)`. `IfStmt` grows `else_branch: Option<ElseBranch>` where `ElseBranch::Block(Block)` is terminal `aliter :` and `ElseBranch::If(Box<IfStmt>)` is the `aliter si` chain. Naming rule documented in `ast.rs` doc-comment: Latin variants for operator surfaces, English for stmt variants tracking the Rust target.
  - **Parser.** Hand-rolled precedence-climbing cascade (or → and → equality → comparison → additive → multiplicative → unary → primary), Rust-precedence-verbatim per PRD §4.4. Multi-word phrase recognition (`maior quam` / `minor vel par` / `maior vel par` / `divisus per` / `non aequat`) via greedy peek at parse level — no lexer changes. `Parser::peek_kind_at(offset)` added for `non aequat` lookahead (clamps past-end peeks to trailing `Eof`). New `parse_while` / `parse_loop` / `parse_break` / `parse_continue`; `parse_if` extended with `aliter` / `aliter si` recursion. Parens grouping in `parse_primary`. Bare phrase components (`maior`, `minor`, `divisus`, `vel par` without head) surface as `UnexpectedToken` with descriptive labels — **zero new `ParseError` variants** per R10's stance.
  - **Emit.** `emit_expr` paren-wraps every BinOp / UnaryOp (parser already encodes correct precedence; paren-everywhere is bulletproof against Rust precedence drift). `BoolLit` → `true` / `false`. New `emit_while` → `while`, `emit_loop` → `loop`, `Break` / `Continue` inline as `break;` / `continue;`. `emit_else_branch` walks the chain by recursing into `emit_if`.
  - **Goldens.** 6 new emit fixtures (`12_arithmetic`, `13_booleans`, `14_comparison`, `15_aliter_chain`, `16_dum`, `17_semper_interrumpe`) and 3 new error fixtures (`errors/08_bare_maior`, `errors/09_missing_colon_dum`, `errors/10_missing_period_interrumpe`).
  - **Test-file decomposition (first pressure-release invocation in v0.2).** `parser/mod.rs` (905 LOC) and `emit.rs` (899 LOC) hit pressure-release territory after R11+R12 growth; both files dominated by test code. Per CLAUDE.md ("If `#[cfg(test)] mod tests` dominates, move it to a sibling file via `#[path = "foo_tests.rs"] mod tests;` — that's a clean decomposition, not a workaround"), tests moved to `parser/mod_tests.rs` (836 LOC) and `emit_tests.rs` (554 LOC). Production-side files all back under the 500-LOC target (parser/mod.rs 73, emit.rs 349). Both sibling test files carry module doc-comment justification per the pressure-release rule.
  - **R10 caveat retired.** `si 1 :` no longer the test ceiling — boolean conditions (`si verum et falsum :`) produce typecheckable Rust end-to-end, closing the deferred condition-typing forward hook.
  - **Total tests: 354** (+65 from 289 at R10 close). `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.
  - Sentrux session_end: `signal_delta` -85 (7089 → 7005), `cycles_change` 0, `coupling_change` 0.0, DSM `above_diagonal` stays 0, `check_rules` passes (4/4 enforced under free tier). `import_edges` 39 → 38 (test-file split removed an inbound edge). New AST nodes + parse functions + emit arms flow downward through the pipeline DAG; no edge inversion.
  - ARCHITECTURE.md §14 closed; §0 reading-order table extended through R11+R12. No new carry-over concerns opened. Carry-over γ (Variabile runtime injection) still pinned for when literals enter the language.

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
- ~~**Round 10 implementation**~~ (done 2026-05-29)
  - **Scope expansion.** R10 framing folded the `si <cond> :` head in as the substrate parent for `parse_block` (alternative was a test-only seam exposing the API). `aliter` chain explicitly held back for R11+R12 to preserve one-feature-per-round on the chain shape.
  - `crates/marain-core/src/ast.rs`: `Block { stmts: Vec<Stmt>, span: Span }`; `IfStmt { cond: Expr, then_block: Block, span: Span }`; `Stmt::If(IfStmt)` variant; `Stmt::span()` dispatch extended. +2 unit tests.
  - `crates/marain-core/src/parser/grammar.rs`: `parse_if` consumes `si` → `parse_expr` → `:` → `parse_block`; `parse_block` consumes `Indent` → loop `parse_stmt` until `Dedent`-or-`Eof` → expects `Dedent`. `parse_stmt` dispatches on `Keyword::Si`. No new `ParseError` variants — generic `UnexpectedToken { expected: &'static str }` with labels `"`:`"`, `"indented block"`, `"end of indented block"` covers every new failure mode (no dedicated `EmptyBlock` / `ExpectedIndent` / `ExpectedColon`).
  - `crates/marain-core/src/emit.rs`: `emit_stmt` gains `indent_level: usize` parameter (resolves ARCHITECTURE §8.10 forward hook); new `push_indent(out, level)` helper; new `emit_if` writes `if <cond> { ... }` with closing `}` at parent indent (no trailing `\n` — the caller adds it). All existing `emit_stmt` callers updated to thread `1` at top level. +5 unit tests including a `top_level_stmts_emit_at_indent_one` regression guard.
  - `crates/marain-core/src/parser/mod.rs`: +10 driver tests covering single-statement body, multi-statement body, nested `si`, integer-literal condition (R10 doesn't gate on type), body at column-0 (no Indent → next stmt is sibling), missing colon, missing condition, body at same indent as parent, `Eof` straight after `:`, span covers `si` through closing `Dedent`.
  - Goldens: `tests/fixtures/10_si_simple.lat`+`.expected.rs` (let + if + dic); `tests/fixtures/11_si_nested.lat`+`.expected.rs` (two `si` heads, deepest body at 12-space indent); `tests/fixtures/errors/07_no_block_after_if.lat`+`.expected.txt` (body at column 0 → `expected indented block, found keyword \`sit\``).
  - **Empty-block discovery.** The PRD §4.12 "comment-only block is a parse error" promise is satisfied structurally rather than via a dedicated `EmptyBlock` variant: R4's `indent.rs` treats blank lines as transparent to the indent stack and R9's lexer extends that to comment-only lines, so `Indent` immediately followed by `Dedent` is impossible to produce from any source. The only "empty block" failure mode is the absence of `Indent` entirely, which `parse_block`'s leading `expect_kind(p, &TokenKind::Indent, "indented block")` surfaces via `UnexpectedToken`. Per CLAUDE.md "don't add for can't-happen," no `EmptyBlock` variant.
  - **R10 emits Rust that won't typecheck** (e.g. `si 1 :` → `if 1 { }`); fixtures are string-compare only — the goldens harness never invokes cargo. R11+R12 lands the Boolean / operator surface that makes the produced Rust real.
  - **Total tests: 289** (+17 from 272 at R9 close). `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.
  - Sentrux session_end: signal_delta +7 (7082 → 7089), cycles_change 0, coupling_change 0.0, DSM `above_diagonal` stays 0, `check_rules` passes (4/4 enforced under free tier). New AST nodes flow downward through the existing parser → emit pipeline; no edge inversion.
  - ARCHITECTURE.md §13 closed (Block Parsing + `si`); §0 reading-order table extended. Carry-over concern: ARCHITECTURE §8.10 `emit_stmt` indent-threading forward hook RESOLVED.

- ~~**Round 9 implementation**~~ (done 2026-05-25)
  - **PRD §4.12 amended** (pre-R9): `//` line comments committed for v0.2; `/* */` reserved syntax with explicit deferred-feature error; doc comments unscoped.
  - **`.sentrux/rules.toml` created** (pre-R9): 20 architectural rules encoded (2 constraints + 18 boundaries) capturing the pipeline DAG from ARCHITECTURE.md §§2,6,7,8. Free tier mechanically checks 4/20; the rest are documented intent. Baseline session_start at signal 7079.
  - New `crates/marain-core/src/lexer/comments.rs` (80 LOC, 7 unit tests): `scan_line_comment` consumes to next `\n` exclusive, leaves `\n` for the existing newline handler.
  - `lexer/cursor.rs` gains `peek_at(offset)` for two-character opener lookahead (3 new unit tests).
  - `lexer/mod.rs` dispatch: start-of-line `//` peek treats the line as blank for indent purposes; mid-line `/` arm triages `//` → comment, `/*` → `LexError::BlockCommentsDeferred`, bare `/` → `LexError::UnexpectedChar`. 9 new driver tests.
  - `lexer/error.rs`: new `BlockCommentsDeferred { span }` variant; message `block comments are reserved syntax; use // for a line comment (PRD §4.12)`; 1 new test asserts message structure.
  - 2 new fixtures: `09_line_comments.lat`+`.expected.rs` (trailing + standalone + blank-line-interleaved); `errors/06_block_comments_deferred.lat`+`.expected.txt` (diagnostic).
  - **Total tests: 272** (+20 from 252 at R8 close). `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.
  - Sentrux session_end: signal_delta +3 (7079 → 7082), cycles_change 0, coupling_change 0.0, DSM `above_diagonal` stays 0, `check_rules` passes. New `lexer/comments.rs` slotted in without inverting any pipeline edge.
  - ARCHITECTURE.md §12 closed; §0 reading-order table extended; §11 η entry now points at §12. Carry-over concern η (comment syntax) RETIRED.
