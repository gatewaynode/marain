# Repository Guidelines

## Project Structure & Module Organization
- `src/`: SvelteKit UI (components, routes, stores, utils).
- `src-tauri/`: Tauri app and Rust backend (`lib.rs`, commands, config).
- `src-tauri/*`: Uses Cargo workspaces to separate concerns into crates.
- `src-tauri/api`: Is the crate for all API functions and features.
- `src-tauri/app`: Is the main crate that orchestrates the app.
- `src-tauri/database`: For handling database stores.
- `src-tauri/enitities`: For managing content schemas and their supporting features.
- `src-tauri/fields`: For managing different field implementations.
- `src-tauri/json-cache`: Cache specific functions and features.
- `src-tauri/schema-manager`: Handles hot-reloading and accessing of entity and config schema values.
- `static/`: Static assets bundled at build.
- `data/`: Databases and logs
- `schemas/`: YAML entity definitions
- `config/`: YAML configuration definitions
- `scripts/`, `build/`: Helper scripts, and build output.

## Build, Test, and Development Commands
- The public repository location is `github.com/gatewaynode/marain`
- Dev (desktop): `bun run tauri dev` — launches the Tauri app.
- Dev (web only): `bun run dev` — Vite dev server for UI.
- Build (desktop): `bun run tauri build` — production bundle.
- Preview (web): `bun run preview` — serves built UI.
- Type check: `bun run check` — Svelte/TS checks.
- Rust tests: `cargo test` in `src-tauri/`.

## Coding Style & Naming Conventions
- Svelte/TS: 2-space indent; components `PascalCase.svelte`; stores/utils `camelCase.ts`.
- Routing: follow SvelteKit `+page.svelte`, `+layout.svelte` patterns.
- Rust: `snake_case` for modules/functions, `CamelCase` for types; prefer `Result` over panics.
- Format: run `cargo fmt` (Rust); use editor formatting for Svelte/TS; keep imports ordered.

## Testing Guidelines
- Rust unit tests live beside code (`#[cfg(test)]`); integration tests per crate; use `rstest` when helpful.
- Name tests with descriptive `snake_case`; focus on core logic and file/IO boundaries.
- Frontend: no formal unit tests; use `bun run check` and manual flows for regressions.
- Run: `cargo test` in `src-tauri/`
- Run: `cargo clippy --all` in `src-tauri/` and resolve and issues
- Run: `cargo fmt` in `src-tauri/` before finishing any task

## Security & Configuration Tips
- Application security concerns are a top priority, always write secure code
- Do not commit secrets or signing identities.
- Ensure all dependencies are using the latest stable version
- Ensure all database inputs are validated and handled as safely as possible (parameterized queries, stored procedures, injection string filtering before querying).

## Documentation
- The documentation in `/documentation/` is always updated if the implementation changes any details
- The `/documentation/REST-API/openapi.json` file should always be kept up to date
- The **Implementation Notes** is always where we store the task completion summary in the `/documentation/DEV-TASKS.md` file

