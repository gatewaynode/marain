# Marain

A staged artisan language that re-skins Rust syntax with Latin keywords. Source-to-source transpiler: `.lat` → tokens → AST → emitted Rust → `cargo` invocation → executable.

**Status: v0.2 in progress.** v0.1 hello-world shipped; v0.2 has landed line comments (R9), indented blocks + `si` (R10), the full operator surface + `aliter`/`dum`/`semper`/`interrumpe`/`continua` (R11+R12), and `functio` declarations + `redde` returns + function calls (R13). Remaining v0.2 work: `pro` loops + range tokens + `nihil.` (R14+R15). Stage 1 (nominative-only Latin, Rust-fixed word order) is the v0.x surface; Stage 2 (full case grammar + free word order + LSP-driven Latin pedagogy) is post-v0.x roadmap.

Named after Marain, the language of the Culture in Iain M. Banks' novels. (Renamed from "Rubigo" on 2026-05-17.)

## Quick start

A program:

```latin
dic "salve, munde".
```

Save as `hello.lat`. Then:

```sh
marain run hello.lat
```

Output:

```
salve, munde
```

That's the v0.1 done line. v0.2 adds enough surface to write real programs — see [Examples](#examples) below.

## Building

```sh
git clone <repo> marain
cd marain
cargo build --release
# Binary lands at target/release/marain
```

Rust toolchain is pinned to **1.94.1 / edition 2024** via `rust-toolchain.toml`. The only dependency is `clap = "=4.5.61"` (CLI arg parsing); both crates carry `#![forbid(unsafe_code)]`.

## Commands

### `marain build <file.lat>`

Transpile and write a self-contained cargo project to `$XDG_STATE_HOME/marain/builds/<basename>-<8hex-hash>/`. Prints the shim path on stdout — `cd $(marain build hello.lat)` works.

```sh
$ marain build hello.lat
/Users/you/.local/state/marain/builds/hello-a96fbdd4
```

The hash is an FNV-1a 32-bit digest of the source's canonical absolute path. Two `hello.lat` files in different directories produce two distinct shims; the same source path always produces the same shim dir (so `marain build` is idempotent and `marain run` reuses the prior `target/`).

### `marain run <file.lat>`

Same as `build`, then invokes `cargo run` on the shim, inheriting stdio so cargo's progress and your program's output go to your terminal live. Exits with cargo's exit code.

```sh
$ marain run hello.lat
salve, munde
```

### `marain --help` / `marain --version`

Standard clap-generated output.

## Error reporting

Source-level errors render as `path:line:col: error: message` (compiler convention):

```sh
$ marain build bad.lat
bad.lat:1:1: error: unexpected character '?'
$ echo $?
1
```

Filesystem / process errors carry a `marain:` prefix so you can tell them apart from source-level diagnostics:

```sh
$ marain build nonexistent.lat
marain: error: failed to read nonexistent.lat: No such file or directory (os error 2)
```

Exit codes: `0` on success, `1` on any driver error (lex / parse / emit / I/O / shim / cargo failure), `2` on argument-parsing errors (clap convention).

## The language today

### Bindings, literals, macros (v0.1)

| Construct | Marain | Rust |
| --- | --- | --- |
| Immutable binding | `sit ^x est 5.` | `let x = 5i64;` |
| Mutable binding | `sit @x est 5.` | `let mut x = 5i64;` |
| stdout macro | `dic <expr>.` | `println!("{}", <expr>);` |
| stderr macro | `queror <expr>.` | `eprintln!("{}", <expr>);` |
| `Vec` literal | `agmen <expr>.` | `vec![<expr>];` |
| `format!` | `forma <expr>.` | `format!("{}", <expr>);` |
| Variable reference | `^x` / `@x` | `x` (sigil discarded at use site) |
| String literal | `"…"` with `\"` `\\` `\n` `\t` `\r` `\0` escapes | same |
| Integer literal | `42`, `1_000_000` | `42i64`, `1000000i64` |

Identifier sigils are mandatory on every variable reference — `@` marks mutable, `^` marks immutable. The mutability lives at the *declaration* (`sit @x` → `let mut x`); use sites discard the sigil.

A Marain identifier that collides with a Rust keyword gets `r#`-escaped at emit time. The five Rust keywords that have no raw-identifier escape (`crate`, `extern`, `self`, `Self`, `super`) surface as a Marain emit error pointing at the source span — no silent mangling.

### Comments (R9)

| Construct | Marain | Notes |
| --- | --- | --- |
| Line comment | `// to end of line` | Consumed by the lexer; emits no token. Comment-only lines don't affect indentation. |
| Block comment | `/* … */` | **Reserved syntax** — rejected with a targeted "use `//`" diagnostic. v0.3+. |

### Operators (R11)

Boolean atoms `verum` / `falsum`, parens for grouping, and the Latin operator surface with Rust precedence:

| Class | Marain | Rust |
| --- | --- | --- |
| Logical or / and | `vel`, `et` | `\|\|`, `&&` |
| Equality | `aequat`, `non aequat` | `==`, `!=` |
| Comparison | `minor quam`, `maior quam`, `minor vel par`, `maior vel par` | `<`, `>`, `<=`, `>=` |
| Additive | `plus`, `minus` | `+`, `-` |
| Multiplicative | `per`, `divisus per`, `modulo` | `*`, `/`, `%` |
| Unary | `non` | `!` |

Multi-word phrases (`maior quam`, `divisus per`, `non aequat`, `minor vel par`, `maior vel par`) are recognized at the parser via greedy completion of a leading word — the lexer emits one token per word. Emitted Rust paren-wraps every binary and unary node, so the lowering is bulletproof against precedence drift.

### Control flow (R10, R12)

| Construct | Marain | Rust |
| --- | --- | --- |
| if | `si <cond> :` | `if cond { … }` |
| else | `aliter :` | `else { … }` |
| else if | `aliter si <cond> :` | `else if cond { … }` |
| while | `dum <cond> :` | `while cond { … }` |
| infinite loop | `semper :` | `loop { … }` |
| break | `interrumpe.` | `break;` |
| continue | `continua.` | `continue;` |

Block bodies are introduced with `:` and delimited by indentation (spaces only — tabs anywhere are a hard lex error). Period `.` terminates statements; `:` terminates block heads; the two are orthogonal.

### Functions (R13)

| Construct | Marain | Rust |
| --- | --- | --- |
| Function (unit return) | `functio <name>(<params>) :` | `fn <name>(<params>) { … }` |
| Function (typed return) | `functio <name>(<params>) dat <Type> :` | `fn <name>(<params>) -> <Type> { … }` |
| Parameter | `^x: Sermo`, `@y: Numerus` | `x: String`, `mut y: i64` |
| Return | `redde <expr>.` or `redde.` | `return <expr>;` or `return;` |
| Call (expression or statement) | `f(<args>)` / `f(<args>).` | `f(<args>)` / `f(<args>);` |

Type names follow PRD §4.9 PascalCase — `sermo` in a type position is a hard parse error pointing at the bad casing. `Sermo` / `Numerus` translate to `String` / `i64` at emit; any other PascalCase name passes through verbatim (so a future `structura Custom` works without an emitter table fork). Top-level `functio`s hoist above `fn main()`; everything else stays inside it.

## Examples

The full set lives in `crates/marain-core/tests/fixtures/`. A few representative pairs:

### Line comments — `09_line_comments.lat`

```latin
// preamble: line comments smoke-test
sit ^x est 42.
// standalone between statements
sit ^y est "hello".

dic ^x.         // trailing after dic
dic ^y.
// trailing standalone at end of file
```

```rust
fn main() {
    let x = 42i64;
    let y = "hello";
    println!("{}", x);
    println!("{}", y);
}
```

### `si` / `aliter si` / `aliter` chain — `15_aliter_chain.lat`

```latin
sit ^x est 2.
si ^x aequat 1 :
    dic "one".
aliter si ^x aequat 2 :
    dic "two".
aliter :
    dic "other".
```

```rust
fn main() {
    let x = 2i64;
    if (x == 1i64) {
        println!("{}", "one");
    } else if (x == 2i64) {
        println!("{}", "two");
    } else {
        println!("{}", "other");
    }
}
```

### Multi-parameter function — `20_functio_multi_param.lat`

```latin
functio add(^a: Numerus, ^b: Numerus) dat Numerus :
    redde ^a plus ^b.
sit ^sum est add(2, 3).
dic ^sum.
```

```rust
fn add(a: i64, b: i64) -> i64 {
    return (a + b);
}

fn main() {
    let sum = add(2i64, 3i64);
    println!("{}", sum);
}
```

### Type translation (pass-through for user types) — `22_functio_translation.lat`

```latin
functio greet(^name: Sermo, ^count: Numerus) dat Sermo :
    redde ^name.
functio identity(^x: Custom) dat Custom :
    redde ^x.
```

```rust
fn greet(name: String, count: i64) -> String {
    return name;
}

fn identity(x: Custom) -> Custom {
    return x;
}

fn main() {
}
```

`Custom` is not in the emit translation table — it passes through verbatim, and rustc adjudicates whether the type exists. The same machinery will absorb future `structura` / `enumeratio` declarations without an emit-table fork.

### A grammar-error diagnostic — `errors/11_generics_lookalike.lat`

```latin
functio f() dat Agmen<T> :
    dic "x".
```

```
11_generics_lookalike.lat:1:22: error: unexpected '<'; generics are deferred to v0.3+ (PRD §4.11.6) and comparisons use `minor quam` / `maior quam` (PRD §4.4)
```

The diagnostic catches a likely user intent (a generic-type annotation) at the lex layer with a targeted message, rather than the generic "unexpected character" path.

## What's NOT in yet

v0.2 deferred to R14+R15 (next round):

- `pro <binding> in <iter> :` for-loops.
- Range tokens `..` / `..=` and `Expr::Range`.
- `nihil.` — the empty-block / pass sentinel.

v0.3+ (post-v0.2):

- Generics — `<T>`, bounds, lifetimes. The `<` / `>` lexer arm currently fires `LexError::GenericsLookalike` against any attempt.
- Closures (`|x| body`), visibility (`publicus` → `pub`), block comments (`/* … */`).
- `structura` / `enumeratio` / `praestatio` / `proprietas` / `modulus` declarations.
- `Variabile` runtime, Python-inspired literals (dict / list / tuple / f-string), triple-quoted strings.
- `marain check` / `marain fmt` / `marain repl`.

Stage 2 (post-v0.x):

- Full case-grammar enforcement, free word order, `.latin` sidecar, LSP-driven Latin pedagogy.

Each is individually re-planned at its round; see `PRD.md` §4.11.6 and §11, and `tasks/TODO.md`.

## Known limitations

- **`rustc` errors pass through verbatim.** If the emitted Rust fails to compile (because you exercised a feature combination Marain emits invalid Rust for), you see raw rustc output with line/column referring to the *emitted* Rust, not your Marain source. Span back-mapping is deferred per PRD §5 (revisit post-v0.5).
- **Cold start.** First `marain run hello.lat` on a fresh shim has to compile `println!` from scratch — typically 1–3 seconds. Subsequent runs reuse the shim's `target/`.
- **No `marain install`.** The CLI doesn't yet symlink user-built programs into `~/.local/bin/`; you'd run `marain run hello.lat` each time, or invoke the shim's executable directly from `~/.local/state/marain/builds/<name>-<hash>/target/debug/<name>`.

## Project layout

```
marain/
  Cargo.toml                     # workspace; resolver "3"
  rust-toolchain.toml            # pinned 1.94.1
  PRD.md                         # Product requirements (the *what*)
  ARCHITECTURE.md                # Implementation architecture (the *how*) — §§1–15 complete
  README.md                      # this file
  crates/
    marain-core/                 # lexer + parser + AST + emit + shim (library)
    marain-cli/                  # the `marain` binary
  tasks/
    TODO.md                      # round tracker
    CONTINUITY.md                # cross-session notes
```

The compiler front-end is hand-rolled per PRD §9 self-supporting: no `logos`, no `chumsky`, no error-derive crates. The CLI uses `clap` (the one permitted dependency, added 2026-05-23 by PRD amendment).

## Documents

- **[`PRD.md`](PRD.md)** — full language design, including Stage 2 grammar plans and Stage 2 LSP roadmap.
- **[`ARCHITECTURE.md`](ARCHITECTURE.md)** — implementation architecture by section: crate layout, source / span model, error model, lexer, parser, AST, codegen, shim, CLI, testing harness (§§1–10 for v0.1), then v0.2 rounds: line comments (§12), block parsing + `si` (§13), operator expressions + control flow (§14), function declarations + calls (§15).
- **[`CLAUDE.md`](CLAUDE.md)** — project conventions and collaboration norms (also covers the 500-LOC file size policy, the N-1 / 30-day dependency rule, Rust idioms, security posture).

## Testing

```sh
cargo test --all
```

415 tests across three layers: per-phase unit tests in-source (`#[cfg(test)] mod tests` or sibling `_tests.rs` files where the test bloc dominates), fixture-walker goldens (`crates/marain-core/tests/{emit,error}_goldens.rs` + 22 emit + 13 error paired fixtures), and behavioral end-to-end (`crates/marain-core/tests/e2e_hello_world.rs` for the library pipeline, `crates/marain-cli/tests/cli_e2e.rs` for the binary).

Golden fixtures are tripwires for unintended drift in emit shape — regenerate intentionally with:

```sh
MARAIN_UPDATE_GOLDENS=1 cargo test -p marain-core --test emit_goldens --test error_goldens
```

## License

MIT. See [`LICENSE`](LICENSE).
