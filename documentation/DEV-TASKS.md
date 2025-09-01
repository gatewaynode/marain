# Development Tasks

This is our project management list, since we'll be taking an agile approach this list will only 
cover the next one or two tasks at a time.

# Tasks

---

## Task 22 Supporting CLI application as an additional binary

- [x] Status: Complete

Create a CLI interface for the CMS as a standalone binary with the Rust `Clap` crate.  This binary, named `marc`, is optionally compiled with the main system but should exist as a distinct file which a user can set in their own system path in the way they feel fit.  The CLI app mimics the same functionality of the `app` crate of pulling in additional system crates for its CLI commands.  Initial functionality for the binary is to:
- Read system status as a sort of command line health check
- List in memory configurations
- Read individual configuration section values


### Acceptance Criteria:

- An additional binary is created which holds the CLI app
- The binary should have access to the same resources as the app crate when launched in the project root
- The binary should fail with an error message that it is not in the project root when launched in any other location
- Maintain the current build priority so the full Tauri build does not break, the CLI app is just an additional binary file
- An initial CLI test suite is created

### **Implementation Notes:**

Successfully implemented the `marc` CLI binary with the following features:

1. **Project Structure**: Created a new crate at `src-tauri/cli/` with proper Cargo.toml configuration
2. **Commands Implemented**:
   - `health` - System health check showing status of database, configuration, schemas, and API
   - `config list` - List all loaded configurations
   - `config get <path>` - Get specific configuration values using dot notation
3. **Project Root Detection**: Implemented robust project root detection that works from any subdirectory
4. **Environment Path Management**: Refactored `EnvPaths` to support testing with base directory override
5. **Output Formats**: Support for text, JSON, and YAML output formats
6. **Testing Strategy**:
   - Fixed path resolution issues by using relative paths and clearing environment variables between tests
   - Added mutex protection for environment variable tests to prevent interference
   - Created comprehensive integration tests that verify CLI behavior from temporary directories
7. **CI/CD Fixes**:
   - Resolved clippy warnings by removing unnecessary borrows in test arrays
   - Fixed security audit failure by renaming package from "marc" to "marain-cli" to avoid conflict with vulnerable crate
   - Binary name remains "marc" for user convenience
8. **Code Quality**: All tests passing, clippy warnings resolved, and code formatted with cargo fmt
9. **Documentation**: Created comprehensive CLI documentation at `documentation/CLI/marc-cli-documentation.md`

The CLI can be built with `cargo build --package marain-cli` and tested with `cargo test --package marain-cli`.

---

## Task TEMPLATE

- [ ] Status: Implementation Design



### Acceptance Criteria:

### **Implementation Notes:**

---
---

# Roadmap


- [ ] Bug fix: Errors on the swagger UI page
```
Resolver error at paths./api/v1/entity/update/{entity_type}/{content_id}.post.responses.200.content.application/json.schema.properties.created_at.$ref
Could not resolve reference: Could not resolve pointer: /components/schemas/DateTime does not exist in document
```
- [ ] Implement [ReDB](https://github.com/cberner/redb) as a persistent event/work queue
- [ ] Implement the broadcast event bus using `tokio::sync::broadcast`
- [ ] Implemeent the cron event signaler with system configuration
    - make sure to have comprehensive error handling and logging in the event of errors here
- [ ] Implement the standard work queue `crossbeam-channel`
- [ ] Add broadcast triggers in standard workflow items
- [ ] Implement the `last_cached`, `cache_ttl` fields across all entities
- [ ] Implement the JSON cache using [ReDB](https://github.com/cberner/redb)
- [ ] Implement the user private store using [ReDB](https://github.com/cberner/redb)
- [ ] Implement the user semi-private content entity
- [ ] Implement the CLAP CLI interface
- [ ] Implement the default admin interface in Svelte5
- [ ] Refine hook system locations and format, add a priority field
- [ ] Implement the Postgres database drivers and system config options
- [ ] Swagger UI custom plugin to show example write payloads for entities.


# Diff Fails From Last Task

**Place diff files that fail to merge here with the task #, filename and line context for manual application.**

