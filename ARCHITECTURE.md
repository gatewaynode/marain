# Marain — Architecture

_Document status: **DRAFT**, v0.1-ARCH-1. Only Round 1 (crate layout) is committed; Rounds 2–8 sections are outlined and labeled **TBD** until their design round closes._

## 0. Reading Order

This document derives from `PRD.md` and assumes the reader has it open. The PRD owns *what* Marain is; this document owns *how* the v0.1 implementation is shaped.

Design proceeds in eight numbered rounds. Each round closes in conversation, then crystallizes into the corresponding section here:

| Round | Section | Status |
| ----- | ------- | ------ |
| 1 | §2 Crate Layout, §3 On-Disk Paths | **Closed** |
| 2 | §4 Source & Span Model | **Closed** |
| 3 | §5 Error Model | **Closed** |
| 4 | §6 Lexer | **Closed** |
| 5 | §7 Parser & AST | **Closed** |
| 6 | §8 Codegen & Cargo Shim | **Closed** |
| 7 | §9 CLI & Driver | **Closed** |
| 8 | §10 Testing Harness | **Closed** |
| 9 | §12 Line Comments | **Closed** |
| 10 | §13 Block Parsing + `si` | **Closed** |
| 11+12 | §14 Operator Expressions + Control Flow | **Closed** |
| 13 | §15 Function Declarations + Calls | **Closed** |
| 14+15 | §16 Loops + Ranges + `nihil` | **Closed** |
| 16 | §17 Reassignment (`fit`) | **Closed** |
| 17 | §18 f-strings (interpolation + concatenation) | **Closed** |

§11 collects forward hooks that anticipate Stage 2 and other post-v0.1 work; it accretes across rounds.

## 1. Overview

Marain is a source-to-source transpiler: `.lat` source → tokens → AST → emitted Rust → `cargo` invocation → executable. The v0.1 done line is hello-world (PRD §7). The compiler front-end is hand-rolled per the self-supporting constraint (PRD §9, CLAUDE.md); no `logos`, no `chumsky`. The CLI uses `clap` (PRD §9, amended 2026-05-23), pinned per the N-1 / 30-day rule.

Pipeline as a diagram:

```
.lat source bytes
  ─▶ lexer    ─▶ tokens   (each carrying a Span)
  ─▶ parser   ─▶ AST      (each node carrying a Span)
  ─▶ emitter  ─▶ Rust source string
  ─▶ shim     ─▶ generated cargo project on disk (XDG state dir)
  ─▶ driver   ─▶ invoke `cargo run`, forward stdout/stderr
```

Stage 2 (post-v0.1, per PRD §4.10) interposes a lowering pass between parser and emitter, and replaces the parser entirely (free word order, case-driven assembly). Round 5 will reserve the seam.

## 2. Crate Layout

### 2.1 Workspace, two crates

```
marain/
  Cargo.toml                         # workspace manifest (no [package])
  Cargo.lock                         # single lockfile, committed
  rust-toolchain.toml                # pinned stable
  crates/
    marain-core/                     # compile pipeline (library)
      Cargo.toml
      src/lib.rs                     # crate root; modules added per round
    marain-cli/                      # the `marain` binary
      Cargo.toml                     # depends on marain-core (path)
      src/main.rs                    # thin shim; modules added per round
```

`default-members = ["crates/marain-cli"]` preserves Rust ergonomics: bare `cargo run --`, `cargo build`, and `cargo test` at the workspace root behave like a single-crate project. Explicit `cargo … -p marain-core` still works.

### 2.2 Rationale

- **Workspace, not single crate.** The Stage 2 LSP will want `marain-core` as a library dependency without dragging the CLI in. A workspace gives us the seam from day one; promoting a single crate later is cheap, but renaming all consumers is not.
- **Two crates, not three.** A separate `marain-build` (shim generator + cargo invocation) is plausible — but at v0.1 footprint the codepath is small enough that the crate boundary costs more than it earns. Promote when (and only when) the shim layer grows real surface area.
- **Where the shim module lives.** `marain-core::shim` generates the cargo shim project *given a target path* (path-agnostic). `marain-cli::paths` resolves XDG (UX concern, not a compiler concern). A future `marain-lsp` depending on `marain-core` inherits zero path policy.
- **No second binary in v0.1.** No `marain-fmt`, no `marain-repl`. PRD §6 lists those as out-of-scope.

### 2.3 Workspace conventions

- Each member inherits `version`, `edition`, and `rust-version` from `[workspace.package]` via `*.workspace = true` to avoid drift.
- `[workspace.lints.rust]` carries `unsafe_code = "forbid"`; each member opts in via `[lints] workspace = true`. Crate-root `#![forbid(unsafe_code)]` becomes redundant once this is in place but is harmless if added per file.
- Resolver `"3"` (required for edition 2024 workspaces).
- `Cargo.lock` is tracked (binary-crate convention; CLAUDE.md Rust guidance).

## 3. On-Disk Paths

### 3.1 What Marain owns, what XDG owns

| Lives in | What |
| --- | --- |
| `./target/` (project-local) | Marain's *own* cargo build artifacts. Standard Rust behavior; unchanged. |
| `$XDG_STATE_HOME/marain/builds/<name>-<hash>/` | One generated cargo shim project per input source file. Inspectable; cd-friendly. |
| `$XDG_STATE_HOME/marain/tmp/` | Atomic-write staging (emit-then-rename) so a partial transpile never leaves a corrupt shim. Pruned aggressively. |
| `~/.local/bin/<name>` | **Not auto-managed in v0.1.** A future `marain install` subcommand will drop user-program-binary symlinks here. |

`$XDG_STATE_HOME` defaults to `~/.local/state` per the XDG Base Directory spec. Resolution lives in `marain-cli/src/paths.rs` (~20 LOC; no `dirs` crate dependency).

### 3.2 Shim project identity

`<name>-<hash>` keys each shim project to its source: `<name>` is the source basename (`hello` for `hello.lat`), `<hash>` is a short (8-hex) digest of the source file's canonical absolute path. The basename gives a human-readable directory; the hash prevents collision when two `hello.lat`s exist in different directories. Content hashing is *not* used here — re-transpiling the same source overwrites its shim rather than minting a new one.

### 3.3 Day-to-day workflow

```
$ marain build hello.lat
  → writes ~/.local/state/marain/builds/hello-a3f29b1c/{Cargo.toml,src/main.rs}

$ marain run hello.lat
  → as above, then invokes `cargo run` inside the shim, forwarding stdout/stderr.
```

The shim is inspectable: `cd ~/.local/state/marain/builds/hello-a3f29b1c && cargo doc` works without any special handling.

## 4. Source & Span Model

### 4.1 Types

```rust
// marain-core/src/span.rs
pub struct FileId(NonZeroU32);          // niche-optimized; Option<FileId> stays 4 bytes
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub file: FileId,
}

impl Span {
    pub fn join(self, other: Self) -> Self;   // debug_assert same file
    pub fn len(self) -> u32;
    pub fn is_empty(self) -> bool;
}

// marain-core/src/source.rs
pub struct SourceFile { /* id, path, text: String, line_starts: Vec<u32> */ }
impl SourceFile {
    pub fn line_col(&self, offset: u32) -> (u32, u32);  // 1-indexed; binary search
}

pub struct SourceMap { /* files: Vec<SourceFile> */ }
impl SourceMap {
    pub fn add(&mut self, path: PathBuf, text: String) -> FileId;
    pub fn get(&self, id: FileId) -> &SourceFile;
}
```

### 4.2 Decisions and rationale

- **Multi-file-ready from day one.** Every span carries a `FileId`, even though v0.1 only ever populates a single file. Trade: ~50% more bytes per AST node now (12–16 bytes vs. 8), vs. a mechanical sweep when modules / Stage-2 LSP cross-file diagnostics land. Stage 2 LSP's multi-file requirement made deferral untenable.
- **`FileId(NonZeroU32)`.** Niche optimization keeps `Option<FileId>` at 4 bytes. Discipline enforced by `FileId::new(u32) -> Option<Self>`. Zero is the sentinel for "absent."
- **`SourceFile` owns a UTF-8 `String`.** Identifiers are ASCII-only per PRD §4.9, but string literals (and eventually comments) tolerate UTF-8. Validate-once-at-load gives a clean error before the lexer ever sees the bytes; `&str` ergonomics for the lexer.
- **Eager line index.** Vec of line-start byte offsets, computed once in `SourceFile::new`. Lazy `OnceCell` rejected: line index is microseconds on Marain's file sizes; lazy adds first-render latency for no realistic savings.
- **`Span::join` is debug-asserted, not `Result`-typed.** Cross-file join is always a compiler bug; `debug_assert_eq!` over `Result` keeps internal AST-combination code noise-free.
- **No global SourceMap.** `marain-cli` owns one, threads `&SourceMap` into the diagnostic renderer and `&SourceFile` into the lexer. Library never holds global state. Standard rustc / GCC / LLVM pattern.
- **`FileId::new` and `FileId::raw` are `pub(crate)`.** External crates obtain `FileId`s only via `SourceMap::add`; they cannot mint one. Tests construct via the crate-private constructor since they live in the same crate.

### 4.3 File layout

- `crates/marain-core/src/span.rs` — `FileId`, `Span`, `Span::join/len/is_empty`. ~110 LOC including tests.
- `crates/marain-core/src/source.rs` — `SourceFile`, `SourceMap`, `compute_line_starts`. ~150 LOC including tests.

Both well under the 500-LOC ceiling; no further decomposition planned for Round 2's surface.

### 4.4 Test coverage

Unit tests live in `#[cfg(test)] mod tests` at the bottom of each file (CLAUDE.md convention).

- `span.rs` — `FileId::new(0)` rejection; `Option<FileId>` size-of-4 (niche check); `Span::join` disjoint / overlapping / commutative; `Span::len` + `is_empty`; debug-assert panic on cross-file join.
- `source.rs` — `compute_line_starts` empty / no-newline / multi-line / trailing-newline; `line_col` at first char / within line / at newline byte / start of next line / within next line; `SourceMap` round-trip; first FileId is 1.

## 5. Error Model

### 5.1 Pattern

Errors layer per-stage; a facade composes them via `From`; diagnostics are the renderable unit.

```rust
// marain-core/src/error.rs (committed in Round 3)
pub enum Severity { Error, Warning }

pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self;
    pub fn warning(span: Span, message: impl Into<String>) -> Self;
    pub fn render(&self, map: &SourceMap) -> String;
    // hello.lat:3:14: error: unexpected character '?'
}

// Materializes when the first stage error lands (Round 4).
pub enum MarainError {
    // Lex(LexError),     // Round 4
    // Parse(ParseError), // Round 5
    // Shim(ShimError),   // Round 6
    // Driver(...),       // Round 7
}

// Per-stage error convention (applied in Round 4+):
//   pub enum LexError { /* variants with explicit span: Span fields */ }
//   impl LexError {
//       pub fn to_diagnostic(&self, map: &SourceMap) -> Diagnostic;
//   }
//   impl From<LexError> for MarainError { fn from(e: LexError) -> Self { Self::Lex(e) } }
```

### 5.2 Decisions

- **Per-stage enums** over one flat enum. Exhaustive `match` catches missed variants at compile time as each stage's error set evolves.
- **`MarainError` facade** composes stage enums via `From`. Hand-rolled `Display` + `std::error::Error` impls; no `thiserror` (PRD §9 self-supporting constraint). `MarainError::to_diagnostic` dispatches to the variant's own method.
- **Fail-fast.** Each phase returns `Result<T, E>`. No `Vec<Diagnostic>` collection in v0.1; lexer/parser do not implement error recovery. Promoting later (returning `(T, Vec<Diagnostic>)`) is a strict superset of the current contract, so the seam survives.
- **Spartan `Diagnostic`.** Severity + Span + message. Renderer emits `path:line:col: severity: message`. No labeled spans, no hint/note slots, no carat-under-source. Adding `hint`/`note` is a backward-compatible field addition; full rustc-style rendering is its own milestone (post-v0.5 per PRD §5 rustc-span-mapping wart).
- **Per-variant spans, not `Spanned<E>` wrapper.** Some variants will need multiple spans (e.g., "unterminated string starting at X, EOF at Y"); wrapping forces those into the `kind` field, obscuring the data. Explicit `span:` fields per variant read better.
- **Diagnostic ≠ Error.** `Diagnostic` is the *output* (renderable, user-facing); `*Error` enums are the *thrown* (carried by `Result`). The boundary is the `to_diagnostic(&SourceMap)` method on each stage error. Decoupling lets us add non-error diagnostics later (Stage-2 grammar hints, lints) without warping the error types.
- **MarainError deferred to Round 4.** No empty facade enum in Round 3; the type materializes when the first stage error (LexError) exists to populate it. Convention is documented here so Round 4 has no design work to repeat.

### 5.3 File layout

- `crates/marain-core/src/error.rs` — `Severity`, `Diagnostic`, `Diagnostic::render`. ~140 LOC including tests as of Round 3.
- Per-stage enums (`LexError`, `ParseError`, …) live in their respective module files and grow with each round.
- `MarainError` facade — declared in `error.rs` once the first stage variant materializes (Round 4).

### 5.4 Test coverage

`error.rs` unit tests cover: `Severity::Display` for both variants; `Diagnostic::error` and `Diagnostic::warning` constructors; render at first-line / first-column; render against a span starting on a later line; render with a subdirectory path; render with an offset column within a line.

### 5.5 Forward hooks

Backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) — rustc→Marain error back-mapping (§6) and a future Stage 2 `GrammarError` (§5). Both follow the existing `From` → `Diagnostic` pattern, so the error-model shape is unchanged; no v0.1 work.

## 6. Lexer

### 6.1 Scope (v0.1)

**In:** keywords (table-driven), sigiled identifiers (`@x` / `^x`), plain identifiers, simple string literals `"..."` with `\"` `\\` `\n` `\t` `\r` `\0` escapes, decimal integer literals (with `_` separators), period, comma, colon, double-colon (`::`), bang (`!`), `LParen` / `RParen` / `LBracket` / `RBracket` / `LBrace` / `RBrace`, indentation (synthetic `Indent` / `Dedent` with spaces-only enforcement and bracket suppression), `Eof`.

**Out (deferred):** triple-quoted strings (`"""..."""`), f-strings, numeric richness (hex/oct/bin prefixes, exponent, floats), multi-word operator phrases (parser-level per PRD §4.4), comments (no PRD spec yet), Stage 2 `(lemma, inflection)`. Comments specifically are an open question — for now, every byte outside a string/integer/ident is a `LexError::UnexpectedChar`, so any future comment syntax must be added before user code can ship them.

### 6.2 Decomposition

```
crates/marain-core/src/
  token.rs                  # TokenKind, Token, Sigil
  lexer/
    mod.rs                  # lex(&SourceFile) -> Result<Vec<Token>, LexError>
    cursor.rs               # byte cursor: peek/advance/slice/pos
    indent.rs               # IndentState: indent stack + bracket depth
    strings.rs              # string literal scanning + escape resolution
    numbers.rs              # decimal integer (with `_` separators)
    idents.rs               # plain + sigiled identifier; keyword dispatch
    keywords.rs             # Keyword enum + lookup/as_str
    error.rs                # LexError + to_diagnostic + Display
```

Seams are semantic, not arbitrary: each file owns one *kind* of token (string, number, ident) or one piece of *mechanism* (cursor, indent state). When `"""..."""` or f-strings land later, they extend `strings.rs`; hex/float literals extend `numbers.rs`; new keywords append to `keywords.rs`. No existing file gets churned.

At v0.1 close, every file lives well under the 500-LOC target. `indent.rs` was the projected pressure-release candidate; the actual implementation lands ~180 LOC, comfortably inside the target.

### 6.3 Decisions

- **Byte-level cursor.** Marain identifiers are ASCII per PRD §4.9, so byte scanning is the natural model. UTF-8 in string literals is preserved via chunked `&str` slicing — special bytes (`"`, `\\`, `\n`) are all ASCII and never collide with UTF-8 continuation bytes, so chunk boundaries always align with char boundaries.
- **Sigiled identifier is one token.** `@x` and `^x` emit a single `SigiledIdent { sigil, name }`. The parser sees one token and knows immediately it's a variable reference. PRD §4.5's "omitting a sigil on any variable reference is a parse error" becomes a parser-level check on the presence/absence of `SigiledIdent` vs. `PlainIdent` in variable positions.
- **Keyword table is exhaustive.** All reserved words land in `keywords.rs`, including operator-word components (`plus`, `et`, `vel`, `quam`, `par`, …) per PRD §4.4. Multi-word phrase recognition (`maior quam` → `≥`) is a parser-level concern; the lexer emits `Keyword(Maior)` + `Keyword(Quam)` as separate tokens.
- **Sigiled idents skip keyword lookup.** `^et` is a variable named `"et"` (PRD §4.5 says the sigil unambiguously marks a variable). Only bare identifier scanning consults the keyword table.
- **Indentation = spaces only.** Tabs anywhere — indentation or mid-line — are a hard `LexError::TabCharacter`. The "mixed tabs/spaces" failure mode from PRD §4.6 is prevented categorically by rejecting all tabs.
- **Bracket depth suppresses indent.** Inside `()`, `[]`, `{}` the line breaks don't produce indent tokens — standard Python rule. Bracket tracking lives in `IndentState`; the driver toggles it on bracket tokens.
- **No `Newline` token.** Newlines are layout per PRD §4.8 (period terminates statements). The lexer tracks newlines internally for indent purposes but emits no token. If proven wrong later, easy to add.
- **Fail-fast errors.** Lexer returns `Result<Vec<Token>, LexError>` per the Round 3 contract. No error recovery, no `Vec<LexError>` collection. First error wins.
- **Keyword lookup is a `match`, not a `HashMap`.** ~30 entries at v0.1; `match` is faster (no hashing, no heap allocation) and as future-proof as the table itself. Promote to a perfect-hash later only if the table grows past ~200 entries.

### 6.4 `LexError` variants (v0.1)

| Variant | Trigger |
| --- | --- |
| `UnexpectedChar { ch, span }` | A character that does not start any token |
| `UnterminatedString { span }` | `"...` without closing quote before EOF or `\n` |
| `TabCharacter { span }` | Tab byte (any position) |
| `InvalidEscape { ch, span }` | Unknown `\X` inside a string |
| `InconsistentIndent { span }` | Dedent to a level not on the indent stack |
| `SigilWithoutIdent { sigil, span }` | `@` or `^` not followed by an identifier |
| `InvalidInteger { text, span }` | Integer literal failed to parse (overflow) |

Each variant carries its span(s) directly; `to_diagnostic()` wraps as a `Diagnostic`.

### 6.5 `MarainError` facade activation

`LexError` is the first stage error to materialize, so this round also activates the `MarainError` facade promised in §5:

```rust
pub enum MarainError {
    Lex(LexError),
}

impl From<LexError> for MarainError { /* … */ }
impl MarainError {
    pub fn to_diagnostic(&self) -> Diagnostic { /* dispatches to LexError */ }
}
impl Display for MarainError { /* delegates */ }
impl std::error::Error for MarainError { /* source() chains */ }
```

`ParseError`, `EmitError`, `ShimError`, etc. extend this enum as their rounds close.

### 6.6 Test coverage

- **Per-file unit tests** — every scanner has isolation tests against a hand-built `Cursor`: simple cases, edge cases, error cases. Indent state machine tests cover indent / dedent (single & cascading) / no-change / inconsistent / bracket suppression / finalize.
- **Driver integration tests** (in `lexer/mod.rs`) — hello-world, sigiled binding, indented block, nested-cascading dedents, blank-line skipping, bracket-suppressed indents, tab-in-indent, tab-mid-line, unexpected char, unterminated string, double-colon vs colon, bang separateness, integer with separators, empty source, whitespace-only source, `DETONATIO` recognition, escape sequences, no-trailing-newline drain, inconsistent indent, multiple statements per line.

100 tests pass at Round 4 close (25 carry-over + 75 new).

### 6.7 Pressure-release tier 1 not invoked

The 500-LOC target held for all eight lexer files. The pressure-release rule (CLAUDE.md "Small and Modular") remained on the shelf for Round 4. The most-likely future invocation site is still `indent.rs` once Stage 2 lands grammar-conditional indentation, or `strings.rs` if triple-quoted + f-string + interpolation logic all converge there.

## 7. Parser & AST

### 7.1 Scope (v0.1)

**In:** top-level statement sequence — `Module = Stmt*` — over five productions:

| Production | Concrete syntax | AST node |
| --- | --- | --- |
| let-binding | `sit <sigiled-ident> est <expr> .` | `Stmt::Let(LetStmt)` |
| no-punct macro call | `<dic\|queror\|agmen\|forma> <expr> .` | `Stmt::MacroCall(MacroCallStmt)` |
| string literal expr | `"…"` | `Expr::StringLit(StringLit)` |
| integer literal expr | `42`, `1_000_000` | `Expr::IntegerLit(IntegerLit)` |
| variable reference | `^x` / `@x` | `Expr::VarRef(SigiledIdent)` |

That set covers the PRD §7 done line (`dic "salve, munde".`) and the most-natural-next-thing (bindings + var references), giving us a parser exercisable by real 2-statement programs (`sit ^x est 5. dic ^x.`).

**Out (deferred):** operator expressions (precedence climbing, multi-word phrase table per PRD §4.4), indented blocks (gated by PRD §4.8 control-structure-head terminator, still TBD), `!`-bearing macros + argument lists, `functio` / `redde` / `si` / `dum` / `pro` / `structura` / `enumeratio`, multi-line continuation (already lexer-level via bracket suppression but not exercised by R5 productions), pattern syntax, types.

### 7.2 Decomposition

```
crates/marain-core/src/
  ast.rs                    # types only; Module/Stmt/Expr + Ident/SigiledIdent wrappers
  parser/
    mod.rs                  # pub fn parse(&[Token]) -> Result<Module, ParseError>
                            # internal `Parser<'tokens>` cursor: peek_kind / peek_span /
                            # current_clone / advance / at_eof
    grammar.rs              # one fn per production: parse_module / parse_stmt /
                            # parse_let / parse_macro_call / parse_expr /
                            # parse_sigiled_ident; expect_keyword / expect_kind helpers
    error.rs                # ParseError enum + to_diagnostic + Display + Error
```

All files comfortably inside the 500-LOC target at this scope: `ast.rs` ~195 LOC, `parser/mod.rs` ~290 LOC, `parser/grammar.rs` ~140 LOC, `parser/error.rs` ~125 LOC (all counts include tests).

Seams are by responsibility: types vs. driver vs. productions vs. errors. When operator expressions land (precedence climbing, phrase table), `grammar.rs` is the file that grows; when new statement forms land, the same. AST file extensions go into `ast.rs` until it crowds 500 LOC, at which point a directory split (`ast/{stmt,expr,item}.rs`) becomes the natural decomposition.

### 7.3 Decisions

- **Recursive descent, hand-written.** Stage 1's Rust-fixed word order (PRD §4.2) is the natural fit for recursive descent. Stage 2 likely throws this away (case-driven assembly, GLR / Earley, per concern α); the Stage 1 parser is a deliberate throwaway and is sized accordingly.
- **AST nodes are enum-of-structs.** `Stmt`, `Expr` are enums whose variants wrap dedicated structs (`LetStmt`, `MacroCallStmt`, `StringLit`, `IntegerLit`). Idiomatic for pattern-matching dispatch and keeps each variant's data cohesive. `Stmt::span()` / `Expr::span()` dispatch through the wrapped struct's `span` field, avoiding scattered span-extraction logic in callers.
- **Identifiers wrap into [`Ident`] and [`SigiledIdent`].** Rather than scattering `inflection: Option<Inflection>` on six different node types, all identifier-bearing positions go through one of these two wrappers. Stage 2 grows `Inflection` once; every consumer follows. Constructors `Ident::new(name, span)` and `SigiledIdent::new(sigil, name, span)` default `inflection: None`, so Stage 1 parsing sites never type the field. Carry-over concern α (ARCHITECTURE.md §11) lands here.
- **`Inflection` is an empty marker struct.** Stage 1 has nothing to put in it; the type exists purely to reserve the `Option<Inflection>` field's shape. Adding real fields in Stage 2 is a backward-compatible structural extension.
- **One AST layer for v0.1.** AST is the emit-ready form. No separate HIR / MIR. When Stage 2 needs a lowering pass (case-driven free-word-order → fixed-form Stage 1 shape), interpose between parser and emitter; the AST type is the seam.
- **Fail-fast.** Parser returns `Result<Module, ParseError>` — no error recovery, no `Vec<Diagnostic>` accumulation. R3's contract carries forward unchanged. Promoting later (parser collects multiple errors before returning) is a strict superset of the current shape, so the seam survives.
- **Token cursor borrows the slice.** `Parser<'tokens>` holds `&'tokens [Token]` and a `pos: usize`. `advance()` mutates `pos`; `current_clone()` returns a `Token` (cheap clone since `String` payloads are small at v0.1 scope). Stage 1 grammar functions clone the leading `Token` to obtain owned `String` payloads for AST construction; this avoids wrestling the borrow checker for a v0.1 throwaway parser.
- **Parser requires `Eof`-terminated input.** Lexer's contract is "always emit `Eof` last." `Parser::new` `debug_assert!`s this so a malformed feed fails loudly in debug builds rather than silently slipping past index bounds.
- **`expect_kind` uses `std::mem::discriminant` equality** for variant matching that ignores payload. Lets one helper handle every "is this the right TokenKind variant" check (`Period`, `Eof`, future `LBracket`, etc.) without growing one helper per kind. The trade is one line of explanation; the alternative is N copy-pasted matchers.
- **Macro-call arity is enforced structurally.** `parse_macro_call` parses exactly one expression, then expects period. `dic.` (no arg) surfaces as `ExpectedExpression { found: Period }`; `dic "a" "b".` as `UnexpectedToken { expected: \`.\` }`. No separate `InvalidMacroArity` variant — the grammar's structure already enforces the rule.
- **`TokenKind: Display`** added to `token.rs` so parser errors can render token names without leaking literal payloads (`"string literal"` not `"\"the contents\""`). Keeps diagnostic text terse and avoids dumping arbitrary user data into error messages.

### 7.4 `ParseError` variants (v0.1)

| Variant | Trigger |
| --- | --- |
| `UnexpectedToken { found, expected, span }` | wrong token at a known position; `expected` is a `&'static str` label (e.g. `` "keyword `est`" ``, `` "`.`" ``) |
| `ExpectedExpression { found, span }` | `parse_expr` saw a token that cannot start an expression |
| `UnknownStatementStart { found, span }` | first token of a statement matches no known statement form |

Each variant carries its span directly; `to_diagnostic()` wraps as a `Diagnostic`. `MarainError::Parse(ParseError)` joins the facade via `From`; `to_diagnostic` / `Display` / `std::error::Error::source` all dispatch through. Convention identical to R4's `LexError` plumbing.

### 7.5 Inflection slot pattern (carry-over α resolution)

```rust
pub struct Inflection;                       // Stage 1: empty; Stage 2: grows fields

pub struct Ident {                           // bare identifier (macro callees, fn names)
    pub name: String,
    pub span: Span,
    pub inflection: Option<Inflection>,
}

pub struct SigiledIdent {                    // every variable reference per PRD §4.5
    pub sigil: Sigil,
    pub name: String,
    pub span: Span,
    pub inflection: Option<Inflection>,
}
```

Stage 2 either: (a) populates `inflection: Some(Inflection { … })` at the same construction sites (parser becomes inflection-aware), or (b) Stage 2 introduces its own constructors that fill the slot. Either path leaves the AST *type* unchanged.

### 7.6 Test coverage

- **`ast.rs`** — `Ident::new` / `SigiledIdent::new` default `inflection` to `None`; `Stmt::span` / `Expr::span` dispatch; `Inflection::default()` constructs.
- **`parser/error.rs`** — message formatting for each variant; `span()` round-trip; `to_diagnostic` carries message + span; `Display` delegates.
- **`parser/grammar.rs`** — covered transitively by the driver tests in `parser/mod.rs`; no dedicated unit tests at v0.1 scope.
- **`parser/mod.rs` driver tests** — hello-world parses; let with integer / string / var-ref RHS; dic of var-ref; multi-statement; multi-statements-on-one-line; empty source; whitespace-only source; `dic.` (no arg); `dic "a" "b".` (trailing garbage); `sit ^x 5.` (missing est); `sit x est 5.` (no sigil); `functio foo.` (unknown statement start); `dic "a"` (missing period at eof); `sit @y est ^x.` (var-ref value); `1_000_000` integer round-trip; let-stmt span covers sit..period; macro-call span covers keyword..period; inflection slot is None after parse; queror / agmen / forma all dispatch; `ParseError` joins facade as `MarainError::Parse`; `Parser::new` panics in debug on missing `Eof`.
- **`error.rs`** — four new tests for the `Parse` variant of `MarainError` mirroring the existing `Lex` variant tests.
- **`token.rs`** — five tests for `TokenKind: Display` covering literal-value redaction, punctuation rendering, keyword rendering, sigil rendering, EOF rendering.

142 tests pass at Round 5 close (100 carry-over from R4 + 42 new).

### 7.7 Pressure-release tier 1 not invoked

All four R5 files land well under the 500-LOC target. The plausible future pressure points are `grammar.rs` (when operator expressions + precedence climbing + multi-word phrase table land) and `ast.rs` (when item types — `structura`, `enumeratio`, `functio` bodies — grow). Neither is in v0.1's path.

### 7.8 Forward hooks

Backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) — inflection content (θ) and the parser→emitter lowering pass (§5); expression-position macros `MacroCallExpr` (§1). Each is an additive AST/seam change: extending `Inflection`, interposing lowering at the `parse() -> Module -> emit()` seam, or adding an `Expr::MacroCall` variant — consumers that ignore the new shape keep compiling.

## 8. Codegen & Cargo Shim

### 8.1 Scope (v0.1)

Two responsibilities, two flat files in `marain-core`:

| Concern | File | Surface |
| --- | --- | --- |
| AST → Rust source string (pure) | `emit.rs` | `pub fn emit(&Module) -> Result<String, EmitError>` |
| Rust source → cargo project on disk | `shim.rs` | `pub fn render_cargo_toml(&str) -> String`; `pub fn write_shim(&Path, &str, &str) -> Result<(), ShimError>` |

Path policy (XDG resolution, `$XDG_STATE_HOME/marain/builds/<name>-<hash>/`) stays out of R6 per §2.2 — that's the R7 CLI's job. `write_shim` takes the target directory as an argument and writes faithfully wherever it's pointed.

### 8.2 Decisions

- **`emit` is path-agnostic and returns a `String`.** Filesystem concerns are pushed entirely into `shim.rs`. The emitter is a pure function on the AST, fully unit-testable without touching disk.
- **`emit` returns `Result<String, EmitError>`.** The only failure mode is a Marain identifier that collides with one of the five Rust reserved words that cannot be raw-escaped (`crate`, `extern`, `self`, `Self`, `super`). Every other R5 production has a mechanical Rust mapping.
- **Rust-keyword escaping is the emitter's job, not the lexer's.** Marain identifier rules (PRD §4.9: ASCII alpha+underscore start, alphanumeric+underscore continue, Rust casing) live in the lexer; *Rust's* keyword set is a Rust concern that we paper over at the boundary. Strict and reserved-for-future keywords from the Rust 2024 reference get `r#` prefixed; the five unescapable words surface as `EmitError::UnescapableRustKeyword`.
- **No silent mangling for unescapable keywords.** An emit-time error pointing at the original Marain source span is more honest than mangling `self` → `self_` (which could silently collide with a separate Marain `^self_` binding). User fixes the name; emitter stays simple.
- **Future-reserved Rust keywords escape today.** `become`, `abstract`, `final`, `typeof`, `unsized`, `virtual`, `priv`, `override`, `box`, `macro` are all included in the escapable set. A Marain program that parses today still emits valid Rust when those words become active in a future Rust edition.
- **`dic <expr>` → `println!("{}", arg)` uniformly, even for string literals.** Avoids the `{}`-as-format-placeholder footgun where `dic "{} brace".` would otherwise emit `println!("{} brace")` and rustc would interpret `{}` as a positional placeholder.
- **Integer literals emit with `i64` suffix.** Forces type to match the lexer's parsed representation; prevents `let x = 5_000_000_000;` defaulting to i32 and overflowing.
- **Sigils discarded at emission.** `^x` and `@x` both emit as bare `x` in expression position. Marain mutability is encoded by the *declaration* sigil (`@` in `sit @x est 5.` → `let mut x`), not by use-site sigil. Borrow syntax (`tenet`) lands later.
- **`ShimError` stands alone — does not join `MarainError`.** It has no `Span` (it wraps `io::Error`), so it cannot produce a source-level `Diagnostic`. The R7 driver will compose `MarainError + ShimError + io::Error` at the binary boundary; keeping `MarainError` confined to source-mappable errors preserves the §5 contract.
- **Atomic-write via sibling staging + `fs::rename`.** Stage `<parent>/.staging-<basename>`, write all files in, remove old target if present, atomic rename. Same-filesystem rename is atomic on Unix; on the small window between "remove old target" and "rename staging in," target is missing. Acceptable for single-user build artifacts (PRD audience, §3).
- **Cargo.toml is the minimum that works, plus an *empty* `[workspace]` table.** `[package]` table with name + version + edition; empty `[workspace]` opts the shim out of any enclosing cargo workspace it may sit inside (the project-local test scratch under `.scratch/`, or any user workspace if a shim ever co-locates with user source). Without it, cargo walks up from the shim's manifest, finds an outer workspace, and rejects the shim as a non-member. No `[[bin]]` (auto-discovered), no `[dependencies]` (v0.1 emits no `use` of external crates, no `Variabile` runtime yet per concern γ).

### 8.3 Emission mapping

| Marain | Rust |
| --- | --- |
| `dic <expr>.` | `println!("{}", <expr>);` |
| `queror <expr>.` | `eprintln!("{}", <expr>);` |
| `agmen <expr>.` | `vec![<expr>];` |
| `forma <expr>.` | `format!("{}", <expr>);` |
| `sit ^x est <expr>.` | `let x = <expr>;` |
| `sit @x est <expr>.` | `let mut x = <expr>;` |
| `"…"` (StringLit) | Rust-escaped: `\\`, `\"`, `\n`, `\t`, `\r`, `\0`, `\u{…}` for other controls |
| `42` (IntegerLit) | `42i64` |
| `^x` / `@x` where `x` is not a Rust keyword | `x` |
| `^if` / `@if` (Rust strict or future-reserved keyword) | `r#if` |
| `^self` / `@self` / `^Self` / `^crate` / `^super` / `^extern` | **`EmitError::UnescapableRustKeyword`** |

For the v0.1 done line:

```
Marain:  dic "salve, munde".
Rust:    fn main() {
             println!("{}", "salve, munde");
         }
```

This output, written via `write_shim` and invoked via `cargo run`, prints `salve, munde` on stdout — verified by `tests/e2e_hello_world.rs` (see §8.8).

### 8.4 `EmitError` variants (v0.1)

| Variant | Trigger |
| --- | --- |
| `UnescapableRustKeyword { name, span }` | Marain identifier matches one of the five Rust keywords (`crate`, `extern`, `self`, `Self`, `super`) that raw-identifier syntax cannot escape |

`MarainError::Emit(EmitError)` joins the facade via `From`; convention identical to R4's `LexError` and R5's `ParseError` plumbing.

### 8.5 `ShimError` variants (v0.1)

| Variant | Trigger |
| --- | --- |
| `CreateDir { path, source }` | `fs::create_dir_all` failed |
| `WriteFile { path, source }` | `fs::write` failed |
| `RemoveDir { path, source }` | `fs::remove_dir_all` failed |
| `Rename { from, to, source }` | `fs::rename` failed |

All variants carry the offending path(s) and wrap the underlying `io::Error` for `std::error::Error::source()` chaining. `Display` renders as `failed to <op> <path>: <io-error-message>` — terse, no double-quoting.

### 8.6 Atomic-write protocol

For target directory `<parent>/<basename>`:

1. Compute `staging = <parent>/.staging-<basename>`.
2. If `staging` exists from a prior crashed invocation → `remove_dir_all`.
3. If `<parent>` doesn't exist → `create_dir_all(<parent>)`.
4. `create_dir_all(<staging>/src)`, write `<staging>/Cargo.toml`, write `<staging>/src/main.rs`.
5. If `<target>` exists → `remove_dir_all`.
6. `fs::rename(<staging>, <target>)`.

Steps 5+6 leave a small window where target is missing; not atomic in the strict POSIX sense, but sufficient for single-user build artifacts. If step 6 succeeds, both files are guaranteed present in the final target. If any step fails, the prior target (if any) is undisturbed except in the window between 5 and 6.

### 8.7 Test coverage

- **`src/emit.rs`** — 32 unit tests:
  - Skeleton: empty module, fn-main bracket match, multi-statement ordering.
  - All R5 productions: hello-world done line, each macro (`dic`/`queror`/`agmen`/`forma`), let with all three RHS forms (integer/string/var-ref), integer suffix, var-ref discards sigil.
  - String escape: quote, backslash, newline, tab, control char, UTF-8 passthrough.
  - Rust-keyword escaping: `r#if`, `r#async` (2018+), `r#gen` (2024+), `r#become` (future-reserved); `dic ^if.` correctly escapes both the binding and the reference.
  - `EmitError` for all five unescapable keywords (`self`, `Self`, `extern`, `crate`, `super`), error span correctness, `to_diagnostic`, `Display`, and `MarainError` facade join.
  - Round-trip both classification predicates over the complete 45+5 keyword tables (catches drift if the constants change without intent).
- **`src/shim.rs`** — 11 unit tests:
  - `render_cargo_toml`: package section, version, edition; absence of `[workspace]` / `[[bin]]`.
  - `write_shim`: fresh write creates both files; creates `src/` subdir; overwrites existing target; cleans up leftover staging from a prior crash; creates missing parent dir.
  - `ShimError`: `Display` includes paths, `source()` chains to `io::Error`, `Rename` variant shows both paths.
  - `staging_path_for`: sibling of target; relative-target-with-no-directory edge case.
  - Disk tests use a `TempDir` RAII guard under the project-local `.scratch/` directory (gitignored; resolved from `CARGO_MANIFEST_DIR`). Cleans up on drop, so panicking tests don't leave debris; surviving debris (from a hard crash) is inspectable in-tree and a single `rm -rf .scratch` from project root clears it.
- **`src/error.rs`** — 4 new unit tests for the `Emit` variant of `MarainError` mirroring the `Lex` / `Parse` variant tests.
- **`tests/e2e_hello_world.rs`** — 1 integration test (§8.8 below).

198 unit tests + 1 integration test pass at Round 6 close (56 new unit tests + the e2e integration).

### 8.8 End-to-end smoke test

`crates/marain-core/tests/e2e_hello_world.rs` exercises the full library pipeline:

1. Lex + parse `dic "salve, munde".\n` → AST.
2. `emit()` → `fn main() {\n    println!("{}", "salve, munde");\n}\n`.
3. `write_shim()` → tempdir gets `Cargo.toml` + `src/main.rs`.
4. `Command::new("cargo").args(["run", ...])` → asserts stdout is `salve, munde`.

The test uses an RAII `TempDir` guard rooted at the project-local `.scratch/` directory (gitignored), unsets `CARGO_TARGET_DIR` for the spawned cargo (so it doesn't race with the test runner's own `target/`), and runs in well under a second on a warm cache. R8 (testing harness) will own systematic e2e coverage; this single test is the smoke test that lives from R6 forward and fails fast on any regression to emit or shim shape.

This is also the operational proof of the PRD §7 v0.1 done line at the library layer — only the CLI wrapper (R7) sits between the user and a working `marain run hello.lat`.

### 8.9 Pressure-release tier 1 not invoked

Both R6 files comfortably under the 500-LOC target. The plausible future pressure points are `emit.rs` (when operator expressions, indented blocks, functions, structs, and Variabile literal forms land — each adds an `emit_<kind>` arm) and `shim.rs` (when the Variabile runtime emission requires writing a third file `src/variabile.rs` per shim). Neither is in v0.1's path.

### 8.10 Forward hooks

Backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) — `Variabile` runtime injection (γ, §3; vendored `src/variabile.rs` emitted by `shim.rs` + a `mod variabile;` prepend, source as a static string per the self-supporting constraint) and rustc-error span back-mapping (§6; an extra emission pass interleaving `// span` comments, no architectural change here).

Resolved here: **multi-statement function bodies** — `emit_stmt` gained the per-block `indent_level` parameter (R10/§13). **Workspace inheritance** — `render_cargo_toml` emits an empty `[workspace]` table unconditionally (R6), so a shim opts out of any enclosing workspace regardless of location.

## 9. CLI & Driver

### 9.1 Scope (v0.1)

The `marain` binary exposes the two PRD §6 subcommands and nothing else:

| Command | Behavior |
| --- | --- |
| `marain build <file.lat>` | Lex + parse + emit + write shim. Prints the shim directory path to stdout on success. |
| `marain run <file.lat>` | Same as `build`, then invokes `cargo run --quiet --manifest-path <shim>/Cargo.toml`, inheriting stdio. Exits with cargo's exit code on success or with `1` on a driver error before cargo runs. |

`marain --help` and `marain --version` come from `clap` for free. Exit 0 on success, 1 on any [`DriverError`].

### 9.2 Decomposition

```
crates/marain-cli/
  Cargo.toml                  # clap = "=4.5.61" features = ["derive"]; marain-core path dep
  src/
    main.rs                   # parse args, dispatch, report error, exit
    args.rs                   # #[derive(Parser)] Cli + #[derive(Subcommand)] Command
    paths.rs                  # XDG state-home resolution + FNV-1a 8-hex + shim_dir_for
    driver.rs                 # dispatch / build / run / write_shim_from_source / transpile
    error.rs                  # DriverError + Display + Debug + Error + report()
```

All five files under the 500-LOC target (largest: `paths.rs` at 277 LOC, `driver.rs` at 274 LOC). Test code is in-file under `#[cfg(test)] mod tests`.

### 9.3 Decisions

- **Dependency choice — `clap` (derive API), pinned `=4.5.61`.** PRD §9 amended 2026-05-23 from "no `clap`" to permit `clap`. Pin is exact (`=`) so Cargo.toml + Cargo.lock together capture the full identity. Version selected per the PRD §9 N-1 / 30-day rule: 4.5.61 is the top of the previous-minor line, released 2026-03-12 (72 days before the pin date); MSRV 1.74 is well under our 1.94.1 toolchain.
- **`#[derive(Parser)]` + `#[derive(Subcommand)]` over the builder API.** Subcommand schema is small (2 commands × 1 positional each); the derive form is half the LOC of the builder equivalent and reads as data, not procedure. The builder API has no advantage for this surface.
- **Hand-rolled XDG resolution (no `dirs` crate).** ~25 LOC including doc-comments. The `dirs` crate is mature but the policy is trivial enough that a dependency on it adds supply-chain surface for no real win. The pure logic lives in [`xdg_state_home_from(state_var, home_var)`] so tests exercise resolution without touching the process environment.
- **Hand-rolled FNV-1a 32-bit (no hashing crate, no `DefaultHasher`).** `DefaultHasher` is process-stable but Rust-version-fragile — output cannot be persisted to disk and reproduced. FNV-1a is the standard non-cryptographic hash for short identifiers; 32-bit output renders as 8 hex chars, which is what the shim-dir name format specifies (§3.2). Known-vector tests (`a` → `e40c292c`, `foobar` → `bf9cf968`) guard against arithmetic drift.
- **`DriverError` composes three families** — source-mappable ([`MarainError`] + [`SourceMap`] for `path:line:col` rendering), filesystem-shim ([`ShimError`]), and other I/O (`io::Error` + free-text context string) — plus a fourth `Cargo { exit_code }` variant for proxying cargo's non-zero exits. [`From`] impls cover the [`ShimError`] case; the other two need constructor functions ([`DriverError::from_source`] takes both the error and the [`SourceMap`]; [`DriverError::from_io`] attaches a context string) because they aggregate state the caller has.
- **Source-error rendering goes through [`Diagnostic::render`]; system errors go through a `marain:` prefix.** `path:line:col: error: msg` for anything the source can be blamed for; `marain: error: <context>: <io-error>` for filesystem and process errors. The two shapes make it obvious which side of the boundary the error came from (mirrors `cargo:` / `rustc:` convention).
- **`build` is split into a public path-resolving wrapper and a private `write_shim_from_source(source, shim_dir)` helper.** The split lets unit tests drive the pipeline to a tempdir target without going through XDG resolution (which reads process env vars and would write under the user's real `~/.local/state`). The workspace `unsafe_code = "forbid"` lint correctly blocks the obvious "set `XDG_STATE_HOME` in the test fixture" workaround; the seam is the right answer.
- **`marain run` uses inherited stdio for the spawned cargo.** `Command::status()` (not `output()`) — cargo's progress and the user program's output go to the user's terminal live, not buffered. `--quiet` suppresses cargo's "Compiling..." lines so the user sees only their program. `CARGO_TARGET_DIR` is unset so the shim uses its own `target/`, not whatever the outer environment may have set.
- **No `--manifest-path` arg to the `marain` binary, no `--release` flag, no `-v`.** Out of scope for v0.1. Add via clap's derive macros when needed.
- **No `marain check` in v0.1.** PRD §6 lists it as post-v0.1.

### 9.4 `DriverError` variants

| Variant | Trigger | Renders as |
| --- | --- | --- |
| `Source { error, map }` | [`MarainError`] from lex / parse / emit | `path:line:col: error: <message>` |
| `Shim(ShimError)` | [`shim::write_shim`] failure (atomic-write protocol step failed) | `marain: error: failed to <op> '<path>': <io-error>` |
| `Io { context, source }` | source-file read failure, canonicalize failure, cargo spawn failure | `marain: error: <context>: <io-error>` |
| `Cargo { exit_code }` | cargo subprocess exited non-zero | `marain: error: cargo exited with status <code>` (or `terminated by signal` if `None`) |

[`From<ShimError>`] for `DriverError`; the other two variants use constructor functions ([`DriverError::from_source`], [`DriverError::from_io`]). `Display` delegates to the inner error / formats system errors with their context. `std::error::Error::source` chains through.

### 9.5 XDG path resolution

Per the XDG Base Directory Specification:

1. If `$XDG_STATE_HOME` is set and absolute, use it.
2. Else if `$HOME` is set, use `$HOME/.local/state`.
3. Else, `.` (pathological fallback — only reached in an environment with no `HOME`).

Relative `$XDG_STATE_HOME` values are silently ignored per the spec (not a hard error). Resolution lives in `paths::xdg_state_home_from(state_var, home_var)` — a pure function taking `Option<&OsStr>` for each, so tests exercise the policy without env-var mutation. The thin wrapper `paths::xdg_state_home()` plumbs the real env values in.

### 9.6 Shim identity

Each `<file.lat>` source canonicalizes to a unique absolute path; the shim project for that source lives at `$XDG_STATE_HOME/marain/builds/<basename>-<hash>/`, where:

- `<basename>` = source's `file_stem` (e.g. `hello.lat` → `hello`); pathological-input fallback is `main`.
- `<hash>` = `fnv1a_8hex(canonical_path.as_bytes())` — 8 lowercase hex chars.

This means `hello.lat` in `~/a/` and `hello.lat` in `~/b/` produce two distinct shims (different canonical paths → different hashes) but the same source path always produces the same shim dir across invocations (so `marain build` is idempotent and `marain run` reuses the prior `target/`).

The hash is not load-bearing for correctness — it's a short disambiguator. FNV-1a 32-bit is the right tool: deterministic across processes and Rust versions (unlike [`std::hash::DefaultHasher`]), known-vector-verifiable, and ~10 LOC. No cryptographic claim, no resistance to deliberate collision.

### 9.7 Test coverage

- **`args.rs`** — 8 tests: clap schema self-consistency (`Cli::command().debug_assert()`), parse `build` / `run` happy paths, missing subcommand → error, missing path → error, unknown subcommand → error, `--help` → `DisplayHelp` error kind, `--version` → `DisplayVersion` error kind.
- **`paths.rs`** — 16 tests: FNV-1a empty / known vectors (`a`, `foobar`) / output shape (8 lowercase hex) / distinguishability; XDG resolution pure cases (absolute state-var / relative ignored → home fallback / state-absent → home / no env → `.`); `shim_dir_for` composition + idempotence + collision-resistance for same basename in different directories + I/O error propagation; `shim_name_for` for `.lat` / no extension / multi-dotted.
- **`error.rs`** — 9 tests: constructor `from_io` attaches context; constructor `from_source` binds map; `From<ShimError>`; `Display` for `Io` / `Cargo` (both exit-code shapes) / `Source` (delegates to inner); `source()` chains for each variant; `Debug` impl doesn't panic across all variants.
- **`driver.rs`** — 7 tests, all via the `write_shim_from_source(source, shim_dir)` seam against a tempdir: hello-world end-to-end, basename → cargo project name, lex / parse error propagation as `Source` variant, missing source file → `Io { context: "failed to read..." }`, second build overwrites first; plus one test on the public `build` that exercises the canonicalize-failure arm (`context: "failed to canonicalize..."`).

199 marain-core unit tests + 40 marain-cli unit tests + 1 integration test (`marain-core/tests/e2e_hello_world.rs`) pass at Round 7 close. No binary-level e2e test in R7 per the stance chosen at design time; R8 owns systematic e2e coverage.

A manual smoke test of the binary itself (not committed) at Round 7 close confirmed: `marain build hello.lat` prints the shim path on stdout; `marain run hello.lat` prints `salve, munde` on stdout; `marain --help` and `marain --version` render via clap; `marain build bad.lat` with a `?` source prints `bad.lat:1:1: error: unexpected character '?'` on stderr and exits 1.

### 9.8 Pressure-release tier 1 not invoked

All five files under the 500-LOC target at Round 7 close. The plausible future pressure points are `driver.rs` (when `marain check` lands, or when `marain run` grows arg-forwarding to the user program) and `paths.rs` (when shim-dir cleanup / GC lands). Neither is in v0.1's path.

### 9.9 Forward hooks

Backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) §6 — `marain check` and `marain install` subcommands (each = one `args::Command` variant + one `driver::dispatch` arm) and rustc-error span back-mapping (a third `Diagnostic`-rendering shape alongside `DriverError::Cargo`).

Resolved here: **binary-level e2e** (R8 — `cli_e2e.rs` spawns the `marain` binary via `env!("CARGO_BIN_EXE_marain")`, asserting stdout / stderr / exit code).

## 10. Testing Harness

### 10.1 Scope (v0.1)

Three layers of automated test coverage at Round 8 close:

| Layer | Where | What it catches |
| --- | --- | --- |
| **Per-phase unit tests** | `#[cfg(test)] mod tests` at the bottom of each source file | Logic bugs inside a single component (lexer scanner, parse production, emit arm, error rendering, etc.) |
| **Fixture-walker goldens** | `marain-core/tests/{emit,error}_goldens.rs` + `tests/fixtures/` | *Unintended* drift in emit shape or diagnostic rendering. Tripwire, not contract (per PRD §7 + concern ε). `MARAIN_UPDATE_GOLDENS=1` regenerates on intentional change. |
| **Behavioral end-to-end** | `marain-core/tests/e2e_hello_world.rs` (library pipeline) + `marain-cli/tests/cli_e2e.rs` (binary) | The user-facing contract from PRD §7. Library e2e exercises lex → parse → emit → shim → real `cargo run`; binary e2e spawns the actual `marain` binary and asserts on stdout / stderr / exit code. |

### 10.2 Decomposition

```
crates/marain-core/
  tests/
    e2e_hello_world.rs            # PRD §7 done-line smoke test (carried from R6)
    emit_goldens.rs               # fixture-walker for emit pipeline
    error_goldens.rs              # fixture-walker for diagnostic rendering
    fixtures/
      01_hello_world.lat          # paired with 01_hello_world.expected.rs
      02_let_integer.lat          # ... etc
      03_let_string.lat
      04_let_mutable.lat
      05_let_then_print.lat
      06_all_macros.lat
      07_integer_separators.lat
      08_rust_keyword_escape.lat
      errors/
        01_unexpected_char.lat    # paired with 01_unexpected_char.expected.txt
        02_unterminated_string.lat
        03_missing_period.lat
        04_unescapable_keyword.lat
        05_no_sigil_in_binding.lat

crates/marain-cli/
  tests/
    cli_e2e.rs                    # binary-level e2e via env!("CARGO_BIN_EXE_marain")
```

All R8 files comfortably under the 500-LOC target (`cli_e2e.rs` ~210 LOC; goldens harnesses ~130 LOC each).

### 10.3 Decisions

- **Three layers, not one.** Unit / golden / behavioral test different things: unit tests catch logic bugs in a single component (fast, deterministic, run-on-save); golden diffs catch unintended drift in cross-component shape (fast, deterministic, mechanical regeneration); behavioral e2e catches user-facing contract regressions (slower because they spawn real subprocesses, but the only ones that prove the actual user experience).
- **Paired-file fixtures (`.lat` + `.expected.{rs,txt}`) over inline snapshots.** Fixtures are self-documenting (open the directory, read the inputs); expected output lives next to its input; adding a fixture is one `.lat` file + `MARAIN_UPDATE_GOLDENS=1 cargo test`. Inline-snapshot crates (`insta`, `expect-test`) would solve the same problem with one extra dependency, which PRD §9 disfavors.
- **`MARAIN_UPDATE_GOLDENS=1` env-var regen.** Single env var, no subcommand. Set it, run the tests, the goldens regenerate; commit the diff (or revert if the change was unintended). Newline-tolerant comparison (`trim_end`) so editors that auto-add a trailing newline don't flake.
- **One `#[test]` per fixture harness, not one-test-per-fixture.** When a refactor breaks the emit shape, you want to see *every* fixture that drifted in one run, not one-at-a-time-fix-rerun-fix. The harness accumulates failures and reports them all together with a clear header per fixture.
- **Fixture path rebased to the bare basename for error rendering.** Loaded into the `SourceMap` as `01_unexpected_char.lat` (not the full disk path), so rendered diagnostics are stable across machines and CI environments. The on-disk file path appears nowhere in the golden text.
- **Binary-level e2e isolates `XDG_STATE_HOME` per test.** `Command::env("XDG_STATE_HOME", <test-scratch>)` rather than `std::env::set_var` — the subprocess gets its own env, our process's env stays untouched, and the workspace `unsafe_code = "forbid"` lint is satisfied without a single `unsafe` block. Two tests can run concurrently without racing over the same shim directory.
- **`cli_e2e.rs` covers exit-code shape explicitly.** Clap argument errors exit 2; driver errors exit 1; cargo exit codes proxy through. Tests assert the specific codes so a refactor that collapses them gets caught.
- **Library-level e2e (`e2e_hello_world.rs`) carried forward, not retired.** It exercises the library pipeline including a real `cargo run` on the generated shim — the same pipeline `marain-cli`'s `driver::run` invokes. Keeping both layers means a regression in either the library shape or the binary's wiring is caught independently. Marginal duplication, real coverage value.

### 10.4 What each fixture asserts

**Emit fixtures (8) — every R5 production at least once:**

| Fixture | Production exercised |
| --- | --- |
| `01_hello_world.lat` | no-punct macro + string literal (PRD §7 done line) |
| `02_let_integer.lat` | immutable binding (`^`) + integer literal |
| `03_let_string.lat` | binding with string RHS |
| `04_let_mutable.lat` | mutable binding (`@` → `let mut`) |
| `05_let_then_print.lat` | binding + var reference + multi-statement |
| `06_all_macros.lat` | all four no-punct macros (`dic` / `queror` / `agmen` / `forma`) |
| `07_integer_separators.lat` | `1_000_000` underscore stripping at emit time |
| `08_rust_keyword_escape.lat` | `^if` → `r#if` (Rust 2024 keyword escape) |

**Error fixtures (5) — one per error family:**

| Fixture | Error variant |
| --- | --- |
| `01_unexpected_char.lat` | `LexError::UnexpectedChar` |
| `02_unterminated_string.lat` | `LexError::UnterminatedString` |
| `03_missing_period.lat` | `ParseError::UnexpectedToken` (period expected at EOF) |
| `04_unescapable_keyword.lat` | `EmitError::UnescapableRustKeyword` (`self`) |
| `05_no_sigil_in_binding.lat` | `ParseError::UnexpectedToken` (sigiled-ident expected) |

**CLI e2e (10) — every PRD §6 user-visible behavior:**

`build` prints shim path; `run` prints `salve, munde`; bad source exits 1 with `path:line:col: error:` (no `marain:` prefix); emit error surfaces as a source diagnostic; `--help` and `--version` exit 0; unknown subcommand / missing path / no subcommand exit 2 (clap convention); missing source file exits 1 with `marain:` prefix.

### 10.5 Test counts at Round 8 close

| Binary | Tests |
| --- | --- |
| `marain-core` unit | 199 |
| `marain-core` integration (`e2e_hello_world`) | 1 |
| `marain-core` integration (`emit_goldens`) | 1 |
| `marain-core` integration (`error_goldens`) | 1 |
| `marain-cli` unit | 40 |
| `marain-cli` integration (`cli_e2e`) | 10 |
| **Total** | **252** |

`cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean. Both crates carry `#![forbid(unsafe_code)]` at the crate root in addition to the workspace `[workspace.lints.rust] unsafe_code = "forbid"` lint — belt-and-braces per CLAUDE.md.

### 10.6 Pressure-release tier 1 not invoked

All R8 files comfortably under the 500-LOC target. The plausible future pressure points are `cli_e2e.rs` (when `marain check` and `marain install` land — each adds a few tests) and the fixture harnesses (when fixture count grows large enough that the single-test-per-harness aggregation becomes slow). Neither is in v0.1's path.

### 10.7 Forward hooks

Feature backlog lives in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md); the notes below are test-harness-specific seams (not features), retained here.

- **`marain check` coverage.** When the subcommand lands (ROADMAP §6), add a `cli_e2e.rs` test asserting it exits 0 on a clean source and exits 1 on a bad source — without invoking cargo.
- **Per-phase token / AST golden fixtures.** Could add `tests/fixtures/tokens/` (`.lat` → `.expected.tokens`) and `tests/fixtures/ast/` (`.lat` → `.expected.ast`) to catch drift at the lex and parse layers separately. Deferred until a real bug demonstrates the need; the current emit-golden coverage already catches most upstream regressions transitively.
- **Performance regression tests.** Not in scope for v0.1. When the Stage 2 parser lands and parser cost becomes nontrivial, a `tests/perf/` directory with `criterion`-style benchmarks (pinned per N-1 / 30-day rule) is the natural extension.
- **Stage 2 `(lemma, inflection)` golden fixtures.** When Stage 2 lands, the existing fixtures stay (Stage-1-mode regression coverage) and a sibling `tests/fixtures/stage2/` houses inflected-form fixtures. The harness pattern (paired files + `MARAIN_UPDATE_GOLDENS=1`) carries over unchanged.

## 11. Stage 2 Forward Hooks

Stage 2 (full case/conjugation grammar, free word order, sidecar `.latin`, LSP) is a milestone tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) §5, gated by PRD open questions S2-1…S2-7. The entries below are the **v0.x design constraints** those choices must not foreclose — seams, not scheduled work:

- **(ζ) Cross-file diagnostics.** The `SourceMap`-as-arg pattern (§4) already supports multi-file grammar contexts (sidecar `.latin` references).
- **(θ) `(lemma, inflection)` tokens.** A backward-compatible optional inflection slot on `PlainIdent` / `SigiledIdent` token variants; consumers that ignore it keep compiling.
- **`marain-lsp` crate seam.** The workspace can take a third member crate without restructuring; `marain-core` is the dependency target.
- **`Variabile` injection (γ).** Vendored support module emitted into the shim (mechanics in ROADMAP §3).

Resolved carry-overs: α (R5 / §7.5), η (R9 / §12).

## 12. Line Comments

Round 9. PRD §4.12 (amended pre-R9) committed `//` line comments for v0.2 with `/* */` reserved-deferred; this round wires the lexer.

### 12.1 Scope (v0.2)

**In:** `//` line comments, consumed to but not including the next `\n` or EOF, emit no token. New `LexError::BlockCommentsDeferred` variant for `/*` with an explicit "use `//`" hint. Bare `/` remains `LexError::UnexpectedChar`.

**Out (deferred):** block comments (`/* */`), doc comments (`///`), comment AST representation (comments are lexer-layer only — parser never sees them).

### 12.2 Decomposition

```
crates/marain-core/src/lexer/
  comments.rs  (new)         scan_line_comment + 7 unit tests
  cursor.rs    (modified)    peek_at(offset) + 3 unit tests
  error.rs     (modified)    BlockCommentsDeferred variant + 1 unit test
  mod.rs       (modified)    start-of-line `//` peek + mid-line `/` dispatch + 9 driver tests
```

All files under target. The largest is `mod.rs` (~215 LOC executable + ~420 LOC tests); the test bloc is the natural decomposition candidate if pressure surfaces (`#[path = "mod_tests.rs"] mod tests;` per CLAUDE.md).

### 12.3 Decisions

_Full rationale: [`tasks/decisions/R09_line_comments.md`](../tasks/decisions/R09_line_comments.md). Summary list below._

- **Comment-only lines transparent to indent state.** Dispatcher peeks two bytes at line start; if `//`, consume and continue without invoking the indent machinery (identical to the blank-line path).
- **Mid-line `//` is the simple case.** Dispatcher hits `/` mid-line, peeks `/`, scans-to-EOL, `continue`s. Indent state already decided at line start.
- **`/*` → `LexError::BlockCommentsDeferred`** with targeted "use `//`" message (PRD §4.12). Dedicated variant also reserves the syntax against future repurposing.
- **Bare `/` stays `UnexpectedChar`.** Division is `divisus per` per PRD §4.4; `/` has no standalone use. Forward-compatible: v0.3 block-comment work only adds an arm.
- **`\n` left for the existing newline handler.** Comment scanner stops at `\n` exclusive; categorically avoids off-by-one bugs in error reporting.
- **`Cursor::peek_at(offset)` joins the cursor API.** Two-byte opener disambiguation; also serves future `..` (R14).

### 12.4 `LexError::BlockCommentsDeferred`

| Field | Type | Notes |
|-------|------|-------|
| `span` | `Span` | Covers the two-byte `/*` |

Rendered: `path:line:col: error: block comments are reserved syntax; use // for a line comment (PRD §4.12)`

Joins `MarainError::Lex` via the existing facade — no new plumbing.

### 12.5 Test coverage

- **`comments.rs`** — 7 unit tests on `scan_line_comment`: empty body; consume to but not including newline; consume to EOF without trailing newline; leaves `\n` for caller; body doesn't lookback into Marain syntax; UTF-8 in body; consecutive `//` stays inside the comment.
- **`cursor.rs`** — 3 new unit tests on `peek_at`: offset 0 matches `peek`; offset N looks ahead without advancing; past-end is `None`.
- **`error.rs`** — 1 new unit test: `BlockCommentsDeferred` message contains "reserved", `//`, and `PRD §4.12`.
- **`lexer/mod.rs` driver** — 9 new tests: trailing comment after statement; standalone comment at top of file; comment-only file (with and without trailing newline); consecutive comment-only lines preserve indent stack; comment-only line inside indented block doesn't dedent; `/*` produces `BlockCommentsDeferred` with two-byte span; `/*` message mentions `//` and "reserved"; bare `/` → `UnexpectedChar { ch: '/' }`.
- **Goldens** — `09_line_comments.lat` (8 lines exercising trailing + standalone + blank-line-interleaved); `errors/06_block_comments_deferred.lat` (1 line, exercises the diagnostic).

**Test count delta: +20.** Workspace total at R9 close: **272** (was 252 at R8 close). `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.

### 12.6 Sentrux signal at R9 close

`session_start` taken before any code change (signal 7079); `session_end` after the round: signal_delta +3 (7079 → 7082), cycles_change 0, coupling_change 0.0, DSM `above_diagonal` stays 0 (clean layering preserved), `check_rules` passes (4/20 rules enforced under free tier; 16 documented as architectural intent in `.sentrux/rules.toml`). The new `lexer/comments.rs` slotted in without inverting any pipeline edge.

### 12.7 Pressure-release tier 1 not invoked

All R9 files comfortably under target. The plausible future pressure site is `lexer/mod.rs`'s test bloc; not yet at threshold.

### 12.8 Forward hooks

Backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) §1 — block-comment activation (the `Some(b'*')` arm swaps from "return error" to a `scan_block_comment` call; `BlockCommentsDeferred` retires) and doc comments `///` (a three-byte-lookahead extension, uncommitted).

Resolved here: **range tokens** — the `peek_at` lookahead added this round backs the `..` / `..=` dispatch shipped in R14 (§16).

## 13. Block Parsing + `si`

Round 10. The parser learns to consume `Indent`/`Dedent` layout tokens and produces its first block-bearing AST node. The `si <cond> :` head (PRD §4.11.2) lands as the parent construct that exercises `parse_block` end-to-end without inventing a test-only seam. `aliter` / `aliter si` chains, `dum`, `semper`, and the full Boolean / operator expression surface remain in R11+R12.

### 13.1 Scope (v0.2)

**In:** `Block { stmts, span }` AST node; `IfStmt { cond, then_block, span }`; `Stmt::If` variant; `parse_block` (consumes `Indent`, parses statements until `Dedent`, returns `Block`); `parse_if` (`si` → `parse_expr` → `:` → `parse_block`); `Stmt::If` emit with nested indent threading on `emit_stmt`. Reuses existing `UnexpectedToken { expected: &'static str }` for the new "expected `:`" / "expected indented block" / "expected end of indented block" failure modes — no new `ParseError` variants.

**Out (deferred):** `aliter` / `aliter si` else chain (R11+R12); `dum` / `semper` / `interrumpe.` / `continua.` (R11+R12); Boolean literals (`verum`, `falsum`) and operator expressions (R11+R12); `nihil.` empty-block sentinel (R14+R15); `functio` declarations with parameter blocks (R13); range tokens `..` / `..=` (R14+R15).

### 13.2 Decomposition

```
crates/marain-core/src/
  ast.rs               (modified)  + Block, IfStmt, Stmt::If; +2 unit tests
  parser/
    grammar.rs         (modified)  + parse_if, parse_block; parse_stmt gains Si dispatch
    mod.rs             (modified)  +10 driver tests covering si + block parsing
  emit.rs              (modified)  emit_stmt gains indent_level: usize; + emit_if, emit_block_body, push_indent helper; +5 unit tests
```

No new files. All modified files comfortably under the 500-LOC target post-R10.

### 13.3 Decisions

_Full rationale: [`tasks/decisions/R10_block_si.md`](../tasks/decisions/R10_block_si.md). Summary list below._

- **`Block` is a newtype with span**, not a bare `Vec<Stmt>`. `span` carries the `Indent.start..Dedent.end` region.
- **Empty-block failure surfaces via `UnexpectedToken "indented block"`** — no dedicated `EmptyBlock` variant. R4+R9 indent transparencies prevent an empty `Indent`-`Dedent` pair structurally; the only failure mode is "no `Indent` at all," which the leading `expect_kind` already covers.
- **No dedicated `ExpectedIndent` / `ExpectedColon` / `ExpectedDedent` variants.** `UnexpectedToken { expected: &'static str }` is the generic vehicle; label strings carry the diagnostic clarity. Variant proliferation has its own future tax.
- **`parse_block` loop exit checks both `Dedent` and `Eof`.** Defensive against own-code bugs (lexer guarantees `Dedent` before `Eof`, but the loop terminates rather than infinite-loops if that contract breaks).
- **`emit_stmt` takes `indent_level: usize`** — resolves §8.10 forward hook. Top-level at `1` (inside `fn main`); each block body recurses at `level + 1`.
- **`emit_if` closes `}` at parent's indent level, no trailing newline.** Caller writes the trailing `\n`; preserves the per-statement-line invariant.
- **`Stmt::If` ships ahead of an executable condition language.** R10's expression set is R5's; `si 1 :` parses and emits but rustc rejects. Goldens are string-compares only. R11+R12 retires the caveat.
- **R10 ships alone** (locked decision A); `aliter` chain deferred to R11+R12 to avoid pre-committing the `else_branch` AST shape.

### 13.4 New AST shape

```rust
pub enum Stmt {
    Let(LetStmt),
    MacroCall(MacroCallStmt),
    If(IfStmt),     // new in R10
}

pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub span: Span,
}

pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,     // covers the Indent..Dedent region
}
```

`Stmt::span()` dispatch extended; carry-over α (inflection slot) untouched (`If` has no identifier-bearing position of its own; the condition's `VarRef` already carries the slot via `SigiledIdent`).

### 13.5 Test coverage

- **`ast.rs`** — 2 new unit tests: `Block` construction; `Stmt::If` span dispatch.
- **`parser/grammar.rs`** — covered transitively by driver tests in `parser/mod.rs`; pattern carried from R5.
- **`parser/mod.rs` driver** — 10 new tests: single-statement body; multi-statement body; nested `si`; integer-literal condition (R10 doesn't gate on type); body at column-0 (no `Indent` → next stmt is sibling, not child); missing colon; missing condition; body at same indent as parent (`UnexpectedToken` with `"indented block"` label); `Eof` straight after `:`; span covers `si` through closing `Dedent`.
- **`emit.rs`** — 5 new unit tests: simple `si` emits `if x { println!(...) }` with correct indent; nested `si` threads indent level (8-space body inside 4-space outer); body with mixed `let` + macro call; top-level regression (indent threading didn't break pre-R10 shape); `si` followed by sibling top-level statement preserves both.
- **Goldens (emit)** — `10_si_simple.lat` (let + if + dic); `11_si_nested.lat` (two `si` heads, deepest at 12-space indent).
- **Goldens (error)** — `errors/07_no_block_after_if.lat` (body at column 0 → `expected indented block, found keyword \`sit\``).

**Test count delta: +17.** Workspace total at R10 close: **289** (was 272 at R9 close). `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean.

### 13.6 Sentrux signal at R10 close

`session_start` taken before any code change (signal 7082); `session_end` after the round: signal_delta +7 (7082 → 7089), `cycles_change` 0, `coupling_change` 0.0, DSM `above_diagonal` stays 0 (clean layering preserved), `check_rules` passes (4/4 enforced under free tier). The new `Block` / `IfStmt` AST nodes flow downward through the existing parser → emit pipeline; no edge inversion.

### 13.7 Pressure-release tier 1 not invoked

All R10 modifications land well under the 500-LOC target. The plausible future pressure sites are `parser/grammar.rs` (when R11+R12 add precedence-climbing for the operator expression family, the multi-word phrase table, and the `aliter` chain), and `parser/mod.rs`'s test bloc (already large; the `#[path = "mod_tests.rs"] mod tests;` decomposition pattern from CLAUDE.md is the obvious next step if pressure surfaces).

### 13.8 Forward hooks

All hooks this round have since shipped. Resolved in later rounds: `aliter` chain, `dum` / `semper` / `interrumpe.` / `continua.`, R10 condition typing (all R11+R12 / §14); `functio` body block (R13 / §15); `nihil.` (R15 / §16). The open backlog lives in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md).

## 14. Operator Expressions + Control Flow

Rounds 11+12, batched per locked decision A. R11 adds the expression-level
operator surface (precedence-climbing parser + Boolean literals + parens
grouping). R12 finishes Stage 1's control-flow set (`aliter` / `aliter si`
chain on the R10 `si`, plus `dum` / `semper` / `interrumpe.` / `continua.`).
R10's `si 1 :` caveat — the parser could produce Rust that wouldn't typecheck
— retires here: real Boolean conditions land in this round.

### 14.1 Scope (v0.2)

**In:**
- Boolean literals `verum` / `falsum` as `Expr::BoolLit` atoms.
- Parens `( expr )` as expression-grouping primary (precedence override).
- Binary operators with Rust precedence (PRD §4.4): `vel` (||) → `et` (&&) →
  `aequat` / `non aequat` (==, !=) → `minor quam` / `maior quam` /
  `minor vel par` / `maior vel par` (<, >, <=, >=) → `plus` / `minus` (+, -)
  → `per` / `divisus per` / `modulo` (*, /, %).
- Unary prefix `non` (!), right-associative by recursion.
- `IfStmt.else_branch: Option<ElseBranch>`; `ElseBranch::Block(Block)` for
  terminal `aliter :`, `ElseBranch::If(Box<IfStmt>)` for `aliter si` chain.
- `Stmt::While(WhileStmt)`, `Stmt::Loop(LoopStmt)`, `Stmt::Break(BreakStmt)`,
  `Stmt::Continue(ContinueStmt)`.
- `Parser::peek_kind_at(offset)` cursor primitive for the `non aequat`
  lookahead and any future two-token disambiguation.

**Out (deferred):** `pro <binding> in <iterable> :` (R14+R15); range tokens
`..` / `..=` (R14+R15); `nihil.` (R14+R15); `functio` declarations (R13);
labeled `break 'name` / `continue 'name`; `break <expr>` (loop expression
value); type checking (delegated to rustc per PRD §5).

### 14.2 Decomposition

```
crates/marain-core/src/
  ast.rs                (modified)  + BoolLit, BinOpExpr + BinOp enum,
                                       UnaryOpExpr + UnaryOp enum,
                                       WhileStmt, LoopStmt, BreakStmt,
                                       ContinueStmt, ElseBranch enum;
                                       IfStmt grows else_branch; +7 tests
  parser/
    mod.rs              (modified)  + peek_kind_at; tests bloc moved out
    mod_tests.rs        (new)       sibling test file (per CLAUDE.md
                                    pressure-release decomposition)
    grammar.rs          (modified)  + parse_or/and/equality/comparison/
                                       additive/multiplicative/unary/primary
                                       cascade; consume_comparison_completer;
                                       parse_while/parse_loop/parse_break/
                                       parse_continue; parse_if grows
                                       aliter / aliter si chain; verum/falsum
                                       atoms + paren grouping in parse_primary
  emit.rs               (modified)  + BoolLit / BinOp / UnaryOp arms in
                                       emit_expr (paren-wrap-always);
                                       + emit_else_branch, emit_while,
                                       emit_loop; break/continue inline;
                                       tests bloc moved out
  emit_tests.rs         (new)       sibling test file (decomposition twin)
```

No new module under `lexer/` — every keyword R11+R12 consumes was already in
R4's table (`verum`, `falsum`, `et`, `vel`, `non`, `plus`, `minus`, `per`,
`modulo`, `aequat`, `maior`, `minor`, `quam`, `par`, `divisus`, `aliter`,
`dum`, `semper`, `interrumpe`, `continua`). The lexer was deliberately
front-loaded in R4 against exactly this round.

### 14.3 Decisions

_Full rationale: [`tasks/decisions/R11_12_operators_control_flow.md`](../tasks/decisions/R11_12_operators_control_flow.md). Summary list below._

- **Latin for op variants, English for stmt variants.** `BinOp::Plus` / `NonAequat` (Latin; operator surface); `Stmt::While` / `Loop` / `Break` / `Continue` (English; mirrors Rust target via PRD keyword names).
- **Precedence climbing, not Pratt.** Seven cascaded `parse_<level>` fns (or → and → equality → comparison → additive → multiplicative → unary). Left-associative via `while`; unary right-associative via tail recursion.
- **Multi-word phrases consumed greedily at parse level.** `consume_comparison_completer` peeks for `quam` / `vel par` after `minor` / `maior`; same shape for `divisus per` and `non aequat`. Lexer stays single-word-per-token.
- **`non` disambiguates via `peek_kind_at`.** At equality level, `non aequat` is `!=`; everything else is unary prefix at `parse_unary`.
- **`Expr::BoolLit` is a new variant**, not folded into `IntegerLit`. Parallels `StringLit` / `IntegerLit` shape.
- **Paren-wrap-always in emit.** Every `BinOp` / `UnaryOp` emits with surrounding parens. Cost: visual noise. Benefit: zero risk of precedence drift in the lowering.
- **Expression-grouping parens (`(expr)`) in primary** unwrap to inner expression — no `ParenExpr` AST node; precedence is structurally encoded in tree shape.
- **`aliter` recognition by next-token after `parse_block` returns.** Indent alignment enforced implicitly by layout tokens.
- **`aliter si` recurses through `parse_if`.** Chain becomes nested `IfStmt.else_branch: Some(If(Box<IfStmt { ... }>))`. Single nested AST shape; emit walks via `emit_else_branch` → `emit_if` recursion.
- **`semper :` emits `loop { … }`** — no `Semper` rename of `Stmt::Loop`; AST name matches Rust target per the naming rule.
- **`interrumpe.` / `continua.` are statements terminated by `.`** — span-only, no payload (no labels, no value-from-break in v0.2).
- **No new `ParseError` variants.** All R11+R12 failures ride on `UnexpectedToken { expected: &'static str }` with descriptive labels.
- **Test files split via `#[path = "…_tests.rs"] mod tests;`.** First pressure-release invocation in v0.2. `parser/mod_tests.rs` (836 LOC) and `emit_tests.rs` (554 LOC); production files at 73 and 349 LOC respectively. Sibling-file decomposition per CLAUDE.md.

### 14.4 New AST shape

```rust
pub enum Stmt {
    Let(LetStmt),
    MacroCall(MacroCallStmt),
    If(IfStmt),
    While(WhileStmt),     // new
    Loop(LoopStmt),       // new
    Break(BreakStmt),     // new
    Continue(ContinueStmt), // new
}

pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_branch: Option<ElseBranch>, // new
    pub span: Span,
}

pub enum ElseBranch {                    // new
    Block(Block),       // aliter :
    If(Box<IfStmt>),    // aliter si <cond> :
}

pub enum Expr {
    StringLit(StringLit),
    IntegerLit(IntegerLit),
    BoolLit(BoolLit),     // new
    VarRef(SigiledIdent),
    BinOp(BinOpExpr),     // new
    UnaryOp(UnaryOpExpr), // new
}

pub enum BinOp {                              // new
    Plus, Minus, Per, DivisusPer, Modulo,
    Aequat, NonAequat,
    MinorQuam, MaiorQuam, MinorVelPar, MaiorVelPar,
    Et, Vel,
}

pub enum UnaryOp { Non }                      // new
```

Carry-over α (inflection slot, R5/§7.5) untouched. The new expr / stmt
variants have no identifier-bearing positions of their own; condition and
operand identifiers carry the slot via the existing `SigiledIdent`.

### 14.5 Test coverage

- **`ast.rs`** — 7 new unit tests covering each new variant's `span()`
  dispatch, `BinOp::as_rust` / `UnaryOp::as_rust` mappings, and `ElseBranch`
  span dispatch (Block + If shapes).
- **`parser/mod_tests.rs`** — 34 new tests:
  - Atoms: `verum` / `falsum` → `BoolLit`.
  - All 13 binary ops recognized in let RHS position.
  - Multiplicative binds tighter than additive (`a plus b per c` → nested).
  - Left-associativity for repeated same-precedence ops.
  - Unary `non` prefix; right-associative `non non verum`.
  - Parens grouping flips precedence.
  - Full precedence cascade (all six levels in one expression).
  - Error path: bare `maior` / `minor` / `divisus` / `minor vel` (no
    completer) all surface as `UnexpectedToken` with descriptive labels.
  - `si` + terminal `aliter`; `si` + `aliter si` chain; multi-arm chain
    `si … aliter si … aliter si … aliter :`.
  - `dum` simple body; `semper` simple body; `semper` with `interrumpe`;
    `dum` with `continua`.
  - Error path: `dum` missing colon; `interrumpe` missing period.
  - `si <cond>` accepts binop conditions.
- **`emit.rs` / `emit_tests.rs`** — 22 new tests:
  - `verum` → `true`, `falsum` → `false`.
  - All 13 binary ops emit the right Rust operator, wrapped in parens.
  - Unary `non` → `(!x)`.
  - Precedence-preservation via paren nesting.
  - `aliter :` → ` else { ... }`.
  - `aliter si … aliter :` → ` else if … else { ... }` chain.
  - `dum <cond> :` → `while <cond> { ... }`.
  - `semper :` → `loop { ... }`.
  - `interrumpe.` → `break;`; `continua.` → `continue;`.
  - R10 caveat retired: `si verum et falsum :` produces typecheckable Rust.
- **Goldens** — 6 new emit fixtures (`12_arithmetic`, `13_booleans`,
  `14_comparison`, `15_aliter_chain`, `16_dum`, `17_semper_interrumpe`) and
  3 new error fixtures (`errors/08_bare_maior`, `errors/09_missing_colon_dum`,
  `errors/10_missing_period_interrumpe`).

**Test count delta: +65.** Workspace total at R11+R12 close: **354** (was
289 at R10 close). `cargo fmt --check`, `cargo clippy --all-targets -D
warnings`, `cargo test --all` all clean.

### 14.6 Sentrux signal at R11+R12 close

`session_start` taken before any code change (signal 7089, the R10 close
number); `session_end` after the round + the test-file split:
`signal_delta` -85 (7089 → 7005), `cycles_change` 0, `coupling_change` 0.0,
DSM `above_diagonal` stays 0 (clean layering preserved), `import_edges`
39 → 38 (the test-file split removed one inbound edge from parser/mod.rs
to its old in-file test bloc). The signal drop tracks the increase in
total LOC and surface area; sentrux's rule engine reports zero violations.

### 14.7 Pressure-release tier 1 invoked (test files only)

R11+R12 is the first round to trip the 500-LOC pressure-release rule. The
decomposition pattern is `#[cfg(test)] #[path = "…_tests.rs"] mod tests;`
per CLAUDE.md's explicit guidance ("that's a clean decomposition, not a
workaround"). Two new files:

- `crates/marain-core/src/parser/mod_tests.rs` — 836 LOC; justification in
  module doc-comment: shared `parse_ok` / `parse_err` helpers exercise one
  cohesive surface, splitting by R-round forces helper chasing.
- `crates/marain-core/src/emit_tests.rs` — 554 LOC; justification in
  module doc-comment: shared `parse_and_emit` / `parse_and_emit_err`
  helpers, one helper set per file matches the convention.

Production-side files all under the 500-LOC target after the split:
`parser/mod.rs` 73 LOC, `parser/grammar.rs` 428 LOC, `emit.rs` 349 LOC,
`ast.rs` 487 LOC. The plausible next pressure point is `parser/grammar.rs`
once R13 adds `functio` parsing (signature + body); a further parse-time
decomposition there would split per syntactic family (declarations,
statements, expressions).

### 14.8 Forward hooks

Open backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md): labeled `break 'name` / `continue 'name` and `break <expr>` (§1; each grows an `Option<Ident>` / `Option<Expr>` field on the stmt); op-name inflection metadata (§5; parallels the carry-over α pattern on `BinOp` variants).

Resolved in later rounds: `functio` declaration block (R13 / §15); `pro` + range tokens (`DotDot` / `DotDotEq`) and `nihil.` (R14+R15 / §16).

## 15. Function Declarations + Calls

Round 13. The parser learns `functio` declarations, `redde` returns, and
function-call expressions; the emitter learns a two-pass module walk that
hoists user-defined functions out of `fn main()`. R10's "lexer catches
generics-attempts with `UnexpectedChar '<'`" gap closes here via a targeted
`LexError::GenericsLookalike` variant. PRD §4.9's PascalCase-for-types rule
becomes parse-enforced.

### 15.1 Scope (v0.2)

**In:**
- `functio <name>(<params>) [dat <Tipus>] : <body>` declarations (PRD §4.11.1).
- `redde [<expr>] .` returns. Bare `redde.` is a unit return (Rust `return;`).
- `<name>(<args>)` call expressions in any expression position.
- `<name>(<args>) .` calls at statement position (`Stmt::Call`).
- Trailing commas in both param lists and call arg lists.
- `Param { name: SigiledIdent, type_ref: TypeRef, span }`; `TypeRef`
  newtype wrapping an `Ident` (reserves a generics-grow seam for v0.3+).
- Type-translation table: `Sermo` → `String`, `Numerus` → `i64`; all other
  PascalCase idents pass through verbatim (B-3 open pass-through).
- PascalCase enforcement in type position via a new
  `ParseError::TypePositionRequiresPascalCase` variant.
- `LexError::GenericsLookalike { ch, span }` for `<` / `>` in source.
- Two-pass emit: top-level `Stmt::Function`s emit above `fn main()`;
  non-function statements land inside `fn main()`; `fn main()` is always
  emitted (cargo requires it).

**Out (deferred):** generics syntax (lex-rejected via the new variant);
closures, lifetimes, where-clauses, visibility modifiers (PRD §4.11.6);
trailing-expression returns (only explicit `redde` is supported); function
values / first-class functions (callee must be a `PlainIdent`, not a
sigiled var-ref).

### 15.2 Decomposition

```
crates/marain-core/src/
  ast.rs                   (modified)  + FunctionStmt, ReturnStmt, CallStmt,
                                          CallExpr, Param, TypeRef; +Stmt
                                          variants Function/Return/Call;
                                          +Expr variant Call; test bloc
                                          moved to sibling per pressure-release
  ast_tests.rs             (new)       sibling test file for ast.rs
  parser/
    mod.rs                 (modified)  + `mod expressions;`
    expressions.rs         (new)       parse_or..parse_primary cascade,
                                          parse_call, make_binop — split out
                                          of grammar.rs per locked C-1 when
                                          grammar.rs crossed 500 LOC
    grammar.rs             (modified)  + parse_function, parse_param_list,
                                          parse_param, parse_type_ref,
                                          parse_return, parse_call_stmt;
                                          dispatch on Functio / Redde and
                                          on PlainIdent+`(` at stmt position;
                                          helpers `expect_kind` /
                                          `expect_keyword` /
                                          `parse_sigiled_ident` promoted to
                                          `pub(super)` for expressions.rs
    error.rs               (modified)  + TypePositionRequiresPascalCase
  lexer/
    mod.rs                 (modified)  + `<` / `>` dispatch arm → new
                                          GenericsLookalike variant
    error.rs               (modified)  + GenericsLookalike { ch, span }
  emit.rs                  (modified)  two-pass emit; + emit_function,
                                          emit_param, emit_type_ref,
                                          emit_return, emit_call,
                                          emit_call_stmt
  emit_tests.rs            (modified)  +R13 emit tests
  parser/mod_tests.rs      (modified)  +R13 parser tests + adjustment to
                                          existing `unknown_statement_start`
                                          test (`functio` is now parsed)
```

File-size status post-split: `ast.rs` 343 LOC ✓, `parser/grammar.rs` 357 ✓,
`parser/expressions.rs` 269 ✓, `emit.rs` 454 ✓, all production-side files
under target. Pressure-release status applies to test siblings only
(`emit_tests.rs` 712, `parser/mod_tests.rs` 1213) and to `lexer/mod.rs`
(665, mostly tests) — each carries the required module-doc justification.

### 15.3 Decisions

_Full rationale: [`tasks/decisions/R13_functio_calls.md`](../tasks/decisions/R13_functio_calls.md). Summary list below._

- **A-1 Function call scope.** Calls IN as both `Expr::Call` and `Stmt::Call`.
- **A-2 Round split.** R13 ships declarations + calls together.
- **B-1 `TypeRef` newtype.** Wraps `Ident`; reserves generics-grow seam (v0.3+ `params: Vec<TypeRef>`).
- **B-2 Bare unit return.** `redde.` → `return;` supported.
- **B-3 Open type pass-through.** `emit_type_ref` is a 2-arm match (`Sermo` → `String`, `Numerus` → `i64`); other PascalCase names pass verbatim.
- **C-1 `grammar.rs` split timing.** Split when threshold crossed (it did); `parser/expressions.rs` extracted.
- **C-2 PascalCase enforcement.** At `parse_type_ref` via `ParseError::TypePositionRequiresPascalCase`.
- **C-3 Trailing commas.** Accepted in both param and arg lists.
- **C-4 `redde` outside function.** Parses cleanly; lands inside `fn main`; rustc rejects the type mismatch.
- **C-5 Empty param list.** Mandatory parens; empty `params` Vec.
- **D-1 Two-pass emit + always-`fn main()`.** Top-level functions hoist above `fn main`; non-function stmts inside; cargo binary-crate requires `fn main`.
- **D-2 Translation table location.** 2-arm match in `emit_type_ref` (no growth pressure).
- **D-3 Param sigil emit.** `^x` → `x`, `@x` → `mut x` (same as `Stmt::Let`).
- **D-4 Call emit shape.** Mechanical sigil-drop + integer-suffix.
- **Reframe `GenericsDeferred` → `LexError::GenericsLookalike`.** Parser variant would have been dead code (`<` not in lex alphabet); moved to lex layer with targeted "deferred to v0.3+" diagnostic.
- **Addition `Stmt::Call`.** Mid-implementation; narrower than `ExprStmt`. Only calls earn statement-position (literals/var-refs have no observable effect).
- **`parse_call` is shared.** `parse_call_stmt` (statement) and `parse_primary` (expression) both call into `parse_call` for the argument list; trailing-comma logic lives in one place.
- **Callee disambiguation in `parse_primary`.** `PlainIdent + (` → call; bare `PlainIdent` → `ExpectedExpression` (PRD §4.5 variables always carry a sigil).
- **`ast.rs` test bloc split.** Sibling `ast_tests.rs` via `#[cfg(test)] #[path = "ast_tests.rs"] mod tests;` — same pattern R11+R12 used.

### 15.4 New AST shape

```rust
pub enum Stmt {
    Let(LetStmt),
    MacroCall(MacroCallStmt),
    If(IfStmt),
    While(WhileStmt),
    Loop(LoopStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Function(FunctionStmt),   // new
    Return(ReturnStmt),       // new
    Call(CallStmt),           // new — side-effect call as statement
}

pub struct FunctionStmt {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,  // None when `dat` omitted
    pub body: Block,
    pub span: Span,
}

pub struct Param {
    pub name: SigiledIdent,
    pub type_ref: TypeRef,
    pub span: Span,
}

pub struct TypeRef {
    pub name: Ident,
    pub span: Span,
}

pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

pub struct CallStmt {
    pub call: CallExpr,
    pub span: Span,
}

pub enum Expr {
    StringLit(StringLit),
    IntegerLit(IntegerLit),
    BoolLit(BoolLit),
    VarRef(SigiledIdent),
    BinOp(BinOpExpr),
    UnaryOp(UnaryOpExpr),
    Call(CallExpr),           // new
}

pub struct CallExpr {
    pub callee: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}
```

Carry-over α (inflection slot, R5 §7.5) untouched on the new nodes —
`Param`'s `name` is a `SigiledIdent` which already carries the slot;
`TypeRef`'s `name` is an `Ident` which carries it too; `CallExpr`'s
`callee` is an `Ident` (no sigil; Stage 1 function calls use bare
PlainIdents).

### 15.5 Test coverage

- **`ast.rs` / `ast_tests.rs`** — +5 new tests covering `FunctionStmt`,
  `ReturnStmt`, `CallExpr`, `CallStmt`, `Param`, and `TypeRef` span
  dispatch and field round-trips.
- **`parser/error.rs`** — +1 test for the new
  `TypePositionRequiresPascalCase` variant's message format.
- **`parser/mod_tests.rs`** — +30 tests: function declarations
  (zero-arg / single-arg / multi-arg / trailing comma / mutable param /
  unknown-type pass-through); returns (with value / bare unit / outside
  function); calls (zero-arg / multi-arg / trailing comma / nested /
  as dic-arg / with binop arg); error paths (missing parens / name /
  colon / return-type-after-`dat` / PascalCase in both param and return
  type / missing colon in param / generics-lookalike via lex layer /
  missing period after redde / bare PlainIdent as expression / call as
  statement / call-stmt missing period); round-trip integration.
- **`lexer/error.rs`** — +1 test that `GenericsLookalike`'s message
  mentions "generics", "v0.3", and the `minor quam` / `maior quam`
  alternatives.
- **`lexer/mod.rs`** — +3 tests: `<` produces `GenericsLookalike`, `>`
  too, message mentions deferral.
- **`emit_tests.rs`** — +18 tests covering top-level partition, fn-main
  synthesis when no top-level non-function stmts exist, type translation
  (`Sermo` → `String`, `Numerus` → `i64`, `Custom` passes through), mut
  param emits `mut`, multi-param comma-separation, bare/value returns,
  call-stmt with trailing semicolon, call-with-args, call-with-binop-arg
  preserves paren-wrap, nested calls, full round-trip (function declared
  + called from main + value used), top-level `redde` inside `fn main`
  for rustc to reject, trailing commas in both lists.
- **Goldens** — 5 new emit fixtures (`18_functio_unit`, `19_functio_typed`,
  `20_functio_multi_param`, `21_functio_call`, `22_functio_translation`)
  and 3 new error fixtures (`errors/11_generics_lookalike`,
  `errors/12_type_pascal_case`, `errors/13_missing_return_type`).
- **Adjustment** — pre-existing `unknown_statement_start_is_error` test
  pointed at `functio foo.` which is now a parseable (though invalid)
  function declaration. Updated to use `est foo.` (the `est` keyword
  has no statement-position dispatch).

**Test count delta: +61.** Workspace total at R13 close: **415** (was 354
at R11+R12 close). `cargo fmt --check`, `cargo clippy --all-targets -D
warnings`, `cargo test --all` all clean.

### 15.6 Sentrux signal at R13 close

`session_start` taken before any code change (signal 7059); `session_end`
after the round + the two file splits: `signal_delta` +14
(7059 → 7073), `cycles_change` 0, `coupling_change` 0.0, DSM
`above_diagonal` stays 0 (clean layering preserved), 0 rule violations.
The signal improvement (vs R11+R12's −85) tracks the file splits reducing
per-file complexity even as new functionality landed.

### 15.7 Pressure-release tier 1 invoked (two splits)

R13 is the first round to trip the 500-LOC threshold on production-side
files. Two splits made:

- **`ast.rs` test sibling split.** Pre-R13: 487 LOC; post-R13 growth: 646
  LOC with test bloc dominating. Per CLAUDE.md sibling-file rule, tests
  moved to `crates/marain-core/src/ast_tests.rs` via
  `#[cfg(test)] #[path = "ast_tests.rs"] mod tests;`. Same pattern R11+R12
  used. ast.rs now 343 LOC, ast_tests.rs 308 LOC; both under target.
- **`parser/grammar.rs` production-side split.** Pre-R13: 428 LOC;
  post-R13 growth: 601 LOC with new statement productions (`parse_function`,
  `parse_param_list`, `parse_param`, `parse_type_ref`, `parse_return`,
  `parse_call_stmt`). Per locked decision C-1 ("split iff it crosses the
  threshold"), the expression cascade (`parse_or` through `parse_primary`,
  plus `parse_call` and `make_binop`) extracted to
  `crates/marain-core/src/parser/expressions.rs`. Cross-cutting helpers
  (`expect_kind`, `expect_keyword`, `parse_sigiled_ident`) promoted to
  `pub(super)` so expressions.rs can call them. grammar.rs now 357 LOC,
  expressions.rs 269 LOC; both under target.

Test-file pressure-release status (per the existing R11+R12 pattern)
applies to `emit_tests.rs` (712 LOC, was 555), `parser/mod_tests.rs`
(1213 LOC, was 832), and now `lexer/mod.rs` (665 LOC, mostly driver tests
that share an in-scope helper set). All three carry updated module-doc
justifications per CLAUDE.md.

### 15.8 Forward hooks

Open backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md): `structura` / `enumeratio` (§2 — the `TypeRef` seam + `emit_type_ref` table absorb them, no fork); generics activation (§1 — `LexError::GenericsLookalike` retires, `parse_type_ref` consumes `<T, U>` into `TypeRef.params`); labeled `break`/`continue`, `break <expr>`, trailing-expression returns, and closures (§1 — closures generalize `Expr::Call`'s callee from `Ident` to `Expr`; the `args: Vec<Expr>` field is already shaped to absorb it).

Resolved in R14+R15 (§16): `pro` + range tokens and `nihil.` — function bodies as first-class block contexts let both drop into the existing `parse_block` mechanism.

## 16. Loops + Ranges + `nihil`

Rounds 14 + 15, batched per locked decision A. The lexer learns the two
range tokens `..` / `..=`; the parser learns range expressions (lowest infix
precedence), the `pro <binding> in <iterable> :` for-loop, and the `nihil.`
do-nothing sentinel. Function and block bodies are already first-class block
contexts (R10/R13), so `pro` bodies and `nihil` statements slot into the
existing `parse_block` mechanism with no structural changes.

### 16.1 Scope (v0.2)

**In:**
- Range tokens `DotDot` (`..`) and `DotDotEq` (`..=`); the `.` lexer arm now
  peeks one/two bytes to distinguish `Period` / `DotDot` / `DotDotEq`.
- Range expressions `a..b` (exclusive) and `a..=b` (inclusive) at a new
  lowest-precedence `parse_range` cascade level (PRD §4.11.5).
- `pro <sigiled-binding> in <iterable> : <body>` for-loops (PRD §4.11.2).
  `iterable` is any expression, so a range literal flows through naturally.
- `nihil.` empty-block sentinel (PRD §4.11.4), emitting Rust `();`.
- `Expr::Range(RangeExpr)`, `Stmt::For(ForStmt)`, `Stmt::Nihil(NihilStmt)`.

**Out (deferred):** open-ended ranges `..b` / `a..` / `..` and `..=b`
(the parser only produces fully-bounded ranges; `RangeExpr`'s `Option`
fields reserve the shape); `pro` over arbitrary iterators beyond ranges and
collection var-refs is unconstrained by design (any expression parses) but
only typechecks if the emitted Rust value is `IntoIterator`; stepped ranges
(`step_by`) and reverse ranges (no Rust `..` analogue) — out of scope.

### 16.2 Decomposition

```
crates/marain-core/src/
  token.rs                 (modified)  + DotDot, DotDotEq variants + Display
  lexer/mod.rs             (modified)  `.` dispatch peeks for `..` / `..=`
  ast.rs                   (modified)  + RangeExpr, ForStmt, NihilStmt;
                                          +Expr::Range; +Stmt::For/Nihil;
                                          span() dispatch extended
  ast_tests.rs             (modified)  + span-dispatch tests for the 3 nodes
  parser/
    expressions.rs         (modified)  + parse_range (new lowest-precedence
                                          level; parse_expr now enters here
                                          then descends to parse_or)
    grammar.rs             (modified)  + parse_for, parse_nihil; dispatch on
                                          Keyword::Pro / Keyword::Nihil
  emit.rs                  (modified)  + emit_for, range arm in emit_expr,
                                          `();` arm for Stmt::Nihil
  emit_tests.rs            (modified)  + R14 emit tests
  parser/mod_tests.rs      (modified)  + R14 parser tests
```

File-size status post-R14 (production-side): `token.rs` 153 ✓, `ast.rs`
381 ✓, `parser/grammar.rs` 383 ✓, `parser/expressions.rs` 292 ✓, `emit.rs`
485 ✓ — all under the 500-LOC target. **No pressure-release split needed
this round** (the R13-close watch-out that `emit.rs` might cross 500 did not
materialize; it landed at 485). Test-file pressure-release status is
unchanged in kind and applies to `parser/mod_tests.rs` (1407 LOC),
`emit_tests.rs` (793), `lexer/mod.rs` (749) — each carries its module-doc
justification.

### 16.3 Decisions

_Full rationale: [`tasks/decisions/R14_15_pro_ranges_nihil.md`](../tasks/decisions/R14_15_pro_ranges_nihil.md). Summary list below._

- **A Round batching.** R14 (`pro` + ranges) and R15 (`nihil`) ship together — small, and `pro` bodies are the natural place to exercise `nihil`.
- **Range precedence.** New `parse_range` is the lowest infix level: `parse_expr` enters at `parse_range`, which parses an `parse_or` lhs, then optionally consumes `..` / `..=` and a `parse_or` rhs. Mirrors Rust's table (ranges below all binary operators).
- **Range operands are `parse_or`, not `parse_range`.** Ranges don't chain (`a..b..c` is not valid Rust); using `parse_or` for both operands makes chaining a parse error naturally, no special-casing.
- **Bounded-only ranges.** Parser produces only `a..b` / `a..=b`; `RangeExpr.start`/`.end` are `Option<Box<Expr>>` so open-ended forms are a future round, not a reshape.
- **`nihil` emit shape `();`** (the open sub-decision; chose `();` over `{}`). A unit statement satisfies the "block needs ≥1 statement" rule without introducing a nested scope.
- **`pro` binding is a `SigiledIdent`.** Same sigil convention as `Stmt::Let` / `Param`: `^i` → `i`, `@i` → `mut i`. A bare (sigil-less) binding is a parse error (`expected sigiled identifier`), consistent with PRD §4.5.
- **Range emit is not paren-wrapped.** Unlike `BinOp`/`UnaryOp` (paren-everywhere), ranges aren't operators in the precedence-drift sense; operands self-wrap via `emit_expr` if they are `BinOp`/`UnaryOp` shapes.

### 16.4 New AST shape

```rust
pub enum Stmt {
    // ... existing variants ...
    For(ForStmt),     // new — pro <binding> in <iter> :
    Nihil(NihilStmt), // new — nihil.
}

pub struct ForStmt {
    pub binding: SigiledIdent,
    pub iter: Expr,
    pub body: Block,
    pub span: Span,
}

pub struct NihilStmt {
    pub span: Span,
}

pub enum Expr {
    // ... existing variants ...
    Range(RangeExpr), // new
}

pub struct RangeExpr {
    pub start: Option<Box<Expr>>,  // always Some from the v0.2 parser
    pub end: Option<Box<Expr>>,    // always Some from the v0.2 parser
    pub inclusive: bool,
    pub span: Span,
}
```

Carry-over α (inflection slot) untouched: `ForStmt.binding` is a
`SigiledIdent`, which already carries the slot.

### 16.5 Test coverage

- **`ast_tests.rs`** — +3 tests: span dispatch through `Expr` for
  `RangeExpr`, through `Stmt` for `ForStmt` and `NihilStmt`.
- **`lexer/mod.rs`** — +5 driver tests: `0..10` → one `DotDot`, `0..=10`
  → one `DotDotEq`, single `.` still `Period`, `...` → `DotDot` + `Period`
  (greedy two-dot wins), trailing `0..10.` → range then `Period`.
- **`token.rs`** — +2 `Display` assertions (`..` / `..=`).
- **`parser/mod_tests.rs`** — +11 tests: ranges in let-RHS (exclusive /
  inclusive), range with binop endpoints, range missing rhs is error,
  range at statement position in `dic`; `pro` over exclusive / inclusive
  range, `pro` with mutable binding, `pro` over a var-ref; error paths
  (`pro` missing `in`, missing sigil on binding, missing colon); `nihil`
  at top level, inside `pro` body, inside `functio` body, missing period.
- **`emit_tests.rs`** — +11 tests: range emits `..` / `..=`, range with
  binop endpoints preserves paren-wrap; `pro` over exclusive / inclusive
  range emits `for … in …`, mutable binding emits `mut`, body indents
  correctly, `pro` over var-ref emits clean iterator; `nihil` emits `();`,
  inside `functio` body at correct indent, inside `pro` body.
- **Goldens** — 4 new emit fixtures (`23_pro_range_exclusive`,
  `24_pro_range_inclusive`, `25_functio_with_nihil`, `26_pro_with_nihil`)
  and 3 new error fixtures (`errors/14_pro_missing_in`,
  `errors/15_pro_missing_sigil`, `errors/16_nihil_missing_period`).

**Test count delta: +35.** Workspace total at R14+R15 close: **450** (was
415 at R13 close). `cargo fmt --check`, `cargo clippy --all-targets -D
warnings`, `cargo test --all` all clean.

### 16.6 Sentrux signal at R14+R15 close

The session baseline was lost to a mid-round console crash; the comparison
is against the recorded R13-close signal (7073). Post-R14 scan:
`quality_signal` **7060** (`signal_delta` −13), DSM `above_diagonal` **0**
(clean downward layering preserved), `import_edges` 38 → 41 (+3, from the
new cross-module type uses: `ForStmt` into emit, `RangeExpr` into
expressions, the new tokens into the parser), 0 cycles, `check_rules` 4/4
pass / 0 violations. The small negative delta tracks genuinely-added
surface area (2 tokens, 3 AST nodes, 3 parser fns, 3 emit arms) with no
offsetting file split this round.

### 16.7 Pressure-release tier 1 not invoked

No production-side file crossed the 500-LOC target this round (largest:
`emit.rs` at 485). The R13-close watch-out — that `emit.rs` plus
`emit_for`/`emit_nihil`/range emit might cross 500 — resolved comfortably
under target. Test siblings remain in their existing pressure-release
status with unchanged justifications.

### 16.8 Forward hooks

Open backlog tracked in [`tasks/ROADMAP.md`](../tasks/ROADMAP.md) §1: open-ended ranges (`..b` / `a..` / `..` / `..=b` — `RangeExpr`'s `Option` fields already model them, so activation is a `parse_range` change and the emit arm already guards both sides with `if let Some`); stepped / reverse iteration (lowers to `.step_by` / `.rev` once method-call syntax exists); `nihil` as an expression (`Expr::Nihil` mirroring `Stmt::Nihil`). `pro` over real collection iterators lands for free once collection literals exist (§2) — any expression already parses as the iterable.

## 17. Reassignment (`fit`)

Round 16. Wires the already-lexed `Keyword::Fit` through parse + emit so a
declared mutable binding can be re-bound — the binding lifecycle's missing half
(initial assignment `sit @x est 0.` worked since R5; reassignment did not). In-spec
(PRD §4.4 reassign copula, `docs/core-lexicon.md:46`); no PRD amendment, just
wiring an existing keyword.

### 17.1 Scope (v0.2)

**In:** `@x fit <expr> .` reassignment statement; `Stmt::Assign(AssignStmt)` with a
bare `SigiledIdent` target; dispatch on a leading `SigiledIdent`; a parse-time
`@`-required check on the target; emit of `name = value;` with NO `mut`.

**Out (deferred):** field / index targets (`@x.y`, `@x[i]` — no method-call or
index syntax yet); compound assignment (`+=` analogue — not in the PRD §4.4 operator
table, so increment stays explicit: `@x fit @x plus 1.`); cross-statement mutability
tracking (reassigning a `sit ^x`-declared binding via `@x` still falls to rustc).

### 17.2 Decomposition

```
crates/marain-core/src/
  ast.rs               (modified)  + AssignStmt; +Stmt::Assign; span() dispatch
  ast_tests.rs         (modified)  + assign span-dispatch test
  parser/grammar.rs    (modified)  + parse_assign; dispatch on SigiledIdent;
                                      Sigil import
  parser/error.rs      (modified)  + ImmutableReassignmentTarget variant + arms
                                      + message test
  parser/mod_tests.rs  (modified)  + R16 parser tests
  emit.rs              (modified)  + emit_assign + Stmt::Assign arm
  emit_tests.rs        (modified)  + R16 emit tests
```

File-size status (production-side): `ast.rs` 396 ✓, `parser/grammar.rs` 411 ✓,
`parser/error.rs` 174 ✓, `emit.rs` **500** (at the target ceiling — the next emit
addition must split or justify). No pressure-release split needed this round.

### 17.3 Decisions

_Full rationale: [`tasks/decisions/R16_fit_reassignment.md`](../tasks/decisions/R16_fit_reassignment.md). Summary list below._

- **A Require `@` target.** A `fit` target must carry the `@` sigil; a `^` target is a hard parse error (`ImmutableReassignmentTarget`, cites PRD §4.5). Surfaces the contradiction at the Marain level, not via a rustc message with no span back-mapping. Purely syntactic; no symbol table.
- **B Emit no `mut`.** `emit_assign` is deliberately not shared with `emit_let`/`emit_param`/`emit_for` — a reassignment is a *use* site, not a binding site, so the `@`→`mut` rule does not apply. The one non-obvious bit; carries an inline comment.
- **C Dispatch on leading `SigiledIdent`.** Unambiguous and previously dead (such a statement was always `UnknownStatementStart`). Wrong-verb case (`@x est 5.`) now yields a clean "expected keyword `fit`".
- **D Bare `SigiledIdent` target.** Field/index targets out of scope until method/index syntax lands; shape mirrors `LetStmt` minus `est`.
- **PRD reconciliation.** PRD line 115's illustrative `^x fit 5` is footnoted as showing the verb contrast only; the §4.5 sigil rule rejects a `^` target. Canonical form is `@x fit 5`.

### 17.4 New AST shape

```rust
pub enum Stmt {
    // ... existing variants ...
    Assign(AssignStmt), // new — @x fit <expr> .
}

pub struct AssignStmt {
    pub target: SigiledIdent, // always Sigil::Mutable (parser-enforced)
    pub value: Expr,
    pub span: Span,
}
```

Carry-over α (inflection slot) untouched: `AssignStmt.target` is a `SigiledIdent`,
which already carries the slot.

### 17.5 Test coverage

- **`ast_tests.rs`** — +1: `Stmt::Assign` span dispatch.
- **`parser/mod_tests.rs`** — +5: `@x fit 5.` parses to `Assign` with `@` target;
  `@x fit @x plus 1.` value is a BinOp; `^x fit 5.` → `ImmutableReassignmentTarget`;
  `@x fit 5` (no period) → `UnexpectedToken`; `@x est 5.` (wrong verb) →
  `UnexpectedToken` expecting `fit`.
- **`parser/error.rs`** — +1: `ImmutableReassignmentTarget` message names `^x`/`@x`
  and cites PRD §4.5.
- **`emit_tests.rs`** — +4: reassign emits assignment without `mut`; increment idiom
  emits `x = (x + 1i64);`; rust-keyword target uses `r#` prefix; unescapable target
  errors.
- **Goldens** — 1 new emit fixture (`27_fit_reassignment`, the accumulator) and 1
  new error fixture (`errors/17_fit_immutable_target`).
- **Manual e2e** — `pro`/`fit` accumulator (`sit @series est 0.` / `pro ^i in 1..=5
  : @series fit @series plus ^i.` / `dic ^series.`) run through `marain run` prints
  `15`. Confirms emitted Rust compiles and executes; the lone `unused_parens`
  warning is the known Task 3 tradeoff.

**Test count delta: +11.** Workspace total at R16 close: **461** (was 450).
`cargo fmt --all`, `cargo clippy --all-targets -D warnings`, `cargo test --all`
all clean.

### 17.6 Sentrux signal at R16 close

Session baseline `quality_signal` **7063**; post-R16 **7057** (`signal_delta` −6,
i.e. *improved* — lower is better in this signal's polarity per R14+R15's −13).
`coupling_change` [0.0, 0.0], `cycles_change` [0, 0], 0 violations, "Quality stable
or improved". No new import edges (the new types stay within already-coupled
modules: `AssignStmt` flows ast → parser/emit, same as `LetStmt`).

### 17.7 Pressure-release tier 1 not invoked

No production file crossed the 500 target — but `emit.rs` landed *exactly* at 500.
The next emit-arm addition must either split `emit.rs` (e.g. `emit/{stmt,expr}.rs`)
or carry a module-doc justification per the pressure-release rule. Flagged as the
R16-close watch-out. **Resolved in R17** (§18.6): the f-string emit arm triggered the
split — expression emitters moved to `emit/expr.rs`, `emit.rs` back to 436 LOC.

### 17.8 Forward hooks

Compound assignment (`+=`-style) is not specced in the PRD §4.4 operator table, so
increment stays explicit (`@x fit @x plus 1.`); revisit only with a PRD amendment.
Field/index targets activate when method-call / index syntax lands. Cross-statement
mutability tracking (catching `sit ^x` declared, `@x` reassigned) is a
name-resolution-era concern.

## 18. f-strings (interpolation + concatenation) — R17

Ships `f"…{^x}…"` (PRD §4.6 / §4.7) as sugar over `format!`, resolving TODO Task 1
(no string composition existed: `plus` is arithmetic-only, no concat operator, no
interpolation). The entire literal — including each `{…}` hole — is resolved in one
lexer pass and lowered to a `format!` call. In-spec activation of a deferred feature;
no PRD amendment.

### 18.1 Pipeline

`f"salve {^nomen}!"` → lexer `scan_fstring` → `TokenKind::FStringLit([Literal("salve "),
Interp{^nomen}, Literal("!")])` → parser pure-lift → `Expr::FString(FStringLit{parts})`
→ emit → `format!("salve {}!", nomen)`. Concatenation is the all-holes form:
`f"{^a}{^b}"` → `format!("{}{}", a, b)`.

### 18.2 Decisions

_Full rationale: [`tasks/decisions/R17_fstrings.md`](../tasks/decisions/R17_fstrings.md). Summary list below._

- **A f-strings are the only composition mechanism.** No concat operator/keyword; PRD §4.7 says multi-value cases are handled by f-strings. Owner-confirmed.
- **B Holes are variable-refs-only.** `{^name}` / `{@name}` only (optional spaces); empty / no-sigil / expression holes and format specs are `InvalidFStringHole`. Owner-locked. Makes the var-only path *cheaper* than full expressions (reuses `scan_sigiled_ident`), not more expensive.
- **C One-pass lexer resolution.** A hole is a slice of the same file, so scanning it inline via `scan_sigiled_ident` gives a correct span + `FileId` for free — no parser sub-lexing, no remapping, no lexer mode-state. Internal `{`/`}` never reach the main dispatch, so they don't perturb indent/bracket state.
- **D Prefix is `f"` (no space).** Unambiguous: variables carry sigils and calls need `(`, so `f"` is never anything else. One-line dispatch guard before the ident arm.
- **E Emit as `format!`; split `emit.rs`.** Literal parts double `{`/`}`; holes contribute `{}` + a trailing arg. Triggered the §17.7 emit-500 split.

### 18.3 New shapes

```rust
// token.rs
pub enum FStringSeg { Literal(String), Interp { sigil: Sigil, name: String, span: Span } }
TokenKind::FStringLit(Vec<FStringSeg>)            // lexer output, holes pre-resolved

// ast.rs
pub enum FStringPart { Literal(String), Interp(SigiledIdent) }
pub struct FStringLit { pub parts: Vec<FStringPart>, pub span: Span }
Expr::FString(FStringLit)
```

Carry-over α (inflection slot) untouched: `Interp` resolves to a `SigiledIdent`, which
already carries the slot. Future widening of `Interp` from `SigiledIdent` to `Expr`
admits expression holes without reshaping the token/AST boundary.

### 18.4 Test coverage

- **`lexer/strings.rs`** — +16: `scan_fstring` mechanics (literal, single/adjacent/mutable holes, `{{`/`}}`, space-padding, escapes, hole span) and errors (empty, no-sigil, expression, unmatched `}`, unterminated).
- **`lexer/mod_tests.rs`** — +4: full-pipeline prefix dispatch, `f "x"` is ident+string, `functio` unaffected, empty-hole lex error; asserts internal braces emit no `LBrace`/`RBrace`.
- **`parser/mod_tests.rs`** — +3: interpolation parts, no-hole single literal, concat in macro-arg position.
- **`emit_tests.rs`** — +6: interpolation/concat lowering, f-string in `let`, no-hole no-args, doubled literal braces, raw-ident escape in a hole.
- **`lexer/error.rs`** — +1: `InvalidFStringHole` message shows the `{^nomen}` example.
- **`ast_tests.rs`** — +1: `Expr::FString` span dispatch.
- **Goldens** — 2 emit (`28_fstring_interpolation`, `29_fstring_concat`) + 2 error (`errors/18_fstring_empty_hole`, `errors/19_fstring_expression_hole`).
- **Manual e2e** — interpolation + concat + integer interpolation run through `marain run` print `Salve, Munde!` / `Concat: SalveMunde` / `Numerus est 42.`.

**Test count delta: +31.** Workspace total at R17 close: **492** (was 461).
`cargo fmt --all`, `cargo clippy --all-targets -D warnings`, `cargo test --all` clean.

### 18.5 Sentrux signal at R17 close

Baseline `quality_signal` **7057**; post-R17 **7033** (`signal_delta` −24, *improved* —
lower is better in this signal's polarity). `coupling_change` [0.0, 0.0],
`cycles_change` [0, 0], 0 violations, "Quality stable or improved".

### 18.6 File-size status / pressure-release

`emit.rs` split into `emit.rs` (436, statements + escapers + `EmitError`) + `emit/expr.rs`
(129, expression emitters incl. `emit_fstring`); the child reaches the private escapers
via `super::`. This discharges the R16 §17.7 watch-out. The same pressure moved
`lexer/mod.rs`'s driver tests to a sibling `lexer/mod_tests.rs` (the split its own
doc-comment had pre-authorized): `mod.rs` 264 ✓, `mod_tests.rs` 553 (justified test
file). `lexer/strings.rs` 431 ✓, `token.rs` 177 ✓, `ast.rs` 417 ✓,
`parser/expressions.rs` 307 ✓.

### 18.7 Forward hooks

Expression holes (`{^a plus ^b}`) and Rust format specs (`{x:>5}`) are deferred:
activation widens `FStringPart::Interp` from `SigiledIdent` to `Expr` and gives the
lexer brace-balanced sub-lexing (or a parser-side hole parse). `dic f"…"` currently
double-wraps (`println!("{}", format!(…))`) — correct; a `dic`-special-case that emits
`println!("…{}…", …)` directly is a future nicety. Triple-quoted strings (`"""…"""`)
remain a separate deferred item (ROADMAP §4).
