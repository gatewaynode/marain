# Backlog — uncommitted proposals

**Doc convention (set 2026-06-10):** `tasks/ROADMAP.md` holds **committed** work —
being there means it will happen. This file holds **uncommitted** work — proposals
that can be pulled into the roadmap or sit indefinitely; nothing here is a promise.

## Multi-frontend initiative — REJECTED 2026-06-16

The three-language family (Marain, Space Latin, Common) over a shared Rust-subset IR
was evaluated in [`docs/architecture/ADR-0001-multi-frontend-ir.md`](../docs/architecture/ADR-0001-multi-frontend-ir.md)
and **rejected** — Marain stays a single Latin-core language. The deciding argument
("the seam gets more expensive every round") was a cost *of* the multi-frontend goal;
retracting the goal removes the cost. See the ADR's *Rejection* section.

Items D1–D4, E2–E5, and L1–L3 are withdrawn with the initiative. Only **E1** survives,
re-scoped below as standalone code-quality work with no frontend-decoupling motive.

## Standalone code-quality items

| # | Item | What | Standalone value (single-language) | Notes |
|---|------|------|-----------------------------------|-------|
| E1 | **Backend decoupling fixes** | The four spots originally framed as Marain vocabulary "leaking" into the backend: (1) `Sigil`→`mut` interpretation in `emit.rs`, (2) Latin macro-name string match + `unreachable!` fallback in `emit_macro_call`, (3) `Sermo`/`Numerus` type table in `emit_type_ref`, (4) Latin-spelled `BinOp` + `as_rust()` in `ast.rs` | **Leak 2 only** is a genuine fix — the `unreachable!` is a latent panic reachable by adding a macro without updating the match; convert to an `EmitError`. Leaks 1/3/4 are "leaks" only relative to a shared backend; for single-language Marain they are just the design and carry little standalone value. | Hours–1 day if all four; the leak-2 hardening alone is the high-value slice. Parked, not scheduled. |
