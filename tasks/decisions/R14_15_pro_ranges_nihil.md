# R14 + R15 — `pro` loops + range tokens + `nihil`

_Closed 2026-05-31. Decision rationale archive. Summary list lives in [`ARCHITECTURE.md` §16.3](../../ARCHITECTURE.md#163-decisions); this file holds the *why* per decision. Note: this round was implemented in one session, then its round-close (this file, §16, status docs) was completed after a mid-round console crash interrupted before the docs/commit step — the code was found intact and gate-clean on resume._

## A: Round batching (R14 + R15 together)

**Decision:** ship `pro` + ranges (R14) and `nihil` (R15) in a single round.

**Why:** both are small, and they compose naturally — a `pro` body is the most natural place to exercise `nihil` (a loop that does nothing on purpose). Per locked decision A (round granularity), batch where dependencies/testing overlap.

**Considered:** separate rounds. Rejected: `nihil` alone is a few lines of parser + one emit arm; a dedicated round would be ceremony without content, and its best fixtures (`25`, `26`) lean on `functio` and `pro` bodies anyway.

## Range precedence: new lowest-precedence `parse_range` level

**Decision:** `parse_expr` now enters at `parse_range`, which parses a `parse_or` lhs, then optionally consumes `..` / `..=` and a `parse_or` rhs. The cascade becomes `range → or → and → equality → comparison → additive → multiplicative → unary → primary`.

**Why:** Rust places ranges below every binary operator (`a + 1..b * 2` is `(a+1)..(b*2)`). Slotting `parse_range` at the top of the cascade reproduces that verbatim, so the precedence is correct without any paren bookkeeping at emit time.

**Considered:** ranges as a primary-level construct (would force explicit parens for `a..b plus c`). Rejected — diverges from Rust precedence, surprises the user this project is sharpening.

## Range operands are `parse_or`, not `parse_range`

**Decision:** both the lhs and rhs of a range descend through `parse_or`, not recursively through `parse_range`.

**Why:** ranges don't chain — `a..b..c` is not valid Rust. Parsing each operand at `parse_or` means a second `..` has no production to attach to and surfaces as a clean parse error at the enclosing context, with zero special-case code. A right-recursive `parse_range` would instead silently accept (or mis-associate) chained ranges.

## Bounded-only ranges; `Option` fields reserve the open forms

**Decision:** the v0.2 parser produces only fully-bounded `a..b` / `a..=b`. `RangeExpr.start` and `.end` are `Option<Box<Expr>>` and are always `Some` from today's parser.

**Why:** the four open-ended Rust forms (`..b`, `a..`, `..`, `..=b`) are real but unused by any v0.2 surface. Modeling them in the AST now (rather than reshaping later) costs two `Option`s and lets the emit arm — which already guards each side with `if let Some` — light up for free when a future round teaches `parse_range` to accept a missing operand. Mirrors the B-1 (`TypeRef`) forward-seam pattern from R13.

## `nihil` emit shape: `();` (the open sub-decision)

**Decision:** `nihil.` emits Rust `();` (a unit statement), not `{}` (an empty block).

**Why:** the purpose of `nihil` is to satisfy "a block must contain at least one statement" without committing to behavior. `();` is a genuine statement that does that and reads as "evaluate unit, discard." `{}` would introduce a nested lexical scope for no reason and is easy to misread as a block delimiter. This resolves the sub-decision recorded open in `tasks/TODO.md` / CONTINUITY (recommendation was `();`).

**Considered:** `{}`. Rejected — adds a scope, no upside.

## `pro` binding is a `SigiledIdent`

**Decision:** the loop binding must carry a sigil (`pro ^i in …` / `pro @i in …`); a bare `pro i in …` is a parse error.

**Why:** consistent with PRD §4.5 (variables always carry a sigil) and with `Stmt::Let` / `Param`. The sigil drops at emit time and `@` adds `mut` — same convention as everywhere else a binding is introduced. Error fixture `15_pro_missing_sigil` pins the diagnostic.

## Range emit is not paren-wrapped

**Decision:** `emit_expr`'s range arm emits `start .. end` (or `..=`) without wrapping the whole range in parens; operands self-wrap if they are `BinOp`/`UnaryOp`.

**Why:** the paren-everywhere rule for `BinOp`/`UnaryOp` exists to be bulletproof against Rust precedence drift between operators. Ranges are the lowest-precedence form and aren't subject to that drift; wrapping them would produce `(0i64..10i64)` noise. Operands keep their own paren-wrap via the existing `emit_expr` recursion, so `(a plus 1)..b` still emits correctly.

## `.` lexer dispatch: peek for `..` / `..=`

**Decision:** the `b'.'` arm in `lexer/mod.rs` advances past the first `.`, then peeks: a second `.` produces `DotDot` (and a following `=` upgrades to `DotDotEq`); otherwise it stays `Period`.

**Why:** `Period` is the statement terminator and must keep lexing as a single token; the range tokens are strictly longer, so greedy maximal-munch on the dot-run is unambiguous. `...` lexes as `DotDot` + `Period` (no three-dot token exists), and a trailing `0..10.` lexes as range-then-terminator — both pinned by lexer tests. `peek_at` (added R9) made the two-byte lookahead trivial.
