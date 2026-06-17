# R17 — f-strings (interpolation + concatenation)

Closed 2026-06-17. Ships `f"…{^x}…"` (PRD §4.6 / §4.7) as sugar over `format!`,
resolving TODO Task 1 (no string composition). The whole literal — including each
`{…}` hole — is resolved in one lexer pass and emitted as a `format!` call. In-spec
(PRD already specs f-strings); no PRD amendment, just activation of a deferred
feature. New AST node `Expr::FString(FStringLit)`; new token `FStringLit(Vec<FStringSeg>)`.

## A — f-strings are the only string-composition mechanism

**Decision:** No concat operator/keyword. `f"{^a}{^b}"` IS the concatenation idiom
(`format!("{}{}", a, b)`); interpolation is `f"salve {^nomen}"`.
**Why:** PRD §4.7 states outright that "multi-value cases are handled by f-strings",
and §4.6 frames f-strings as the sugar over `format!`. `plus` is arithmetic-only and
`&str + &str` isn't valid Rust regardless. A separate operator would diverge from the
PRD and need a vision-doc amendment.
**Considered:** (a) a Latin concat keyword (e.g. some word → `+`/`push_str`) —
rejected: PRD-divergent, redundant with f-strings. (b) f-strings only — chosen.
**Owner-confirmed** 2026-06-16.

## B — Holes are variable-refs-only

**Decision:** A hole contains exactly one sigiled variable (`{^name}` / `{@name}`,
optional surrounding spaces). Empty holes (`{}`), no-sigil holes (`{nomen}`),
expression holes (`{^a plus ^b}`), and Rust format specs (`{x:>5}`) are all
`LexError::InvalidFStringHole`. Full expressions are deferred to a later round.
**Why:** Owner-locked (2026-06-16). Keeps the surface minimal and predictable, and —
decisively — lets the lexer resolve a hole by reusing `scan_sigiled_ident`, so the
entire f-string is produced in one pass with correct spans (decision C). The reuse
makes the var-only restriction *cheaper* to implement than full expressions would
have been, not more expensive.
**Result:** A hole is read directly to a `SigiledIdent`; emit reuses the ordinary
`VarRef` escaping path. `dbg!`-style format specs and computed holes are a clean
future extension (the AST `FStringPart::Interp` would widen from `SigiledIdent` to
`Expr`, and the lexer would need brace-balanced sub-lexing).

## C — One-pass lexer resolution (no parser sub-lexing)

**Decision:** `scan_fstring` (in `lexer/strings.rs`) scans the literal end-to-end,
calling `scan_sigiled_ident` on each hole; the token `FStringLit(Vec<FStringSeg>)`
carries already-resolved segments. The parser does a pure 1:1 lift to the AST.
**Why:** Because a hole is a slice of the *same* source file, scanning it inline
gives the `SigiledIdent` a correct source span and `FileId` for free — no re-lexing a
fragment, no span/offset remapping, no second lexer entry point. Internal `{`/`}` are
consumed inside the scanner, so they never reach the main dispatch and never perturb
the indent/bracket state machine.
**Considered:** (a) raw-token + parser re-lex of the hole substring — rejected: a
fresh `SourceFile` yields a wrong `FileId` and fragment-local spans needing remap;
error spans get fiddly. (b) lexer mode-stack emitting interleaved hole tokens —
rejected: spreads f-string state across the main loop, churning the clean per-kind
scanners. (c) one self-contained `scan_fstring` reusing `scan_sigiled_ident` — chosen,
viable precisely *because* holes are var-only (decision B).

## D — Prefix is `f"` with no intervening space

**Decision:** The f-string prefix is the byte `f` immediately followed by `"`. `f "x"`
(space) is a plain ident `f` then a string; `functio`/`fit` and `f(...)` take the
ident path.
**Why:** Variables always carry a sigil, so a bare `f` is only ever a keyword /
function name / no-punct macro — and a function call needs `(`, never `"`. So `f"` is
unambiguous. Mirrors Rust/Python. The dispatch is a one-line guard
(`b'f' if cursor.peek_at(1) == Some(b'"')`) placed before the generic ident arm.

## E — Emit as `format!`; split `emit.rs`

**Decision:** `emit_fstring` (in new `emit/expr.rs`) builds `format!("…{}…", a, …)`:
literal parts go to the format string with `{`/`}` doubled to `{{`/`}}`; each hole
contributes one `{}` placeholder and one trailing arg. `emit.rs` was at exactly the
500-LOC target, so the expression emitters (`emit_expr`, `emit_call`, `emit_fstring`)
moved to a child `emit/expr.rs`; the shared escapers + `EmitError` stay in `emit.rs`
and the child reaches them via `super::` (descendant access to private items).
**Why:** Per CLAUDE.md, the 500-LOC tier requires a real decomposition when one
exists; the file had a clean statement/expression seam, so split rather than justify.
Brace-doubling is single-pass over source chars so a control char's `\u{..}` escape
braces are not themselves doubled.
**Result:** `dic f"…"` double-wraps as `println!("{}", format!(…))` — correct, if
slightly redundant; a `dic`-special-case optimization is a future nicety, out of scope.
The same pressure also moved `lexer/mod.rs`'s driver tests to a sibling
`lexer/mod_tests.rs` (the split its own doc-comment had pre-authorized).

## PRD reconciliation

f-strings were listed as deferred-past-v0.1 (PRD §365, lexicon "deferred"). PRD §4.6 /
§4.7 already describe the surface; a footnote now records the v0.3 shipped subset
(var-only holes; full-expression holes and format specs deferred). Triple-quoted
strings (`"""…"""`) remain deferred — not part of this round.
