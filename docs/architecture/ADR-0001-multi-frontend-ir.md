# ADR-0001 — Multi-Frontend Architecture: Shared Rust-Subset IR

- **Status:** **Rejected** (2026-06-16) — see *Rejection* below. Drafted 2026-06-10 from a feasibility discussion.
- **Deciders:** John (project owner)
- **Relates to:** PRD.md, ARCHITECTURE.md §7.3/§7.8 (AST-as-seam decision, deferred lowering pass), tasks/ROADMAP.md §3 (`Variabile`), §5 (Lowering/IR pass)
- **Backlog:** tasks/BACKLOG.md (only the standalone E1 leak fixes survive the rejection)

## Rejection (2026-06-16)

The owner rejected the multi-frontend initiative; Marain stays a single Latin-core
language. Rationale:

- **The deciding argument cut both ways.** "The seam gets more expensive every round"
  (see *Timing pressure*) was the case *for* moving fast on the IR — but that cost
  exists *only because of* the multi-frontend goal. Retract the goal and the timing
  pressure vanishes; there is no seam to keep cheap.
- **The premise was retracted.** The "Do nothing" alternative was rejected below on the
  grounds that "two frontends are now concretely intended." Space Latin and Common are
  no longer intended, so that alternative is no longer a fallback — it is simply correct.
- **Identity & pedagogy.** Refocusing on the Latin core preserves the project's stated
  dual goal (study Latin / keep Rust thinking sharp). The maintenance multiplier (three
  lexicons / fixture suites / doc sets) and the noted pedagogy dilution are avoided.

**What survives the rejection:**
- **E1 (the four backend "leaks")** stays in `tasks/BACKLOG.md` as *standalone
  code-quality* items, untied to any frontend-decoupling goal. Note that for a
  single-language compiler only leak 2 (the `unreachable!` macro fallback) is a genuine
  robustness fix; leaks 1/3/4 are "leaks" only relative to a shared backend and are
  otherwise just Marain's design.
- **ROADMAP §5's "Lowering / IR pass"** remains, but reverts to its original scope: a
  *Stage-2 normalization* idea (free word order → canonical form), independent of and no
  longer urgent because of this initiative.

The remainder of this ADR is retained verbatim as the rejected proposal.

## Context

Marain today is a single-language pipeline:

```
lexer → tokens → parser → AST (Module) → emit() → Rust source → shim (cargo project)
```

The owner intends to grow this into a small family of input languages, all emitting Rust:

1. **Marain** — the existing Latin re-skin of Rust (Classical-leaning lexicon).
2. **Space Latin** (working name) — a regularized Latin that smooths the inconsistencies
   of Classical and Ecclesiastical Latin. Intended to share Marain's grammar wholesale;
   it differs in lexicon and morphology only.
3. **Common** — a simplified-English, Python-flavored frontend (indentation blocks,
   dynamic-feeling values).

The question evaluated: is modularizing the parser and emitter to support multiple
frontends feasible, and what does it cost?

### Current state (evidence, as of R15 close)

The architecture already anticipated this cut. ARCHITECTURE.md §7.3 locked: *"AST is
the emit-ready form… When Stage 2 needs a lowering pass, interpose between parser and
emitter; the AST type is the seam."* ROADMAP §5 carries a deferred "Lowering / IR pass"
item. Several components are frontend-neutral today:

- `emit()` is a pure function `emit(&Module) -> Result<String, EmitError>` — no
  filesystem, no lexer/parser dependencies beyond AST types.
- `shim.rs` (cargo project generation, atomic write) knows nothing about Marain.
- `span.rs`, `source.rs`, and the `Diagnostic` machinery are language-agnostic.
- The lexer's structural machinery (cursor, indent state machine, string/comment
  handling) is reusable; only keyword recognition is language-specific.

Marain vocabulary leaks into the backend in exactly **four places** (the complete
decoupling bill at current surface — 12 statement kinds, 8 expression kinds, ~486 LOC
of emitter):

| # | Leak | Location | Fix direction |
|---|------|----------|---------------|
| 1 | `Sigil` interpreted as mutability (`@` → `mut`) in four emit sites (let / param / for / expr) | `emit.rs` | IR node carries `mutable: bool`; sigils are Marain surface syntax resolved at lowering |
| 2 | Latin macro names string-matched (`"dic"/"queror"/"agmen"/"forma"`) with `unreachable!` fallback | `emit.rs` (`emit_macro_call`) | IR node carries the *target* macro (`println`/`eprintln`/`vec`/`format`) as an enum; also removes a latent fragility (frontend invariant enforced in backend) |
| 3 | Latin type names mapped in the emitter (`"Sermo"` → `String`, `"Numerus"` → `i64`) | `emit.rs` (`emit_type_ref`) | Translation table moves to the frontend's lowering; IR carries Rust type names |
| 4 | `BinOp` variants spelled in Latin (`Aequat`, `DivisusPer`, …) with `as_rust()` | `ast.rs` | IR `BinOp` uses Rust-named variants; Latin spellings stay in the Marain AST |

The `Inflection` slots (`Option<Inflection>`, always `None` in Stage 1) are inert for
other frontends but are Marain-specific baggage in a shared type; they belong on the
frontend AST, not the IR.

## Decision (proposed)

Introduce a **shared Rust-subset IR** (working name `rust_ir`) between frontends and
the emitter, and restructure the pipeline as:

```
[Marain lexer+parser]      → Marain AST      ─┐
[Space Latin: same parser,                    ├─ lower → rust_ir → emit() → shim
 swapped lexicon tables]   → Marain AST       │
[Common lexer+parser]      → Common AST      ─┘
```

Key structural points:

1. **`rust_ir` is essentially today's AST with the four leaks fixed** and variants
   named for their Rust lowering targets. Spans are retained for diagnostics.
2. **Marain and Space Latin share one parser**, parameterized by lexicon tables
   (keywords, macro names, type names). A new Latin variant is a data file, not code.
3. **Common gets its own parser** (English word order) but reuses the lexer machinery
   (indent state machine, cursor, strings, comments) and the IR/emitter/shim.
4. **`Variabile` graduates from a Marain roadmap feature (ROADMAP §3, carry-over γ) to
   shared runtime infrastructure** — one vendored module emitted by the shim, used by
   all frontends. The IR must represent `Variabile` operations (dynamic index, dict/
   list/tuple construction) from its first design pass.
5. **Crate layout: defer.** Keep everything as modules in `marain-core` until a second
   frontend actually exists; the workspace seam (ARCH §2.2) makes later promotion to
   `frontend-marain` / `rust-ir` / `rust-emit` crates cheap.

### The semantic contract (the load-bearing rule)

> **Every frontend lowers to the shared IR; if a construct cannot, it is not in that
> language.**

This single sentence bounds the whole initiative. It makes the IR the constitution of
the language family. Concretely for Common: it gets exactly as Python-like as
`Variabile` + Rust semantics permit. Actual Python semantics — implicit aliasing of
mutable containers, duck typing on user types, exceptions as control flow — are
structurally ruled out; chasing them would force `Rc<RefCell<…>>` emission or an
interpreter-style runtime and erode the emitter with special cases.

## Feasibility assessment

### By language

- **Space Latin — Tier 1, nearly free.** Lexicon swap over the shared Latin parser.
  Engineering cost ≈ the table extraction; the lexicon authoring is the point of the
  exercise, not a cost. *Bonus:* a regularized morphology makes the hardest Stage 2
  items (case-driven free word order, `.latin` sidecar declensions, inflection
  metadata) easier — consistent declensions, no irregular-form special-casing. Space
  Latin is a plausible Stage 2 *testbed*, with Classical Marain inheriting the engine
  once it works on the regular case. The carry-over α inflection slots and the θ
  inflection-token hook need no change.
- **Common — Tier 2, conditional on the semantic contract.** It is the consumer that
  justifies the IR's existence (Space Latin alone would only justify lexicon tables).
  Most of its "Python-ness" is already on Marain's own roadmap — `Variabile` (γ),
  dict/list/tuple literals, f-strings, triple-quoted strings — so the net-new cost is
  its parser, its lexicon, and a written semantics spec. Without the semantic
  contract pinned, Common degrades to an unbounded Tier 3 liability.

### Timing pressure

The seam gets more expensive every round. v0.3 candidates (structs, enums, generics,
closures, `Variabile`, f-strings) will roughly triple the AST/emitter surface. At
today's ~20 node types the IR cut is a mechanical, afternoon-scale-to-one-round
change; after v0.3 it is a real migration. **If accepted, the IR round should land
before v0.3 feature work begins.**

### Effort estimates (rough)

| Work item | Estimate |
|-----------|----------|
| Vocabulary-leak fixes only (minimum insurance, valuable regardless) | hours–1 day |
| `rust_ir` introduction + retarget emitter + Marain lowering pass | 1–2 normal rounds (~600–900 LOC incl. tests, mostly mechanical) |
| Lexicon table extraction (keywords / macros / types as data) | days |
| Space Latin lexicon | authoring effort, ongoing |
| Common spec (semantic contract + grammar) + parser | its own milestone, sized after spec |

## Gating decisions (must close in the PRD rewrite before implementation)

1. **Common's semantic contract** — adopt the "lowers to shared IR or it's out" rule
   explicitly. (Without this, do not proceed past the leak fixes.)
2. **Space Latin divergence rule** — pin "differs from Marain in lexicon and
   morphology only; grammar is shared" so it cannot drift into a second grammar.
3. **Stage 2 ordering** — decide whether free word order is a Marain-family feature
   and whether it lands on Space Latin first (regular case) before Classical Marain.

## Consequences

### Positive

- IR designed against two known consumers, not speculative generality — a product
  family, not a framework.
- Leak fixes improve the single-language design on their own terms (removes the
  `unreachable!` macro dispatch fragility; moves frontend vocabulary out of the
  backend) even if the initiative stalls.
- `Variabile`, f-strings, and collection literals become shared work, amortized
  across three languages instead of built for one.
- Regularized-Latin testbed de-risks Stage 2's hardest parser work.

### Negative / risks

- **Project identity shift.** Marain becomes *a* frontend rather than *the* language;
  Stage 2 becomes one frontend's roadmap. This contradicts the PRD's current framing
  and requires a deliberate PRD amendment, not silent drift.
- **Maintenance multiplier:** three lexicons, three fixture suites, three doc sets —
  bounded by sharing one Latin grammar engine, one IR, one emitter, one shim, but
  real. Pedagogy goals (Latin study) dilute across languages.
- **Scope gravity on Common.** "Python-like" invites CPython-behavior expectations;
  the semantic contract is the only structural defense. Treat any proposal that
  cannot lower to the IR as a language-design rejection, not an emitter feature
  request.
- One AST-shaped layer becomes two (frontend AST + IR) — more types to keep in sync,
  though the lowering passes are mechanical at current scope.

## Alternatives considered

- **Parameterize the emitter with translation tables, no IR** (macro/type tables as
  emitter inputs). Cheaper today; sufficient for Space Latin alone. Rejected because
  Common's different surface grammar and `Variabile` semantics would push per-language
  conditionals into the emitter — exactly the coupling this initiative removes.
- **Emit via `syn`/`proc-macro2` or an existing Rust-AST crate.** Rejected: violates
  the self-supporting dependency constraint (CLAUDE.md), and string emission of a
  controlled Rust subset has been trouble-free.
- **Do nothing until a second frontend is real.** Viable fallback: fix only the four
  leaks now (cheap, beneficial regardless) and defer the IR. Rejected as the default
  because the v0.3 surface growth makes deferral markedly more expensive and two
  frontends are now concretely intended.
