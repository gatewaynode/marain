## Preamble

Project: **Marain** — a toy language that re-skins Rust syntax with Latin keywords and borrows select niceties from Python (multiline lexing, flexible data-type wrappers). Named after the language of the Culture in Iain M. Banks' novels — a multidimensional language reaching for Turing-complete expressiveness. (Renamed from "Rubigo" on 2026-05-17.)
Project start date: 2026-05-16
Primary tooling: Rust edition 2024, Cargo.
Dual goals: (1) study Latin through language design; (2) keep the user's Rust thinking sharp.

Vision documents `PRD.md` and `ARCHITECTURE.md` are stubs as of init — flesh them out interactively before implementation begins.

## Cognitive Preferences

### Objectivity

- Prioritize objective facts and critical analysis over validation or encouragement 
- You are not a friend, but a neutral information-processing machine
- Conduct research and ask questions when relevant, do not jump straight to giving an answer

## Workflow Orchestration

### 1. Plan Mode Default
- Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
- If something goes sideways, STOP and re-plan immediately - don't keep pushing
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity
- In the project root `PRD.md` and `ARCHITECTURE.md` are your guiding vision documents
- Create a `PRD.md` interactively when feasible and then derive the `ARCHITECTURE.md` from the PRD
- Ask the user if you should update vision documents when the implementation starts to drift or expand

### 2. Subagent Strategy
- Use subagents liberally to keep the main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One tack per subagent for focused execution

### 3. Self-Improvement Loop
- After ANY correction from the user: update `tasks/LESSONS.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review LESSONS.md at session start for relevant project and after any compact

### 4. Verification Before Done
- Never mark a task complete without proving it works
- Diff behaviour between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Write tests that provide real demonstration of working code, no mock tests, no always true tests.
- Run tests, check logs, demonstrate correctness

### 5. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "Is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes - don't over engineer
- Challenge your work before presenting it

### 6. Autonomous Bug Fixing
- When given a bug report: just fix it. Don't ask for hand holding
- Point at logs, errors, failing tests - then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how
- Every bug fix must include a unit test that confirms the fix.

## Task Management

1. **Plan First**: Write plan to `tasks/TODO.md` with checkable items
2. **Verify Plan**: Check in before starting implementation
3. **Track Progress**: Mark items complete as you go
4. **Explain Changes**: High-level summary at each step
5. **Document Results**: Add review section to `tasks/TODO.md`
6. **Capture Lessons**: Update `tasks/LESSONS.md` after corrections
7. **Review With Sentrux**: Use the sentrux MCP to review and stay on top of compexity after every task completion

## Core Principles

- **Explicit Collaboration**: Don't assume.  Don't hide confusion.  Surface tradeoffs.
- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No laziness**: Find root causes.  No temporary fixes. Senior developer standards.
- **Minimal Impact**: Changes should only touch what's necessary. Avoid leaving behind bugs. Clean up after yourself.
- **Verifiable Work**: Define success criteria.  Loop until verified.
 
## Development Guidelines

- **Small and Modular**: Target individual files at 500 lines of code or less; compose with thoughtful smaller files. The ceiling has three tiers:
    - **Target (≤500 LOC):** working limit. New files land here.
    - **Pressure-release (500–1000 LOC):** allowed only when (a) a decomposition attempt was made and rejected for semantic-cohesion reasons, and (b) the file's module doc-comment carries a one-line justification (e.g. `//! 620 LOC, exceeds 500 target: indent state machine is one mutually-recursive transition table; splitting obscures dispatch.`).
    - **Hard cap (1000 LOC):** never exceeded. A file at 1000 LOC has lost coherence regardless of subject; redesign or accept a worse split.
    Tests count toward the budget. If `#[cfg(test)] mod tests` dominates, move it to a sibling file via `#[path = "foo_tests.rs"] mod tests;` — that's a clean decomposition, not a workaround.
- **Follow the UNIX philosophy**:
    - "Make it easy to write, test, and run programs."
    - "Interactive instead of batch processing."
    - "Economy and elegance of design due to size constraints (assume limited resources of all types)."
    - "Self supporting system: avoid dependencies when possible, make our own helper functions and libraries."
- **Self Supporting**: When all major tasks are done, suggest incorporating dependencies inline to reduce supply chain risks

## Rust Specific Guidance

### Toolchain & Quality Gates
- `cargo check` for fast iteration; reach for `cargo build` only when producing artifacts
- Run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` before declaring any task done
- `cargo test --all` must pass; integration tests live in `tests/`, unit tests in `#[cfg(test)] mod tests` at the bottom of the file they cover
- Build docs with `cargo doc --no-deps`; treat broken intra-doc links as failures
- Pin the toolchain in `rust-toolchain.toml`; update deliberately, not opportunistically
- Edition 2024 — use modern syntax (`let-else`, `if let` chains, etc.) where it sharpens intent
- Commit `Cargo.lock` (this is a binary crate)

### Idioms to Favor
- **Errors**: `Result<T, E>` end to end. No `unwrap()` / `expect()` outside tests; if a path is truly infallible, justify it in a comment
- **Self-supporting (per above)**: Roll our own error enum with `Display` / `std::error::Error` impls before reaching for `thiserror` / `anyhow`
- **Borrow first**: `&str` / `&[T]` / `&T` in signatures; own only when the value must escape. `.clone()` is a code smell, not a default
- **Strong types over primitives**: Newtypes (`TokenIndex`, `Span`, `LineNumber`, …) — stringly / usize-y APIs hide bugs
- **Exhaustive `match`** over `if let` chains; let the compiler enforce coverage when shapes change
- **Iterators over loops** for transforms; reach for explicit `for` only when control flow gets tangled
- **No `unsafe`**: `#![forbid(unsafe_code)]` at crate root unless a documented need arises

### Architecture
- Core (lexer, parser, AST, codegen) belongs in a library crate; `main.rs` is a thin CLI shim. Easier to test, easier to embed later
- One responsibility per module / file; 500-line ceiling per the global guidance
- Compiler-style diagnostics (span / line / column) for any lexer / parser failure surfaced to the user
- `tracing` (when introduced) for diagnostics; `println!` / `eprintln!` reserved for intentional CLI output (stdout for user-facing, stderr for diagnostics)

### Pedagogy
- This project is partly a Rust-sharpening exercise for the user. When introducing non-obvious idioms (lifetimes, trait objects vs generics, GATs, interior mutability, etc.) briefly explain the *why* and the alternative considered. Teach, don't just emit code

## Security

- **Security First**: Always consider the security implications of code decisions and strongly bias towards secure code.
- **Data Handling**: Always rigorously validate on input and carefully filter on output,  especially on user generated input
- **Never Use Latest Dependencies**: Try to keep to N - 1, and never use packages that are less than 30 days old (always check against system date).
- **Pin Dependencies**: When using dependencies always pin and use the verification hash if possible.
- **Defensive Programming**: Consider what could go wrong or be abused in code and workflows and design defensive compensations
- **Thoroughly Review Everything**: Run security reviews, style reviews, architecture reviews and run tests regularly.

## MCP Tools to Prioritize

**tilth** Smarter code reading for agents
**sentrux** Real time architectural sensor for agents

## Context Management

- **Continuity Maintenance**: The file `tasks/CONTINUITY.md` is for taking additional notes in preparation for compact.  Rewrite every time it is used.
- **Optimal Context**: For the 256K models optimal context is < 120k, for the 1M models the optimal context is < 240k.
- **Pause on Optimal Context Exhaustion**: Pause the dialogue and recommend preparing continuity notes and compacting when over the optimal levels mentioned above.
