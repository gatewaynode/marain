# Marain CMS

**EXPERIMENTAL!** Please do not use this for anything important yet.

[![CI](https://github.com/gatewaynode/marain/actions/workflows/rust.yml/badge.svg)](https://github.com/gatewaynode/marain/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/badge/rust-1.79-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-v2-blue.svg)](https://tauri.app/)

A flexible, headless CMS, written in Rust.

## Background

So that was my goal, make a CMS from scratch in languages I like, that qualifies as a large coding project from scratch, using semi-autonomous LLM coding models.  CMS's are something I'm very deeply familiar with so creating one in the way I'd want to build it from scratch today has been a fun side effect of pushing coding LLMs to their limits.  I've learned a lot about the semi-autonomous model (not to be confused with yolo coding, or suggestion based coding), and I'm enjoying it.  So I think I'm going to finish this project and start testing it in production soon.

### My findings so far.  

- Context rot is real, most models, even with compensating tooling will become less effective after passing 50% of their maximum context window.  And unfortunately compaction isn't safe, there is too much nuance and detail lost in the process to keep working on difficult features after compaction.  It's better to start over on the feature and better manage your context than to use compaction.
    - This means you have to set narrow goals, move to modularization of an app early so you can achieve feature builds in one pass, one smallish piece at a time.
- Most models seem to hallucinate around very difficult problems, lending proof to the theoretical attention limits of even SOTA hardware(it almost seems like frustration).  Maybe it's some sort of cognitive consistency pattern?  Seems more like straight confabulation though.  Anyway, isolating tough problems to where you can start on them with minimal context seems to work well to get around these quirks.
- Distraction is a real problem with coding LLMs, small things can send them rabbit holing off on tangents.  And when this happens the work will be incomplete, even if you provide detailed contextual requirements.

All that said.  After I figured out how to work with the two models I ended up using, Claude Opus 4.1 and Gemini Pro 2.5 in KiloCode, development has been blazingly fast and relatively surprise free.  This should be ready for general testing within the next few weeks.

# Roadmap

- [x] Configuration as code
- [x] Schema as code
- [x] Sqlite integration
- [x] Axum based REST API
- [x] K/V store fast cache for prequeried content
- [x] Switch to using ULIDs instead of UUIDs for cache keys and entity/field IDs
- [x] Centralized content management features and functions in the `content` crate
- [x] User management with the `user` crate with separation of sensitive user data
- [x] User profiles as entities
- [ ] Implement Cedar based policy engine for authorization control
- [ ] Implement work queue persistence layer
- [ ] Implement the event broadcast bus using `tokio::sync::broadcast`
- [ ] Implemeent the cron event signaler with system configuration
- [ ] Implement the standard work queue `crossbeam-channel`
- [ ] Add broadcast triggers in standard workflow items
- [x] Implement the CLAP CLI interface
- [ ] Implement the default admin interface in Svelte5
- [ ] Refine hook system locations and format, add a priority field
- [ ] Document custom module API and write up a how-to guide
- [ ] Build stand alone Rust project outside of Tauri (currently possible with manual work, this is to automate it)
- [ ] Implement the Postgres database drivers and system config options
- [ ] Swagger UI custom plugin to show example write payloads for entities
- [ ] GRPC API
- [ ] Cloud offload of internal components to serverless cloud components via configuration 

# Development

This is a Tauri project, but I've been focused on the headless backend.  I'm not looking for any code contributions, but if you feel inclined I might accept PRs.  Bug reports are more welcome in the issues section here on Github.  This project is not licensed in anyway, as I am not sure about the legal context of trying to license semi-autonomous AI written code.  So just assume any contributions are only *intended* to go into a GPLv3 or AGPLv3 project once I get some legal review.

Working on the project assumes you have some knowledge of Tauri, Svelte5, Rust, and Cargo workspaces.  Using an LLM that has those prerequsites instead of you yourself is a bad idea.  That said I don't mind LLM generated code, but please write the PR comment yourself.

If you are going to use an LLM for PRs make sure you have them read the `documentation/*` files and the `AGENTS.md` as context in the beginning of your prompt.  Update documentation and add tests.

## 🚀 Getting Started

This project uses `bun` for the frontend and `cargo` for the Rust backend.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Bun](https://bun.sh/docs/installation)
- Platform-specific dependencies for Tauri (see the [Tauri documentation](https://tauri.app/v1/guides/getting-started/prerequisites))

### Running the Application

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/mhenderson-so/marain-cms.git
    cd marain-cms
    ```

2.  **Install frontend dependencies:**
    ```sh
    bun install
    ```

3.  **Create your environment file:**
    Copy the `EXAMPLE.env` file to `.env` and generate a session secret key.
    ```sh
    cp EXAMPLE.env .env
    # On macOS/Linux
    NEW_KEY=$(openssl rand -base64 32)
    echo "SESSION_SECRET_KEY=\"$NEW_KEY\"" >> .env
    ```

4.  **Run the Tauri development server:**
    This command will build and launch the desktop application with hot-reloading for both the frontend and backend.
    ```sh
    bun run tauri dev
    ```

### Running in Web-Only Mode

If you only want to work on the Svelte UI without launching the Tauri desktop application, you can run the Vite development server directly:

```sh
bun run dev
```