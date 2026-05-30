# R11+R12 — Operator expressions + control flow

_Closed 2026-05-29. Decision rationale archive. Summary list lives in [`ARCHITECTURE.md` §14.3](../../ARCHITECTURE.md#143-decisions); this file holds the *why* per decision._

R11 added the expression-level operator surface (precedence-climbing parser + Boolean literals + parens grouping). R12 finished Stage 1's control-flow set (`aliter` / `aliter si` chain on the R10 `si`, plus `dum` / `semper` / `interrumpe.` / `continua.`). Batched per locked decision A. R10's `si 1 :` caveat — the parser could produce Rust that wouldn't typecheck — retired here.

## Latin for op variants, English for stmt variants

**Decision:** `BinOp::Plus` / `BinOp::DivisusPer` / `BinOp::NonAequat` (Latin, with compound names for multi-word phrases); `Stmt::While` / `Stmt::Loop` / `Stmt::Break` / `Stmt::Continue` (English, matching the Rust lowering target).

**Rule:** enum variants whose name mirrors a *user-facing Marain keyword* track the Rust target; variants that name an *operator surface* use the Latin spelling because the parser sees Latin tokens, not Rust symbols.

**Existing `Stmt::Let` / `Stmt::If`** unchanged (no rename sweep).

## Precedence climbing, not Pratt

**Decision:** seven cascaded `parse_<level>` functions, one per Rust precedence rung (low to high: or, and, equality, comparison, additive, multiplicative, unary).

**Why:** all binary levels are left-associative via `while`-loop iteration; unary is right-associative via tail recursion. Roughly the textbook recursive-descent shape; Pratt parsing would buy nothing at this op count.

## Multi-word phrases consumed greedily at parse level

**Decision:** `consume_comparison_completer` fires when the parser sees `minor` or `maior` at comparison level: peeks for `quam` (→ `<` / `>`) or `vel par` (→ `<=` / `>=`). Bare `minor` / `maior` is a hard `UnexpectedToken` error. `divisus per` and `non aequat` use the same shape (advance, expect completer).

**Why:** the lexer remains multi-word-phrase-unaware — one token per word per PRD §4.4. Phrase recognition is a parser-level concern; trying to do it in the lexer would require lookahead and break the simple table-driven keyword dispatch.

## `non` disambiguates via one-token lookahead

**Decision:** at equality level, `non` followed by `aequat` is the binary `!=` operator; everything else is the unary prefix (handled at `parse_unary`).

**Why:** the equality level pre-empts so `parse_unary` only ever sees prefix `non`. `Parser::peek_kind_at(offset)` added for this case; clamps past-end peeks to the trailing `Eof` token (lexer guarantees Eof is last).

## Boolean literals are a new `Expr::BoolLit` variant

**Decision:** new variant, not a fold into `IntegerLit`.

**Why:** parallels `StringLit` / `IntegerLit` shape (struct with `value` + `span`); emit produces bare `true` / `false`. Folding into `IntegerLit` would have required a runtime tag for "is this Boolean?" — false economy.

## Paren-wrap-always in emit

**Decision:** every `BinOp` / `UnaryOp` emits with surrounding parens (`(a + b)`, `(!x)`).

**Why:** the parser tree already encodes correct precedence; paren-everywhere ensures emission is bulletproof against precedence drift in the Rust target.

**Cost:** visual noise in the emitted Rust.

**Benefit:** zero risk of operator-precedence subtleties leaking through the lowering.

## Expression-grouping parens (`(expr)`) in primary

**Decision:** `parse_primary` accepts `( expr )` as a grouping primary; unwraps to the inner expression.

**Why:** cost is one match arm; benefit is users don't have to memorize Rust's precedence table to write arithmetic that overrides defaults. No `ParenExpr` AST node — precedence is structurally encoded in the tree shape after parsing.

## `aliter` recognition by indent-aligned next-token

**Decision:** after `parse_block` returns from the then-body, the parser peeks for `Aliter`.

**Why:** if the user writes `aliter` at the wrong indent, the lexer's Dedent cascade has already moved past the `si`'s context — `Aliter` either won't be the next token, or it'll belong to an outer construct. Indent alignment is enforced implicitly by the layout tokens, not by a parser check.

## `aliter si` recurses through `parse_if`

**Decision:** a chain `aliter si … aliter si … aliter :` becomes `IfStmt { else_branch: Some(If(Box<IfStmt { else_branch: Some(If(Box<IfStmt { else_branch: Some(Block(...)) }>)) }>)) }`.

**Why:** single nested AST shape (TODO.md sub-decision #1 confirmed); emit walks the chain by recursing into `emit_if` from `emit_else_branch`.

**Considered:** flat `else_chain: Vec<ElseBranch>` shape. Rejected as harder to walk in emit and less natural for arbitrary `else if … else if` nesting depth.

## `semper :` emits `loop { … }` (no `Semper` rename of `Stmt::Loop`)

**Decision:** AST variant is `Stmt::Loop`, not `Stmt::Semper`.

**Why:** matches the naming rule (variants tracking Rust keywords use Rust names). PRD §4.11.2 keyword `semper` ("always") drives parser dispatch but doesn't bleed into the AST.

## `interrumpe.` and `continua.` are statements terminated by `.`

**Decision:** both carry just a `span`; no payload (unlabeled, no value-from-break in v0.2 — TODO.md sub-decision #7).

**Why:** Rust's labeled `break 'name` and `break <expr>` are out of scope for v0.2; deferring them keeps the AST shape minimal. When added, the structs grow `Option<Ident>` / `Option<Expr>` fields.

## No new `ParseError` variants

**Decision:** every R11+R12 failure mode rides on `UnexpectedToken { expected: &'static str }` with descriptive labels (`"per to complete divisus per"`, `"quam or vel par to complete maior comparison"`, etc.).

**Why:** consistent with R10's stance — variant proliferation has its own future tax. The label string carries the diagnostic specificity a dedicated variant would.

## Test files split via `#[path = "…_tests.rs"] mod tests;`

**Decision:** test bloc extracted to sibling files: `parser/mod_tests.rs` (836 LOC) and `emit_tests.rs` (554 LOC). Production files at 73 and 349 LOC respectively after the split.

**Why:** R11+R12 growth pushed `parser/mod.rs` to 905 LOC and `emit.rs` to 899 LOC, both in pressure-release territory dominated by test code. Per CLAUDE.md ("If `#[cfg(test)] mod tests` dominates, move it to a sibling file … that's a clean decomposition, not a workaround"), tests moved out.

**Pattern:** first pressure-release invocation in v0.2. Sibling test files remain in pressure-release (one cohesive helper set per file); module doc-comment carries the required justification.
