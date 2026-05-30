# Continuity — R13 closed, function surface complete

_Rewritten 2026-05-30 (post-R13). Captures: functio + redde + calls landed end-to-end, two pressure-release splits invoked (ast.rs sibling, parser/grammar.rs production), reframe of ParseError::GenericsDeferred → LexError::GenericsLookalike, naming-convention split (Latin op surfaces vs English Stmt variants tracking Rust target) holds for new Stmt::Call addition. Rewrite on next use._

## Where We Are

v0.2 implementation is mid-stride. **R13 (functions + calls) closed clean.** Next round entry point is **R14+R15 (`pro` + range tokens + `nihil`)** — sketched in `tasks/TODO.md` under `## v0.2 implementation plan` but not yet framed.

**Session arc (chronological):**

1. **Re-aligned** via PRD / ARCHITECTURE / CONTINUITY (rewritten at R11+R12 close) / TODO. Verified R13 framing assumptions against current file state.
2. **R13 framing slate** with locked baseline (A / B-3 / C-2 / mandatory parens / unit return / generics deferred) and ASCII-labeled sub-decisions A-1 through F-3. User responded inline by label — clean and tight.
3. **Reframe surfaced before any code landed.** I framed `ParseError::GenericsDeferred` (E-1), then noticed it would never fire: `<` and `>` are not in Marain's lex alphabet, so the lexer's `UnexpectedChar` triggers first. Surfaced three paths (a/b/c — Greek originally, switched to ASCII per new feedback memory after the user noted keyboard-context cost). User picked option b: move to `LexError::GenericsLookalike`. Cost of the reframe: zero (no code churned).
4. **Implementation in six tracked tasks** (TaskCreate-managed): AST → Lexer → Parser → Emit → Tests+Goldens → Gates+Docs.
5. **Code changes landed across 9 files:**
   - `ast.rs` — new Stmt variants Function/Return/Call, new Expr::Call, supporting structs.
   - `lexer/error.rs` + `lexer/mod.rs` — GenericsLookalike variant + dispatcher arm.
   - `parser/error.rs` — TypePositionRequiresPascalCase variant.
   - `parser/grammar.rs` — parse_function, parse_param_list, parse_param, parse_type_ref (with PascalCase enforcement), parse_return, parse_call_stmt; dispatch on Functio/Redde and on `PlainIdent + (`.
   - `parser/expressions.rs` (NEW) — extracted from grammar.rs per locked C-1 when grammar.rs crossed 500.
   - `parser/mod.rs` — added `mod expressions;`.
   - `emit.rs` — two-pass `emit()`, emit_function, emit_param, emit_type_ref (translation table), emit_return, emit_call, emit_call_stmt.
6. **Stmt::Call gap discovered during golden generation.** Original framing didn't account for `saluta().` at statement position — calls as expressions worked but parse_stmt had no dispatch arm. Added Stmt::Call(CallStmt) + dispatch arm `PlainIdent + (` → parse_call_stmt mid-implementation. Mechanical; same file footprint.
7. **One existing test adjusted.** `unknown_statement_start_is_error` used `functio foo.` (an "unknown" statement pre-R13). Now `functio` parses, so test trips on a different error. Updated to `est foo.` (the `est` keyword has no statement-position dispatch).
8. **Pressure-release tier 1 invoked twice.** (1) `ast.rs` test bloc dominated → moved to sibling `ast_tests.rs` via `#[cfg(test)] #[path = "ast_tests.rs"] mod tests;`. (2) `parser/grammar.rs` crossed 500 LOC of production code → expression cascade extracted to `parser/expressions.rs` per locked C-1. Helpers (`expect_kind`, `expect_keyword`, `parse_sigiled_ident`) promoted to `pub(super)`.
9. **Sentrux session_end after splits:** signal 7059 → 7073 (Δ +14, **improved** despite added surface area). File splits reduced per-file complexity faster than R13 added it. 0 cycles change, 0 coupling change, DSM `above_diagonal` stays 0.
10. **R13 close docs:** ARCHITECTURE §15 (15.1 through 15.8); §0 reading-order extended; `tasks/TODO.md` R13 checked off + completion entry appended at the top of `## Completed`.

### Test count at session close

**415 tests passing** workspace-wide (was 354 at R11+R12 close; +61 from R13). Per binary: marain-core lib 362 (was 301), e2e_hello_world 1, emit_goldens 1, error_goldens 1, marain-cli unit 40, cli_e2e 10. `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.

### Sentrux at session close

| Metric | Baseline (R13 open) | R13 close |
|--------|---------------------|-----------|
| Quality signal | 7059 | 7073 |
| DSM above_diagonal | 0 | 0 |
| Cycles | 0 | 0 |
| Cycles change | — | 0 |
| Coupling change | — | 0.0 |
| Rule violations | 0 | 0 |

`session_end` reports "Quality stable or improved." First round with a positive signal_delta in v0.2 — the file splits did real architectural work, not just LOC redistribution.

## File State

### Added this session

- `crates/marain-core/src/ast_tests.rs` — sibling test file split from `ast.rs`, 308 LOC, doc-comment justified
- `crates/marain-core/src/parser/expressions.rs` — split from `parser/grammar.rs` per locked C-1, 269 LOC; owns the precedence cascade + `parse_call` + `make_binop`
- `crates/marain-core/tests/fixtures/18_functio_unit.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/19_functio_typed.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/20_functio_multi_param.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/21_functio_call.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/22_functio_translation.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/errors/11_generics_lookalike.lat` + `.expected.txt`
- `crates/marain-core/tests/fixtures/errors/12_type_pascal_case.lat` + `.expected.txt`
- `crates/marain-core/tests/fixtures/errors/13_missing_return_type.lat` + `.expected.txt`

### Modified this session

- `crates/marain-core/src/ast.rs` — new Stmt variants Function/Return/Call, new Expr::Call, supporting structs, test bloc moved to sibling; now 343 LOC
- `crates/marain-core/src/lexer/error.rs` — GenericsLookalike { ch, span } variant + message + 1 test; 173 LOC
- `crates/marain-core/src/lexer/mod.rs` — `<` / `>` dispatcher arm before catch-all + 3 driver tests + doc-comment justification for pre-existing pressure-release size; 665 LOC
- `crates/marain-core/src/parser/error.rs` — TypePositionRequiresPascalCase variant + message + 1 test; 149 LOC
- `crates/marain-core/src/parser/grammar.rs` — +parse_function/param_list/param/type_ref/return/call_stmt; expression cascade extracted to expressions.rs; helpers promoted to `pub(super)`; now 357 LOC
- `crates/marain-core/src/parser/mod.rs` — `mod expressions;`; now 74 LOC
- `crates/marain-core/src/parser/mod_tests.rs` — +30 new tests + `unknown_statement_start_is_error` adjusted to use `est` (since `functio` now parses); doc-comment LOC updated; now 1213 LOC
- `crates/marain-core/src/emit.rs` — two-pass emit, emit_function, emit_param, emit_type_ref (2-arm translation), emit_return, emit_call, emit_call_stmt; now 454 LOC
- `crates/marain-core/src/emit_tests.rs` — +18 R13 emit tests; doc-comment LOC updated; now 712 LOC
- `ARCHITECTURE.md` — §0 reading-order row added for R13; new §15 (full round close: 15.1 through 15.8)
- `tasks/TODO.md` — R13 entry checked off; completion entry appended to `## Completed`

### Untouched but worth noting

- `crates/marain-core/src/lexer/keywords.rs` — 37 keyword entries unchanged (R13's `Functio`, `Dat`, `Redde` were already in R4's table).
- `crates/marain-core/src/parser/mod.rs` — only `mod expressions;` added; the driver itself is unchanged.
- `PRD.md` — no spec changes this session.
- `docs/core-lexicon.md` — no changes.
- `hello.lat` at repo root (untracked) — user's manual-test scratchpad, unchanged.
- `tasks/LESSONS.md` — still empty.
- `tasks/BUGS.md` — still empty.

## What's Next (next session's entry point)

**R14+R15 — `pro` + ranges + `nihil`** (batched per locked decision A). Per `tasks/TODO.md`:

- New lexer tokens `DotDot` and `DotDotEq` — `peek_at` already exists in cursor (added R9), so the lookahead pattern is in place.
- `pro <sigiled-binding> in <iterable> :` parsing — iterable is any expression so a range literal `0..10` flows through naturally.
- `nihil.` parses as `Stmt::Nihil(NihilStmt { span })` and emits as `()` statement or empty block.
- Open sub-decision (per TODO.md): emit shape for `nihil.` — `();` vs `{}`. Recommendation: `();`.

**Likely R14+R15 framing slate (not yet committed):**

1. **Lexer additions.** Add `DotDot` and `DotDotEq` to `TokenKind`. Dispatcher in `lexer/mod.rs` adds `b'.'` arm that peeks for `.` then `=` to produce the three variants (`Period` / `DotDot` / `DotDotEq`). Period is currently single-character; the new dispatch needs to be careful not to break statement-terminator parsing.
2. **Range expressions at the expression level.** Two shapes — `lhs..rhs` (exclusive) and `lhs..=rhs` (inclusive). Cascade slot: lowest infix precedence in Rust, so a new `parse_range` level below `parse_or`. Or: ranges as a primary-level construct (would require explicit parens for `a..b plus c`). Rust treats ranges as low-precedence infix; matching that is cleaner.
3. **`Expr::Range(RangeExpr)`** AST shape. Likely `RangeExpr { start: Option<Expr>, end: Option<Expr>, inclusive: bool, span }` to cover all 6 Rust range variants (`a..b`, `a..=b`, `..b`, `..=b`, `a..`, `..`). v0.2 may only need `a..b` and `a..=b` — defer the others.
4. **`pro <sigiled-binding> in <iterable> :`** — new `Stmt::For(ForStmt)`. ForStmt { binding: SigiledIdent, iter: Expr, body: Block, span }. Parser dispatch on `Keyword::Pro`. Emit: `for <binding> in <iter> { ... }` with sigil-drop on binding.
5. **`Stmt::Nihil(NihilStmt { span })`** — parser dispatch on `Keyword::Nihil`. Emit: `();` per recommendation.
6. **Pressure-release watch.** parser/grammar.rs at 357 LOC; +parse_for + parse_nihil ≈ +40 LOC = ~400 LOC. Comfortable. parser/expressions.rs at 269 LOC; +parse_range ≈ +30 LOC = ~300 LOC. Comfortable. emit.rs at 454 LOC; +emit_for + emit_nihil + emit_range ≈ +50 LOC = ~500 LOC — could push over. Watch.

**Other open sub-decisions** (each in its own round, all noted in `tasks/TODO.md`):
- R14+R15: emit shape for `nihil.` — `();` vs `{}`. Recommendation `();`.
- Range arity (which of the 6 variants ship in v0.2) — likely just `a..b` and `a..=b`.

## Carry-over Concerns (status at session close)

| Concern | Status |
| ------- | ------ |
| (α) AST inflection slot | **RESOLVED** in R5 (Stage 1 untouched; TypeRef.name carries it via Ident; CallExpr.callee carries it via Ident) |
| (β) 500-LOC lexer | **RESOLVED** in R4 |
| (γ) `Variabile` runtime injection | **PINNED** for when Variabile literals enter the language |
| (δ) Hand-rolled CLI parsing | **RETIRED** 2026-05-23 |
| (ε) Test strategy | **RETIRED** in R8 |
| (ζ) Cross-file Stage 2 diagnostics | **PINNED**, future-only |
| (η) Comment syntax | **RETIRED** 2026-05-25 via PRD §4.12 + R9/§12 |
| (θ) Stage 2 `(lemma, inflection)` tokens | **PINNED** for Stage 2 |
| Workspace inheritance for shims | **RESOLVED** in R6 |
| R10's `si 1 :` typecheck caveat | **RETIRED** in R11+R12 |
| ARCHITECTURE §8.10 emit indent-threading forward hook | **RESOLVED** in R10 |
| `ParseError::GenericsDeferred` original framing | **RETIRED** by R13 reframe — now `LexError::GenericsLookalike` |

Three concerns remain pinned: γ (Variabile), ζ (cross-file Stage 2), θ (Stage 2 inflection tokens). All are post-v0.2.

## Decisions Locked This Session

| Topic | Decision |
| ----- | -------- |
| Function call scope | A-1 = (a): calls IN as both expression (`Expr::Call`) and statement (`Stmt::Call`) — declare-without-call would leave the feature untestable via cargo. |
| Round split | A-2: R13 ships declarations + calls together; no R13a/R13b split. |
| TypeRef newtype | B-1: wrap `Ident` in `TypeRef` newtype rather than scattering bare `Ident` across type-position consumers. Reserves a generics-grow seam (v0.3+ will add `params: Vec<TypeRef>` field). |
| Bare unit return | B-2: support `redde.` (no value) as Rust `return;`. Costs nothing parser-side. |
| Callee shape | B-3 (call): callee is `Ident` only, not SigiledIdent. Function-value semantics (`^f(...)`) deferred indefinitely. |
| grammar.rs split timing | C-1 = (b): land R13 in grammar.rs, split iff threshold crossed (it did). Pre-emptive split rejected as predicted-pressure not measured-pressure. |
| PascalCase enforcement | C-2 = (a): enforce at parse_type_ref via new `ParseError::TypePositionRequiresPascalCase`. PRD §4.9 commits to "hard error, not a lint"; lexer has no type-position context so check happens at parse time. |
| Trailing commas | C-3: accept in both param lists and arg lists. Diff-friendly, no surface cost. |
| `redde` outside function | C-4 = (a): parse cleanly, emit `return ...;` inside fn main, let rustc reject. Single-pass parser doesn't track nesting state. |
| Empty param list | C-5: confirmed mandatory parens; `functio foo() :` parses cleanly with empty params Vec. |
| Two-pass emit + always-fn-main | D-1: top-level Stmt::Function hoists above fn main; non-function stmts go inside fn main; fn main always emitted (cargo binary-crate requirement). |
| Translation table location | D-2 = (b): match arm in emit_type_ref (2 entries, no growth pressure). |
| Param sigil emit | D-3: `^x: Sermo` → `x: String`; `@x: Sermo` → `mut x: String`. Same sigil-drop rule as Stmt::Let. |
| Call emit shape | D-4: `foo(^x, 5)` → `foo(x, 5i64)`. Mechanical. |
| GenericsDeferred reframe | β path: original `ParseError::GenericsDeferred` would be dead code (`<` not in Marain's lex alphabet). New `LexError::GenericsLookalike { ch, span }` catches `<` / `>` at the lex layer with targeted "deferred to v0.3+" message. |
| Stmt::Call addition | Mid-implementation: original framing didn't account for call-at-statement-position; added Stmt::Call(CallStmt) wrapping CallExpr + period span. Narrower than generic ExprStmt — literals/var-refs have no observable effect, only calls earn statement-position. |
| Helper promotion | expect_kind / expect_keyword / parse_sigiled_ident promoted to `pub(super)` so parser/expressions.rs can call them after the grammar.rs split. |
| ast.rs sibling split | Per CLAUDE.md test-dominance rule. ast.rs → 343 LOC + ast_tests.rs 308 LOC. |
| ASCII option labels | New feedback memory captured: use a/b/c not α/β/γ. User has English keyboard; Greek labels are a context switch. |

## Collaboration Patterns (refined this session)

- **Numbered/lettered sub-decision slates continue to work cleanly.** R13 used `A-1 / A-2 / B-1 / B-2 / B-3 / C-1 / ... / F-3` — easier to reference in conversation than R11+R12's `1`–`12`. User replied inline by label.
- **Reframe-before-code is cheap, reframe-after-code is expensive.** Caught the `ParseError::GenericsDeferred` dead-code issue during framing follow-through, before any code landed. Zero churn cost. Per the loaded `reframe-vs-push-through` memory: the moment a design starts fighting itself, surface the reframe explicitly with options rather than push through.
- **Mid-implementation framing gaps are recoverable when narrow.** The Stmt::Call addition mid-Task #10 was a real framing gap — calls-as-statements wasn't sub-decisioned. But it was a small mechanical addition (one variant + one dispatch arm + one emit arm). Documented in CONTINUITY rather than re-framed. Worth watching: if mid-implementation additions get bigger, that's a signal the framing slate was too thin.
- **Pressure-release threshold crossings are now expected, not exceptional.** R11+R12 invoked once (test-file split); R13 invoked twice (one test-file, one production). Splits cost ~5-15 min each and the sentrux signal *improved* in R13 — measurable evidence the splits are doing real work.
- **Goldens regen via `MARAIN_UPDATE_GOLDENS=1` continues reliable.** All 8 new fixtures (5 emit + 3 error) matched predictions on first run after the Stmt::Call fix. Spot-checked outputs before close.
- **Helper promotion is the natural seam when splitting a parser module.** grammar.rs → grammar.rs + expressions.rs needed shared `expect_kind` / `expect_keyword` / `parse_sigiled_ident`. Promoted to `pub(super)`; clean.
- **TaskCreate/TaskUpdate cadence per major work phase** (6 tasks for R13) worked well for keeping parallel streams coherent. Tasks 6-9 functionally independent; 10-11 sequential.

## Tactical Notes

- Date: 2026-05-30 (one day after R11+R12 close).
- `hello.lat` at repo root (untracked, one line: `dic "salve, munde".`) unchanged.
- Lexer keyword count: unchanged at 37 (R13 added no keywords; `Functio`, `Dat`, `Redde` were already there from R4).
- `.sentrux/rules.toml` exists; `session_end` reports 0 rule violations.
- ARCHITECTURE §0 reading-order table now reaches Round 13 / §15.
- v0.2 implementation rounds remaining: R14+R15 (`pro` + ranges + `nihil`).
- `tasks/notes/v0.2_loops_final_decisions.md` (created 2026-05-25) still holds B-3 type-system rationale — refer to if R14+R15 framing needs the pass-through policy refresh.
- `tasks/CONTINUITY.md` rewritten (not appended) per CLAUDE.md.
- No new bugs this session; `tasks/BUGS.md` still empty.
- No user corrections worth a lesson this session — the framing was course-corrected mid-flight (GenericsDeferred reframe) but caught before code landed, and the Greek-letter slip self-corrected via new feedback memory.
- File-size status across modified prod files at close: ast.rs 343, ast_tests.rs 308 (sibling), parser/grammar.rs 357, parser/expressions.rs 269 (new), parser/mod.rs 74, emit.rs 454 — all under 500 target.
- Test-side pressure-release files at close: emit_tests.rs 712 (justified), parser/mod_tests.rs 1213 (justified), lexer/mod.rs 665 (now justified after silent gap).
- Two new memory files written:
  - `feedback_ascii_option_labels.md` — use a/b/c not Greek letters in framing slates.

## When You Resume

If user opens with "let's frame R14+R15" or similar:

1. Read `tasks/TODO.md` first (especially `## v0.2 implementation plan` for R14+R15's open sub-decisions inline in its bullet).
2. Read `tasks/notes/v0.2_loops_final_decisions.md` for context on the batching rule (decision A) and B-3 type pass-through.
3. Read PRD §4.11.2 (control flow heads, `pro` row), §4.11.4 (`nihil`), §4.11.5 (range syntax) — likely cold after compact.
4. Run sentrux `scan` then `session_start` to baseline before R14+R15 code lands.
5. Open the R14+R15 framing — start with the 6-point slate in this doc's "What's Next" section. R14+R15 is medium-sized: 2 new lexer tokens (DotDot, DotDotEq), 3 new AST nodes (RangeExpr, ForStmt, NihilStmt), ~40 new tests estimated; ARCHITECTURE §16 to be written at close.
6. Use ASCII labels (a/b/c, 1/2/3) for any framing sub-decisions — Greek letters are a context switch for the user.
7. Watch parser/expressions.rs LOC during R14+R15 if range expressions add a new cascade level.
8. Per CLAUDE.md round-closing protocol: cargo fmt + clippy + test --all; sentrux `session_start` / `session_end` bracket; ARCHITECTURE.md section drafted in conversation then committed; TODO.md round entry checked off + completion summary appended.

If user opens with anything else, be flexible — R14+R15 isn't urgent and doc state is fully coherent. Other plausible directions: manual-test R13's function-and-call surface via `hello.lat` and `marain run`, polish `docs/core-lexicon.md` to capture the R13 keyword landings (functio / dat / redde already documented but their R13 status updates would refresh), or skip ahead to post-v0.2 planning.
