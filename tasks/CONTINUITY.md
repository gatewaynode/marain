# Continuity — R9 closed, v0.2 architecture locked, sentrux baselined

_Rewritten 2026-05-25 mid-evening. Captures: A/B/C/D architectural decisions locked, PRD §4.12 amendment (line comments), `.sentrux/rules.toml` created, R9 (line comments) implemented end-to-end, ARCHITECTURE.md §12 closed. Rewrite on next use._

## Where We Are

v0.2 is in active implementation. R9 (line comments) closed clean; **R10 (block parsing) is the next round's entry point** and has not yet been framed. Three architectural decisions for v0.2 are locked (B/C/D); round granularity (A) is also locked.

**Session arc (chronological):**

1. **Re-aligned** via CONTINUITY/PRD/ARCHITECTURE/TODO. Surfaced four open architectural questions (A round granularity, B type system scope, C lowering pass, D comment syntax) for v0.2.
2. **Locked A** in conversation: batched where dependencies overlap. R9 alone (everything depends on it), R10+R11 batched (expressions feed control flow), R12 alone, R13+R14 batched (small).
3. **Locked B/C/D** via the question-slate cadence. User answered in `tasks/notes/v0.2_loops_final_decisions.md`:
   - **B-3**: open type pass-through with a 2-entry emitter translation table (`Sermo`→`String`, `Numerus`→`i64`); generics rejected with `ParseError::GenericsDeferred`.
   - **C-2**: defer lowering pass; parser→Ast direct; Stage 2 hook stays documented in ARCHITECTURE §7.8.
   - **D**: `//` line comments only; `/* */` reserved syntax with explicit deferred-feature error; `///` doc comments unscoped.
4. **PRD §4.12 amendment** + supporting doc updates landed in one parallel batch (ARCHITECTURE §11 η entry, lexicon Structural Punctuation table, `tasks/TODO.md` new `## v0.2 implementation plan` section with rounds R9–R14 ordered).
5. **Sentrux baseline.** `session_start` at signal 7079. `.sentrux/rules.toml` created with 20 architectural rules (2 constraints + 18 boundaries) encoding the pipeline DAG from ARCHITECTURE §§2,6,7,8. Free tier mechanically checks 4/20; the remaining 16 are documented intent. **`test_gaps` reports a misleading ~5% coverage** — tool-calibration artifact (sentrux's heuristic doesn't see inline `#[cfg(test)] mod tests`); actual coverage is 272 tests passing.
6. **R9 framing** → user approved without pushback.
7. **R9 implementation** in a single parallel-Write batch: `lexer/comments.rs` (new, 80 LOC + 7 tests), `lexer/cursor.rs` (`peek_at` + 3 tests), `lexer/error.rs` (`BlockCommentsDeferred` variant + 1 test), `lexer/mod.rs` (start-of-line + mid-line dispatch + 9 driver tests), 4 fixture files. Quality gates clean on first try (one `cargo fmt` nit auto-fixed). Sentrux `session_end`: signal_delta +3 (7079→7082), 0 cycles change, 0 coupling change, DSM `above_diagonal` stays 0, `check_rules` passes.
8. **R9 close docs**: ARCHITECTURE §12 (full round write-up), §0 reading-order table extended through R9, §11 η entry collapsed to one-liner pointer; `tasks/TODO.md` R9 checked off + completion entry appended.

### Test count at session close

**272 tests passing** workspace-wide (was 252 at v0.1/R8 close; +20 from R9). Per binary: marain-core unit 219, e2e_hello_world 1, emit_goldens 1, error_goldens 1, marain-cli unit 40, cli_e2e 10. `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.

### Sentrux at session close

| Metric | Baseline (R9 open) | R9 close |
|--------|--------------------|----------|
| Quality signal | 7079 | 7082 |
| Acyclicity score | 10000 | 10000 |
| DSM above_diagonal | 0 | 0 |
| Cycles | 0 | 0 |
| Rules pass | 4/20 enforced | 4/20 enforced |

`session_end` reports "Quality stable or improved."

## File State

### Added this session

- `.sentrux/rules.toml` — 20 architectural rules (pipeline DAG) encoding ARCHITECTURE §§2,6,7,8 invariants
- `crates/marain-core/src/lexer/comments.rs` — `scan_line_comment` + 7 unit tests
- `crates/marain-core/tests/fixtures/09_line_comments.lat` + `.expected.rs`
- `crates/marain-core/tests/fixtures/errors/06_block_comments_deferred.lat` + `.expected.txt`
- `tasks/notes/v0.2_loops_final_decisions.md` — user's B/C/D answers

### Modified this session

- `PRD.md` — new §4.12 (Comments) committing `//` line comments + `/* */` reserved-deferred + `///` unscoped
- `ARCHITECTURE.md` — §0 reading-order row added for R9, §11 η entry collapsed to one-liner pointer, new §12 (Line Comments, full round close — scope, decomposition, decisions, error variant, test coverage, sentrux signal, pressure-release, forward hooks)
- `docs/core-lexicon.md` — Structural Punctuation table gains `//` and `/*` rows
- `tasks/TODO.md` — new `## v0.2 implementation plan` section (R9–R14 ordered with B-3/C-2/D decisions inline + per-round open sub-decisions); R9 checked off; R9 completion entry appended to Completed
- `crates/marain-core/src/lexer/cursor.rs` — `peek_at(offset)` API + 3 unit tests
- `crates/marain-core/src/lexer/error.rs` — `BlockCommentsDeferred { span }` variant + 1 unit test
- `crates/marain-core/src/lexer/mod.rs` — start-of-line `//` peek (comment-only lines blank for indent); mid-line `/` dispatch (`//`→comment, `/*`→`BlockCommentsDeferred`, `/`→`UnexpectedChar`); 9 new driver tests

### Untouched but worth noting

- `crates/marain-core/src/parser/` — still v0.1 scope; will gain its first v0.2 surface in R10 (block parsing).
- `crates/marain-core/src/emit.rs` — still v0.1 scope; R10 may need block-aware emit; R11+R12 will exercise it heavily.
- `crates/marain-core/src/lexer/keywords.rs` — 37 keyword entries (unchanged this session; R9 added no keywords).
- `tasks/LESSONS.md` — still empty (no user corrections this session worth capturing).
- `tasks/BUGS.md` — still empty.
- `hello.lat` at repo root (untracked) — user's manual-test scratchpad, unchanged.

## What's Next (next session's entry point)

**R10 — Block parsing.** Per locked decision A, R10 ships alone. Every other v0.2 round depends on the parser consuming `Indent`/`Dedent` tokens (which the lexer has emitted since R4) and exposing a block-parsing API.

**Likely R10 framing slate (not yet committed):**

1. **Block-binding AST shape.** A `Block { stmts: Vec<Stmt>, span: Span }` newtype, or just `Vec<Stmt>` inline at each block-bearing parent? Recommend the newtype — Span carries the indented region, consumers don't recompute.
2. **Empty-block rule** (closes the open R10 sub-decision in TODO.md). `nihil.` required, or empty `Indent`/`Dedent` allowed? Recommend `nihil.` required (matches PRD §4.11.4 spirit). Empty-comment-only blocks are already a parse error per PRD §4.12.
3. **Test substrate without a parent construct.** R10 lands before R11 (control-flow heads) and R12 (functio). To exercise `parse_block` end-to-end before R11, either (a) test-only API exposure, or (b) bundle a minimal `if` head into R10 just for the substrate. Recommend (a) — keeps single-feature-per-round discipline.
4. **New `ParseError` variants.** Likely `ExpectedIndent`, `ExpectedDedent`, `EmptyBlock`. Confirm trio in R10 framing.
5. **Block nesting limit?** Marain's lexer already enforces well-formed Indent/Dedent pairing; parser doesn't need depth checks. Don't over-engineer.

**Other open sub-decisions** (each addressed in its own round, all noted in `tasks/TODO.md`):
- R11+R12: `else if` chain shape — single nested AST node vs two emitter-assembled (recommendation: single).
- R13: mandatory parens for zero-arg `functio foo() :` (recommendation: yes, per PRD §4.11.1).
- R13: unit return — `dat` clause omission already PRD-committed; confirm emit produces no `-> ()`.
- R14+R15: emit shape for `nihil.` — `();` vs `{}` (recommendation: `();`).

## Carry-over Concerns (status at session close)

| Concern | Status |
| ------- | ------ |
| (α) AST inflection slot | **RESOLVED** in R5 |
| (β) 500-LOC lexer | **RESOLVED** in R4; mod.rs at ~635 incl. tests after R9 — production code still well under target |
| (γ) `Variabile` runtime injection | **PINNED** for when Variabile literals enter the language |
| (δ) Hand-rolled CLI parsing | **RETIRED** 2026-05-23 (PRD §9 amended; clap pinned) |
| (ε) Test strategy | **RETIRED** in R8 |
| (ζ) Cross-file Stage 2 diagnostics | **PINNED**, future-only |
| (η) Comment syntax | **RETIRED** 2026-05-25 via PRD §4.12 + R9/§12 |
| (θ) Stage 2 `(lemma, inflection)` tokens | **PINNED** for Stage 2 |
| Workspace inheritance for shims | **RESOLVED** in R6 |

Three concerns remain pinned: γ (Variabile), ζ (cross-file Stage 2), θ (Stage 2 inflection tokens). All are post-v0.2.

## Decisions Locked This Session

For full rationale see `tasks/notes/v0.2_loops_final_decisions.md` and PRD §4.12.

| Topic | Decision |
| ----- | -------- |
| Round granularity (A) | Batched where dependencies overlap; one-feature-per-round otherwise. R10 solo; R11+R12 batch; R13 solo; R14+R15 batch. |
| Type system scope (B-3) | Open pass-through; emitter translation table (`Sermo`→`String`, `Numerus`→`i64`); generics rejected with `ParseError::GenericsDeferred`. |
| Lowering pass (C-2) | Deferred; parser→Ast direct; Stage 2 hook stays documented in ARCHITECTURE §7.8. |
| Comment syntax (D) | `//` line comments for v0.2; `/* */` reserved syntax with explicit deferred-feature `LexError`; `///` doc comments unscoped. |
| `.sentrux/rules.toml` adoption | 20 rules (2 constraints + 18 boundaries); free tier checks 4 mechanically; rest documented intent. Re-run `check_rules` per round. |
| ARCHITECTURE round numbering | R{n} → §{n+3} starting from R9 (R9=§12, R10=§13, …). §11 stays the cross-cutting Stage 2 forward-hooks accretion zone. |

## Collaboration Patterns (refined this session)

- **Question slate cadence proven for architecture too.** Same format worked for B/C/D as for v0.2 vocabulary in the prior session — sub-options + recommendation + rationale per question; user answered in `tasks/notes/v0.2_loops_final_decisions.md`. The `tasks/notes/` location matched user's referent this time (the previous session's misplacement was a one-time miss; the directory now exists).
- **PRD/lexicon/ARCH/TODO four-file batch for spec changes.** PRD §4.12 + ARCHITECTURE §11 η + lexicon Structural Punctuation + TODO v0.2 plan all landed in one parallel-write batch. Clean. Replicate for future spec changes.
- **Sentrux MCP per-round cadence.** `session_start` at round-open, `session_end` at round-close. Free tier mechanically enforces ~4 rules; the rest document architectural intent. DSM `above_diagonal` is the load-bearing canary for layering regressions.
- **Write-based full-file rewrites over Edit chaining for big touches.** R9's `lexer/mod.rs` had 4 separate insertion points; a single `Write` of the entire file (after a fresh `Read`) was faster and safer than 4 sequential `Edit` calls competing on file state. Reserve `Write` for files just-read fresh; `Edit` for surgical localized changes.
- **Task-tool nudge ignored** per session convention — the harness fires the reminder regularly; consistent to ignore.

## Tactical Notes

- Date: 2026-05-25.
- `hello.lat` at repo root (untracked, one line: `dic "salve, munde".`) is the user's manual-test scratchpad.
- Lexer keyword count: unchanged at 37 (R9 added no keywords; comments are punctuation/layout).
- `.sentrux/rules.toml` exists and `check_rules` passes. Re-run after each round (cheap, free).
- ARCHITECTURE §0 reading-order table now reaches Round 9 / §12.
- v0.2 implementation rounds R10–R15 are sketched in `tasks/TODO.md` under `## v0.2 implementation plan`. Read it first when opening R10.
- The `tasks/notes/` directory now exists (created when user landed `v0.2_loops_final_decisions.md`).
- `tasks/CONTINUITY.md` rewritten (not appended) per CLAUDE.md.
- No new bugs this session; `tasks/BUGS.md` still empty.
- No user corrections this session; `tasks/LESSONS.md` still empty.

## When You Resume

If user opens with "let's frame R10" or similar:

1. Read `tasks/TODO.md` first (esp. `## v0.2 implementation plan` for round ordering + open R10/R11/R13/R14 sub-decisions).
2. Read `tasks/notes/v0.2_loops_final_decisions.md` for B/C/D rationales.
3. Read PRD §§4.11–4.12 (likely cold after compact).
4. Run `sentrux session_start` to baseline before R10 code lands.
5. Open the R10 framing — start with the 5-point slate in this doc's "What's Next" section. Block parsing is a small but pivotal round: parser gains its first v0.2 surface, AST gains `Block` (or similar) node, `ParseError` gains 1–3 new variants. Estimate ~30–50 new tests; ARCHITECTURE §13 to be written at close.
6. Per CLAUDE.md round-closing protocol: cargo fmt + clippy + test --all; sentrux `session_start` / `session_end` bracket; ARCHITECTURE.md section drafted in conversation then committed; TODO.md round entry checked off + completion summary appended.

If user opens with anything else, be flexible — R10 isn't urgent and the doc state is coherent enough to support a different direction (more lexicon polish, manual-testing the new comment syntax on real `.lat` code, R11+R12 framing skipping ahead, etc.).
