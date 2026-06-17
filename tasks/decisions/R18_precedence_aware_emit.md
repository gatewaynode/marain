# R18 — Precedence-aware emit + compiling control-flow e2e

Closed 2026-06-17. Resolves TODO Task 3 (`unused_parens` in emitted Rust) and the
ROADMAP §6 "v0.2 done-line e2e" gap. Reverses R11+R12's paren-everywhere emit
(ARCH §14).

## A. Minimal-paren emit (reverses paren-everywhere)

**Decision:** `emit_expr` wraps a binary/unary operand in parens only when Rust's
precedence/associativity would otherwise re-parse it differently. Replaces the
R11+R12 rule that wrapped every `BinOp`/`UnaryOp` unconditionally.

**Why:** The emitted Rust is a user-facing artifact — readability is a stated project
goal (Rust-sharpening, pedagogy). Paren-everywhere produced `let x = ((1 + (2 * 3)) -
4);` where idiomatic Rust is `let x = 1 + 2 * 3 - 4;`. It also emitted `if (cond)`,
`let x = (…)`, `return (…)`, `x = (…)` — every one a `unused_parens` warning under
`-D warnings` (the Task 3 defect).

**Considered:** (b) outermost-strip — keep paren-everywhere for nested operands, only
drop the top-level parens in lint-checked slots. Smaller, lower-risk, preserves §14's
spirit; clears the *warnings* but leaves non-idiomatic nested parens (`a + (b * c)`).
Rejected by the owner in favor of idiomatic output.

**Result:** All BinOp/UnaryOp goldens regenerated; redundant parens vanished only where
removable (e.g. `12_arithmetic`, `14_comparison`, `15_aliter_chain`, `27_fit`).

## B. The emit precedence table follows Rust's grammar, NOT the parser cascade

**Decision:** `BinOp::rank()` ranks operators by **Rust's** precedence table — all six
relational operators (`== != < > <= >=`) at one **non-associative** level — even though
`parser/expressions.rs` splits equality above comparison and builds both with
left-associative loops.

**Why (the trap):** Rust forbids chained comparisons (`a < b < c` is a syntax error),
and groups all six relationals at one level. Our parser is left-associative and
two-level, so `a minor quam b minor quam c` → AST `(a<b)<c` and `a aequat b minor quam
c` → AST `a==(b<c)`. A naive "all binary ops left-assoc" minimal-paren rule would emit
`a < b < c` / `a == b < c` — **invalid Rust**, where paren-everywhere had emitted the
valid (if type-wrong) `((a<b)<c)`. So §14's blanket was silently covering one real
divergence. The emit table and the parser cascade answer different questions (how Marain
*groups* vs. what Rust needs to *re-parse identically*); they coincide for arithmetic /
logical ops and diverge for relationals. Emit must follow the target grammar.

**Result:** Relationals classified `Associativity::None` → both equal-precedence sides
parenthesized. Tests `chained_relational_left_child_keeps_parens`,
`mixed_relational_keeps_parens_for_rust_grouping`. (Rejecting chained comparisons at
*parse* time, as Rust does later, is deferred — R18 only emits valid Rust.)

## C. Compiler-enforced exhaustiveness over a heuristic-friendly form

**Decision:** `BinOp::rank()` is a single exhaustive `match` (no catch-all) returning
`(precedence, associativity)`; `precedence()`/`associativity()` are thin accessors.

**Why:** A future operator (method-call, closure, cast — all on the roadmap) cannot
compile until it is given a rank and associativity here. That is the safety
paren-everywhere bought by brute force, now bought by the type system without the noisy
output. Pairing the two facts (always consulted together) into one table keeps them in
sync.

**Caught (sentrux):** `rank()` trips the cyclomatic "complex function" heuristic (5 → 6)
— a flat 13-variant lookup table where cyclomatic complexity overstates cognitive load.
Accepted deliberately: an exhaustive match is the correct, safest form (the Rust idiom
CLAUDE.md prescribes); a discriminant-indexed array would be less readable and more
bug-prone. Net signal still improved (7033 → 7035), 0 coupling/cycle change.

## D. Compiling e2e asserts BOTH warning-clean build AND computed value

**Decision:** New `tests/e2e_control_flow.rs` compiles an accumulator program with
`RUSTFLAGS=-D warnings` and runs it, asserting a clean build AND stdout `20`.

**Why:** The emit goldens are string-compare only — they never invoke cargo, so the
`unused_parens` warning slipped past them for rounds (Task 3 root cause). A build-only
check catches the warning regression but **not** a precedence *miscompile* that still
compiles. The program relies on `*` binding tighter than `+` (`summa + i*2`, not
`(summa+i)*2`), so the value `20` is what proves minimal-paren preserved Rust precedence.

**Result:** Closes ROADMAP §6 done-line e2e. Guards both the lint and precedence
semantics against future emit regressions.

## Verification
fmt / clippy `-D warnings` clean · `cargo test --all` 502 passing (+10) · e2e green ·
`cargo doc` only the 2 pre-existing `marain-cli` intra-doc warnings · sentrux
7033 → 7035, 0 violations beyond the accepted lookup-table flag.
