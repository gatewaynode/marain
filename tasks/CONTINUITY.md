# Continuity — R16 (`fit` reassignment) shipped; choose the next round

_Rewritten 2026-06-16 (R16 close). Last session scoped Task 0; this session
**implemented and closed it as R16**. Code + tests + docs all landed and verified.
Next session: pick the next round (candidates below). Rewrite on next use._

## What Just Shipped — R16 (`fit` reassignment)

The binding lifecycle's missing half is now complete: a declared mutable binding
can be re-bound. `@x fit <expr> .` parses and emits `x = <expr>;`.

- **Five edit sites landed** (all green): `ast.rs` (`Stmt::Assign(AssignStmt)`),
  `parser/grammar.rs` (`parse_assign` + `SigiledIdent` dispatch), `parser/error.rs`
  (`ImmutableReassignmentTarget`), `emit.rs` (`emit_assign`, **no `mut`**), plus
  tests across `ast_tests.rs` / `mod_tests.rs` / `emit_tests.rs` and 2 golden
  fixtures (`27_fit_reassignment`, `errors/17_fit_immutable_target`).
- **Locked decision honored:** `@` target required; `^` target is a parse error.
- **Verified:** `cargo fmt --all` / `clippy --all-targets -D warnings` /
  `test --all` all clean (**461 tests**, +11). Sentrux stable (signal 7063 → 7057,
  improved; 0 violations). **e2e:** a `pro`/`fit` accumulator run through
  `marain run` prints `15` — emitted Rust compiles and executes.
- **Archived:** ARCH §17 (+ §0 reading-order row), `tasks/decisions/R16_fit_reassignment.md`
  (+ DECISIONS index row), PRD line-115 footnote, ROADMAP §1 row marked shipped.

## Immediate Next Action — frame the next round

No round is in flight. v0.2 is now genuinely feature-complete (R9–R16). Candidates,
roughly ordered by leverage:

1. **v0.2 done-line e2e + commit** (ROADMAP §6, near-term, NOT v0.3-gated). The
   goldens are string-compare only — they never compile their output. A test that
   runs emitted control-flow/`fit` Rust through `cargo build -D warnings` would have
   caught Task 3 and guards future emit regressions. **Pairs naturally with Task 3.**
2. **Task 3 — `unused_parens`** (`tasks/TODO.md`). Paren-everywhere emit (ARCH §14)
   warns on `if`/`while` conds, `let`/`redde` values, and — confirmed in R16 — `fit`
   assignment RHS (`x = (x + 1i64);`). Two fix options in TODO: (a) precedence-aware
   emit [elegant, reverses §14], (b) outermost-strip [surgical]. Either regenerates
   all goldens via `MARAIN_UPDATE_GOLDENS=1`.
3. **v0.3 framing** — type system / user-defined types (ROADMAP §2), f-strings
   (Task 1 / ROADMAP §4), or `Variabile` (γ). Unframed; needs a PRD pass first.

Recommendation: **#1 + #2 together** — small, closes a real testing gap, and clears
the only warning Marain currently emits. Owner's call on ordering.

## Watch-outs (carry into next round)

- **`emit.rs` is at exactly 500 LOC** (the target ceiling). The NEXT emit-arm
  addition must split it (e.g. `emit/{stmt,expr}.rs`) or carry a module-doc
  pressure-release justification. This bites whoever does Task 3 or any v0.3 emit
  work. Flagged in ARCH §17.2 / §17.7.
- Test files already over target with justifications: `parser/mod_tests.rs` (~1490),
  `emit_tests.rs` (~830), `lexer/mod.rs` (749). Sibling-split pattern (`#[path]`)
  is the clean move if any grows further.

## Where We Are (state)

**Marain is a single Latin-core language** — multi-frontend initiative rejected
2026-06-16 (ADR-0001). v0.2 feature-complete R9–R16; lexer→parser→AST→emit→goldens
end to end. **461 tests.** Only external dep is `clap` (pinned). Pipeline:
`.lat → lexer → tokens → parser → AST → emit → Rust → shim (cargo) → run`.

## Commit State — ⚠️ R16 PARTIALLY committed (5 files still untracked)

R16 tracked edits are committed and **pushed** as `2eb497a` ("Assignment and
reassignment implementation 'fit'"), on `main`, in sync with `origin/main`.

**BUT 5 new files were NOT in that commit — still untracked, owner must `git add`:**
- `crates/marain-core/tests/fixtures/27_fit_reassignment.lat`
- `crates/marain-core/tests/fixtures/27_fit_reassignment.expected.rs`
- `crates/marain-core/tests/fixtures/errors/17_fit_immutable_target.lat`
- `crates/marain-core/tests/fixtures/errors/17_fit_immutable_target.expected.txt`
- `tasks/decisions/R16_fit_reassignment.md`

Without these, a fresh checkout loses the `fit` golden fixtures and the decision
archive that ARCH §17 / DECISIONS index link to. `2eb497a` is pushed, so this is a
**follow-up commit** (not an amend). **Owner handles all git/CI** — do not stage or
commit; just remind. (The R16 doc/source edits in `2eb497a` are safe and complete;
only these 5 additions are outstanding.)

## Open Decisions

- **Next round: UNCHOSEN** (see candidates above).
- **v0.3 unframed** — frame from `ARCH §16.8`/`§17.8` + `PRD §4.11`/§4.3 when ready.
- **E1 leak fixes** parked in BACKLOG; only leak 2 (`unreachable!` → `EmitError`) has
  standalone single-language value.

## Carry-over Concerns

All Stage-2 / post-v0.2, unchanged: **γ (Variabile)** plain roadmap item (ROADMAP §3);
**ζ** cross-file Stage-2 diagnostics; **θ** Stage-2 inflection tokens.

## When You Resume

1. **Commit check** — R16 is committed as `2eb497a` (pushed), but **5 files are still
   untracked** (see "Commit State" above). If `git status` still shows them, remind the
   owner — do NOT `git add`/commit yourself (owner handles all git/CI).
2. **Pick a round** from "Immediate Next Action"; enter plan mode to frame it.
3. **Mind the `emit.rs` 500-LOC ceiling** before adding any emit arm.
4. Round-close ritual (CLAUDE.md §7): sentrux `session_start` baseline BEFORE code;
   on close → decisions file + DECISIONS row + ARCH §N.3 + ROADMAP + check off TODO +
   rewrite this file. Use **ASCII labels (a/b/c)** for any framing slate.

## Tactical Notes

- Date 2026-06-16. Project renamed Rubigo → Marain (repo dir still `rubigo`).
- Doc convention (load-bearing): **ROADMAP = committed, BACKLOG = uncommitted.**
- Golden harnesses auto-collect fixtures; regenerate with `MARAIN_UPDATE_GOLDENS=1`
  then eyeball the `.expected.rs`/`.expected.txt` diff.
- CLI: `marain run <file.lat>` transpiles + executes; `marain build` emits the shim
  project path. `cargo run -p marain-cli -- run …` during dev.
