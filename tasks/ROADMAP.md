# Marain — Roadmap

Work the project is **committed to**, grouped by theme, with source citations.
Created 2026-05-31 by consolidating items that were scattered across PRD.md
(§4.11.6, §4.12, §4.3, Stage-2 open questions), ARCHITECTURE.md ("Forward hooks"
subsections + §11 Stage 2 hooks), `tasks/CONTINUITY.md` (pinned carry-overs γ/ζ/θ),
and `docs/core-lexicon.md` ("proposed" rows).

## How this doc works

Planning-doc convention (set 2026-06-10):

- **ROADMAP.md** (this file) = **committed** work. Being here means it will happen;
  only the ordering is open. Items live here until scheduled.
- **`tasks/BACKLOG.md`** = **uncommitted** work — proposals that can be pulled into
  the roadmap or sit indefinitely. Nothing in BACKLOG.md is a promise.
- **`tasks/TODO.md`** = active work. When a roadmap item is scheduled, copy it into
  TODO.md as a numbered task; leave it here (optionally marked *in TODO*) so the
  roadmap stays complete.
- Citations are **section-level** (e.g. `PRD §4.11.6`, `ARCH §16.8`) rather than line
  numbers, so they survive doc edits.
- **Triage pending:** this doc predates the committed/uncommitted split and was
  written as a candidate pool — some sections (notably "v0.3 candidates") are not yet
  true commitments. Triage each section into ROADMAP (committed) vs BACKLOG
  (uncommitted) during a future PRD pass.

## Status snapshot (2026-05-31)

- **v0.1** (the "Salve, Munde" done line, PRD §7) — **shipped** (rounds R1–R8).
- **v0.2** (Control Flow & Functions, PRD §4.11) — **feature-complete** across R9–R15
  (comments, blocks, `si`/`aliter`, operators, `dum`/`semper`/`interrumpe`/`continua`,
  `functio`/`redde`/calls, `pro`/ranges/`nihil`). Commit + a runnable v0.2 done-line e2e
  still outstanding.
- No v0.3 round is framed yet. Candidates for that framing live below (pending the
  committed/uncommitted triage noted above). (`tasks/BACKLOG.md` now holds only the
  standalone E1 code-quality item — the multi-frontend initiative there was rejected
  2026-06-16; see ADR-0001.)

## Active findings (currently tracked in `tasks/TODO.md`)

These were surfaced during manual testing and live in TODO.md, not here, but they
intersect the backlog so they're cross-referenced:

- **TODO Task 1** — no string concatenation/interpolation → resolved by **f-strings** (§4 below).
- **TODO Task 2** — `fit` reassignment specced but unimplemented (§1 below; in-spec, highest-priority gap).
- **TODO Task 3** — `unused_parens` in emitted Rust (an emit-quality fix, not a roadmap feature).

---

## 1. v0.3 candidates — language surface

### Formally deferred to v0.3+ in the PRD

| Item | What | Source | Notes |
|------|------|--------|-------|
| **Closures** | `\|x\| body`, `move \|x\|` | PRD §4.11.6 | Capture rules (Fn/FnMut/FnOnce, by-ref vs by-move) deserve their own decision round. Generalizes `Expr::Call`'s callee from `Ident` to `Expr` (ARCH §15.8). |
| **Generics** | `<T>`, bounds, lifetimes, const generics | PRD §4.11.6, ARCH §15.8 | Large surface; defer alongside the type-system layer. Lexer currently hard-errors on `<`/`>` via `LexError::GenericsLookalike`; activation flips that arm to emit a token and teaches `parse_type_ref` to consume `<T, U>` into the `TypeRef` seam reserved in R13. |
| **Visibility** | `pub` → proposed `publicus` | PRD §4.11.6 | No module boundaries in v0.2 to gate against; lands with the module system (§2). |
| **Block comments** | `/* */` | PRD §4.12, ARCH §12.8 | Reserved; lexer errors via `LexError::BlockCommentsDeferred`. Nesting + termination semantics TBD. |

### Reassignment (`fit`) — in-spec, highest-priority gap

| Item | What | Source | Notes |
|------|------|--------|-------|
| **`fit` reassignment** | `@x fit @x plus 1.` — mutate a declared mutable binding | PRD §4.4 (§94/§115), lexicon (`fit` → `=` reassign), **TODO Task 2** | Specced and lexed (`Keyword::Fit`) but never parsed/emitted — no `parse_stmt` arm, no `Stmt::Assign`. Skipped across R9–R15. Small, in-spec (no PRD amendment): add `Stmt::Assign { target: SigiledIdent, value: Expr }`, dispatch on `Keyword::Fit`, emit `target = value;`. Unblocks all mutation/increment code. |

### Near-term forward hooks (control flow & expressions)

| Item | What | Source | Notes |
|------|------|--------|-------|
| **Open-ended ranges** | `..b`, `a..`, `..`, `..=b` | ARCH §16.8 | `RangeExpr`'s `Option` start/end fields already model these; emit arm already guards both sides with `if let Some`. Activation = `parse_range` change only, no AST reshape. |
| **Labeled `break`/`continue`** | `break 'name`, `continue 'name` | ARCH §14/§15.8 | Needs `Option<Ident>` on Break/Continue. |
| **`break <expr>`** | loop break producing a value | ARCH §14/§15.8 | Needs `Option<Expr>` on `BreakStmt`. |
| **Method-call syntax** | `receiver.method()` | ARCH §16.8 | Unblocks stepped/reverse iteration (`.step_by(n)` / `.rev()`). |
| **Trailing-expression returns** | `{ 42 }` with no `redde` | ARCH §15.8 | Awkward against the period-terminator design (§4.8); **deferred indefinitely** unless a real use case appears. |
| **`nihil` as an expression** | unit-valued `Expr::Nihil` | ARCH §16.8 | Statement-only today; no current use case. |
| **Expression-position macros** | `MacroCallExpr` — macros usable inside expressions, e.g. `sit ^x est forma "salve {nomen}".` | ARCH §7.8 | Today macros are statement-only (`MacroCallStmt`). Adds an `Expr::MacroCall` variant; the Stmt/Expr split keeps statement-only forms honest. Pairs naturally with f-strings (§4). |

## 2. Type system & user-defined types (tagged "R16+")

| Item | What | Source | Notes |
|------|------|--------|-------|
| **User-defined type keywords** | `structura`/struct, `enumeratio`/enum, `modulus`/mod, `praestatio`/impl, `proprietas`/trait | lexicon §"proposed; not yet in lexer", PRD §4.3, ARCH §15.8 | Proposed keywords, not yet lexed. The `TypeRef` pass-through seam + `emit_type_ref` translation table are built to absorb them (user types pass through verbatim today). `modulus` interacts with Visibility (§1). |
| **Stdlib type names** | `Agmen<T>`/Vec, `Vocabularium<K,V>`/HashMap, `Fortasse<T>`/Option, `Eventus<T,E>`/Result | lexicon §"Proposed Standard-Library Type Translations", PRD §4.3 | Depend on generics (§1) + collection literals (§3). |

## 3. Dynamic value type — `Variabile` (carry-over γ, pinned)

| Item | What | Source | Notes |
|------|------|--------|-------|
| **`Variabile` enum** | tagged union `Numerus \| Decimalis \| Sermo \| Bool \| Nihil \| Index(Vec) \| Vocabularium(HashMap)` | PRD §3 (dynamic value wrapper), CONTINUITY carry-over γ | When literals land, `shim.rs` emits a vendored `variabile.rs` module + prepends `mod variabile; use variabile::Variabile;` to `main.rs`. Vendored, not a dependency (self-supporting constraint). |
| **`Variabile` literals** | `{clavis: valor}`, `[unum, duo]`, `(x, y)` | PRD §3 | Concise dict/list/tuple syntax; depends on the enum landing. |

## 4. String handling

| Item | What | Source | Notes |
|------|------|--------|-------|
| **F-strings** | `f"salve {nomen}"` | PRD §3, §4.6 | The PRD's intended mechanism for string composition (replaces a concat operator — see TODO Task 1). Needs string-interior parsing. |
| **Triple-quoted strings** | `"""…"""` | PRD §4 (Python-niceties), lexicon | Multiline string literals. |

## 5. Stage 2 — Latin grammar + LSP (its own milestone)

Stage 2 (full case/conjugation grammar, free word order) is gated: **PRD open questions
S2-1 through S2-7 must close and a separate Stage-2 grammar spec must be drafted before
any ARCHITECTURE Stage-2 work begins** (PRD §"Stage 2 gating round"). Captured here so
the architectural seams aren't lost:

| Item | What | Source |
|------|------|--------|
| **Free word-order parser** | case-driven role resolution; engine TBD (RD+backtracking / GLR / Earley / constraint propagation) | PRD S2-3 |
| **Sidecar `.latin` format** | declension/inflection metadata; diff-friendly + tool-regenerable; format TBD | PRD S2-1 |
| **LSP server** | JSON-RPC over stdio; editor order Zed → Lapce → Helix; deterministic suggestion layer first, optional LLM layer later | PRD §4.10, S2-4, S2-7 |
| **Stage1 ↔ Stage2 interop** | cross-stage `modulus` imports; signature details TBD | PRD S2-2 |
| **Ambiguity resolution** | multiple-parse handling (first-wins / hard error / user-disambiguate) | PRD S2-5 |
| **Migration UX** | "upgrade Stage 1 → Stage 2" tooling | PRD S2-6 |
| **(θ) Inflection tokens** | optional `(lemma, inflection)` slot on identifier tokens | ARCH §11, CONTINUITY carry-over θ |
| **(ζ) Cross-file diagnostics** | multi-file Stage-2 grammar context | ARCH §11, CONTINUITY carry-over ζ |
| **Lowering / IR pass** | parser → IR → emitter normalization layer; `Module` becomes the *output* of lowering rather than of parsing | ARCH §7.8 (deferred by design) |
| **Op-name inflection metadata** | adding inflection metadata to `BinOp` variants (spelled at lemma level today: `DivisusPer`, `MinorQuam`, …), paralleling the carry-over α pattern on identifier nodes | ARCH §14.8 |

## 6. Tooling & diagnostics

| Item | What | Source | Notes |
|------|------|--------|-------|
| **`marain check`** | lex + parse (+ name-resolve) without invoking rustc | ARCH §9.9 | One `args::Command` variant + one driver dispatch arm. |
| **`marain install`** | symlink a built user program into `~/.local/bin/<name>` | ARCH §9.9 | Adds `~/.local/bin` to the path table. |
| **v0.2 done-line e2e** | run emitted control-flow Rust through `cargo build -D warnings` | TODO Task 3 note | Goldens are string-compare only; a compiling e2e would have caught Task 3 and guards future emit regressions. Near-term, not v0.3-gated. |
| **Rustc error back-mapping** | map rustc errors back onto Marain source spans | PRD §"Non-Goals" (revisit **post-v0.5**) | Currently cargo output passes through verbatim. Listed as a non-goal with a revisit horizon, not an active plan. |

## 7. Non-goals (explicitly NOT planned)

Listed to keep them distinct from the backlog and prevent re-litigation (PRD §"Non-Goals"):

- `unsafe` blocks expressible in Marain (author can drop to raw Rust; Marain source stays safe-only)
- `async` / `await`
- FFI / `extern` blocks
- Procedural macros authored in Marain
- Self-hosting (Marain compiler written in Marain)
- `crates.io` publishing

(Editor/LSP integration was *removed* from non-goals — it's now an essential Stage-2 roadmap item, §5.)
