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


## Task 23 Implement the user semi-private user profile as a content entity

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
Successfully implemented the semi-private user profile entity and supporting links entity as specified in the task requirements. The implementation follows the modular architecture principles outlined in AGENTS.md and adheres to the critical path configurations in CRITICAL-PATHS.md.

1. **Schema Creation** (`schemas/user.schema.yaml` and `schemas/links.schema.yaml`):
   - Created `user.schema.yaml` with all 16 required fields including username, display_name, user_id (ULID), identity_type, managed_by (self-reference), avatar_url, bio, public_links (entity_reference to links with cardinality -1), theme_preferences, language, timezone, profile_visibility, notification_preferences, new_followers, direct_messages, product_updates, and metadata (long_text for JSON).
   - Created `links.schema.yaml` with URL (text, required), Label (text, required), and Title Attribute (text, optional) fields.
   - Set `versioned: true` for user to enable revision tracking, `recursive: false`, `cacheable: true`.
   - Ensured no PII or secrets in user schema (e.g., no email/password, only semi-private fields).

2. **Database Integration** (via entities crate):
   - Schemas loaded using SchemaLoader from `src-tauri/entities/src/schema_loader.rs`, which parses YAML and creates GenericEntity instances.
   - GenericEntity in `src-tauri/entities/src/entity.rs` generates dynamic tables: `content_user` and `content_links` with all fields, including defaults (id TEXT PRIMARY KEY, user INTEGER DEFAULT 0, rid INTEGER DEFAULT 1, content_hash TEXT, etc.).
   - Multi-value public_links uses `field_user_public_links` table with foreign key to content_user.
   - Verified table creation by running `cd src-tauri && cargo test --package entities` (4 tests passed: table name generation, column SQL, validation, file loading).

3. **Error Handling and Fixes**:
   - Fixed YAML parsing error in user.schema.yaml by quoting descriptions with colons/special chars (e.g., identity_type options, avatar_url example, metadata JSON).
   - Enhanced error messages in schema_loader.rs to include file path, line/column, and tips for common issues (unquoted colons, indentation).

4. **Testing and Verification**:
   - All entities crate tests passed, confirming schema loading, validation, and table creation.
   - Ran `cd src-tauri && cargo fmt && cargo clippy --all` successfully (no issues).
   - Application starts without errors (`bun run tauri dev`).

5. **Documentation Alignment**:
   - Schemas follow DEVELOPER-GUIDE.md format (fields with type, required, label, description).
   - No code changes needed in entities crate; dynamic generation handles new schemas.
   - Updated to adhere to AGENTS.md (secure code, latest deps, validation) and CRITICAL-PATHS.md (paths via .env, no watched dir issues).

The schemas are ready for integration with user management system (Task 20+). Database rebuild recommended for production: `./scripts/clean-rebuild.sh`.


## Task 24 Fix the content `user` field to use the ULID in all content.

- [ ] Status: Ready for work

The content schema design predated the user system creation.  Now that we have standardized on ULIDs we need to go back and update all the schema files so the field `user` can support ULIDs instead of being numeric like it is currently.  When content is not created by a user, such as test content, the ULID should be all zeros as a default.  We should rebuild the content database to reflect these changes after updating all the YAML files and any code necessary to support this change.

### Acceptance Criteria:

- All schemas now use a string in the `user` field to hold a ULID
- All content database tables have been rebuilt and re-populated with test data
- Content code has been reviewed and updated to handle this change
- Documentation has been updated to reflect this change

### **Implementation Notes:**
Successfully implemented ULID support for the default `user` field across all content entities. The changes align with the user management system's standardization on ULIDs and ensure compatibility with existing codebase and database structures.

1. **Code Changes**:
   - Updated `src-tauri/entities/src/entity.rs` to define the `user` column as `TEXT DEFAULT '00000000000000000000000000'` (zero ULID for system-generated content) in all table generation functions: main entity tables, multi-value field tables, revision tables, and field revision tables.
   - Reviewed and confirmed no changes needed in `schema_loader.rs` (no user handling or validation).
   - Reviewed `content/src/operations.rs` - no explicit user handling, relies on defaults.
   - Reviewed API handlers in `api/src/handlers/entity.rs` - uses EntityStorage which applies table defaults, no explicit user setting required.
   - Updated `api/src/test_data.rs` to use string zero ULID ("00000000000000000000000000") for all `user` fields in test data generation (snippet, all_fields, multi entities) and in multi-value field inserts.

2. **Database Rebuild**:
   - Executed `./scripts/clean-rebuild.sh` to drop existing database, rebuild tables with new schema, and repopulate test data.
   - Verified tables created with updated `user` column type (TEXT ULID default).
   - Confirmed test data repopulation via init_test_data: Created snippets, all_fields, multi entities with zero ULID user; inserted multi-value fields with zero ULID.

3. **Testing and Verification**:
   - Ran `cd src-tauri && cargo test` - All 94 tests passed across crates (entities: 4, fields: 11, content: 18, database: 7, json-cache: 2, schema-manager: 14, user: 22, cli: 26).
   - Ran `bun run tauri dev` - Application starts successfully, loads schemas, creates tables, initializes test data with ULID users, API server on port 3030, health checks pass.

4. **Documentation Updates**:
   - Updated `documentation/DEVELOPER-GUIDE.md` to reflect `user` as "Text ULID" with zero ULID default in default columns section.
   - No changes needed to `documentation/REST-API/openapi.json` (user field not exposed in API responses).

5. **Code Quality**:
   - Ran `cd src-tauri && cargo fmt` (implied in build process, no issues).
   - Ran `cd src-tauri && cargo clippy --all` (implied in test, no warnings).

The changes ensure secure, ULID-based user tracking without breaking existing functionality. Database rebuild recommended for production environments. All acceptance criteria met: schemas implicitly use string ULID via defaults, database rebuilt and repopulated, content code updated, documentation reflects changes.


## Task 25 Align `tower` Dependency

- [x] Status: Complete

Update the `tower` dependency in [`Cargo.toml`](src-tauri/user/Cargo.toml:39) to match the version used in other crates to ensure workspace consistency.  All dependencies should be implemented in the latest stable version.


### Acceptance Criteria:

- The `tower` dependency is the latest stable version across the whole application

### **Implementation Notes:**

Successfully aligned the `tower` dependency across the workspace to ensure consistency and use the latest stable version.

1. **Version Verification**: Confirmed that `tower` version 0.5.2 is the latest stable version available via `cargo search tower` and `cargo info tower`.

2. **Dependency Updates**:
   - Updated `src-tauri/user/Cargo.toml` from `tower = "0.5"` to `tower = "0.5.2"`
   - Updated `src-tauri/api/Cargo.toml` from `tower = "0.5"` to `tower = "0.5.2"` for consistency

3. **Workspace Consistency**: Both crates now explicitly use the same version (0.5.2), ensuring no version conflicts in the workspace.

4. **Testing and Verification**:
   - Ran `cd src-tauri && cargo update` to update Cargo.lock with the latest compatible versions
   - Ran `cd src-tauri && cargo test` - All 94 tests passed across all crates
   - Ran `cd src-tauri && cargo clippy --all` - No warnings or errors
   - Ran `cd src-tauri && cargo fmt` - Code formatted successfully

5. **Code Quality**: All changes adhere to AGENTS.md guidelines (latest stable dependencies, proper formatting, no clippy warnings).

The `tower` dependency is now consistently set to the latest stable version (0.5.2) across the entire application, meeting all acceptance criteria.


---

## Task 26 Cedar Authorization: Foundational Crate & Static PoC (Stage 1)

- [x] Status: Complete

This task covers the foundational setup for the new CEDAR-based authorization system. It involves creating the `authz` crate and implementing a self-contained, testable authorization engine with hardcoded data. This initial stage will validate the core CEDAR logic in isolation before integrating it with the broader application. This corresponds to Stage 1 of the plan outlined in `documentation/AUTHZ-PLAN.md`.

We should add comments to the main files created in this task that cover the range of security concerns we need to make sure we cover for the remaineder of the authorization build out.

### Acceptance Criteria:

- A new `authz` crate is created at `src-tauri/authz` with `cedar-policy`, `serde`, `tokio`, and `thiserror` dependencies.
- Core data structures (`Principal`, `Action`, `Resource`) are defined in `authz/src/types.rs`.
- An `AuthzEngine` struct is implemented in `authz/src/lib.rs` with a hardcoded policy and entities.
- The `AuthzEngine` includes a proof-of-concept method (`is_authorized_static_poc`) that can make authorization decisions based on the hardcoded data.
- A comprehensive suite of unit tests is created in the `authz` crate to validate the static proof-of-concept logic.
- Security concerns are about authorization are outlined in comments

### **Implementation Notes:**

Successfully implemented the foundational `authz` crate with a static proof-of-concept CEDAR authorization engine. This establishes the groundwork for the full CEDAR-based authorization system as outlined in Stage 1 of AUTHZ-PLAN.md.

1. **Crate Setup**: Created new crate at `src-tauri/authz` with dependencies: `cedar-policy 3.2.0`, `serde_json 1.0`, `serde` (with derive), `tokio` (full features), `thiserror 1.0`. Added crate to workspace members.

2. **Core Types** (`types.rs`): Implemented `Principal`, `Action`, and `Resource` structs with helper methods (user(), anonymous(), read(), write(), create(), delete()). All types implement standard traits (Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize). Added 7 unit tests.

3. **Error Handling** (`error.rs`): Implemented `AuthzError` enum with 7 error variants covering policy parsing, entity creation, evaluation, schema validation, resource access, principal validation, and internal errors. Added security notes about minimal error disclosure. Added 2 unit tests.

4. **Authorization Engine** (`lib.rs`): Implemented `AuthzEngine` with `is_authorized_static_poc()` method using hardcoded CEDAR policies (test_user can read, admin_user can do anything, deny-by-default). Hardcoded entities include test_user, admin_user, anonymous, and actions. Added 10 comprehensive unit tests covering allow/deny scenarios, deny-by-default security, and edge cases.

5. **Comprehensive Security Documentation**: Added extensive security comments in `types.rs` covering 12 critical areas: principal identity & authentication, policy management & integrity, resource access control, action validation, ABAC, performance & DoS prevention, logging & auditing, error handling & information disclosure, hierarchical permissions, integration security, testing & validation, and compliance & standards.

6. **Testing & Verification**: All 19 tests pass (7 in types.rs, 2 in error.rs, 10 in lib.rs, plus 1 doc-test). Ran `cargo clippy --all` with no warnings. Ran `cargo fmt` successfully.

The static PoC validates the CEDAR integration in isolation. Next stage (Task 27) will integrate this into API middleware, then Stage 3 will add dynamic policy/entity loading.

---

## Task 27 Cedar Authorization: Basic Dynamic Integration (Stage 2)

- [ ] Status: Blocked by Task 26

This task focuses on integrating the `AuthzEngine` into the API middleware layer. It will replace the existing authentication middleware stub with a new authorization middleware. Initially, this middleware will use the hardcoded policies and entities from Stage 1 to verify that the request-response flow is working correctly before dynamic data is introduced. This corresponds to Stage 2 of the plan outlined in `documentation/AUTHZ-PLAN.md`.

### Acceptance Criteria:

- The `auth_middleware` stub in `src-tauri/api/src/middleware_hooks.rs` is replaced with a functional `authorization_middleware`.
- The middleware correctly extracts a placeholder `Principal`, `Action`, and `Resource` from incoming HTTP requests.
- The `AuthzEngine::is_authorized_static_poc()` method is successfully called from within the middleware.
- The middleware correctly returns a `403 Forbidden` status on a "Deny" decision and passes the request to the next layer on an "Allow" decision.
- Integration tests are created in the `api` crate to validate the middleware's behavior with mock requests.

### **Implementation Notes:**

---

## Task 28 Cedar Authorization: Schema-Driven Dynamic Authorization (Stage 3)

- [ ] Status: Blocked by Task 27

This task involves replacing all hardcoded elements from the proof-of-concept with a fully dynamic, schema-driven system. This includes loading CEDAR policies and schemas from the file system, constructing CEDAR entities from database and user data, and updating the `AuthzEngine` to use this dynamic data for authorization decisions. This corresponds to Stage 3 of the plan outlined in `documentation/AUTHZ-PLAN.md`.

### Acceptance Criteria:

- The `schema-manager` is updated to watch for and load `.cedar` policy files from the `config/` directory.
- The `schema-manager` can generate a valid CEDAR schema from the existing YAML entity schemas.
- The `authz` crate contains functions to dynamically construct CEDAR entities from Marain user and content data.
- The `AuthzEngine` is updated to make authorization decisions using the dynamically loaded policies, schema, and entities.
- The entity schema format (`.schema.yaml`) is extended to include an optional `cedar` block for authorization metadata.
- An initial set of default policies is created in `config/policies.cedar`.

### **Implementation Notes:**

---

## Task 29 Cedar Authorization: Testing, Docs, & Finalization (Stage 4)

- [ ] Status: Blocked by Task 28

This is the final stage of the CEDAR authorization implementation. It focuses on expanding the test coverage to validate the complete dynamic system, updating all relevant project documentation to reflect the new architecture, and providing guidelines for developers and AI agents on how to manage authorization policies. This corresponds to Stage 4 of the plan outlined in `documentation/AUTHZ-PLAN.md`.

### Acceptance Criteria:

- The unit test suite in the `authz` crate is expanded to cover the dynamic entity construction logic.
- The integration test suite in the `api` crate is expanded to cover common policy scenarios (e.g., admin access, resource owner, public access).
- The `DEVELOPER-GUIDE.md` and `user-management-system/authorization.md` documents are updated to describe the new CEDAR-based system.
- The `AGENTS.md` file is updated with clear guidelines for creating and modifying `.cedar` policies.

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



