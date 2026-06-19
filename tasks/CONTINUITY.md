# Continuity — R18 shipped+committed; next = v0.3 type-system PRD pass (unchanged)

_Rewritten 2026-06-19. This session was a `/catchup` re-orient + one doc-only task
(converted `docs/keywords.html` → `docs/keywords.md`). No code, no round work — the v0.3
direction below is carried forward verbatim from the R18-close rewrite. New loose end: two
uncommitted docs files (see Commit State). Rewrite on next use._

## This Session (2026-06-19) — doc-only delta

- **`docs/keywords.html` → `docs/keywords.md`** (NEW, uncommitted). Clean markdown of the
  Rust Reference "Keywords" page: strict / reserved / weak, all three edition callouts
  (2018 `async`/`await`/`dyn`; 2018 `try`; 2024 `gen`; 2015→2018 `dyn` promotion), the
  `compile_fail` example with HTML entities decoded. Stripped mdBook scaffolding
  (`[lex.keywords.*]` rule anchors, `<wbr>`, dead relative links). The source `.html`
  remains on disk; owner not yet asked to delete it.
- **Relevance to v0.3:** this is the authoritative Rust keyword inventory — directly useful
  when framing the Latin keyword mapping for user-defined types (`structura`/`enumeratio`/
  `modulus`/`praestatio`/`proprietas` vs. Rust `struct`/`enum`/`mod`/`trait`/`...`). Pull it
  in during the PRD pass to confirm no Marain keyword collides with a reserved Rust word at
  emit time.
- Nothing else touched. **502 tests** still the baseline (not re-run this session — no code
  changed).

## Immediate Next Action — PRD pass for the v0.3 type system (UNCHANGED)

**Direction chosen by the owner 2026-06-17: the v0.3 type system** (option a of the
direction slate). The arc: user-defined types (`structura` / `enumeratio` / `modulus` /
`praestatio` / `proprietas`) → generics → stdlib type names (`Agmen` / `Vocabularium` /
`Fortasse` / `Eventus`). ROADMAP §2; depends on generics + collection literals.

**Entry point = a PRD framing pass** (CLAUDE.md: create/extend the PRD interactively, then
derive ARCHITECTURE from it). The type system is too large to jump straight to a round
spec — frame scope + ordering first. The same PRD pass resolves the still-open ROADMAP
"Triage pending" note: triage §1's "v0.3 candidates" pool into committed (→ ROADMAP) vs
uncommitted (→ BACKLOG) now that the direction is known.

Sequencing for the PRD pass: lead with the **decision-bearing forks** — (1) static
user-defined types vs. also pulling in the dynamic `Variabile` wrapper (γ, ROADMAP §3);
(2) how much generics surface to commit to up front (the lexer currently hard-errors on
`<`/`>` via `GenericsLookalike`; activation flips that arm); (3) visibility (`publicus`)
+ modules (`modulus`) ordering. Use **ASCII labels (a/b/c)** for framing slates.

Cleanup status (as of R18 close): `BACKLOG.md` clean; `TODO.md` Tasks 0/1/2/3 all DONE
(tracker empty, ready for the next round); the only loose end is the ROADMAP §1/§2 triage,
which is coupled to — and falls out of — the PRD pass above.

## Watch-outs (carry into next round)

- **emit.rs / emit/expr.rs both have headroom.** `emit/expr.rs` ~190 LOC (R18 added
  `expr_precedence` + 3 paren helpers); `emit.rs` 436. New expr arms → `emit/expr.rs`,
  stmt arms → `emit.rs`. Escapers + `EmitError` in `emit.rs`, reached via `super::`.
- **Test files over target (justified, sibling-split is the move):**
  `parser/mod_tests.rs` (~1550), `emit_tests.rs` (935), `lexer/mod_tests.rs` (553). All
  carry doc-comment justifications.
- **`rank()` complexity flag** is accepted noise (lookup table). Don't "fix" it by
  de-matching — that loses compiler-enforced exhaustiveness. If a future op lands, add it
  to the single `rank()` match.
- **Future paren hard case:** closures (`|x| body`, Rust's lowest greedy-right
  precedence) will be the genuinely tricky minimal-paren case when they land.
- **Keyword-collision check (new):** with `docs/keywords.md` now on hand, verify v0.3
  Latin keywords don't emit to / shadow a Rust strict-or-reserved word.

## Where We Are (state)

**Marain is a single Latin-core language** (multi-frontend rejected, ADR-0001). v0.2
feature-complete R9–R16; R17 added f-strings; R18 made emit idiomatic (minimal-paren) +
added the compiling control-flow e2e. Pipeline: `.lat → lexer → tokens → parser → AST →
emit → Rust → shim (cargo) → run`. **502 tests.** Only external dep is `clap` (pinned).

## Commit State — R18 committed; 2 doc files uncommitted

R18 landed in commit `6f41acd` (on top of R17's `d7030b2`); that tree was clean. **This
session added uncommitted changes the owner should handle:**

- `docs/keywords.md` — NEW (untracked), the conversion output.
- `docs/keywords.html` — present on disk (untracked unless already tracked; the
  conversion did not modify it).
- `tasks/CONTINUITY.md` — MODIFIED (this rewrite).

Recommended git action (owner executes — never me): `git add docs/keywords.md
tasks/CONTINUITY.md` (+ decide whether to track or delete `docs/keywords.html`), then
commit, e.g. `docs: add markdown conversion of Rust keywords reference`. Owner handles
all git/CI — surface + recommend, never stage/commit.

## Open Decisions

- **v0.3 direction: CHOSEN = type system** (2026-06-17). Next: PRD framing pass (see
  Immediate Next Action), which also discharges the ROADMAP §1/§2 triage.
- **v0.3 type-system scope still unframed** — frame from PRD §4.3/§4.11 + ARCH §15.8;
  forks listed in Immediate Next Action.
- **`docs/keywords.html` disposition** — keep both, or delete the HTML now that the `.md`
  exists? Not yet asked. Owner's call.
- **E1 leak fixes** parked in BACKLOG; only leak 2 (`unreachable!` → `EmitError`) has
  standalone single-language value.

## Carry-over Concerns

Unchanged, Stage-2 / post-v0.2: **γ (Variabile)** (ROADMAP §3); **ζ** cross-file Stage-2
diagnostics; **θ** Stage-2 inflection tokens. Plus the R18 closure-paren-emit note above.

## When You Resume

1. **Commit check** — R18 is committed (`6f41acd`). Uncommitted now: `docs/keywords.md`
   (new), `tasks/CONTINUITY.md` (this rewrite), and the `docs/keywords.html` source.
   Surface the git action; don't run it (owner handles git/CI).
2. **Start the v0.3 type-system PRD pass** (direction chosen 2026-06-17). Lead with the
   decision-bearing forks in Immediate Next Action; extend `PRD.md` interactively, then
   derive ARCHITECTURE. Triage ROADMAP §1/§2 as part of the same pass. Use
   `docs/keywords.md` to sanity-check Latin↔Rust keyword mapping.
3. Once the PRD scopes a first round, enter plan mode to frame it.
4. Round-close ritual (CLAUDE.md §7): sentrux `session_start` baseline BEFORE code; on
   close → decisions file + DECISIONS row + ARCH §N + ROADMAP + check off TODO + rewrite
   this file. Use **ASCII labels (a/b/c)** for any framing slate.

## Tactical Notes

- Date 2026-06-19. Project renamed Rubigo → Marain (repo dir still `rubigo`).
- Doc convention (load-bearing): **ROADMAP = committed, BACKLOG = uncommitted.**
- Golden harnesses auto-collect fixtures; regenerate with `MARAIN_UPDATE_GOLDENS=1` then
  eyeball the `.expected.rs`/`.expected.txt` diff.
- CLI: `marain run <file.lat>` transpiles + executes; `cargo run -p marain-cli -- run …`
  during dev. R18 e2e program is inline in `tests/e2e_control_flow.rs`.
