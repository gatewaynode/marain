# R10 — Block parsing + `si`

_Closed 2026-05-29. Decision rationale archive. Summary list lives in [`ARCHITECTURE.md` §13.3](../../ARCHITECTURE.md#133-decisions); this file holds the *why* per decision._

The parser learns to consume `Indent` / `Dedent` layout tokens and produces its first block-bearing AST node. The `si <cond> :` head (PRD §4.11.2) lands as the parent construct that exercises `parse_block` end-to-end. `aliter` / `aliter si` chains, `dum`, `semper`, and the operator expression surface remain in R11+R12.

## `Block` is a newtype, not a bare `Vec<Stmt>`

**Decision:** `Block { stmts: Vec<Stmt>, span: Span }` — wrap statements with their span.

**Why:** the `span` field carries the indented region (`Indent.start..Dedent.end`) so consumers don't recompute it. The newtype also gives `IfStmt::then_block` a name that reads as a block, not a list of statements that happen to be a block.

## Empty blocks are a parse error — mechanism is `ExpectedIndent`, not `EmptyBlock`

**Decision:** no dedicated `EmptyBlock` variant; the leading `expect_kind(p, &TokenKind::Indent, "indented block")` in `parse_block` catches the failure mode.

**Why:** R4's indent state machine treats blank lines as transparent; R9 extended that to comment-only lines. Both transparencies mean the lexer cannot produce an `Indent` immediately followed by a `Dedent` from any source — there is always at least one statement token between them. So the only way to get an "empty block" failure is to have no `Indent` at all (body on the same column as the parent, or `Eof` straight after the `:`). The leading `expect_kind` covers both cases with the same `UnexpectedToken` shape.

**Pattern:** per CLAUDE.md "don't add for can't-happen," no `EmptyBlock` variant ships.

## No dedicated `ExpectedIndent` / `ExpectedColon` / `ExpectedDedent` variants

**Decision:** R5's `UnexpectedToken { expected: &'static str }` is the generic vehicle for every "wrong token at a known position" failure.

**Why:** label strings (`":"`, `"indented block"`, `"end of indented block"`) give the user the same diagnostic clarity a dedicated variant would. Variant proliferation has its own future tax (more `match` arms, more renderer code paths); skip until a variant earns its keep with something `UnexpectedToken` cannot say.

## `parse_block` loop checks for both `Dedent` and `Eof`

**Decision:** the loop exit predicate matches `Dedent | Eof`, not just `Dedent`.

**Why:** the lexer guarantees a closing `Dedent` before `Eof` (R4 `indent.rs::finalize`), so `Eof` mid-block is structurally impossible from any well-formed token stream. Including `Eof` in the loop's exit predicate is one extra discriminant check that prevents an infinite loop if the lexer ever violates its contract — defensive against a *future bug in our own code*, not against valid input. The trailing `expect_kind(p, &TokenKind::Dedent, ...)` then fires `UnexpectedToken { found: Eof }` if the loop exited on `Eof`, surfacing the broken-lexer state instead of hanging.

## `emit_stmt` takes `indent_level: usize` (resolves §8.10 forward hook)

**Decision:** `emit_stmt(out, stmt, indent_level)` — depth parameter instead of hard-coded `"    "`.

**Why:** top-level statements are at `1` (inside `fn main`); each block body recurses at `level + 1`. `push_indent` writes four spaces per level. Regression coverage (`top_level_stmts_emit_at_indent_one`) confirms pre-R10 shape preserved.

## `emit_if` closes `}` at parent's indent level, no trailing newline

**Decision:** `emit_if` writes through to closing `}` at the parent's indent; caller (`emit_stmt`) writes the trailing `\n`.

**Why:** preserves the per-statement-line invariant of `emit_stmt`. Shape: `if <cond> {\n<body at level+1>\n<level-indent>}` followed by `\n` from the caller. Matches the Rust formatter's output for if-statements as block-statements.

## `Stmt::If` parses ahead of an executable condition language

**Decision:** ship `Stmt::If` in R10 even though R5's expression set (string / int / var-ref) cannot produce typecheckable conditions.

**Why:** `si 1 :` parses and emits as `if 1 { ... }` — which rustc will reject. Goldens are string-compares only (no `cargo run` in their harness), so the emit fixtures are exercised end-to-end through the parser+emitter without paying the rustc cost. R11+R12 (Boolean literals + operator expressions) made the produced Rust actually typecheck and retired the caveat.

**Documented here** so a confused future reader doesn't try to `cargo run` an R10 fixture by hand.

## R10 ships alone (locked decision A)

**Decision:** R10 contains only `si :`; `aliter` chain (`aliter :` / `aliter si <cond> :`) deferred to R11+R12.

**Why:** `aliter`'s chain shape (`aliter :` vs `aliter si <cond> :`) and the matching `Else::If(...)` AST shape are R11+R12 work; folding them in here would pre-commit a decision the next round should own. The single `si :` head is sufficient substrate for R10 to demonstrate `parse_block` end-to-end.

**Considered:** including `aliter` as the natural pair to `si`. Rejected to keep round-decision boundaries clean.
