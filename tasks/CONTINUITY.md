# Continuity — R18 (precedence-aware emit) shipped; v0.3 = type system (PRD pass next)

_Rewritten 2026-06-17 (R18 close). This session implemented and closed Task 3 as R18:
minimal-paren emit + a compiling control-flow e2e; then set the future direction to the
v0.3 type system. Code + tests + docs all landed and verified. NOT yet committed — owner
handles git. Next session: the v0.3 type-system PRD framing pass. Rewrite on next use._

## What Just Shipped — R18 (precedence-aware emit + control-flow e2e)

Emitted Rust is now idiomatic — no redundant parens — and the goldens-never-compile gap
is closed. Resolves TODO Task 3 and the ROADMAP §6 done-line e2e.

- **Minimal-paren emit (reverses ARCH §14 paren-everywhere).** `emit/expr.rs` wraps an
  operand only when Rust's precedence/associativity would re-parse it differently.
  Driven by `BinOp::rank() -> (precedence, associativity)` in `ast.rs`; `emit_operand` +
  `operand_needs_parens` + `regroups_at_equal_precedence` do the decision.
- **The trap (decision B):** the emit table follows **Rust's** grammar, NOT the parser
  cascade. Rust ranks all six relationals (`== != < > <= >=`) at ONE non-associative
  level (`a < b < c` is a syntax error); the parser is left-assoc + two-level. So
  `a minor quam b minor quam c` → `(a < b) < c` must keep parens, and
  `a aequat b minor quam c` → `a == (b < c)`. This is the one place §14's blanket was
  silently load-bearing. Covered by dedicated tests.
- **Exhaustive `rank()` (decision C):** no catch-all → a future operator can't compile
  until ranked. Compiler-enforced safety replacing brute-force parens. Sentrux flags
  `rank()` as a "complex function" (lookup table) — **accepted**, an exhaustive match is
  the correct form; net signal improved 7033 → 7035.
- **Compiling e2e** (`tests/e2e_control_flow.rs`): accumulator compiled with
  `RUSTFLAGS=-D warnings` AND run; asserts clean build (kills `unused_parens`) AND stdout
  `20` (proves `summa + i*2`, not `(summa+i)*2` — catches precedence miscompiles a
  build-only check can't).
- **Verified:** fmt / clippy -D warnings / `test --all` clean (**502 tests**, +10).
  `cargo doc` only the 2 pre-existing `marain-cli` intra-doc warnings. All goldens
  regenerated + eyeballed (6 fixtures: 12/13/14/15/20/27).
- **Archived:** ARCH §14.9 (+ §0 row, §14.3 bullet struck), `tasks/decisions/R18_precedence_aware_emit.md`
  (+ DECISIONS row), ROADMAP §6 e2e + Task 3 shipped, TODO Task 3 DONE.

## Immediate Next Action — PRD pass for the v0.3 type system

**Direction chosen by the owner 2026-06-17: the v0.3 type system** (option a of the
direction slate). The arc: user-defined types (`structura` / `enumeratio` / `modulus` /
`praestatio` / `proprietas`) → generics → stdlib type names (`Agmen` / `Vocabularium` /
`Fortasse` / `Eventus`). ROADMAP §2; depends on generics + collection literals.

**Entry point = a PRD framing pass** (CLAUDE.md: create/extend the PRD interactively, then
derive ARCHITECTURE from it). The type system is too large to jump straight to a round
spec — frame scope + ordering first. The same PRD pass resolves the still-open ROADMAP
"Triage pending" note: triage §1's "v0.3 candidates" pool into committed (→ ROADMAP) vs
uncommitted (→ BACKLOG) now that the direction is known.

Cleanup status (checked this session): `BACKLOG.md` clean; `TODO.md` Tasks 0/1/2/3 all
DONE (tracker empty, ready for the next round); the only loose end is the ROADMAP §1/§2
triage, which is coupled to — and falls out of — the PRD pass above.

Sequencing for the PRD pass: lead with the **decision-bearing forks** — (1) static
user-defined types vs. also pulling in the dynamic `Variabile` wrapper (γ, ROADMAP §3);
(2) how much generics surface to commit to up front (the lexer currently hard-errors on
`<`/`>` via `GenericsLookalike` — activation flips that arm); (3) visibility (`publicus`)
+ modules (`modulus`) ordering. Use **ASCII labels (a/b/c)** for framing slates.

## Watch-outs (carry into next round)

- **emit.rs / emit/expr.rs both have headroom.** `emit/expr.rs` now ~190 LOC (added
  `expr_precedence` + 3 paren helpers); `emit.rs` 436. New expr arms → `emit/expr.rs`,
  stmt arms → `emit.rs`. Escapers + `EmitError` in `emit.rs`, reached via `super::`.
- **Test files over target (justified, sibling-split is the move):**
  `parser/mod_tests.rs` (~1550), `emit_tests.rs` (935 — grew +R18), `lexer/mod_tests.rs`
  (553). All carry doc-comment justifications.
- **`rank()` complexity flag** is accepted noise (lookup table). Don't "fix" it by
  de-matching — that loses compiler-enforced exhaustiveness. If a future op lands, add it
  to the single `rank()` match.
- **Future paren hard case:** closures (`|x| body`, Rust's lowest greedy-right
  precedence) will be the genuinely tricky minimal-paren case when they land. The
  exhaustive `rank()` match forces us to think about it rather than silently miscompile.

## Where We Are (state)

**Marain is a single Latin-core language** (multi-frontend rejected, ADR-0001). v0.2
feature-complete R9–R16; R17 added f-strings; R18 made emit idiomatic + added the
compiling e2e. Pipeline: `.lat → lexer → tokens → parser → AST → emit → Rust → shim
(cargo) → run`. **502 tests.** Only external dep is `clap` (pinned).

## Commit State — R18 NOT yet committed ⚠️

R18 work is complete and verified but **uncommitted** (owner handles all git/CI — surface
+ recommend, never stage/commit). Changed/new files this round:
- `crates/marain-core/src/ast.rs` (+`Associativity`, `BinOp::rank/precedence/associativity`)
- `crates/marain-core/src/emit/expr.rs` (minimal-paren emit + helpers)
- `crates/marain-core/src/ast_tests.rs`, `emit_tests.rs` (precedence/assoc/trap tests)
- `crates/marain-core/tests/e2e_control_flow.rs` (NEW)
- 6 regenerated goldens under `crates/marain-core/tests/fixtures/`
- docs: ARCHITECTURE.md, tasks/DECISIONS.md, tasks/decisions/R18_precedence_aware_emit.md
  (NEW), tasks/ROADMAP.md, tasks/TODO.md, tasks/CONTINUITY.md
Recommended commit message: `feat: R18 — precedence-aware (minimal-paren) emit + compiling control-flow e2e`.

## Open Decisions

- **v0.3 direction: CHOSEN = type system** (2026-06-17). Next: PRD framing pass (see
  Immediate Next Action), which also discharges the ROADMAP §1/§2 triage.
- **v0.3 type-system scope still unframed** — frame from PRD §4.3/§4.11 + ARCH §15.8;
  forks listed in Immediate Next Action.
- **E1 leak fixes** parked in BACKLOG; only leak 2 (`unreachable!` → `EmitError`) has
  standalone single-language value.

## Carry-over Concerns

Unchanged, Stage-2 / post-v0.2: **γ (Variabile)** (ROADMAP §3); **ζ** cross-file Stage-2
diagnostics; **θ** Stage-2 inflection tokens. R18 added a future note: closure paren
emit (see Watch-outs).

## When You Resume

1. **Commit check** — R18 is complete but UNCOMMITTED; tree is dirty. Don't
   `git add`/commit yourself (owner handles git/CI). Confirm with the owner it landed.
2. **Start the v0.3 type-system PRD pass** (direction chosen 2026-06-17). Lead with the
   decision-bearing forks in Immediate Next Action; extend `PRD.md` interactively, then
   derive ARCHITECTURE. Triage ROADMAP §1/§2 as part of the same pass.
3. Once the PRD scopes a first round, enter plan mode to frame it.
4. Round-close ritual (CLAUDE.md §7): sentrux `session_start` baseline BEFORE code; on
   close → decisions file + DECISIONS row + ARCH §N + ROADMAP + check off TODO + rewrite
   this file. Use **ASCII labels (a/b/c)** for any framing slate.

## Tactical Notes

- Date 2026-06-17. Project renamed Rubigo → Marain (repo dir still `rubigo`).
- Doc convention (load-bearing): **ROADMAP = committed, BACKLOG = uncommitted.**
- Golden harnesses auto-collect fixtures; regenerate with `MARAIN_UPDATE_GOLDENS=1` then
  eyeball the `.expected.rs`/`.expected.txt` diff.
- CLI: `marain run <file.lat>` transpiles + executes; `cargo run -p marain-cli -- run …`
  during dev. R18 e2e program is inline in `tests/e2e_control_flow.rs`.
