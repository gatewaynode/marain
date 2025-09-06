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


## Task 22 Implement the user semi-private user profile as a content entity

- [ ] Status: Ready for work


This is the first part of implementing user management, create the less sensitive content type for holding user information.  It must be free from PII data and secrets.  This entity will require a supporting "links" entity with a standard set of fields for displaying hyperlinks.

These are the fields the user entity content type requires:
- username: "creative_coder"
- display_name: "Alex Doe"
- user_id: "01H8XGJWBWBAQ4Z4M9D5K4Z3E1" the ULID
- identity_type: "human", Other potential values: "bot", "agent", "corporation", "dog"
- managed_by: "01H8XGJWBWBAQ4Z4M9D5K4Z3E0" # user_id of the managing human or group if the user is an alias or non-human entity
- avatar_url: "https://example.com/path/to/user/avatar.png"
- bio: "Developer and designer focused on creating beautiful and functional web experiences."
- public_links: a cardinality 5 entity reference to the links entity
- theme_preferences: "light" or "dark"
- language: "en-US"
- timezone: "America/New_York"
- created_at: "2023-10-27T10:00:00Z"
- profile_visibility: "public" # Other potential values: "private", "friends_only"
- notification_preferences: a select list "none", "security", "important", "all"
- new_followers: true
- direct_messages: true
- product_updates: false
- metadata: A JSON string with stuff like
   - profile_views: 1024
   - last_badge_earned: "pioneer"
   - custom_status_emoji: "🚀"

The link entity requires:
- URL
- Label
- Title Attribute

### Acceptance Criteria:

- Two new schema files have been created, one for links, and one for user profiles
- The database tables have been confirmed to be in place and working

### **Implementation Notes:**


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

---

## Task 23 Documentation Refinements for Consistency and Quality

- [x] Status: Complete

### Description:
Refined key documentation files to achieve consistent quality, depth, and completeness, focusing on points of refinement identified in architecture and docs. Updates include tailored examples, enhanced diagrams, cross-references to DEVELOPER-GUIDE.md, and added security best practices sections across user management and hot-reload docs.

### Changes Made:
- **entity-management.md**: Tailored trait-object example to Marain Entity trait with Article/User structs; added Mermaid sequence diagram for schema loading/hot-reload flow; included links to schema examples (snippet.schema.yaml, etc.).
- **user-management-system.md**: Expanded core components with implementation details (magic links, passkeys, ULID/UUID bridge, file refs); enhanced Mermaid graph with data flows; added Security Best Practices section.
- **authentication.md**: Added cross-references to DEVELOPER-GUIDE.md for DB/caching/ULID; inserted Security Best Practices subsection under Session Management.
- **authorization.md**: Updated Data Model with DB flexibility ref; added Security Best Practices under Implementation Guidelines, with cross-refs to auth and API lifecycle.
- **user-management.md**: Enhanced User Schema with modeling ref; added Security Best Practices under Implementation Guidelines, linking to auth/authz and fields crate.
- **hot-reload-summary.md**: Added ref to dynamic schema in State Management Changes; appended Cross-References and Security Best Practices section for integration and validation.

### Verification:
- Manual review confirms uniform structure (e.g., consistent sections: Core Components, Implementation Guidelines, Security Best Practices).
- Cross-references ensure navigation to DEVELOPER-GUIDE.md for broader context.
- No syntax errors in Mermaid diagrams or YAML examples.
- Suggested commands: `cd src-tauri && cargo fmt && cargo clippy --all`, `bun run check` (no issues expected as docs only).

### **Implementation Notes:**
Documentation now provides a cohesive, secure, and comprehensive guide, improving developer experience and maintainability. All updates align with project principles: modularity, configuration-as-code, and security priority.

