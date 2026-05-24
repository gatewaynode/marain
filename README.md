# Marain

A staged toy language that re-skins Rust syntax with Latin keywords. Source-to-source transpiler: `.lat` → tokens → AST → emitted Rust → `cargo` invocation → executable.

**Status: v0.1.** The hello-world done line works end-to-end. Stage 1 (nominative-only Latin, Rust-fixed word order) is the v0.1 surface; Stage 2 (full case grammar + free word order + LSP-driven Latin pedagogy) is post-v0.1 roadmap.

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

That's the whole v0.1 done line.

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

## The language at v0.1

Five productions in Stage 1:

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

A two-statement program:

```latin
sit ^answer est 42.
dic ^answer.
```

emits

```rust
fn main() {
    let answer = 42i64;
    println!("{}", answer);
}
```

Identifier sigils are mandatory on every variable reference — `@` marks mutable, `^` marks immutable. The mutability lives at the *declaration* (`sit @x` → `let mut x`); use sites discard the sigil.

A Marain identifier that collides with a Rust keyword gets `r#`-escaped at emit time. The five Rust keywords that have no raw-identifier escape (`crate`, `extern`, `self`, `Self`, `super`) surface as a Marain emit error pointing at the source span — no silent mangling.

## What's NOT in v0.1

Per PRD §7, intentionally tiny:

- No comments. Every byte the lexer doesn't recognize is a `LexError::UnexpectedChar`. (Yes, `// note` will fail — this is a known PRD gap.)
- No operators. `2 plus 3` doesn't parse yet; precedence / multi-word phrase table lands later.
- No control flow. No `si` / `dum` / `pro`; no indented blocks beyond what the lexer tracks internally.
- No functions beyond the implicit `main`. No `functio` declarations.
- No structs, enums, traits, modules.
- No `Variabile` runtime, no Python-inspired literals (dict / list / tuple / f-string), no triple-quoted strings.
- No Stage 2: no case grammar enforcement, no free word order, no `.latin` sidecar, no LSP.
- No `marain check` / `marain fmt` / `marain repl`.

Each is deferred until after v0.1 ships and is individually re-planned.

## Known v0.1 limitations

- **`rustc` errors pass through verbatim.** If the emitted Rust fails to compile (because you exercised a feature combination Marain emits invalid Rust for), you see raw rustc output with line/column referring to the *emitted* Rust, not your Marain source. Span back-mapping is deferred per PRD §5 (revisit post-v0.5).
- **Cold start.** First `marain run hello.lat` on a fresh shim has to compile `println!` from scratch — typically 1–3 seconds. Subsequent runs reuse the shim's `target/`.
- **No `marain install`.** v0.1 doesn't symlink user-built programs into `~/.local/bin/`; you'd run `marain run hello.lat` each time, or invoke the shim's executable directly from `~/.local/state/marain/builds/<name>-<hash>/target/debug/<name>`.

## Project layout

```
marain/
  Cargo.toml                     # workspace; resolver "3"
  rust-toolchain.toml            # pinned 1.94.1
  PRD.md                         # Product requirements (the *what*)
  ARCHITECTURE.md                # Implementation architecture (the *how*) — §§1–10 complete
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
- **[`ARCHITECTURE.md`](ARCHITECTURE.md)** — implementation architecture by section: crate layout, source / span model, error model, lexer, parser, AST, codegen, shim, CLI, testing harness. All sections complete for v0.1.
- **[`CLAUDE.md`](CLAUDE.md)** — project conventions and collaboration norms (also covers the 500-LOC file size policy, the N-1 / 30-day dependency rule, Rust idioms, security posture).

## Testing

```sh
cargo test --all
```

252 tests across three layers: per-phase unit tests in-source (`#[cfg(test)] mod tests`), fixture-walker goldens (`crates/marain-core/tests/{emit,error}_goldens.rs` + 13 paired fixtures), and behavioral end-to-end (`crates/marain-core/tests/e2e_hello_world.rs` for the library pipeline, `crates/marain-cli/tests/cli_e2e.rs` for the binary).

Golden fixtures are tripwires for unintended drift in emit shape — regenerate intentionally with:

```sh
MARAIN_UPDATE_GOLDENS=1 cargo test -p marain-core --test emit_goldens --test error_goldens
```

## License

MIT. See [`LICENSE`](LICENSE).
