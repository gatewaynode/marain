# Continuity — post-v0.1 manual validation, v0.2 vocabulary locked

_Rewritten 2026-05-25 end-of-session. Captures: MIT license addition, equality-operator split (`aequat`), `docs/core-lexicon.md` creation + expansion, v0.2 vocabulary scoping + lexer keyword reservation. Rewrite on next use._

## Where We Are

v0.1 is shipping and manually validated (user ran `hello.lat` end-to-end against the README examples). v0.2 vocabulary is fully scoped and committed across PRD / lexicon / lexer. No v0.2 implementation work has begun — that's the next session's entry point.

**Session arc (chronological):**

1. **MIT license added.** `LICENSE` at repo root (copyright `gatewaynode`); `license = "MIT"` in `[workspace.package]`; `license.workspace = true` in both crates; `README.md` License section updated.
2. **Equality-operator split** (carry-over PRD §4.4 design footgun). Previously `est` was overloaded for both `=` (init) and `==` (equality) with position-based disambiguation. User flagged the readability concern (`sit ^valid est ^answer est 42.` parses but reads jarringly). Resolved: new keyword `aequat` (3rd-sg. present indicative active of `aequare`, "equals") for `==`; `non aequat` replaces `non est` for `!=`. PRD §§4.2/4.4 amended; lexicon updated; lexer reserves `Aequat`.
3. **Lexicon doc created.** User drafted `docs/core-lexicon.md` stub; I expanded to a complete spec covering sigils, all Marain keywords (organized by purpose), multi-word operator phrases, Rust-keyword boundary handling (mapped / auto-escaped / unescapable), structural punctuation, stdlib type proposals, numeric and string literals. Latent comment-typo in `lexer/keywords.rs` (`Maior`/`Minor`/`Par` annotations were wrong per PRD §4.4) fixed as a side effect.
4. **v0.2 vocabulary scoping** (this session's main work). I drafted an 11-question slate covering function syntax (parameter shape, return-type indicator, deferred features), loop vocabulary (`loop`/`break`/`continue`/`for`/range), and block syntax (head terminator, function-body intro, empty block). User answered in `tasks/questions_and_answers_1.md`. Decisions committed across:
   - **PRD.md**: §4.2 (imperative-mood row updated — `interrumpe!` → `interrumpe`, `continua` added); §4.8 (control-head terminator locked to `:`, S1-2 leftover resolved); new §4.11 "Control Flow & Functions (v0.2)" with six subsections (function decls / conditional+loop heads / control transfer / empty block / range syntax / out-of-scope); §11 (S1-2 entry updated).
   - **docs/core-lexicon.md**: 7 new entries in Control flow & declarations table; new "Structural Punctuation" section; range-Latinization rejection note (L-5).
   - **crates/marain-core/src/lexer/keywords.rs**: 7 new `Keyword` variants reserved (`Dat`, `Semper`, `Interrumpe`, `Continua`, `In`, `Nihil`, `Aliter`). Round-trip test mechanically covers them.

**Eight new Marain keywords total this session**: `aequat`, `dat`, `semper`, `interrumpe`, `continua`, `in`, `nihil`, `aliter`. Lexer's keyword table now has **37 entries** (was 29 at v0.1/R4 close).

### Test counts at session close

Unchanged from v0.1 close — **252 tests pass**, `cargo fmt --check` and `cargo clippy --all-targets -D warnings` clean. No new tests added (all changes were doc + keyword reservations; existing `round_trip_all_keywords` test covers new entries mechanically).

## File State

### Added this session

- `LICENSE` — MIT, copyright 2026 `gatewaynode`
- `docs/core-lexicon.md` — ~270 lines after all expansions
- `tasks/questions_and_answers_1.md` — user's answers to the v0.2 vocab slate

### Modified this session

- `Cargo.toml` + both crate `Cargo.toml`s — `license = "MIT"` / `license.workspace = true`
- `README.md` — License section
- `PRD.md` — §4.2 (imperative row), §4.4 (equality split), §4.8 (`:` for control heads), §11 (S1-2 resolved), new §4.11 (v0.2 spec)
- `crates/marain-core/src/lexer/keywords.rs` — `Maior`/`Minor`/`Par` comment fixes + 8 new `Keyword` variants

### Untouched but worth noting

- `ARCHITECTURE.md` — still describes v0.1 (§§1–10 closed); needs v0.2 implementation rounds added when implementation starts.
- `tasks/TODO.md` — all 8 v0.1 rounds checked off; no v0.2 entries yet (planning is next session).
- `tasks/LESSONS.md` — still empty.
- `tasks/BUGS.md` — still empty.
- `crates/marain-core/src/parser/` — no parser support for any v0.2 keywords; writing `functio` or `dum` today tokenizes correctly but parse-fails.
- `crates/marain-core/src/emit.rs` — no emit support for any v0.2 construct.
- `..` / `..=` range tokens documented in lexicon but NOT YET in `token.rs` / lexer; defer until parser needs them (likely R13).
- `hello.lat` at repo root (untracked) — user's manual-test artifact.

## What's Next (next session's entry point)

User stated: "we can plan out the tasks to implement." This is **v0.2 implementation planning**.

Likely shape — needs the same round-by-round approach that worked for v0.1. Rough sketch (NOT yet PRD/TODO-committed; ordering TBD by user):

1. **R9 — block parsing.** Lexer already emits `Indent`/`Dedent`; parser doesn't consume them. Make `parse_block(&[Token]) -> Vec<Stmt>` work. Closes the v0.1-to-v0.2 gap that every other v0.2 feature depends on.
2. **R10 — operator expressions.** Precedence climbing for `plus` / `minus` / `per` / `modulo` / `aequat` / `non aequat` / `maior quam` / etc. + `et` / `vel` / `non`. Needed for loop conditions, function bodies of any value, and to make `aequat` real.
3. **R11 — control flow** (`si` + `aliter` + `dum` + `semper` + `interrumpe` + `continua`). Builds on R9 + R10.
4. **R12 — functions** (`functio` + `dat` + `redde` + parameters). Multi-statement bodies. Forces `emit_stmt` depth tracking (ARCHITECTURE.md §8.10 forward hook).
5. **R13 — `pro` + ranges** (`..` / `..=`). New lexer tokens (`DotDot`, `DotDotEq`). Iterable parsing.
6. **R14 — `nihil`** + minimal type system. `nihil` is one statement (mechanical). Types: at minimum need a few named tokens for function signatures.

**Open architectural questions for the round-1 framing conversation:**

- **A. Round granularity.** Single combined rounds vs. split? R11 and R12 both need blocks; R10 and R11 both need expressions. Natural batching opportunities — or keep tight one-feature-per-round per CLAUDE.md.
- **B. Type system scope for v0.2.** Minimal alphabet (`Sermo` + `Numerus`)? Or "any identifier in type position passes through; rustc rejects what doesn't resolve"? Or defer types entirely and have v0.2 functions all infer (`functio foo() :`, no annotations)? PRD §4.11.1 commits to `dat <Tipus>` syntax, so types need at least placeholder grammar.
- **C. Lowering pass between parser and AST?** PRD §4.10 + §4.11 anticipate a Stage-1 AST that emit can chew directly. Some v0.2 constructs lower naturally (`aliter si` → `else if` chain); some might want desugaring (`pro x in 0..10` → Rust's `for x in 0..10`). Decide whether to interpose.
- **D. Comment syntax (η, still open).** User hasn't hit it in manual testing yet, but writing real `.lat` files for v0.2 functions without comments will get painful. Likely blocking once R10/R11 opens. PRD amendment is the gating step.

These are questions for the next session's planning conversation, not work to do now.

## Carry-over Concerns (status at session close)

| Concern | Status |
| ------- | ------ |
| (α) AST inflection slot | **RESOLVED** in R5 |
| (β) 500-LOC lexer | **RESOLVED** in R4; still within target after v0.2 keyword adds |
| (γ) `Variabile` runtime injection | **PINNED** for when Variabile literals enter the language |
| (δ) Hand-rolled CLI parsing | **RETIRED** 2026-05-23 (PRD §9 amended; clap pinned) |
| (ε) Test strategy | **RETIRED** in R8 |
| (ζ) Cross-file Stage 2 diagnostics | **PINNED**, future-only |
| (η) **Comment syntax** | **OPEN** — likely surfaces during v0.2 implementation; PRD amendment is the gating step |
| (θ) Stage 2 `(lemma, inflection)` tokens | **PINNED** for Stage 2 |
| Workspace inheritance for shims | **RESOLVED** in R6 |

No new concerns surfaced this session. The `aequat` split was a v0.1 PRD-design self-correct (no new architectural commitment beyond what §4.4 already had); the v0.2 vocabulary work added words but not constraints.

## Decisions Locked This Session

For full rationale see `tasks/questions_and_answers_1.md` and PRD §4.11.

| Topic | Decision |
| ----- | -------- |
| Equality operator | `aequat` (`==`), `non aequat` (`!=`). `est` is `=` (init) only. |
| Function parameter list | Rust-style `(^nomen: Tipus, ...)` parens, comma-separated, nominative case (Stage 1) |
| Function return type | `dat <Tipus>` (3rd-sg. indicative active of `dare`); omit for unit return |
| Closures / generics / `pub` | Deferred to v0.3+ |
| Infinite loop | `semper :` (adverb-exception to §4.2 verb-mood pattern) |
| Break | `interrumpe.` (imperative; no `!`) |
| Continue | `continua.` (user's call over my `perge` rec — easier English-speaker onboarding) |
| For-binding word | `in` (`pro ^x in ^xs :`) |
| Range syntax | Keep Rust's `..` / `..=`. Latin alternatives logged in lexicon for future revisit. |
| Block-head terminator | `:` (Python-style). Covers `si` / `aliter` / `dum` / `pro` / `semper` / `functio` heads. |
| Empty block | `nihil.` (Python `pass` in Latin) |
| Else / else-if | `aliter :` / `aliter si <cond> :` (adverb-exception per `semper`) |
| License | MIT, copyright `gatewaynode`; pinned via workspace inheritance |
| Lexicon location | `docs/core-lexicon.md`; single doc covers sigils + Marain keywords + Rust-boundary handling + structural punctuation |

## Collaboration Patterns (refined this session)

- **Spec-first cadence for vocabulary changes.** Decide the words via question slate → sync PRD + lexicon + lexer in one batch → run gates → done. Three small files, mechanical edits, ~5 min per pass. Locked across both `aequat` (4-edit pass) and the v0.2 vocab (12-edit + 8-edit two-pass).
- **Question slate format works for multi-decision rounds.** 11 questions with options + my recommendation + one-line rationale each. User answered in a markdown file; one diverged from my rec (`continua` vs `perge`), one added a requirement (L-5 documenting alternatives). Faster than per-question `AskUserQuestion` for this many decisions.
- **PROPOSED items get picked up.** When I left `aliter` as PROPOSED in §4.11.2 at end of the first v0.2 pass, user locked it immediately rather than letting it drift. Surfacing > silently-deferring.
- **`tasks/notes/` doesn't exist.** User referenced `tasks/notes/questions_and_answers_1.md` but the file landed at `tasks/questions_and_answers_1.md` (no `notes/` subdir). Check both on first reference.
- **Sentrux MCP review** per CLAUDE.md still has NOT been run any session. Running it before R9 opens (baselining complexity at end of v0.1 + vocabulary spec) is the natural moment.
- **Quality gate cadence** — `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --all` after every commit-boundary change. Verified three times this session.
- **Task-tool nudge ignored** per session convention.

## Tactical Notes

- Date is 2026-05-25.
- `hello.lat` at repo root (untracked, one line: `dic "salve, munde".`) is the user's manual-test scratchpad.
- Lexer keyword count: 37 (29 v0.1 + `aequat` + 6 v0.2 batch + `aliter`). Historical "29 entries" claims in TODO.md / older notes remain unchanged — they describe what shipped at R4 close.
- `LICENSE` is canonical SPDX MIT text with `Copyright (c) 2026 gatewaynode`.
- Working dir still `/Users/john/code/rubigo/`; Claude Code memory dir still `~/.claude/projects/-Users-john-code-rubigo/`. Not renamed despite the Rubigo→Marain project rename.
- `tasks/CONTINUITY.md` rewritten (not appended) per CLAUDE.md.
- `tasks/questions_and_answers_1.md` is the first such doc — future rounds may follow `_2.md`, `_3.md`, etc. or be consolidated.
- No new tests added — vocabulary changes are covered by existing `round_trip_all_keywords`. v0.2 implementation rounds will add per-feature tests.

## When You Resume

If user opens with "let's plan v0.2 implementation":

1. Read PRD §4.11 first (new content; may not be cached after compact).
2. Re-read `tasks/questions_and_answers_1.md` for decision rationales.
3. Open the v0.2 round-framing conversation. Block parsing (R9) is the obvious first move — everything depends on it.
4. Single-feature-per-round per CLAUDE.md "one feature per session" guidance (PRD §10 risk mitigation).
5. ARCHITECTURE.md §11 will gain new round headings; the §0 reading-order table will extend.
6. Consider running sentrux MCP for complexity baseline before opening R9.

If user opens with anything else (manual testing more features, comment syntax decision, lexicon polish), be flexible — the plan above is one path, not the only.
