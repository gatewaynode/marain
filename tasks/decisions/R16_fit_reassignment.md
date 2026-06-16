# R16 — `fit` reassignment

Closed 2026-06-16. Wires the already-lexed `Keyword::Fit` through parse + emit so
a declared mutable binding (`sit @x est 0.`) can be re-bound (`@x fit @x plus 1.`).
The binding lifecycle's missing half across R9–R15. In-spec (PRD §4.4 reassign
copula, `docs/core-lexicon.md:46`) — no PRD amendment, just wiring.

## A — Require an `@` (mutable) sigil on the reassignment target

**Decision:** A `fit` target must carry `@`. A `^`-sigiled target is a hard parse
error (`ParseError::ImmutableReassignmentTarget`); the message names both the
offending `^x` and the fix `@x`, and cites PRD §4.5.
**Why:** PRD §4.5 makes the sigil the mutability marker at *every* use site, so
reassigning a `^` binding is a self-contradiction the author should see at the
Marain level — not deferred to a rustc "cannot assign twice to immutable" message
(no span back-mapping exists yet; rustc output would point at emitted Rust).
**Considered:** (a) accept any sigil and let rustc adjudicate — rejected: worse
diagnostic, wrong language layer. (b) require `@` — chosen.
**Result:** Purely syntactic check on the target token; no symbol table needed. It
does NOT catch the cross-statement case (a binding *declared* `sit ^x` but targeted
as `@x`) — that still falls to rustc. Accepted: full mutability tracking is a
name-resolution-era concern, out of scope for a single-statement parse check.

## B — Reassignment emits NO `mut` (use site, not binding site)

**Decision:** `emit_assign` emits `<name> = <value>;` and deliberately does not
share code with `emit_let`/`emit_param`/`emit_for` — it never emits `mut`.
**Why:** The `@`→`mut` rule applies only where a binding is *introduced*. A
reassignment is a *use* of an existing binding (`@x fit …` re-binds the `mut x`
already declared by `sit @x est …`); emitting `mut x = …` would be invalid Rust.
This is the one non-obvious bit, so the divergence carries an inline comment.
**Result:** `sit @x est 0.` → `let mut x = 0i64;`; `@x fit @x plus 1.` →
`x = (x + 1i64);`. The RHS paren-wrap is the pre-existing Task 3 `unused_parens`
tradeoff (paren-everywhere emit, ARCH §14), now confirmed to fire on assignment
RHS as predicted — not a regression and not this round's fix.

## C — Dispatch on a leading `SigiledIdent` at statement position

**Decision:** `parse_stmt` gains `TokenKind::SigiledIdent { .. } => parse_assign`.
**Why:** Unambiguous and previously dead: before R16, a statement opening with a
sigiled ident was always `UnknownStatementStart` (only keywords and `PlainIdent(`
open statements). No conflict with `parse_call_stmt` (PlainIdent) or any keyword
arm. The verb mismatch case (`@x est 5.`) now produces a clean "expected keyword
`fit`" error from `expect_keyword` rather than a dispatch miss.

## D — Target shape is a bare `SigiledIdent` for v0.2

**Decision:** `AssignStmt.target` is a `SigiledIdent`; field targets (`@x.y`) and
index targets (`@x[i]`) are out of scope.
**Why:** No method-call or index syntax exists in the language yet, so there is
nothing to target. The shape mirrors `LetStmt` minus the `est` copula. Add richer
targets when method/index syntax lands.

## PRD reconciliation

PRD line 115's illustrative `^x fit 5` (in the verb-disambiguation example
"`sit ^x est 5` initializes, `^x fit 5` reassigns, `si ^x aequat 5` compares") is
self-contradictory under decision A and is now footnoted in PRD as illustrating the
verb contrast *only* — the §4.5 sigil rule rejects a `^` reassignment target. The
canonical reassignment form is `@x fit 5`.
