# Marain Core Lexicon

The core translations from Rust to nominative Latin for Marain v0.1 (Stage 1).
Source of truth for keyword and operator decisions; see `PRD.md` §§4.2–4.7 for
design rationale, and `crates/marain-core/src/lexer/keywords.rs` for the
keywords the lexer actually accepts today.

Three layers covered:

1. **Sigils** — Marain-original; mark every variable reference (PRD §4.5).
2. **Reserved Marain keywords** — words the lexer rejects as bare identifiers
   in v0.1 (29 today).
3. **Rust keywords** — words that need handling at the Marain↔Rust boundary,
   either via a Marain equivalent or via the emitter's `r#`-escape policy.

## Marain Sigils

| Sigil | Description |
| ----- | ----------- |
| `^` | Immutable variable reference (`^x` ≈ `x` for immutable binding) |
| `@` | Mutable variable reference (`@x` ≈ `x` for `let mut` binding) |

Sigils are required on every variable reference — declarations, expressions,
function parameters, struct fields, destructuring, method receivers. Omitting
one is a parse error. There is no `mut` keyword in Marain; the `@` sigil IS
the mutability marker. Sigils don't appear in emitted Rust — `^x` and `@x`
both lower to bare `x` at use sites; mutability is encoded only at the
declaration site (`sit @x` → `let mut x`).

## Reserved Marain Keywords (v0.1)

All entries present in the lexer today (`lexer/keywords.rs`, 29 entries). A
bare identifier matching one of these tokenizes as a keyword, not an
identifier. Sigiled identifiers (`^name`, `@name`) skip the keyword table
entirely, so `^et` is a variable named `et`, not the `et` operator.

Note: Marain does not use a macro sigil (`!`), so the common single-argument
macros are reserved as their own keywords (PRD §4.7).

### Bindings & values

| Marain | Rust | Notes |
| ------ | ---- | ----- |
| `sit` | `let` | introduces a binding |
| `est` | `=` (init) | indicative copula "is" (PRD §4.4) |
| `fit` | `=` (reassign) | indicative present "becomes"; only valid on a previously declared binding |
| `verum` | `true` | |
| `falsum` | `false` | |
| `ego` | `self` | Marain `^ego` / `@ego` map to Rust `self` / `&mut self` |
| `tenet` | `&` / `&mut` | borrow; `tenet ^x` = `&x`, `tenet @x` = `&mut x` (PRD §4.5) |

### No-punct macros (PRD §4.7)

| Marain | Rust | Notes |
| ------ | ---- | ----- |
| `dic` | `println!` | stdout, single expression argument |
| `queror` | `eprintln!` | stderr — "I complain" |
| `agmen` | `vec!` | sequence / list builder |
| `forma` | `format!` | returns a `Sermo` |
| `DETONATIO` | `panic!` | sanctioned ALL-CAPS exception (PRD §4.2); `!`-bearing, not no-punct, but the name is reserved |

### Control flow & declarations

| Marain | Rust | Notes |
| ------ | ---- | ----- |
| `functio` | `fn` | function declaration |
| `redde` | `return` | |
| `si` | `if` | |
| `dum` | `while` | |
| `pro` | `for` | |

### Logical operators

| Marain | Rust | Notes |
| ------ | ---- | ----- |
| `et` | `&&` | conjunction "and" |
| `vel` | `\|\|` | conjunction "or" (also component of `minor vel par`, `maior vel par`) |
| `non` | `!` | negation prefix; also component of `non aequat` |

### Arithmetic & comparison (atomic words)

Component words for multi-word operator phrases below. Individually meaningful
where indicated; otherwise only valid as part of a phrase.

| Marain | Rust | Notes |
| ------ | ---- | ----- |
| `plus` | `+` | |
| `minus` | `-` | |
| `per` | `*` | preposition "by"; also component of `divisus per` |
| `modulo` | `%` | ablative of `modulus` |
| `aequat` | `==` | 3rd-sg. indicative active of `aequare`, "equals"; also component of `non aequat` |
| `maior` | (phrase only) | "greater"; only in `maior quam` / `maior vel par` |
| `minor` | (phrase only) | "less"; only in `minor quam` / `minor vel par` |
| `quam` | (phrase only) | "than"; only in `maior quam` / `minor quam` |
| `par` | (phrase only) | "equal"; only in `maior vel par` / `minor vel par` |
| `divisus` | (phrase only) | "divided"; only in `divisus per` |

## Multi-word operator phrases (PRD §4.4)

Recognized at the parser level via a fixed phrase table; the lexer emits one
token per component word. Component words above are reserved today so a future
parser expansion doesn't break existing source.

| Marain | Rust | Notes |
| ------ | ---- | ----- |
| `non aequat` | `!=` | "does not equal" |
| `maior quam` | `>` | "greater than" |
| `minor quam` | `<` | "less than" |
| `maior vel par` | `>=` | "greater or equal" |
| `minor vel par` | `<=` | "less or equal" |
| `divisus per` | `/` | "divided by" |

Operator parsing and precedence are not yet wired up in the v0.1 parser
(operator expressions are deferred — `2 plus 3` doesn't parse today).

## Rust Keywords at the Marain↔Rust Boundary

Rust 2024 keywords that need a position at the boundary. Three cases per row:

- **Mapped** — a Marain keyword (above) stands in for this Rust keyword.
- **Auto-escaped** — the emitter wraps the identifier in `r#` automatically
  (`emit.rs::is_rust_reserved_escapable`). You can use the Rust word as a
  sigiled Marain identifier and it'll emit as `r#word`. This is a footgun
  more than a recommendation — prefer the Marain mapping where one exists.
- **Unescapable** — raw-identifier syntax cannot escape these. Surfacing one
  as a Marain identifier is an `EmitError::UnescapableRustKeyword`. The five:
  `crate`, `extern`, `self`, `Self`, `super`.

### Strict Rust keywords

| Rust | Marain | Notes |
| ---- | ------ | ----- |
| `_` | `_` | wildcard / disposable designator |
| `as` | — | no mapping; auto-escaped |
| `async` | — | non-goal (PRD §8); auto-escaped |
| `await` | — | non-goal (PRD §8); auto-escaped |
| `break` | — | no mapping; auto-escaped (PRD §4.2 floats `interrumpe`) |
| `const` | — | no mapping; auto-escaped |
| `continue` | — | no mapping; auto-escaped |
| `crate` | — | **unescapable** |
| `dyn` | — | no mapping; auto-escaped |
| `else` | — | no mapping; auto-escaped |
| `enum` | `enumeratio` | PRD §4.3 (proposed; not yet in lexer) |
| `extern` | — | non-goal — no FFI in Marain (PRD §8); **unescapable** |
| `false` | `falsum` | |
| `fn` | `functio` | PRD §4.3 (proposed; not yet in lexer) |
| `for` | `pro` | |
| `if` | `si` | |
| `impl` | `praestatio` | PRD §4.3 (proposed; not yet in lexer) |
| `in` | — | no mapping; auto-escaped |
| `let` | `sit` | |
| `loop` | — | no mapping; auto-escaped |
| `match` | — | no mapping; auto-escaped |
| `mod` | `modulus` | PRD §4.3 (proposed; not yet in lexer) |
| `move` | — | no mapping; auto-escaped |
| `mut` | `@` (sigil) | mutability is a sigil prefix on the variable, not a keyword (PRD §4.5) |
| `pub` | — | no mapping; auto-escaped |
| `ref` | — | no mapping; auto-escaped (Marain uses `tenet` for borrows) |
| `return` | `redde` | |
| `self` | `ego` | Marain `ego` lowers to Rust `self`, so practical conflict is rare; the literal Rust word is **unescapable** |
| `Self` | — | no mapping; **unescapable** |
| `static` | — | no mapping; auto-escaped |
| `struct` | `structura` | PRD §4.3 (proposed; not yet in lexer) |
| `super` | — | no mapping; **unescapable** |
| `trait` | `proprietas` | PRD §4.3 (proposed; not yet in lexer) |
| `true` | `verum` | |
| `type` | — | no mapping; auto-escaped |
| `unsafe` | — | non-goal — no `unsafe` in Marain (PRD §8); auto-escaped |
| `use` | — | no mapping; auto-escaped |
| `where` | — | no mapping; auto-escaped (Stage 2 will house ablative bounds here per PRD §4.2) |
| `while` | `dum` | |

### Weak keywords (Rust)

Reserved only in specific syntactic contexts in Rust. Passed through verbatim
by Marain today (no special handling beyond the normal identifier rules).

| Rust | Marain | Notes |
| ---- | ------ | ----- |
| `'static` | — | lifetimes deferred past v0.1 (PRD §4.5) |
| `dyn` | — | weak in some contexts; also listed as strict above |
| `macro_rules` | — | macro authoring not in Marain (PRD §8) |
| `raw` | — | weak in raw-pointer / raw-identifier contexts |
| `safe` | — | weak in `extern` context |
| `union` | — | no Marain equivalent |

### Reserved-for-future keywords (Rust)

All auto-escaped by the emitter (`is_rust_reserved_escapable`), so a Marain
program using one as a sigiled identifier today (`sit ^box est 5.` → `let r#box = 5i64;`)
keeps producing valid Rust after the word becomes active in some future
edition.

| Rust | Marain | Notes |
| ---- | ------ | ----- |
| `abstract` | — | future-reserved |
| `become` | — | future-reserved |
| `box` | — | future-reserved |
| `do` | — | future-reserved |
| `final` | — | future-reserved |
| `gen` | — | active in edition 2024+; auto-escaped |
| `macro` | — | future-reserved |
| `override` | — | future-reserved |
| `priv` | — | future-reserved |
| `try` | — | future-reserved |
| `typeof` | — | future-reserved |
| `unsized` | — | future-reserved |
| `virtual` | — | future-reserved |
| `yield` | — | future-reserved |

## Proposed Standard-Library Type Translations (PRD §4.3)

PRD-blessed names for the eventual type-system layer. Not yet enforced by the
lexer; the current emit pipeline doesn't generate `use` statements, so these
names don't appear in Rust output yet. Listed here per PRD §4.3 ("Full
vocabulary will be enumerated in `docs/lexicon.md`").

| Rust | Marain | Notes |
| ---- | ------ | ----- |
| `String` | `Sermo` | "speech" |
| `Vec<T>` | `Agmen<T>` | "army / column"; shares root with the `agmen` macro |
| `HashMap<K, V>` | `Vocabularium<K, V>` | "vocabulary" |
| `Option<T>` | `Fortasse<T>` | "perhaps" |
| `Result<T, E>` | `Eventus<T, E>` | "outcome" |

## Numeric literals

Numeric literals are Arabic, not Latin (PRD §4.3). `0`–`9` digits, `_`
separators, the standard prefixes (`0x`, `0o`, `0b` — not yet in v0.1's
lexer), decimal points, and exponent notation are preserved as-is from Rust.
Latin number-words (`quinque`, `decem`, …) are not legal numeric literals.

| Marain source | Rust output |
| ------------- | ----------- |
| `42` | `42i64` |
| `1_000_000` | `1000000i64` |

The v0.1 emitter forces every integer literal to `i64` so large literals don't
silently default to `i32` and overflow.

## String literals

Single-quoted strings `"..."` with escapes `\"`, `\\`, `\n`, `\t`, `\r`, `\0`.
Triple-quoted strings (`"""…"""`) and f-strings (`f"salve {nomen}"`) are
PRD-promised (§4.6) but deferred past v0.1.
