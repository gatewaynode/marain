# Marain — Product Requirements Document

_Document status: **DRAFT**, v0.1-PRD-1. Sections marked PROPOSED await author review._

## 1. Overview

Marain is a Rust-flavored toy language that re-skins Rust syntax with Latin words, where Latin grammatical cases and verb moods carry semantic meaning, and which borrows a small set of ergonomic features from Python (indentation-significant blocks, triple-quoted strings, a dynamic value wrapper, and concise dict/list/tuple/f-string literals).

The language is **staged** (§4.10): Stage 1 is a nominative-only Latin re-skin with Rust-fixed word order — the v0.1 target. Stage 2 is an opt-in, post-v0.1 layer that activates the full Latin case/conjugation grammar and enables free word order, paired with an LSP that turns the language into an interactive Latin pedagogy environment. Stage 3 is the Rust target.

The Marain compiler is a source-to-source transpiler: `.lat` source (+ optional sidecar `.latin` for Stage 2) → tokens → AST → Rust source → `cargo`/`rustc` → executable. Marain owns only the front end; correctness, ownership, lifetimes, type checking, and codegen are delegated to Rust.

## 2. Goals

1. **Latin study aid.** Writing Marain exercises Latin **vocabulary and verb-mood recognition** in Stage 1 (the v0.1 surface). Active declension and conjugation practice — plus creative word-order composition — lands in Stage 2 paired with an LSP-driven pedagogical surface (§4.10).
2. **Rust thinking-sharpener.** Building Marain (lexer, parser, AST, transpiler) keeps the author's Rust skills active.

These goals are dual and equal. A design decision that meaningfully compromises either is suspect.

## 3. Audience & Distribution

Primary user: the author. Secondary: a small enthusiast circle (Latin students who code, Rust hobbyists who like language design). A README plus a handful of runnable examples is required; broad UX investment is not.

Distribution: public GitHub repo, source-only. No `crates.io` publish until well past v1.

## 4. The Language

### 4.1 Core Premise

Marain is, roughly, Rust expressed through Latin morphology. Anything Rust can express, Marain can express. The translation is structural, not magical: a Marain `functio` becomes a Rust `fn`. After transpilation, the borrow checker and type system are pure Rust.

### 4.2 Latin Grammar as Syntax — staged across compilation stages

Marain's Latin-grammar surface is **staged** (see §4.10). The table below describes the *target* mapping that Stage 2 enforces. **Stage 1 — the v0.1 default — simplifies dramatically:** all identifier-position tokens are written in the **nominative case only**, and Rust-fixed word order applies. Verb mood (subjunctive / imperative / indicative) is the only grammatical-form distinction Stage 1 enforces; the case rows below are inert in Stage 1.

**Stage 2** (opt-in, post-v0.1) activates the full table. The lexer normalizes each identifier-like token into a `(lemma, inflection)` pair, the parser checks inflection against role and emits Latin-grammar errors when it does not (e.g., _"expected accusative argument, got nominative `textus`; did you mean `textum`?"_), and case markings (not token position) drive parsing — enabling free word order within a statement. Stage 2 also pairs with an LSP that surfaces declension and conjugation suggestions interactively (§4.10, §8 roadmap).

| Latin form                       | Syntactic role                                                  | Illustrative example                              |
| -------------------------------- | --------------------------------------------------------------- | ------------------------------------------------- |
| **Nominative case** (noun)       | Declarations — the thing being introduced                       | binding names, declared `functio`/`structura`/`enumeratio` names, type names |
| **Genitive case** (noun)         | Paths, fields, ownership-transfer source                        | `moduli::res` (module's thing); `puellae nomen` (the girl's name) |
| **Dative case** (noun)           | Function parameters, reassignment targets                       | parameter lists carry dative forms                |
| **Accusative case** (noun)       | Call arguments, `match` scrutinees                              | `dic textum` (say the-text)                       |
| **Ablative case** (noun)         | Trait bounds, lifetime annotations                              | `cum Ostentatione` (with display)                 |
| **Subjunctive mood** (verb)      | Bindings — "let it be"                                          | `sit` (`let`), `sit mutabilis` (`let mut`)        |
| **Imperative mood** (verb)       | Macros and control-flow keywords that perform an action         | `dic!` (println), `pone!` (insert); `interrumpe` (break), `continua` (continue) |
| **Indicative mood** (verb)       | Statements / functions that report or compute                   | `functio` bodies, `redde` (return)                |

**Declaration-site wins (Stage 1 rule).** In Stage 1, an identifier is inflected *once*, at its first introduction, into the nominative. Every subsequent reference echoes that form unchanged — the lexer does not re-inflect per use-site role. This is **"first to define is followed"** — it accepts Latin-grammatical roughness downstream in exchange for stable identifier spelling and a dramatically simpler lexer (one form per identifier, recorded at declaration; later occurrences must match it verbatim).

In **Stage 2**, this rule relaxes: identifiers may take different inflections at different use sites — that is precisely how case marking enables free word order. The Stage 2 grammar spec is the source of truth for that mode; "first to define is followed" applies only to Stage 1.

**Out-of-phase exception: `DETONATIO!`.** Rust's `panic!` is rendered `DETONATIO!` — a nominative-case noun in ALL CAPS, deliberately violating the imperative-for-action-macros rule above. The visual discordance is the point: detonation is a terminal, unrecoverable act and the source should *not* read like surrounding code. This is the only sanctioned exception to the case-and-mood table; further exceptions require explicit justification of equivalent semantic weight.

**Operators are Latin function words, not symbols.** Marain replaces `=`, `==`, `<`, `+`, `&&`, etc. with the appropriate Latin verb (`est` "is", `fit` "becomes", `aequat` "equals"), preposition (`per` "by", `plus` "more"), or conjunction (`et` "and", `vel` "or") — see §4.4 for the full table. Assignment vs. equality is disambiguated by *verb choice* (`est` for initialization, `fit` for reassignment, `aequat` for comparison), not by token shape (`=` vs `==`). Precedence and associativity inherit from Rust verbatim.

**Identifier sigils are a third orthographic layer** beyond declensions and verb moods. Every variable reference carries `@` (mutable) or `^` (immutable) as its first character — see §4.5. This is a Marain-original convention, not Latin grammar; it earns its keep by making mutability visible at every use site and by disambiguating user-named identifiers from the dense vocabulary of Latin keywords and operator words.

This table is the **target contract for Stage 2**. Stage 1 only honors the verb-mood rows (subjunctive, imperative, indicative); the case rows are documentation of what Stage 2 will enforce. The specific keyword vocabulary lives separately in `docs/lexicon.md` (TBD), not in this PRD.

### 4.3 Whole-Language Re-Skin

Keywords, builtin types, stdlib names, error messages, and tool output are all Latin. A canonical translation table is the source of truth for compiler and docs alike. Sketch:

| Rust            | Marain (proposed) |
| --------------- | ----------------- |
| `let x = 5`     | `sit ^x est 5`          |
| `let mut x = 5` | `sit @x est 5`          |
| `fn`            | `functio`         |
| `struct`        | `structura`       |
| `enum`          | `enumeratio`      |
| `impl`          | `praestatio`      |
| `trait`         | `proprietas`      |
| `mod`           | `modulus`         |
| `String`        | `Sermo`           |
| `Vec<T>`        | `Agmen<T>`        |
| `HashMap<K, V>` | `Vocabularium<K, V>` |
| `Option<T>`     | `Fortasse<T>`     |
| `Result<T, E>`  | `Eventus<T, E>`   |
| `println!`      | `dic!`            |
| `panic!`        | `DETONATIO!`      |

Full vocabulary will be enumerated in `docs/lexicon.md`. The PRD does not commit to any particular word — Section 4.2 commits only to the **system** (cases and moods carry role).

**Numeric literals stay Arabic.** `0`–`9` digits, the usual `_` separators, `0x` / `0o` / `0b` prefixes, decimal points, and exponent notation are all preserved as-is from Rust. Latin number-words (`quinque`, `decem`, …) are not legal numeric literals. Roman-numeral support is a separate, opt-in question — see S1-6.

### 4.4 Operators and Connectives

Marain expresses all of Rust's operators as Latin function words. Symbol-form operators (`=`, `==`, `+`, `&&`, etc.) are **not** legal Marain source. Specific word choices below are PROPOSED — final forms live in `docs/lexicon.md`.

| Rust             | Marain (proposed)   | Latin role                                |
| ---------------- | ------------------- | ----------------------------------------- |
| `=` (init)       | `est`               | indicative copula ("is")                  |
| `=` (reassign)   | `fit`               | indicative present ("becomes")            |
| `==`             | `aequat`            | 3rd-sg. present indicative active of `aequare`, "equals" |
| `!=`             | `non aequat`        | negated equality                          |
| `<`              | `minor quam`        | adjective + "than"                        |
| `<=`             | `minor vel par`     | "less or equal"                           |
| `>`              | `maior quam`        | adjective + "than"                        |
| `>=`             | `maior vel par`     | "greater or equal"                        |
| `+`              | `plus`              | adverb                                    |
| `-`              | `minus`             | adverb                                    |
| `*`              | `per`               | preposition ("by")                        |
| `/`              | `divisus per`       | passive participle + preposition          |
| `%`              | `modulo`            | technical / ablative of `modulus`         |
| `&&`             | `et`                | conjunction                               |
| `\|\|`           | `vel`               | conjunction (inclusive)                   |
| `!`              | `non`               | adverb (negation)                         |
| `true` / `false` | `verum` / `falsum`  | —                                         |

**Multi-word operators.** `maior quam`, `divisus per`, `non aequat`, `minor vel par`, `maior vel par` are recognized at the parser level via a fixed phrase table; the lexer emits one token per word. The component words (`quam`, `vel`, `par`, `divisus`, `per`, `non`, `aequat`, …) are reserved keywords and may not be used as identifiers. (`aequat` is also a standalone keyword — the equality operator — not only a phrase component.)

**Precedence and associativity inherit from Rust verbatim.** `a plus b per c` parses as `a plus (b per c)`. The author memorizes Rust precedence anyway; Marain borrows the same table.

**Assignment vs. equality is disambiguated by verb choice, not by token shape.** `sit ^x est 5` initializes, `^x fit 5` reassigns, `si ^x aequat 5` compares. The `=` / `==` confusion class disappears — at the cost of memorizing which Latin verb carries which sense.

### 4.5 Identifier Sigils

Every variable reference in Marain carries a single-character sigil that marks mutability. This is a Marain-original convention, not Latin grammar.

| Sigil | Meaning   | Example |
| ----- | --------- | ------- |
| `@`   | mutable   | `@x`    |
| `^`   | immutable | `^x`    |

Two payoffs justify the verbosity:

1. **Mutability is visible at every use site.** Readers do not scan back to the declaration. The `@` sigil is visually loud by design; `^` is quiet.
2. **Variables are unambiguously distinct from Latin keywords and operator words.** With operators expressed as Latin function words (§4.4), a bare `bonum` is ambiguous (variable? adjective?); `^bonum` is plainly a variable.

**Where sigils appear (everywhere a variable name does):** declarations, expression references, function parameters, function call arguments, struct field names, pattern destructuring, method receivers, and the self-equivalent `ego` (`^ego`, `@ego`). Omitting a sigil on any variable reference is a parse error. The `mutabilis` keyword falls out — `@` IS the mutability marker.

**Per-field struct mutability is decorative in v0.1.** Every struct field accepts its own sigil for visual symmetry (`structura Puella { ^nomen: Sermo, @aetas: Numerus }`), but Rust's "the whole struct binding is mutable or not" model is what actually transpiles. Genuine per-field mutability semantics are deferred — see S1-10.

**Borrows replace `&` and `&mut`.** Marain uses the keyword `tenet` ("holds", indicative 3rd-sg. of `teneo`) plus the borrowed variable carrying its own sigil:

| Rust         | Marain        |
| ------------ | ------------- |
| `&x`         | `tenet ^x`    |
| `&mut x`     | `tenet @x`    |
| `&self`      | `tenet ^ego`  |
| `&mut self`  | `tenet @ego`  |

Lifetime annotations and reference-of-reference syntax are deferred past v0.1.

### 4.6 Python-Inspired Niceties

- **Indentation-significant blocks.** No braces. The lexer emits synthetic `INDENT`/`DEDENT` tokens. Spaces only; mixed tabs/spaces in one file is a hard lex error.
- **Triple-quoted multiline strings.** `"""…"""` preserves embedded newlines and the layout the author wrote.
- **Dynamic value wrapper.** A `Variabile` tagged enum: `Numerus | Decimalis | Sermo | Bool | Nihil | Index(Vec<Variabile>) | Vocabularium(HashMap<String, Variabile>)`. Transpiles to a hand-rolled Rust enum (vendored, not a dependency). Lets the author write loose, JSON-shaped data without ceremony.
- **Concise literals + f-strings.** `{clavis: valor}`, `[unum, duo]`, `(x, y)`, `f"salve {nomen}"` — surface sugar over `HashMap`, `Vec`, tuples, and `format!`.

### 4.7 Macro Call Syntax

Marain splits macros into two classes.

**Punctuation-free macros (no `!`, no parens).** A small reserved subset of common single-argument macros may be called as bare words followed by one expression argument. The argument is a single expression terminated by the end of the statement; multi-value cases are handled by f-strings.

| Rust         | Marain (proposed) | Notes                                  |
| ------------ | ----------------- | -------------------------------------- |
| `println!`   | `dic`             | stdout                                 |
| `eprintln!`  | `queror`          | stderr ("I complain")                  |
| `vec![…]`    | `agmen […]`       | sequence/list builder                  |
| `format!(…)` | `forma …`         | returns a `Sermo`                      |

These names are **reserved keywords**. They cannot be redefined, shadowed, or used as function / struct / enum / module names; variable references cannot collide because variables always carry a sigil (`@`/`^`). The subset is intentionally tiny in v0.1; growing it requires explicit PRD revision.

**Punctuation-bearing macros (`!` preserved).** Everything else — `dbg!`, `assert!`, `assert_eq!`, the out-of-phase `DETONATIO!`, all multi-argument macros, and any user-defined macros — uses Rust's shape: `name!(arg1, arg2, …)`. The `!` is the reader's signal that the call is a macro and may behave unlike a function.

**Disambiguation rules:**

- A bare word at statement or expression position is one of: a keyword, a function name, or a no-punct macro. It is never a variable reference (variables always carry a sigil).
- Function calls always carry parentheses (`functio_mea(@x)`). No-punct macros never do (`dic ^x`). The lexer dispatches on the keyword table; the parser dispatches on parens.
- The no-punct form is strictly single-argument. Multi-argument needs the `!`+parens form.

**Cost acknowledged.** Stripping `!` from `dic`, `agmen`, `forma`, `queror` removes the visual "this is compile-time magic, beware" cue Rust's `!` carries. The mitigation is that the subset is small, fixed, and the chosen words are immediately learnable. Users opting into custom macros never lose the `!`.

### 4.8 Statement Terminator

Every statement ends with a period `.`, followed by either a newline or a space. The period is the canonical terminator; the following whitespace is non-semantic.

```
sit ^x est 5.
dic ^x.

sit ^y est 7. dic ^y.       // two statements on one line, separated by `. `

sit ^z est computa(
    @a,
    @b,
).                           // multi-line expression: no period mid-expression, period at the end
```

This is intentionally **not** Python-style newline-termination. The period is the unambiguous statement boundary; newlines are layout. Two consequences:

- Multi-line expressions need no continuation marker (no backslash, no implicit-paren rule). The lexer keeps reading until it sees a period at top expression-nesting depth.
- Indentation (§4.6) still defines *block* membership, but no longer carries *statement* boundaries. The two concerns are orthogonal.

**Control-structure heads end with a colon `:`.** `si <cond> :`, `dum <cond> :`, `pro <binding> :`, `semper :`, and `functio <sig> :` all use the same single-character block introducer; the body is the indented block that follows. Resolved 2026-05-25 (closing S1-2's leftover); see §4.11 for the v0.2 syntax.

### 4.9 Identifier Lexical Rules

**ASCII-only identifiers.** Diacritics (macrons, breves, any non-ASCII marks) are forbidden in Marain source, including in source examples within documentation. Macrons exist to mark vowel length for spoken use; that buys nothing in a programming-language wrapper. Latin prose in docs (descriptive paragraphs, citations) may use macrons freely; *fenced source samples* must not. Rationale: keeps the v0.1 lexer ASCII-only, avoids Unicode-normalization questions, sidesteps spelling drift between `Sermō` and `Sermo`.

**Case style follows Rust.** Multi-word identifiers use `snake_case` for functions, methods, variables, fields, and modules; `PascalCase` for types (structs, enums, traits); `SCREAMING_SNAKE_CASE` for constants. This is enforced at the lexer or early-pass level (mismatch is a hard error, not a lint), keeping the language opinionated and the source predictable. `DETONATIO!` (§4.2) honors the const-style convention by coincidence and remains the documented all-caps exception for the panic macro.

### 4.10 Compilation Stages

Marain's path to executable is three stages, with the middle stage optional.

**Stage 1 — Nominative source (`.lat`).** Default and the only stage required to ship a working program. Source uses nominative-only identifiers, verb-mood differentiation (subjunctive bindings, imperative macros, indicative statements), Rust-fixed word order, and all the operator / sigil / macro / terminator rules of §§4.4–4.9. Stage 1 is intentionally the least-expressive surface — it sidesteps the full lexicon and parser cost of Latin grammar while preserving the Latin reading experience. **v0.1 ships Stage 1 only.**

**Stage 2 — Latin-rich layer (`.lat` + sidecar `.latin`).** Optional, post-v0.1. The user (or the LSP, on their behalf) writes richer Latin: identifiers declined per their role, verbs conjugated across tense and person, and creative word orderings enabled by case marking. The Marain tool generates and maintains a sidecar **`.latin`** file alongside each `.lat` source. The sidecar is **tool-managed, not user-edited** — it records the per-file grammar usage map: which declension forms and conjugations are in play, which orderings are intentional. The lexer consults the sidecar to narrow its load per file; only the grammar features that file actually uses are activated, keeping the lexer cost proportional to the user's Latin ambition.

Free word order is fully enabled in Stage 2. The parser uses case markings (not token position) to assemble meaning: `dic textum` and `textum dic` lower to the same AST. Ambiguity — e.g., two nominatives in one statement where the grammar permits multiple subjects — is a hard parse error with a Latin-grammar diagnostic. The Stage 2 parser is significantly more complex than Stage 1's (likely case-driven token reassembly rather than pure recursive descent); see §10.

The **LSP is the primary pedagogical surface** for Stage 2 and is no longer a non-goal (see §8). When the user is in Stage 1, the LSP suggests Stage 2 alternatives ("you could express this in accusative-fronted order: `textum dic`"). In Stage 2, the LSP validates inflection, offers conjugation/declension help, and curates the sidecar. Stage 2 without LSP support is technically usable but pedagogically thin — most of the Latin-learning value of Marain lives in the LSP-driven interactive surface.

**Editor targets and extension model.** The Stage 2 LSP stays entirely inside the Rust ecosystem; the project deliberately avoids any JavaScript / Node dependency in build, packaging, or extension surface (§9). Editor targets, in roadmap order: **Zed**, **Lapce**, and **Helix**. Zed and Lapce share a WebAssembly-based extension model, which is the planned packaging surface for Marain's per-editor integration; Helix consumes the LSP via its native LSP client and needs no project-specific extension code. The LSP itself is a standalone Rust binary speaking JSON-RPC over stdio (the standard LSP transport), so any future spec-compliant editor can attach without project work. Other editors (VS Code, JetBrains family, Sublime, …) are explicitly not roadmap items; community ports are welcome but unsupported.

**Suggestion engines (deterministic + cognitive).** The Stage 2 LSP is expected to host two suggestion layers. The **deterministic** layer is driven directly off the Latin grammar spec: always-correct declension and conjugation alternatives, grammar-violation diagnostics, completion of inflected forms. It is required and ships first. The **cognitive / LLM** layer is a longer-horizon addition: it proposes stylistic and pedagogically-shaped reformulations ("how a Silver-Age author might phrase this clause") that cannot be derived mechanically from grammar rules alone. The deterministic core must be solid before LLM-layer work begins, and the LLM layer must remain optional and clearly visually distinguished from deterministic suggestions in editor UX (see S2-7).

**Stage 3 — Rust target.** Stages 1 and 2 both lower into a common simplified AST. The transpiler emits Rust source from this AST and invokes `cargo`/`rustc` to produce the executable (see §5 for mechanics). Stage 3 is mechanical and stage-symmetric — the source's grammar richness has been fully resolved by the time it reaches Stage 3.

### 4.11 Control Flow & Functions (v0.2)

v0.2 extends Stage 1 with function declarations, conditional / loop constructs, indented blocks, and the supporting keywords. Seven new Marain keywords land — `dat`, `semper`, `interrumpe`, `continua`, `in`, `nihil`, `aliter` — plus the structural `:` block-head terminator (resolving S1-2's leftover, see §4.8) and the `..` / `..=` range punctuation borrowed from Rust.

#### 4.11.1 Function declarations

```
functio <nomen>(<sigiled-name>: <Tipus>, ...) dat <ReturnTipus> :
    <body statements>
```

- `functio` introduces the declaration (reserved in v0.1; PRD §4.2).
- Parameters use Rust-style parens around comma-separated `<sigiled-name>: <Tipus>` pairs. Stage 1 uses nominative inflection per "first to define is followed" (§4.2); Stage 2 will activate dative.
- `dat` ("gives", 3rd-sg. present indicative active of `dare`) is the return-type indicator — single-word keyword, parallel verb form with `est` / `fit` / `aequat`. Omit the `dat <Tipus>` clause for unit return.
- Head terminates with `:` (§4.8); body is the indented block that follows.
- `redde <expr>.` returns a value.

Examples:

```
functio saluta() :
    dic "salve, munde".

functio echo(^x: Sermo) dat Sermo :
    redde ^x.
```

#### 4.11.2 Conditional and loop heads

| Construct | Marain | Rust |
| --------- | ------ | ---- |
| if | `si <cond> :` | `if cond { ... }` |
| else | `aliter :` | `else { ... }` |
| else if | `aliter si <cond> :` | `else if cond { ... }` |
| while | `dum <cond> :` | `while cond { ... }` |
| for | `pro <sigiled-binding> in <iterable> :` | `for x in iter { ... }` |
| infinite loop | `semper :` | `loop { ... }` |

- `si`, `dum`, `pro` reserved in v0.1 (PRD §4.2); v0.2 wires their parser support.
- `semper` ("always") — new keyword. Adverb, not a verb — sanctioned exception to §4.2's verb-mood pattern (no Latin verb cleanly maps to "loop forever"; `dum verum :` was the alternative considered and rejected as wordy).
- `in` — new keyword. Latin `in` and English/Rust `in` converge.
- `aliter` ("otherwise") — new keyword. Adverb, sanctioned exception to §4.2's verb-mood pattern (parallel justification to `semper`: Latin has no verb mapping cleanly to "alternative branch"). Used standalone as `aliter :` (`else`) or chained as `aliter si <cond> :` (`else if`); the parser recognizes the two-token sequence and lowers to Rust's `else if`.

#### 4.11.3 Control transfer

- `interrumpe.` — `break` (imperative 2nd-sg. of `interrumpere`). New keyword.
- `continua.` — `continue` (imperative 2nd-sg. of `continuare`). New keyword.

Both are statements; both terminate with `.` per §4.8. (§4.2's imperative-mood example row was amended at the same time — `interrumpe!` corrected to `interrumpe`, since `break` is a keyword in Rust and the `!` was vestigial.)

#### 4.11.4 Empty block

- `nihil.` — Python's `pass` in Latin. New keyword; one statement that does nothing. Used where a block must be non-empty syntactically but no behavior is wanted.

```
functio stub() :
    nihil.

dum verum :
    nihil.
```

#### 4.11.5 Range syntax

Range literals borrow Rust's `..` and `..=` punctuation verbatim:

```
pro ^i in 0..10 :
    dic ^i.

pro ^i in 0..=10 :
    dic ^i.
```

Latinizing the range form (`ab 0 ad 10`, `0 ad 10`, `0 usque ad 10`) was considered and rejected for v0.2: the Rust shape stays compact across all six range variants (`a..b`, `a..=b`, `..b`, `..=b`, `a..`, `..`) while the Latin phrasing would balloon. The alternatives are documented in `docs/core-lexicon.md` for future revisits.

#### 4.11.6 Out of v0.2 scope (deferred to v0.3+)

- **Closures** (`|x| body`, `move |x|`) — capture rules (Fn / FnMut / FnOnce, by-ref vs by-move) deserve a separate decision round.
- **Generics** (`<T>`, bounds, lifetimes, const generics) — large surface; defer alongside the type-system layer.
- **Visibility** (`pub` → PROPOSED `publicus`) — no module boundaries in v0.2 to gate against.

## 5. Execution Model

- **Pipeline (Stage 1, v0.1).** `.lat` source → tokens → nominative-only AST → Rust source → `cargo run` on a generated shim crate → executable.
- **Pipeline (Stage 2, post-v0.1).** `.lat` source + sidecar `.latin` → enriched tokens (case/conjugation-aware) → grammar-validated AST (with free-word-order reassembly) → same nominative-AST lowering → Rust emission → executable. The sidecar regenerates on every Stage 2 build; drift between `.lat` and `.latin` is detected via a content hash and triggers automatic rebuild of the sidecar.
- **No runtime of our own** beyond a small vendored support module (the `Variabile` enum and a handful of helpers), inlined into transpiled output so the result is a self-contained Rust project.
- **Marain source spans propagate** into emitted Rust as comments where useful, so hand-inspecting the output remains tractable.
- **Mapping rustc errors back onto Marain source spans is deferred** past v0.1. v0.1 lets `cargo`'s output through verbatim; this is a known UX wart and is acknowledged in README.

## 6. CLI

The `marain` binary exposes (v0.1 subset in **bold**):

- **`marain build <file.lat>`** — transpile to a sibling generated Rust project.
- **`marain run <file.lat>`** — transpile, then invoke `cargo run` on the generated project, forwarding stdout/stderr.
- `marain check <file.lat>` — lex + parse + name-resolve without invoking rustc. (Post-v0.1.)
- `marain fmt`, `marain repl` — not in this PRD.

## 7. v0.1 Scope — "Salve, Munde"

**v0.1 ships Stage 1 only** (§4.10). All of Stage 2 — full case/conjugation grammar, free word order, sidecar `.latin`, LSP — is post-v0.1 roadmap and stays out of v0.1's footprint, dependency budget, and code paths.

The v0.1 done line:

> The author writes `hello.lat` containing the single statement `dic "salve, munde".` — runs `marain run hello.lat` — and sees `salve, munde` on stdout.

Minimum required to ship v0.1:

- Lexer that recognizes the small set of Latin tokens used in hello-world (keywords + macro-call + string literal + minimal punctuation).
- Parser that accepts a single top-level macro invocation.
- AST → Rust emitter for that one shape.
- `marain build` and `marain run` subcommands.
- Generated cargo-shim crate.
- One end-to-end integration test that compiles + runs hello-world and asserts on stdout.

Explicitly **not** required for v0.1: full declension enforcement, indentation blocks, dynamic `Variabile`, Python literals, triple-quoted strings, f-strings, structs, enums, control flow, functions beyond the implicit `main`. Each is deferred until after v0.1 ships and is re-planned.

## 8. Non-Goals (for v0.1 and likely beyond)

**Removed from non-goals:** Editor / LSP integration is now *essential to Stage 2* (§4.10) and a first-class roadmap item, not a non-goal.



- `unsafe` blocks expressible in Marain. (Author can drop to raw Rust if needed; Marain source stays safe-only.)
- `async` / `await`.
- FFI / `extern` blocks.
- Procedural macros authored in Marain.
- Mapping rustc errors back onto Marain spans (revisit post-v0.5).
- Self-hosting (Marain compiler written in Marain).
- `crates.io` publishing.

## 9. Constraints

- **Rust stable channel.** No nightly features in compiler or generated code.
- **Rust-only ecosystem footprint.** No JavaScript / Node / npm in the build pipeline, the LSP, or any editor-extension surface. WebAssembly (where used for Zed / Lapce extensions, §4.10) is produced via the Rust toolchain's `wasm32-unknown-unknown` target — never via a Node-based build chain.
- **Self-supporting (per CLAUDE.md).** Lexer and parser use no external crates unless a strong case is documented. No `logos`, no `chumsky`. **CLI uses `clap`** (amended 2026-05-23 from the original "no `clap`" rule per author preference) — mature, full-featured, and the right level for the project's CLI workflow; pinned per the N-1 / 30-day rule below.
- **N-1 / 30-day rule** for any dependency we do add; pin with verification hashes.
- **500 LOC per file ceiling.**
- **Edition 2024.**

## 10. Risks

| Risk                                                                                                              | Likelihood | Mitigation                                                                                                       |
| ----------------------------------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------- |
| Strict-declension design (Stage 2 full case enforcement) proves too rigid for some Rust constructs (generics, closures, patterns, for-loops, tuple destructuring) | High (Stage 2 only) | Staging (§4.10) defers this entirely past v0.1. Stage 1 sidesteps it via nominative-only. Stage 2 spec must enumerate case mappings for each Rust construct before activation; iterate in small chunks per the user's plan |
| Stage 2 free-word-order parser is significantly more complex than Stage 1 (likely case-driven token reassembly rather than recursive descent)                     | High (Stage 2) | Ship Stage 1 first. Defer Stage 2 parser architecture to v0.2+ planning. Stage 2 may iterate through several parser prototypes before settling                                                                                |
| Sidecar `.latin` drifts out of sync with `.lat` source                                                            | Medium (Stage 2) | Content-hash the `.lat` source into the sidecar header; mismatch triggers automatic regeneration. Document the sidecar as tool-managed; do not surface it as user-editable in tooling                                       |
| LSP scope-creep now that LSP is essential to Stage 2 pedagogy (§4.10) and carries both a deterministic and (eventually) an LLM-driven suggestion layer | Medium-High (post-v0.1) | Treat the LSP as its own milestone with its own scope gate. Stage 2 syntax must remain hand-writable without LSP support; the LSP makes it pleasant, not possible. The deterministic layer ships first and must be solid before LLM work begins; LLM cost / latency / hallucination risks and editor-UX confusion risks tracked under S2-7 |
| Tying the editor extension model to Zed / Lapce / Helix locks the LSP UX to a Rust-ecosystem niche; broader adoption (VS Code, JetBrains) would require either a JS-bridge (violates §9 Rust-only footprint) or an unscoped editor-extension rewrite | Low-Medium (post-v0.1) | Accepted trade for §9's no-JavaScript constraint and for staying inside the author's editor preferences. Helix's native LSP client gives a zero-extension fallback. Community ports to other editors are welcome but unsupported (§4.10) |
| Whole-language re-skin demands a sprawling lexicon (every stdlib item)                                            | High       | Grow the lexicon program-by-program. v0.1's vocabulary is whatever hello-world needs and nothing more            |
| Indentation + Python literals + declensions in one lexer is non-trivial                                           | Medium (mitigated for v0.1)     | Lexer decomposed into 8 files under `lexer/` (cursor / indent / strings / numbers / idents / keywords / error / mod), one concern per file — `ARCHITECTURE.md` §6. Indentation + sigils + Latin keywords landed in Round 4 under the 500-LOC target with no pressure-release invoked. Declensions defer to Stage 2 |
| Without rustc-error span mapping, errors confuse non-author users                                                 | Medium     | Limited blast radius (small audience); README calls out the limitation; revisit post-v0.5                        |
| Scope creep from dual goals (Latin learning may pull for breadth; Rust learning for depth)                        | Medium     | Re-plan after each milestone; one feature per session; refer back to Goals in Section 2 when arbitrating         |
| Author loses interest before reaching v1                                                                          | Medium     | v0.1 is intentionally tiny so the project produces a working artifact within a small number of sessions          |
| Latin scholarship errors in the proposed lexicon embarrass the project                                            | Low-Medium | Treat `docs/lexicon.md` as living; accept corrections; cite a single dictionary (e.g., Lewis & Short) as arbiter |
| Arithmetic-heavy code reads heavily (`2 plus 3 per 4`) once operators are words                                   | Medium     | Accepted trade for the prose-Latin aesthetic; README examples calibrate expectations                             |
| Reserved-keyword footprint grows large (every operator word) and may shadow natural Latin identifiers              | Medium     | `docs/lexicon.md` tracks reserved words; identifiers that collide must pick a synonym                            |
| Sigils on every variable reference (`@x`, `^y`) increase visual density across all code                            | Low        | Accepted trade for visible mutability + variable/keyword disambiguation; calibrate via README examples           |
| Decorative per-field struct sigils suggest semantic capability Rust does not natively support                       | Medium     | Document the gap in §4.5; treat S1-10 as load-bearing before any code relies on per-field mutability  |

## 11. Open Questions (Stage-1 must close before drafting `ARCHITECTURE.md`)

### Stage 1 open questions

S1-1. ~~**Section 4.2 mapping**~~ — **RESOLVED.** Approved with three amendments: ablative narrowed to trait bounds + lifetimes; dative narrowed to parameters + reassignment targets, governed by the "first to define is followed" rule (§4.2); vocative row removed; `DETONATIO!` added as the sanctioned out-of-phase exception.

S1-2. ~~**Hello-world canonical form**~~ — **RESOLVED.** Macro-`!` split per §4.7 (small no-punct subset, `!` preserved otherwise). Statement terminator per §4.8 (period followed by newline or space). Control-structure-head terminator resolved 2026-05-25: `:` (Python-style); see §4.8 and §4.11.

S1-3. ~~**Diacritics in identifiers.**~~ **RESOLVED** (§4.9). Forbidden in source and in source examples in docs; descriptive Latin prose in docs may use macrons.

S1-4. ~~**Identifier case style.**~~ **RESOLVED** (§4.9). Rust-style: `snake_case` for fns/vars/fields/modules, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for consts.

S1-5. **Verb voice.** Does Latin's active/passive distinction encode anything in Marain? (E.g., consumed-by-move vs. borrowed.) **Deferred to Stage 2** (§4.10) — Stage 1 verb forms are fixed keywords with no voice variation.

S1-6. **Roman-numeral literals.** Arabic numerals are the default and always legal (per §4.3). Should Roman numerals additionally be accepted as numeric literals, presumably opt-in via a prefix (e.g., `r"XIV"`)? Tempting flourish; not required.

S1-7. **Lexicon governance.** Single author-maintained `docs/lexicon.md` is assumed. Confirm.

S1-8. **Multi-word operator phrase table.** §4.4 commits to parser-level phrase recognition for `maior quam`, `divisus per`, etc., with the component words reserved as keywords. Confirm, or relax to context-sensitive parsing later?

S1-9. **Boolean literal forms.** §4.4 proposes `verum` / `falsum`. Accept, or substitute alternates?

S1-10. **Per-field struct mutability.** §4.5 lets every struct field carry its own sigil decoratively. Later, promote to real per-field mutability semantics (a Marain-original feature) or keep documentation-only and inherit Rust's struct-level model permanently?

S1-11. **`self` keyword form.** §4.5 uses `ego` (nominative "I"). Accept, or substitute (`me`, `mihi`, …)?

S1-12. **Borrow syntax details.** §4.5 uses indicative `tenet` ("holds"). (a) Accept, or prefer imperative `tene` ("hold!") in parallel with action macros (`dic!`, `pone!`)? (b) Within `tenet`, does the sigil describe the borrow's mutability (allowing `tenet ^x` to immutably borrow a mutable variable) or must it match `x`'s declaration?

### Stage 2 / LSP open questions (post-v0.1)

S2-1. **Sidecar `.latin` file format.** Plain text? JSON? S-expression? Custom binary? Needs to be diff-friendly for VCS and tool-regenerable.

S2-2. **Stage 1 ↔ Stage 2 module interop.** Can a Stage 1 file `modulus`-import a Stage 2 file, and vice versa? Likely yes since both lower to a common AST, but signature-level details (do declined parameter forms appear in cross-stage signatures?) need spec.

S2-3. **Stage 2 parser architecture.** Recursive descent + backtracking, GLR, Earley, constraint propagation, or something else? Free word order with case-driven role resolution is the parser's hardest job.

S2-4. **LSP transport and editor packaging.** **Partial answer (§4.10):** standalone LSP server speaking JSON-RPC over stdio; editor targets in roadmap order Zed → Lapce (both via the shared WebAssembly extension model) → Helix (native LSP client, no extension code). **Still open:** minimum viable Stage-2 pedagogy feature set for the first LSP cut — which of {hover-explains-inflection, code-action-decline-this-identifier, diagnostic-on-grammar-violation, completion of inflected forms, sidecar-curation surface} are v0.2 vs v0.3?

S2-5. **Stage 2 ambiguity resolution.** When the case grammar admits multiple parses (e.g., two accusatives in one statement), is the resolution "first wins by source order," "hard error," or "user-disambiguates via explicit Latin prepositions"?

S2-6. **Migration UX.** Tooling for "upgrade this Stage 1 file to Stage 2" — does it auto-decline identifiers, or surface declensions to the user for confirmation? Tied to LSP scope (S2-4).

S2-7. **Hybrid suggestion architecture (deterministic + LLM).** §4.10 commits the Stage 2 LSP to two suggestion engines: a **deterministic** layer grounded in the Latin grammar spec (always-correct alternatives, validations, completions) shipping first, and a longer-horizon **cognitive / LLM** layer that proposes stylistic and pedagogically-shaped reformulations not derivable from grammar rules alone. Open sub-questions: (a) LLM hosting model — local inference (llama.cpp / Candle / Ollama, Rust-friendly), hosted API (Anthropic / OpenAI / etc.), or BYOL? (b) how does the LLM layer surface in editor UX without being confused with deterministic / grammar-correct suggestions? (c) latency, cost, and hallucination budget — what guardrails keep LLM output from polluting Stage 2 source with ungrammatical Latin? (d) does the LLM layer ship as a separate optional binary so the LSP core remains zero-dep on any model runtime, preserving Helix's no-extension-code path?

## 12. Success Criteria for This PRD

This PRD is ready to derive `ARCHITECTURE.md` from when:

1. ~~Section 4.2 (grammar mapping) is approved or amended.~~ **DONE** (S1-1).
2. ~~The hello-world canonical form is committed.~~ **DONE** (S1-2: §4.7 + §4.8).
3. ~~S1-3 and S1-4 are resolved (they affect lexer rules directly).~~ **DONE** (S1-3 + S1-4: §4.9).

**All §12 gates are met for Stage 1.** The PRD is ready to derive `ARCHITECTURE.md` for the v0.1 Stage-1 implementation.

Stage 2 has its own gating round before any `ARCHITECTURE.md` work for it: questions S2-1 through S2-7 in §11 must close, and the Stage 2 grammar specification must be drafted as a separate document.

Remaining Stage 1 open questions (S1-5 through S1-12, deferred) may be deferred into `ARCHITECTURE.md` or post-v0.1 planning.
