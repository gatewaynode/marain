# Continuity — R17 (f-strings) shipped; choose the next round

_Rewritten 2026-06-17 (R17 close). This session implemented and closed f-strings
(TODO Task 1) as R17. Code + tests + docs all landed and verified. Next session:
pick the next round (candidates below). Rewrite on next use._

## What Just Shipped — R17 (f-strings)

String composition now exists. `f"salve {^nomen}"` lowers to `format!("salve {}", nomen)`;
concatenation is the all-holes form `f"{^a}{^b}"` → `format!("{}{}", a, b)`. One
mechanism for both — no concat operator (PRD §4.7). Resolves TODO Task 1.

- **Owner-locked scope:** holes are **variable-refs-only** (`{^x}` / `{@x}`). Empty /
  no-sigil / expression holes and Rust format specs are `InvalidFStringHole`.
- **One-pass lexer:** `scan_fstring` resolves each hole inline via `scan_sigiled_ident`
  (correct spans + `FileId`, no parser sub-lexing, no lexer mode-state). Internal
  `{`/`}` never reach the main dispatch, so they don't perturb indent/bracket state.
  Prefix is `f"` with no space (unambiguous: variables carry sigils, calls need `(`).
- **New shapes:** `TokenKind::FStringLit(Vec<FStringSeg>)` (lexer, holes pre-resolved),
  `Expr::FString(FStringLit{ parts: Vec<FStringPart> })`; parser does a pure 1:1 lift.
- **emit.rs split** (discharges the R16 watch-out): expression emitters →
  `emit/expr.rs` (129); `emit.rs` back to 436. Also moved `lexer/mod.rs` driver tests
  to sibling `lexer/mod_tests.rs` (mod.rs 264 ✓).
- **Verified:** fmt / clippy -D warnings / `test --all` clean (**492 tests**, +31).
  Sentrux improved (7057 → 7033, 0 violations). e2e through `marain run` prints
  `Salve, Munde!` / `Concat: SalveMunde` / `Numerus est 42.`.
- **Archived:** ARCH §18 (+ §0 row, §17.7 marked resolved), `tasks/decisions/R17_fstrings.md`
  (+ DECISIONS row), PRD §4.6 footnote, lexicon update, ROADMAP §4 row shipped + Task 1
  struck, TODO Task 1 marked DONE.

## Immediate Next Action — frame the next round

No round is in flight. v0.2 feature-complete (R9–R16); R17 was the first v0.3-era
language feature. Candidates, roughly by leverage:

1. **Task 3 — `unused_parens` + v0.2 done-line e2e** (`tasks/TODO.md`, ROADMAP §6).
   Paren-everywhere emit (ARCH §14) warns on `if`/`while` conds, `let`/`redde`/`fit`
   RHS. Two fixes in TODO: (a) precedence-aware emit [elegant, reverses §14], (b)
   outermost-strip [surgical]. Pairs with a compiling e2e (goldens are string-compare
   only — they never compile their output). **Note:** f-string emit does NOT add new
   `unused_parens` sites (format args aren't paren-wrapped), so R17 didn't worsen this.
2. **More f-string surface** — expression holes (`{^a plus ^b}`) and/or format specs
   (`{x:>5}`). Widen `FStringPart::Interp` from `SigiledIdent` to `Expr` + give the
   lexer brace-balanced sub-lexing (or parser-side hole parse). Owner deferred these
   in R17; pull in only on request.
3. **v0.3 framing** — type system / user-defined types (ROADMAP §2), `Variabile` (γ,
   ROADMAP §3), or triple-quoted strings (ROADMAP §4). Needs a PRD pass first.

Recommendation: **#1** — closes the only warning Marain emits and the goldens-never-
compile gap, small and self-contained. Owner's call.

## Watch-outs (carry into next round)

- **emit.rs=500 watch-out is RESOLVED** (split into `emit.rs` 436 + `emit/expr.rs` 129).
  New emit arms go in `emit/expr.rs` (expressions) or `emit.rs` (statements); both have
  headroom. Escapers + `EmitError` live in `emit.rs`, reached from the child via `super::`.
- Test files over target with justifications: `parser/mod_tests.rs` (~1550),
  `emit_tests.rs` (~870), `lexer/mod_tests.rs` (553). Sibling-split is the established
  move; all carry doc-comment justifications.

## Where We Are (state)

**Marain is a single Latin-core language** (multi-frontend rejected, ADR-0001). v0.2
feature-complete R9–R16; R17 added f-strings. Pipeline:
`.lat → lexer → tokens → parser → AST → emit → Rust → shim (cargo) → run`. **492 tests.**
Only external dep is `clap` (pinned).

## Commit State — R17 NOT yet committed ⚠️

R17 is complete in the working tree but **uncommitted**. Owner handles all git/CI —
surface + recommend, never stage/commit. Suggested commit grouping when the owner is
ready:
- **Source:** `crates/marain-core/src/{token.rs, ast.rs, lexer/mod.rs, lexer/strings.rs,
  lexer/error.rs, lexer/mod_tests.rs (new), parser/expressions.rs, emit.rs,
  emit/expr.rs (new), ast_tests.rs, emit_tests.rs, parser/mod_tests.rs}`.
- **Fixtures (new):** `tests/fixtures/{28_fstring_interpolation, 29_fstring_concat}.{lat,expected.rs}`
  + `tests/fixtures/errors/{18_fstring_empty_hole, 19_fstring_expression_hole}.{lat,expected.txt}`.
- **Docs:** `ARCHITECTURE.md`, `PRD.md`, `docs/core-lexicon.md`, `tasks/{ROADMAP.md,
  TODO.md, DECISIONS.md, CONTINUITY.md}`, `tasks/decisions/R17_fstrings.md` (new).
- ⚠️ Don't forget `git add` for the **new** files (lexer/mod_tests.rs, emit/expr.rs,
  the 4 fixtures+2 expected, R17_fstrings.md) — a tracked-only commit will miss them
  (the R16 lesson). `tasks/LESSONS.md` may still be uncommitted from R16.

## Open Decisions

- **Next round: UNCHOSEN** (see candidates above).
- **v0.3 still largely unframed** — frame from ARCH §16.8/§17.8/§18.7 + PRD §4.3/§4.11.
- **E1 leak fixes** parked in BACKLOG; only leak 2 (`unreachable!` → `EmitError`) has
  standalone single-language value.

## Carry-over Concerns

Unchanged, Stage-2 / post-v0.2: **γ (Variabile)** (ROADMAP §3); **ζ** cross-file
Stage-2 diagnostics; **θ** Stage-2 inflection tokens (the `SigiledIdent` inflection
slot in f-string holes already carries α forward).

## When You Resume

1. **Commit check** — confirm the owner committed R17 (incl. the new files listed
   above). Don't `git add`/commit yourself (owner handles git/CI).
2. **Pick a round** from "Immediate Next Action"; enter plan mode to frame it.
3. Round-close ritual (CLAUDE.md §7): sentrux `session_start` baseline BEFORE code;
   on close → decisions file + DECISIONS row + ARCH §N + ROADMAP + check off TODO +
   rewrite this file. Use **ASCII labels (a/b/c)** for any framing slate.

## Tactical Notes

- Date 2026-06-17. Project renamed Rubigo → Marain (repo dir still `rubigo`).
- Doc convention (load-bearing): **ROADMAP = committed, BACKLOG = uncommitted.**
- Golden harnesses auto-collect fixtures; regenerate with `MARAIN_UPDATE_GOLDENS=1`
  then eyeball the `.expected.rs`/`.expected.txt` diff.
- CLI: `marain run <file.lat>` transpiles + executes; `cargo run -p marain-cli -- run …`
  during dev. f-string e2e scratch file was `/tmp/r17_e2e.lat`.
