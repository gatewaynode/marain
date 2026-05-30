# R13 — `functio` declarations + `redde` returns + function calls

_Closed 2026-05-30. Decision rationale archive. Summary list lives in [`ARCHITECTURE.md` §15.3](../../ARCHITECTURE.md#153-decisions); this file holds the *why* per decision._

## A-1: Function call scope

**Decision:** (a) — calls IN as both expression (`Expr::Call`) and statement (`Stmt::Call`).

**Why:** declaring functions without supporting calls would leave the feature untestable via `marain run`. The two halves are functionally co-dependent.

**Considered:** (b) declarations only. Rejected: un-callable functions can't be exercised end-to-end.

## A-2: Round split

**Decision:** R13 ships declarations + calls together; no R13a/R13b split.

**Why:** the two halves are co-dependent (callable functions need both surfaces); splitting forces a contrived intermediate state with no end-to-end story.

## B-1: TypeRef newtype

**Decision:** wrap `Ident` in a `TypeRef` newtype rather than scattering bare `Ident` across type-position consumers.

**Why:** reserves a generics-grow seam — v0.3+ will add `params: Vec<TypeRef>` to `TypeRef`. Sites that destructure by `.name` still work; new generic-bearing sites pattern-match the new field. Bare `Ident` everywhere would require a sweep when generics land.

**Considered:** (a) bare `Ident` scattered. Rejected as forward-incompatible.

## B-2: Bare unit return

**Decision:** support `redde.` (no value) as Rust `return;`.

**Why:** parser cost is zero (`Option<Expr>` on `ReturnStmt`); the spec is symmetric with `return;` in Rust.

## B-3: Callee shape

**Decision:** callee is `Ident` only, not `SigiledIdent`.

**Why:** function-value semantics (`^f(...)` for first-class functions) deferred indefinitely. Function names per PRD §4.5 are bare PlainIdents (no sigil because they're not variables).

## C-1: `grammar.rs` split timing

**Decision:** (b) land R13 in `grammar.rs`, split iff threshold crossed.

**Why:** pre-emptive split would have been predicted-pressure not measured-pressure. The 500-LOC rule's first tier triggers on *actual* size, not anticipation.

**Result:** the threshold did cross (`grammar.rs` reached 601 LOC mid-R13). Split executed: expression cascade (`parse_or` through `parse_primary`, plus `parse_call` and `make_binop`) extracted to `parser/expressions.rs` (269 LOC). `grammar.rs` at 357 LOC post-split. Helpers (`expect_kind`, `expect_keyword`, `parse_sigiled_ident`) promoted to `pub(super)` so `expressions.rs` can call them.

## C-2: PascalCase enforcement

**Decision:** (a) enforce at `parse_type_ref` via new `ParseError::TypePositionRequiresPascalCase`.

**Why:** PRD §4.9 commits to "hard error, not a lint" for type-name casing. The lexer has no type-position context (an identifier could be a variable name, a function name, or a type name depending on surrounding syntax), so the check has to happen at parse time when type position is known.

## C-3: Trailing commas

**Decision:** accept in both param lists and arg lists.

**Why:** diff-friendly (adding a param doesn't touch the previous line); no surface cost (one-line accept-and-stop in the loop).

## C-4: `redde` outside function

**Decision:** (a) parse cleanly, emit `return ...;` inside `fn main()`, let rustc reject.

**Why:** single-pass parser doesn't track function-nesting state. Tracking it just to produce one error message is the kind of complexity Stage 1 doesn't want. rustc's `error[E0308]: mismatched types` (`return` value type mismatches `()` for `fn main()`) is a perfectly adequate diagnostic.

**Considered:** parse-time nesting check. Rejected for cost.

## C-5: Empty param list

**Decision:** mandatory parens; `functio foo() :` parses cleanly with an empty `params: Vec<Param>`.

**Why:** symmetric with Rust (`fn foo() { ... }`). Going parens-optional would force a lookahead between `functio foo :` (no params) and `functio foo (...)` (with params) for no readability gain.

## D-1: Two-pass emit + always-`fn main()`

**Decision:** top-level `Stmt::Function`s hoist above `fn main()`; non-function statements land inside `fn main()`; `fn main()` is always emitted (cargo binary-crate requirement).

**Why:** preserves the v0.1 hello-world shape (statements still go into `fn main()`); cargo refuses to compile a binary crate without `fn main()`. The partition is one explicit `matches!` filter per pass — no flag, no Vec-of-functions intermediate.

## D-2: Translation table location

**Decision:** (b) match arm in `emit_type_ref` (2 entries today: `Sermo` → `String`, `Numerus` → `i64`).

**Why:** 2 entries have no growth pressure. A `HashMap` or const-table would cost more than it saves; when entries multiply (post-`structura` / `enumeratio`), promote.

## D-3: Param sigil emit

**Decision:** `^x: Sermo` → `x: String`; `@x: Sermo` → `mut x: String`.

**Why:** same sigil-drop rule as `Stmt::Let`. Mutability lives at the declaration; use sites discard sigils.

## D-4: Call emit shape

**Decision:** `foo(^x, 5)` → `foo(x, 5i64)`. Mechanical sigil-drop on var-refs and integer-suffix on int literals.

## Mid-implementation reframe: `GenericsDeferred` → `GenericsLookalike`

**Decision:** the originally-framed `ParseError::GenericsDeferred` would have been dead code — `<` and `>` aren't in Marain's lex alphabet, so the lexer's `UnexpectedChar` would fire long before any parser code ran. Replaced with `LexError::GenericsLookalike { ch, span }` at the lex layer.

**Caught:** during framing follow-through, before any code landed.

**Cost:** zero churn (no code to roll back).

**Pattern:** matches the [[feedback-reframe-vs-push-through]] memory — when a design starts fighting itself (parser variant unreachable because lexer fires first), surface the reframe with options rather than push through.

## Mid-implementation addition: `Stmt::Call`

**Decision:** added `Stmt::Call(CallStmt)` wrapping `CallExpr` + period span.

**Why:** original framing didn't sub-decision call-at-statement-position. Discovered during golden generation when `saluta().` failed to parse (`statement cannot begin with identifier`). Narrower than a generic `ExprStmt` — literals and var-refs at statement position have no observable effect, so only calls earn statement-position treatment in v0.2.

**Cost:** one variant + one dispatch arm (`PlainIdent + (` lookahead) + one emit arm. Mechanical.

**Lesson noted:** mid-implementation framing gaps are recoverable when narrow, but a signal to watch — if mid-implementation additions get bigger, the framing slate was too thin.
