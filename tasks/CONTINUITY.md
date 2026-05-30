# Continuity — R11+R12 closed, expression + control-flow surface complete

_Rewritten 2026-05-29 (post-R11+R12). Captures: locked naming principle for AST + op variants (English/Latin split), full operator precedence cascade landed, control-flow statement set complete for Stage 1, first pressure-release invocation in v0.2 (test-file split). Rewrite on next use._

## Where We Are

v0.2 implementation is mid-stride. **R11+R12 (operator expressions + control flow) closed clean.** Next round entry point is **R13 (`functio` declarations)** — sketched in `tasks/TODO.md` under `## v0.2 implementation plan` but not yet framed.

**Session arc (chronological):**

1. **Re-aligned** via PRD / ARCHITECTURE / CONTINUITY (stale at R9) / TODO. Skimmed §13 to confirm R10 closed.
2. **R11+R12 framing slate** with 12 numbered sub-decisions (user requested numbering for tighter feedback). User answered each crisply — including a brief pivot on naming convention: first user picked "(c) Latin everywhere, rename old `Stmt::Let` → `Stmt::Sit`, `Stmt::If` → `Stmt::Si`," then course-corrected after seeing the implications: **AST keeps English variant names**, BinOp / UnaryOp variants stay Latin per #10. Reverted to position (a).
3. **Implementation in five tracked tasks** (TaskCreate-managed): AST → parser → emit → tests+goldens → quality gates + docs. Sentrux session_start at 7089.
4. **Code changes landed in three substantial writes:**
   - `ast.rs` rewrite: new variants, BinOp/UnaryOp enums + `as_rust` methods, naming-rule note in module doc-comment.
   - `parser/grammar.rs` rewrite: 7-level precedence-climbing cascade, multi-word phrase consumption, four new control-flow parsers, aliter chain via `parse_if` recursion.
   - `emit.rs` surgical edits: imports, new arms in `emit_stmt` and `emit_expr`, new functions `emit_else_branch` / `emit_while` / `emit_loop`.
   - Plus `parser/mod.rs` got `Parser::peek_kind_at(offset)` for `non aequat` lookahead.
5. **Test additions** in three batches: 7 ast unit tests, 34 parser tests, 22 emit tests, 6 emit goldens, 3 error goldens. Goldens auto-generated via `MARAIN_UPDATE_GOLDENS=1`; all matched the predicted outputs (paren-wrap-always emit shape, descriptive `UnexpectedToken` labels for bare phrase components).
6. **Pressure-release tier 1 invoked for the first time in v0.2.** Both `parser/mod.rs` (905 LOC) and `emit.rs` (899 LOC) crossed 500 after R11+R12 growth, dominated by test code. Per CLAUDE.md, split into sibling `mod_tests.rs` (836) and `emit_tests.rs` (554) via `#[cfg(test)] #[path = "…_tests.rs"] mod tests;`. Production-side files all back under target. Both new sibling test files carry module-doc justification per the pressure-release rule.
7. **Sentrux session_end after split:** signal 7089 → 7005 (Δ−85), pass=true, 0 cycles change, 0 coupling change, DSM `above_diagonal` stays 0. Signal drop tracks added surface area, not architectural degradation.
8. **R11+R12 close docs:** ARCHITECTURE §14 (full round write-up — scope, decomposition, decisions, AST shape, test coverage, sentrux, pressure-release, forward hooks); §0 reading-order extended; `tasks/TODO.md` R11+R12 checked off + completion entry appended at the top of `## Completed`.

### Test count at session close

**354 tests passing** workspace-wide (was 289 at R10 close; +65 from R11+R12). Per binary: marain-core unit 301, e2e_hello_world 1, emit_goldens 1, error_goldens 1, marain-cli unit 40, cli_e2e 10. `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.

### Sentrux at session close

| Metric | Baseline (R11+R12 open) | R11+R12 close |
|--------|-------------------------|----------------|
| Quality signal | 7089 | 7005 |
| Acyclicity score | 10000 | 10000 |
| DSM above_diagonal | 0 | 0 |
| Cycles | 0 | 0 |
| Import edges | 39 | 38 |
| Rules pass | 4/4 enforced | 4/4 enforced |

`session_end` reports "Quality stable or improved."

## File State

### Added this session

- `crates/marain-core/src/parser/mod_tests.rs` — sibling test file, 836 LOC, doc-comment justified
- `crates/marain-core/src/emit_tests.rs` — sibling test file, 554 LOC, doc-comment justified
- `crates/marain-core/tests/fixtures/12_arithmetic.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/13_booleans.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/14_comparison.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/15_aliter_chain.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/16_dum.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/17_semper_interrumpe.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/errors/08_bare_maior.lat` + `.expected.txt`
- `crates/marain-core/tests/fixtures/errors/09_missing_colon_dum.lat` + `.expected.txt`
- `crates/marain-core/tests/fixtures/errors/10_missing_period_interrumpe.lat` + `.expected.txt`

### Modified this session

- `crates/marain-core/src/ast.rs` — full rewrite with new variants + naming-rule doc-comment; 487 LOC (487 incl. 7 new tests at the bottom)
- `crates/marain-core/src/parser/mod.rs` — `peek_kind_at` added, test bloc moved to sibling; now 73 LOC
- `crates/marain-core/src/parser/grammar.rs` — full rewrite with precedence cascade + control-flow parsers; 428 LOC
- `crates/marain-core/src/emit.rs` — surgical edits for new variants + control-flow emit; test bloc moved to sibling; now 349 LOC
- `ARCHITECTURE.md` — §0 reading-order row added for R11+R12; new §14 (full round close: 14.1 through 14.8)
- `tasks/TODO.md` — R11+R12 entry checked off; completion entry appended to `## Completed`

### Untouched but worth noting

- `crates/marain-core/src/lexer/` — no changes (R4 was front-loaded against exactly this round; every R11+R12 keyword was already in the table).
- `crates/marain-core/src/lexer/keywords.rs` — 37 keyword entries unchanged.
- `tasks/LESSONS.md` — still empty (no user corrections this session worth capturing — the brief naming-pivot mid-framing was a course-correct, not a lesson).
- `tasks/BUGS.md` — still empty.
- `hello.lat` at repo root (untracked) — user's manual-test scratchpad, unchanged.
- `PRD.md` — no spec changes this session.
- `docs/core-lexicon.md` — no changes (R11+R12 keywords already documented from prior rounds).

## What's Next (next session's entry point)

**R13 — Function declarations.** Per locked decision A, R13 ships alone (functio is large surface). Per locked decision B-3, types are open pass-through with a 2-entry emitter translation table (`Sermo`→`String`, `Numerus`→`i64`); generics rejected with `ParseError::GenericsDeferred`.

**Likely R13 framing slate (not yet committed — fresh framing round expected):**

1. **`Stmt::Function(FunctionStmt)` AST shape** with `name: Ident`, `params: Vec<Param>`, `return_type: Option<TypeRef>`, `body: Block`, `span`. `Param { name: SigiledIdent, type_ref: TypeRef, span }`. `TypeRef` likely a newtype wrapping `Ident` (Stage 1 has no generics); the emitter translation table handles `Sermo`→`String` etc.
2. **`Stmt::Return(ReturnStmt)`** for `redde <expr>.` — straightforward.
3. **Mandatory parens** on signature per PRD §4.11.1, including for zero-arg `functio foo() :` (per locked sub-decision).
4. **Unit return** when `dat` clause omitted — emit produces no `-> ()` annotation; let Rust infer (per locked sub-decision).
5. **`ParseError::GenericsDeferred`** new variant — first new ParseError variant since R5 (R10 / R11+R12 both stayed on `UnexpectedToken`). Justification: generics carry a deferred-feature message that `UnexpectedToken { expected: "type identifier (generics deferred to v0.3+)" }` can roughly approximate but a dedicated variant gives the parser a clean place to localize the deferred-feature messaging if the rule grows.
6. **Pressure-release watch on `parser/grammar.rs`** — currently 428 LOC; R13 adds function-signature parsing (params loop + type parsing + return-type optional clause). Could push it over 500. Consider whether to pre-emptively split into `parser/{statements,expressions,types}.rs` family or wait for actual pressure.

**Other open sub-decisions** (each in its own round, all noted in `tasks/TODO.md`):
- R14+R15: emit shape for `nihil.` — `();` vs `{}` (recommendation: `();`).

## Carry-over Concerns (status at session close)

| Concern | Status |
| ------- | ------ |
| (α) AST inflection slot | **RESOLVED** in R5 |
| (β) 500-LOC lexer | **RESOLVED** in R4 |
| (γ) `Variabile` runtime injection | **PINNED** for when Variabile literals enter the language |
| (δ) Hand-rolled CLI parsing | **RETIRED** 2026-05-23 |
| (ε) Test strategy | **RETIRED** in R8 |
| (ζ) Cross-file Stage 2 diagnostics | **PINNED**, future-only |
| (η) Comment syntax | **RETIRED** 2026-05-25 via PRD §4.12 + R9/§12 |
| (θ) Stage 2 `(lemma, inflection)` tokens | **PINNED** for Stage 2 |
| Workspace inheritance for shims | **RESOLVED** in R6 |
| R10's `si 1 :` typecheck caveat | **RETIRED** in R11+R12 (boolean conditions now produce typecheckable Rust) |

Three concerns remain pinned: γ (Variabile), ζ (cross-file Stage 2), θ (Stage 2 inflection tokens). All are post-v0.2.

## Decisions Locked This Session

| Topic | Decision |
| ----- | -------- |
| AST naming rule | English for Stmt variants (track Rust target); Latin for BinOp/UnaryOp variants (track operator surface). Existing `Stmt::Let`/`If` unchanged. Compound Latin words allowed for multi-word ops (`DivisusPer`, `MinorVelPar`, `NonAequat`). Documented in `ast.rs` module doc-comment. |
| Else-chain AST shape | `IfStmt.else_branch: Option<ElseBranch>` where `ElseBranch::Block` is terminal `aliter` and `ElseBranch::If(Box<IfStmt>)` is `aliter si` chain (single nested shape; recursion source for `emit_else_branch`). |
| Operator precedence | Rust precedence verbatim per PRD §4.4 — 7-level cascade or → and → equality → comparison → additive → multiplicative → unary → primary. All binary levels left-associative; unary `non` right-associative by recursion. |
| Multi-word phrase resolution | Greedy at parse level — `minor`/`maior` advance + peek for `quam` or `vel par`; `divisus` peek for `per`; `non` peek for `aequat` at equality level. Bare components are hard errors with descriptive `UnexpectedToken` labels. |
| `non` disambiguation | One-token lookahead via new `Parser::peek_kind_at`. `non aequat` at equality level is binary `!=`; everything else is unary prefix (handled at `parse_unary`). |
| Boolean literal AST | New `Expr::BoolLit` variant (parallels `IntegerLit` / `StringLit` shape); emit produces bare `true`/`false`. |
| Emit precedence safety | Paren-wrap-always — every BinOp/UnaryOp emits with surrounding parens. Bulletproof against Rust-precedence drift; the visual noise is accepted. |
| `(expr)` grouping in primary | Supported — one match arm in `parse_primary`. No `ParenExpr` AST node (precedence is structurally encoded post-parse). |
| `aliter si` parsing | `parse_if` recurses through itself for the chain; no special "is chain" detection — the same code handles single-arm and multi-arm. |
| Stmt::Loop name | English `Loop` for the AST (matches Rust target); `semper` keyword drives parser dispatch. |
| `interrumpe.` / `continua.` shape | Statement-level, period-terminated, no payload (unlabeled, no break-value). Labeled break + break-value are post-v0.2. |
| Zero new ParseError variants | R11+R12 rides entirely on `UnexpectedToken { expected: &'static str }` with descriptive labels. Continues R10's stance against variant proliferation. |
| Test-file decomposition pattern | `#[cfg(test)] #[path = "…_tests.rs"] mod tests;` per CLAUDE.md. First v0.2 invocation. Each sibling file carries module-doc justification (shared helper set, one cohesive surface). |

## Collaboration Patterns (refined this session)

- **Numbered sub-decisions for framing slates.** User asked for numbering on the second framing message after the first one was hard to respond to. Adopted: every framing slate now numbers sub-decisions, user replies inline by number. **Cleanest framing-feedback shape encountered to date — replicate going forward.**
- **Course-corrections mid-framing are cheap if caught early.** User answered the framing slate, then re-read the implications, caught a misread (#10 + position (c) interaction), and corrected before any code landed. No churn cost. Surfaces a meta-pattern: spend a beat re-confirming the *implications* of a decision before locking, not just the decision itself.
- **Goldens auto-generation via `MARAIN_UPDATE_GOLDENS=1` is reliable when emit shape is well-understood.** All 9 new goldens (6 emit + 3 error) matched predictions exactly on first run. The pattern is: write the `.lat` source, mentally predict the `.expected.{rs,txt}`, run with the env var, spot-check the generated file. Faster than hand-writing expecteds.
- **Sentrux session_start → session_end bracket per round** continues to work. `scan` must precede `session_start` (learned this round — the tool errors with `No scan data` otherwise).
- **Test-file decomposition is mechanical** when the trigger is "test bloc dominates a production file." `sed 's/^    //'` extracts and dedents in one shot; the `#[path]` attribute is a clean two-line replacement in the production file. Total cost: ~5 minutes including format/clippy/test re-verification.
- **Sibling test files retain module-doc justification.** Even when the split itself is the decomposition, the resulting sibling can still exceed 500 LOC if the test bloc is large enough; in that case, the justification doc-comment is mandatory per CLAUDE.md's pressure-release rule.
- **TaskCreate/TaskUpdate cadence per major work phase** — created 5 tasks at round-open, updated through pending → in_progress → completed. Worked well for keeping the parallel streams (AST + parser + emit + tests + docs) coherent.

## Tactical Notes

- Date: 2026-05-29.
- `hello.lat` at repo root (untracked, one line: `dic "salve, munde".`) is the user's manual-test scratchpad.
- Lexer keyword count: unchanged at 37 (R11+R12 added no keywords; everything was already in R4's table).
- `.sentrux/rules.toml` exists and `check_rules` passes (4/4 enforced under free tier).
- ARCHITECTURE §0 reading-order table now reaches Round 11+12 / §14.
- v0.2 implementation rounds remaining: R13 (`functio`), R14+R15 (`pro` + ranges + `nihil`).
- `tasks/notes/v0.2_loops_final_decisions.md` (created 2026-05-25) still holds B/C/D rationales — refer to if R13 framing needs the type-system policy refresh.
- `tasks/CONTINUITY.md` rewritten (not appended) per CLAUDE.md.
- No new bugs this session; `tasks/BUGS.md` still empty.
- No user corrections this session; `tasks/LESSONS.md` still empty. (The naming course-correct doesn't merit a lesson — it was caught immediately, no code churned.)
- File-size status across modified prod files at close: ast.rs 275 prod LOC, parser/mod.rs 70 prod LOC, parser/grammar.rs 428 LOC, emit.rs 346 prod LOC — all under 500 target.
- Pressure-release watch for R13: `parser/grammar.rs` at 428 LOC will likely cross 500 once `parse_function` (signature + params loop + type parsing + return-type optional + body) lands. Consider pre-emptive split into `parser/{statements,expressions,types}.rs` family during R13 framing.

## When You Resume

If user opens with "let's frame R13" or similar:

1. Read `tasks/TODO.md` first (esp. `## v0.2 implementation plan` for round ordering + R13's open sub-decisions inline in its bullet).
2. Read `tasks/notes/v0.2_loops_final_decisions.md` for B-3 (type pass-through + generics deferred) rationale.
3. Read PRD §4.11.1 (function declaration spec) — likely cold after compact.
4. Run `sentrux scan` then `session_start` to baseline before R13 code lands.
5. Open the R13 framing — start with the 6-point slate in this doc's "What's Next" section. R13 is medium-sized: AST gains 2 new statement types (FunctionStmt + ReturnStmt) and one new struct (Param). Parser gains function-signature parsing + redde. Emit gains the type translation table (B-3). Estimate ~40-60 new tests; ARCHITECTURE §15 to be written at close.
6. Number sub-decisions in any framing slate (proven this round to work well).
7. Watch parser/grammar.rs LOC during R13 — pre-emptive split may be cleanest.
8. Per CLAUDE.md round-closing protocol: cargo fmt + clippy + test --all; sentrux `session_start` / `session_end` bracket; ARCHITECTURE.md section drafted in conversation then committed; TODO.md round entry checked off + completion summary appended.

If user opens with anything else, be flexible — R13 isn't urgent and the doc state is fully coherent. Other plausible directions: manual-test the new operator + control-flow syntax via `hello.lat` and `marain run`, polish `docs/core-lexicon.md` to capture the R11+R12 operator landings, or skip ahead to R14+R15 framing.
