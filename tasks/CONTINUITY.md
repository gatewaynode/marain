# Continuity — Marain v0.1 Architecture Complete

_Rewritten 2026-05-23 at end-of-session, after R7 + R8 closed. Rewrite on next use._

## Where We Are

**All 8 architecture-design rounds are closed and disk-committed. v0.1 ships.** The full lex → parse → emit → shim → `cargo run` pipeline works end-to-end at the binary level: `marain run hello.lat` containing `dic "salve, munde".` prints `salve, munde` on stdout. The PRD §7 done line is contractually enforced by automated tests at four layers.

User has NOT yet manually exercised the binary beyond the smoke runs I executed at R7 + R8 close. Per user direction: "tomorrow we'll start manual testing and planning next steps." This compact preserves state for that session.

### Test counts at session close

| Binary | Tests |
| --- | --- |
| `marain-core` unit | 199 |
| `marain-core` integration (`e2e_hello_world`) | 1 |
| `marain-core` integration (`emit_goldens`) | 1 (aggregates 8 fixtures) |
| `marain-core` integration (`error_goldens`) | 1 (aggregates 5 fixtures) |
| `marain-cli` unit | 40 |
| `marain-cli` integration (`cli_e2e`) | 10 |
| **Total** | **252** |

`cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test --all` all clean. `#![forbid(unsafe_code)]` at both crate roots + workspace `[workspace.lints.rust] unsafe_code = "forbid"`.

## File State

### Code (final v0.1 layout)

```
marain/
  Cargo.toml                              # workspace; resolver "3"; [workspace.lints.rust] unsafe_code = "forbid"
  Cargo.lock                              # tracked (binary crate convention)
  rust-toolchain.toml                     # pinned channel = "1.94.1"
  .gitignore                              # /target + /.scratch
  CLAUDE.md                               # 500-LOC three-tier rule; collaboration norms
  AGENTS.md                               # symlink → CLAUDE.md
  PRD.md                                  # §9 amended 2026-05-23 to permit clap; §10 risk row marked "mitigated"
  ARCHITECTURE.md                         # §§1–10 all CLOSED; §11 forward hooks
  README.md                               # rewritten end of v0.1; quickstart + commands + limitations
  crates/
    marain-core/
      Cargo.toml
      src/
        lib.rs                            # #![forbid(unsafe_code)]; pub mod ast/emit/error/lexer/parser/shim/source/span/token
        span.rs                           # FileId(NonZeroU32), Span, join/len/is_empty
        source.rs                         # SourceFile + SourceMap registry
        error.rs                          # Severity, Diagnostic, MarainError { Lex, Parse, Emit }
        token.rs                          # Sigil, TokenKind (20 variants), Token, TokenKind: Display
        ast.rs                            # Module/Stmt/Expr; Ident/SigiledIdent w/ Option<Inflection>
        emit.rs                           # emit(&Module) -> Result<String, EmitError>; full Rust 2024 keyword escape
        shim.rs                           # render_cargo_toml + write_shim (atomic-write); ShimError standalone
        lexer/                            # 8-file decomposition (mod / cursor / indent / strings / numbers / idents / keywords / error)
        parser/                           # 3-file (mod / grammar / error)
      tests/
        e2e_hello_world.rs                # library-level smoke (PRD §7 done line via cargo run)
        emit_goldens.rs                   # fixture walker, MARAIN_UPDATE_GOLDENS=1 to regen
        error_goldens.rs                  # fixture walker for diagnostic rendering
        fixtures/
          01_hello_world.{lat,expected.rs}
          02_let_integer.{lat,expected.rs}
          03_let_string.{lat,expected.rs}
          04_let_mutable.{lat,expected.rs}
          05_let_then_print.{lat,expected.rs}
          06_all_macros.{lat,expected.rs}
          07_integer_separators.{lat,expected.rs}
          08_rust_keyword_escape.{lat,expected.rs}
          errors/
            01_unexpected_char.{lat,expected.txt}
            02_unterminated_string.{lat,expected.txt}
            03_missing_period.{lat,expected.txt}
            04_unescapable_keyword.{lat,expected.txt}
            05_no_sigil_in_binding.{lat,expected.txt}
    marain-cli/
      Cargo.toml                          # clap = "=4.5.61" features ["derive"]; marain-core path dep
      src/
        main.rs                           # #![forbid(unsafe_code)]; parse args + dispatch + exit
        args.rs                           # clap derive: Cli + Command { Build, Run }
        paths.rs                          # XDG hand-rolled + FNV-1a 8-hex + shim_dir_for + shim_name_for
        driver.rs                         # dispatch/build/run + private write_shim_from_source seam
        error.rs                          # DriverError + Display + Debug + Error + report()
      tests/
        cli_e2e.rs                        # 10 binary-level e2e via env!("CARGO_BIN_EXE_marain")
  tasks/
    TODO.md                               # all 8 rounds closed (in-flight + completed sections)
    LESSONS.md                            # still empty (no corrections worth recording this session)
    BUGS.md                               # still empty
    CONTINUITY.md                         # this file
    notes/                                # empty
  docs/                                   # empty dir; lexicon.md still not created
  data/                                   # empty
  tests/                                  # workspace-root tests dir, empty (per-crate tests are authoritative)
  .scratch/                               # gitignored; project-local test artifacts
```

**LOC budget posture:** every file under the 500-LOC target. Largest at v0.1 close: `paths.rs` 277, `driver.rs` 274, `error.rs` (cli) 271, `cli_e2e.rs` ~210. Pressure-release never invoked across the entire stack.

**Dependency footprint:** `clap = "=4.5.61"` (pinned exact per N-1 / 30-day rule); transitively pulls `clap_builder`, `clap_lex`, `anstream`, `anstyle*`, `colorchoice`, `strsim`, `utf8parse`, `is_terminal_polyfill`. Cargo.lock holds verification hashes. No other dependencies in either crate.

## Round Closure Index (all 8 closed)

| # | Section | Status | Key code artifact |
| - | ------- | ------ | ----------------- |
| 1 | §2 Crate Layout, §3 On-Disk Paths | **Closed** 2026-05-22 | workspace + 2 crates + rust-toolchain.toml |
| 2 | §4 Source & Span Model | **Closed** 2026-05-22 | span.rs + source.rs |
| 3 | §5 Error Model | **Closed** 2026-05-22 | error.rs |
| 4 | §6 Lexer | **Closed** 2026-05-23 | token.rs + 8-file lexer/ |
| 5 | §7 Parser & AST | **Closed** 2026-05-23 | ast.rs + 3-file parser/ |
| 6 | §8 Codegen & Cargo Shim | **Closed** 2026-05-23 | emit.rs + shim.rs + e2e test |
| 7 | §9 CLI & Driver | **Closed** 2026-05-23 | 5-file marain-cli/src/ + clap = "=4.5.61" |
| 8 | §10 Testing Harness | **Closed** 2026-05-23 | 3 test files + 13 fixtures + UPDATE_GOLDENS |

## What's Next (tomorrow's session entry point)

User-stated: "manual testing and planning next steps." Most likely shape:

1. **Manual exploration.** User writes their own `.lat` files outside the test harness — `marain build`, `marain run`, intentionally break things, see what the diagnostics look like in real use. Surface UX papercuts I won't have seen.
2. **Triage the papercuts.** Likely candidates:
   - Comment syntax (concern η — currently any unrecognized byte → `LexError::UnexpectedChar`; user will hit this the moment they instinctively type `// note`).
   - Help text terseness — `marain --help` is clap-default; subcommand help may need polish.
   - Cargo error pass-through is verbatim (PRD §5 deferred): when `marain run` hits a rustc-side error from the emitted Rust, the user sees raw rustc output with no mapping back to Marain spans. This is the documented v0.1 wart.
   - Path collisions if two `.lat` files have the same canonical path content (shouldn't happen — symlinks resolved by `canonicalize`).
   - First-build cold time (cargo pulls down stdlib for a fresh shim). Per-shim `target/` reuse helps after the first run.
3. **v0.2 scoping.** Open candidates (no priority order yet):
   - **Comment syntax** (concern η, PRD gap). One-line `--` or `#`? Block comments? Needs PRD amendment first.
   - **Variabile runtime injection** (concern γ — PRD §4.6). Shim grows a third writer for vendored `src/variabile.rs`; emit prepends `mod variabile; use variabile::Variabile;`.
   - **Operator expressions + multi-word phrase table** (PRD §4.4). `a plus b per c`, `maior quam`, `divisus per`. Requires precedence climbing in parser.
   - **Indented blocks + control flow** (PRD §4.6 + §4.8). `si … :`, `dum … :`, `pro … :`. Forces the `tasks/CONTINUITY` PRD §4.8 control-structure-head terminator gap to close.
   - **`functio` declarations** (PRD §4.2). Multi-statement function bodies. `emit_stmt` takes an indent-level parameter (forward hook §8.10).
   - **`marain check` subcommand** (PRD §6). Lex + parse + name-resolve without invoking rustc. Sub-second feedback loop for diagnostics.
   - **rustc-error span back-mapping** (PRD §5, declared post-v0.5). Probably not v0.2; called out for completeness.
   - **`docs/lexicon.md`** (S1-7). The canonical translation table. Currently empty.
4. **`tasks/LESSONS.md` is still empty.** No corrections this session worth recording — every pushback was a re-scoping signal, not a mistake. Keep the bar high; don't pad.

## Open Items for Next Session

In priority order, **assuming the user wants to exercise v0.1 manually first**:

1. **Run `marain run hello.lat` from a hand-typed source.** Most of the manual-testing value lives here — see what the user-facing path feels like cold.
2. **Triage UX papercuts** the user finds during step 1. Add real findings to `tasks/BUGS.md` (currently empty) or to `tasks/TODO.md` as a "v0.1 follow-ups" section.
3. **Decide v0.2 scope.** Use the candidate list above. Likely one feature per session per CLAUDE.md "one feature per session" risk-mitigation guidance (PRD §10).
4. **Comment-syntax PRD amendment (concern η).** If the user finds the no-comments thing painful in manual testing, this jumps to top priority. Otherwise pin for v0.3.
5. **`docs/lexicon.md` seed.** Even a minimal lexicon doc would help future-me cross-reference keyword choices. Low priority; the keyword table in `lexer/keywords.rs` is the live source of truth.

## Carry-over concerns (status at v0.1 close)

| Concern | Status |
| ------- | ------ |
| (α) AST inflection slot | **RESOLVED** in R5 via `Ident` / `SigiledIdent` wrappers w/ `Option<Inflection>` |
| (β) 500-LOC lexer | **RESOLVED** in R4. Pressure-release never invoked across entire v0.1 stack |
| (γ) `Variabile` runtime injection | **PINNED** for Stage 2 / when Variabile literals enter the language; plan in ARCHITECTURE.md §8.10 |
| (δ) Hand-rolled CLI parsing (no clap) | **RETIRED** 2026-05-23: PRD §9 amended per user preference; clap pinned `=4.5.61` |
| (ε) Test strategy | **RETIRED** in R8: three-layer coverage (unit + golden + behavioral e2e) per ARCHITECTURE.md §10 |
| (ζ) Cross-file Stage 2 diagnostics | PINNED, future-only |
| (η) Comment syntax | **OPEN** — PRD gap; first real footgun in manual testing. Likely v0.2/v0.3 candidate |
| (θ) Stage 2 `(lemma, inflection)` tokens | PINNED for Stage 2 |
| Workspace inheritance for shims | **RESOLVED** in R6 (empty `[workspace]` table in render_cargo_toml) |

## Decisions Locked This Session (R7 + R8)

Carry-over from prior sessions (R1–R6) — see git log, prior CONTINUITY (replaced), and ARCHITECTURE.md §§2–8.

| # | Decision | Why |
| - | -------- | --- |
| R7.1 | `clap = "=4.5.61"` exact pin via `[dependencies]`; features `["derive"]` | N-1 of 4.6.1; released 72 days before pin; MSRV 1.74 ≤ our 1.94.1; `=` makes Cargo.toml + Cargo.lock together the full identity |
| R7.2 | `#[derive(Parser)] Cli` + `#[derive(Subcommand)] Command` over builder API | Schema is small; derive is half the LOC and reads as data not procedure |
| R7.3 | Hand-rolled XDG resolution (no `dirs` crate) | ~25 LOC; trivial policy not worth supply-chain surface |
| R7.4 | Hand-rolled FNV-1a 32-bit (no `DefaultHasher`) | `DefaultHasher` is Rust-version-fragile; cannot be persisted to disk and reproduced. FNV-1a is the standard non-cryptographic hash for short identifiers; known-vector-verifiable |
| R7.5 | `DriverError { Source{error, map}, Shim(ShimError), Io{context, source}, Cargo{exit_code} }` | Source binds SourceMap so report() can render; Shim auto-`From`; Io needs context wrapping; Cargo for exit-code proxying |
| R7.6 | Source errors → `Diagnostic::render`; system errors → `marain:` prefix | Two visually-distinct shapes mirror cargo:/rustc: convention; reader knows which side of the boundary the error came from |
| R7.7 | Split `build` into public path-resolving wrapper + private `write_shim_from_source(source, shim_dir)` seam | Workspace `unsafe_code = "forbid"` blocks env mutation in tests; cleaner to inject the target than to override env |
| R7.8 | `cargo` invoked with `--quiet --manifest-path <shim>/Cargo.toml`, `CARGO_TARGET_DIR` removed, inherited stdio | User sees only their program output; `--manifest-path` skips cwd-walk-up for workspace; env removal prevents race with parent test runner's target/ |
| R7.9 | Exit codes: 0 success / 1 driver error / cargo's code proxied for run failures / clap returns 2 for arg errors | Distinguishable; matches POSIX + clap convention |
| R7.10 | Manual `Debug` for `DriverError` (don't add Debug to SourceMap) | Hand-rolling Debug in CLI is small; adding Debug to a foundational core type is a surface change unrelated to R7 |
| R7.11 | `marain build` prints shim path to stdout (chosen via AskUserQuestion) | Scriptable; user picked over silent-success and over print-with-prefix |
| R7.12 | Binary-level e2e deferred to R8 (chosen at R7 framing) | R8 is the testing-harness round; don't pull harness work forward |
| R8.1 | Three-layer coverage (per-phase unit / fixture-walker goldens / behavioral e2e) | Each catches a different class of regression at a different cost; only e2e proves user-facing behavior |
| R8.2 | Paired-file fixtures (`.lat` + `.expected.{rs,txt}`) over inline snapshots | Self-documenting; adds zero dependencies (vs. `insta` / `expect-test`); aligns with PRD §9 self-supporting |
| R8.3 | `MARAIN_UPDATE_GOLDENS=1` env var to regen | Single env var, no subcommand; trim-end-tolerant comparison so trailing-newline editors don't flake |
| R8.4 | One aggregating `#[test]` per harness, not one-per-fixture | Regression shows every drifted fixture in one run; cheaper triage |
| R8.5 | Error fixtures load the bare basename into SourceMap (not full disk path) | Rendered diagnostics are machine-stable across CI / different repo locations |
| R8.6 | CLI e2e isolates `$XDG_STATE_HOME` per test via `Command::env` (subprocess env, not ours) | Two tests can run concurrently; workspace `unsafe_code = "forbid"` satisfied without any `unsafe` block |
| R8.7 | CLI e2e asserts specific exit-code shape (1 vs 2 vs cargo's) | Refactor that collapses them gets caught |
| R8.8 | Library e2e (`e2e_hello_world.rs`) carried forward, not retired | Marginal duplication; covers a regression in either library shape or binary wiring independently |
| R8.9 | `#![forbid(unsafe_code)]` added to both crate roots (in addition to workspace lint) | Belt-and-braces per CLAUDE.md; survives a crate being forked out of the workspace |

## Standing decisions from PRD (pointers only)

Re-read `PRD.md` §§4–9 for details:

- §4.1 Source-to-source transpile to Rust (Stage 3)
- §4.2 Latin grammar as staged syntax; "first to define is followed" Stage-1 rule; `DETONATIO!` sanctioned ALL-CAPS exception
- §4.4 Operators as Latin function words; `est`/`fit` disambiguation; Rust precedence
- §4.5 Sigils `@`/`^` on every variable reference; `tenet` for borrows; `ego` for self
- §4.6 Python niceties (indentation, triple-quoted, Variabile, dict/list/tuple/f-string)
- §4.7 Macro split: no-punct subset (`dic`, `queror`, `agmen`, `forma`) + `!`-bearing otherwise
- §4.8 Period-terminated statements
- §4.9 ASCII-only identifiers, Rust-style casing
- §4.10 3-stage compilation
- §6 CLI v0.1: `marain build`, `marain run`
- §7 v0.1 done line: `dic "salve, munde".` — **operationally proven at four test layers**
- §9 Constraints: stable Rust 1.94.1, edition 2024, no JS/Node/npm, self-supporting; `clap` is the lone permitted dep (amended 2026-05-23); N-1 / 30-day rule; 500 LOC/file target
- §11 Open: S1-5..S1-12 deferred; S2-1..S2-7 post-v0.1; (η) comment syntax still a real footgun

## Collaboration Patterns (refined this session)

- **`AskUserQuestion` for round framings.** Used at R7 (chose "Print path, defer binary e2e"). Skipped at R8 — proposed framing tersely in text and executed; user trusted the framing. **Pattern: when scope is clear and there's no real fork, propose-and-execute beats ask. Save the question for decisions with multiple defensible answers.**
- **One drilling question per round.** Multi-axis bundled into single AskUserQuestion; user processes them together as coherent stances.
- **Pushback in answers is re-scoping, not rejection.** R6 Rust-keyword reframe was the textbook example. R7 saw none — clean.
- **The user pushes for project-local artifacts** (`.scratch/`, not `/tmp`). Verified again in R7 + R8 (test scaffolding lives under `.scratch/` with RAII cleanup).
- **The user prefers mature mainstream libs over hand-rolling at decision points where pedagogy isn't the goal** (clap over hand-rolled arg parsing). **Pattern: hand-rolling pays for itself when the surface is the thing being studied (lexer / parser / emit); when it's mechanical infrastructure (arg parsing, hashing for short IDs, XDG resolution), the calculus changes per case.** Hand-rolled survived for XDG (~25 LOC, no real lib advantage) and FNV-1a (~10 LOC, `DefaultHasher` unsuitable); clap won for arg parsing (real ergonomic advantage).
- **Demand "explicit deny" interpretations literally.** When user asked for "explicit deny to unsafe blocks in the Cargo.toml," the Cargo.toml workspace lint was already at strictest setting (`forbid` > `deny`). I confirmed the existing state AND added crate-root `#![forbid(unsafe_code)]` as belt-and-braces per CLAUDE.md. **Pattern: when a user asks for X and X already exists, confirm visibly and then add the adjacent belt-and-braces — don't just say "already there."**
- **End-of-turn summaries 1–2 sentences.** List of changed sections welcome; restating the changes themselves is not.
- **User likes consequences spelled out.** When a decision has downstream effects, articulate them. Earns green-lights.
- **`feedback_reframe_vs_push_through` still applies.** When a design is fighting itself, surface a reframe before compromising.
- **The task-tool nudge is ignored** per session convention (TODO.md tracks rounds; no in-flight tasks worth `TaskCreate`-ing).
- **Quality gate cadence per round close:** `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --all`. If fmt fails: `cargo fmt --all` then re-check. Verified across all 8 rounds.
- **Disk artifacts land per round.** Write code + tests + ARCHITECTURE.md section + TODO.md update in one batch, run gates, declare round closed. Verified across all 8 rounds.
- **Demonstrate, don't claim.** R6 closed with manual end-to-end run BEFORE marking done. R7 closed with manual binary smoke. R8 closed with `MARAIN_UPDATE_GOLDENS=1` to seed, then verify the goldens look right by `cat`-ing them.

## Tactical Notes

- Today's date is 2026-05-23. This session covered R7 + R8 (plus the clap PRD amendment + the crate-root `#![forbid(unsafe_code)]`). Prior session covered R5 + R6.
- The "first to define is followed" rule (PRD §4.2) is Stage-1-only. Preserve staging if §4.2 reopens.
- `DETONATIO!` is the only sanctioned ALL-CAPS keyword. New exceptions need equivalent semantic-weight justification.
- `AGENTS.md` is a symlink to `CLAUDE.md` — only edit one.
- PRD §11 uses prefixed labels (`S1-*` / `S2-*`) in paragraph form. Preserve if extending.
- Working directory on disk is still `/Users/john/code/rubigo/`; Claude Code memory dir is still `~/.claude/projects/-Users-john-code-rubigo/`. Not renamed despite the Rubigo → Marain project rename. User's call whether to `mv`.
- `tasks/CONTINUITY.md` is rewritten (not appended) every time used (per CLAUDE.md).
- `.scratch/` (gitignored) is the convention for any test artifact that needs disk; resolved via `CARGO_MANIFEST_DIR/../../.scratch/`. Used by shim.rs tests, e2e_hello_world.rs, driver.rs tests, paths.rs tests, and cli_e2e.rs.
- The shim's `Cargo.toml` includes empty `[workspace]` — baked into `render_cargo_toml` and tested. This lets a shim sit anywhere (including under `.scratch/` inside the outer workspace) without cargo rejecting it as a non-member.
- Hash function for shim identity is FNV-1a 32-bit (8 hex chars). Known vectors tested (`a` → `e40c292c`, `foobar` → `bf9cf968`). Lives in `marain-cli/src/paths.rs`.
- `clap` is the lone dependency. Pinned exactly to `4.5.61` (released 2026-03-12, 72 days before pin; latest at pin was 4.6.1 released 2026-04-15). Verification hashes in Cargo.lock.
- `MARAIN_UPDATE_GOLDENS=1 cargo test -p marain-core --test emit_goldens --test error_goldens` regenerates goldens. Useful when intentionally changing emit shape or diagnostic text.
- Two `#![forbid(unsafe_code)]` attrs at crate roots + one workspace `[workspace.lints.rust] unsafe_code = "forbid"` = belt-and-braces unsafe ban. Adding `unsafe` anywhere is a compile error.
- Sentrux MCP review per CLAUDE.md has NOT been run any session yet. Worth running before opening v0.2 to baseline complexity now that the v0.1 surface is settled.
- README.md was rewritten this session from the v0.1 stub to a real quickstart + commands + limitations document.
