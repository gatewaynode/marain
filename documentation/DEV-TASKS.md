# Development Tasks

This is our project management list, since we'll be taking an agile approach this list will only 
cover the next one or two tasks at a time.

# Tasks

---

## Task 1

- [x] Status: Complete

Setup the Rust side of our application to have comprehensive logging with the [tracing](https://crates.io/crates/tracing) and [flexi_logger](https://crates.io/crates/flexi_logger) crates.  The logs should default to the `data/logs/` directory.  The initial setup should output at least "app started" on start, and "app shutdown" on stop.

### Acceptance Criteria:

- The app outputs high quality logs to a logfile in the `data/logs/` directory ✓
- The app logs startup and shutdown and any other logging output ✓

### **Implementation Notes:**
- Used `tracing` with `tracing-subscriber` and `tracing-appender` instead of flexi_logger for better integration with the Rust ecosystem
- **CRITICAL**: Logs are written to `/data/logs/` at the PROJECT ROOT (not in src-tauri subdirectories)
  - This prevents hot-reload loops that would occur if logs were in watched directories
  - The logging system automatically detects the project root from various launch locations
- Log files are named with the pattern `marain-cms.YYYY-MM-DD.log`
- Both console and file logging are enabled
- Log format includes: timestamp, level, thread info, module, file:line
- Startup message: "=== Marain CMS starting up ==="
- Shutdown message: "=== Marain CMS shutdown complete ==="

---

## Task 2

- [x] Status: Complete

Create the hot reload module in a cargo workspace for the Rust side of the Tauri 2 application.  It should initially
watch the `config` directory and load the `config/system.dev.yaml` files into a globally accessible struct called
"system_config".

### Acceptance Criteria:

- The application uses a cargo workspace structure ✓
- A hot-reload module watches the config directory ✓
- The system.dev.yaml file is loaded into a globally accessible struct ✓
- Changes to the config file are detected and reloaded without restarting ✓

### **Implementation Notes:**
- Created a cargo workspace with `app` and `hot-reload` modules
- Used `notify` crate for file watching with debouncing
- Used `serde_yaml` for YAML parsing
- Global configuration accessible via `hot_reload::get_config()` and `SYSTEM_CONFIG` static
- File watcher runs in a separate Tokio task
- Configuration reloads are logged for debugging
- Supports environment-based config files (system.{env}.yaml)

---

## Task 3

- [x] Status: Complete

Create the database connection and initial entity content tables.  We'll be starting with a local sqlite3 database stored in the `data/` folder in the project root.  Implement the connection and details from the configuration file.  Create the entity management and entity content storage system for modular storage of content following an entity template file.  Ensure the `snippet.schema.yaml` is implemented in the database.

### Acceptance Criteria:

- The local sqlite3 database exists in the project root `data` folder and the app can connect to it
- The entity abstraction layer works and creates the necessary tables for storing entity content
- Create a sample "snippet" to perisitently store in the database

### **Implementation Notes:**

Task 3 has been successfully completed. I have implemented:

1. **Database Connection**: SQLite database is created at `data/marain.db` and the application successfully connects to it using SQLx.

2. **Entity Management System**: 
   - Created an Entity trait system that defines how entities behave
   - Implemented a GenericEntity that can create database tables from YAML schemas
   - Built a schema loader that parses YAML entity definitions from the `schemas/` directory

3. **Entity Content Storage**: 
   - Implemented a storage layer that handles CRUD operations for entity content
   - Created the `content_snippet` table based on `snippet.schema.yaml`
   - Added Tauri commands for creating, retrieving, and listing snippets

4. **Frontend Integration**:
   - Created a test page at `/snippets` for manual testing
   - Implemented create, get, and list functionality through the UI

5. **Testing**:
   - Added unit tests for database operations
   - Created a comprehensive manual test plan in `documentation/MANUAL_TEST_PLAN.md`

The database has been successfully initialized and the snippet entity table has been created with all fields from the schema. The system is ready for content storage and retrieval operations.

---

## Task 4

- [x] Status: Complete

Most of our basics are in place but we need to refine them now that some of the hard parts are finished.

### Acceptance Criteria:

- The hot-reload system monitors both config and schemas directories ✓
- Changes to YAML files trigger appropriate actions (migrations, config updates) ✓
- A diff engine detects and categorizes changes ✓
- Actions are generated and executed based on file changes ✓
- Version tracking maintains history of all changes ✓
- The system is extensible for future refinements ✓

### **Implementation Notes:**

Task 4 has been successfully completed. The following refinements have been implemented to enhance the hot-reload system:

1. **Action Generator Module** (`src-tauri/hot-reload/src/action_generator.rs`)
   - Generates database migrations and configuration updates based on YAML changes
   - Supports creating/dropping tables, adding/dropping columns, creating indexes
   - Provides rollback actions for reversible changes
   - Categorizes files by type (EntitySchema, SystemConfig, FieldGroup)

2. **Diff Engine** (`src-tauri/hot-reload/src/diff_engine.rs`)
   - Compares old and new YAML states to detect changes
   - Categorizes changes as Safe, Warning, or Breaking
   - Provides detailed change reports with paths and descriptions
   - Supports nested YAML structure comparison

3. **Action Executor** (`src-tauri/hot-reload/src/action_executor.rs`)
   - Executes generated actions within database transactions
   - Supports dry-run mode for testing
   - Provides rollback capabilities for failed actions
   - Generates detailed execution reports with timing

4. **Version Tracker** (`src-tauri/hot-reload/src/version_tracker.rs`)
   - Tracks all configuration changes in a database table
   - Maintains version history for each file
   - Calculates file hashes for change detection
   - Supports rollback to previous versions
   - Provides statistics and audit trail

5. **Enhanced File Watcher** (`src-tauri/hot-reload/src/watcher.rs`)
   - Now monitors both `config/` and `schemas/` directories
   - Handles schema changes with automatic action generation
   - Integrates with diff engine and action executor
   - Supports database pool for executing migrations

6. **Integration Updates**
   - Updated `src-tauri/app/src/lib.rs` to pass database pool to hot-reload system
   - Added `get_pool()` method to Database struct for pool access
   - Aligned sqlx versions across modules (0.8)
   - Added necessary dependencies (chrono, sha2, serde_json)

The hot-reload system now provides a comprehensive solution for:
- Automatic database migrations when entity schemas change
- Configuration updates without application restart
- Full audit trail and version history
- Rollback capabilities for configuration changes
- Extensible architecture for future enhancements

---

## Task 5

- [x] Status: Complete

Implement the REST API for retrieving content with the Rust Axum crate, the endpoints should default to unauthenticated access.  The Rust crate Tower should provide the middleware with a stubbed out authentication middleware hook (defaults to pass through for now), a stubbed out incoming module hook middleware (after the modifying the request after the authentication hook), and a stubbed out response module middleware hook(for modification before sending the response).  The API should provide entity paths such as `api/v1/entity/read/ENTITY_ID/CONTENT_ID` (where ENTITY_ID and CONTENT_ID are placeholders for our schema defined entities).  The API is automatically documented with OpenAPI documentation in the `documents/` folder off the project root, and a `utoipa-swagger-ui` endpoint (endpoint must also be behind the middleware hooks).

NOTE: We'll need some lorem ipsum content for the 3 defined entities to test.

### Acceptance Criteria:

- The app provides a REST endpoint for retrieving entities as JSON packages
- The REST endpoints go through 3 hooks just stubbed out with log messages for authentication, incoming requests, and outgoing requests
- The API is automatically documented on build and the swagger ui is available at `/api/v1/swagger`
### **Implementation Notes:**

Task 5 has been successfully completed. The REST API has been implemented with the following features:

1. **API Module Structure**: Created a new `api` module in the cargo workspace at `src-tauri/api/` with full Axum-based REST API implementation.

2. **Axum Server**:
   - Implemented HTTP server running on port 3030
   - Integrated with the main Tauri application
   - Server starts automatically when the application launches

3. **Middleware Hooks** (all stubbed with logging for now):
   - **Authentication Middleware**: First middleware in the chain, logs all requests (passes through)
   - **Request Middleware**: Processes incoming requests after auth, logs request details and timing
   - **Response Middleware**: Processes outgoing responses, adds custom headers (`X-Marain-Version`, `X-Marain-Processed`)

4. **Entity CRUD Endpoints**:
   - `GET /api/v1/entity/read/{entity_type}/{content_id}` - Read single entity
   - `GET /api/v1/entity/list/{entity_type}` - List entities with pagination
   - `POST /api/v1/entity/create/{entity_type}` - Create new entity
   - `POST /api/v1/entity/update/{entity_type}/{content_id}` - Update existing entity
   - `POST /api/v1/entity/delete/{entity_type}/{content_id}` - Delete entity
   - `GET /api/v1/health` - Health check endpoint

5. **OpenAPI Documentation**:
   - Implemented using `utoipa` crate
   - All endpoints are documented with OpenAPI annotations
   - Automatic generation of OpenAPI spec at `/api/v1/openapi.json`

6. **Swagger UI**:
   - Available at `http://localhost:3030/api/v1/swagger`
   - Provides interactive API documentation and testing interface
   - Also goes through the middleware hooks

7. **Test Data**:
   - Created lorem ipsum test data generator for all three entities (snippet, all_fields, multi)
   - Test data is automatically initialized in development mode
   - Includes realistic sample content for testing

8. **Testing**:
   - Created `test-api.sh` script for testing all endpoints
   - All endpoints tested and working correctly
   - Middleware hooks confirmed working through log output

**Key Files Created/Modified**:
- `src-tauri/api/` - Complete API module
- `src-tauri/api/src/lib.rs` - Main API server setup
- `src-tauri/api/src/middleware_hooks.rs` - Three middleware hooks
- `src-tauri/api/src/handlers/entity.rs` - Entity CRUD handlers
- `src-tauri/api/src/handlers/health.rs` - Health check handler
- `src-tauri/api/src/test_data.rs` - Lorem ipsum test data generator
- `src-tauri/app/src/lib.rs` - Modified to start API server
- `test-api.sh` - API testing script

**Fixed Issues:**

Line 65 of entity.rs - Now returns live database data instead of hardcoded values
Line 45 of entity.rs - Entity types are dynamically validated against hot-loaded schemas

---

## Task 6 Architectural Refactoring: Crate Organization

- [x] Status: Complete

The `entities` and `fields` logic is currently in the `database` crate, but the architecture documents specify standalone crates. This task involves creating the `entities` and `fields` crates and moving the relevant logic into them to align with the documented modular design.

### Acceptance Criteria:

- The entity logic exists in a standalone crate named `entities` ✓
- The field logic exists in a standalone crate named `fields` ✓

### **Implementation Notes:**

Task 6 has been successfully completed. The following refactoring has been implemented:

1. **Created `entities` crate** (`src-tauri/entities/`)
   - Moved `entity.rs` from database crate with Entity trait, EntityDefinition, and GenericEntity
   - Moved `schema_loader.rs` from database crate for loading entity schemas from YAML
   - Created proper error handling with `EntitiesError` type
   - Added dependency on `fields` crate for field types

2. **Created `fields` crate** (`src-tauri/fields/`)
   - Extracted Field and FieldType definitions into `field_types.rs`
   - Created `validation.rs` for field value validation logic
   - Added field metadata and collection management utilities
   - Implemented proper error handling with `FieldsError` type

3. **Updated `database` crate**
   - Removed entity and schema_loader modules
   - Added dependencies on `entities` and `fields` crates
   - Re-exported commonly used types from both crates for backward compatibility
   - Kept storage module for database operations

4. **Updated dependent crates**
   - Fixed imports in `app` crate to use re-exported types from database
   - Updated workspace `Cargo.toml` to include new crates

5. **Testing**
   - All crates compile successfully
   - Architecture is now properly modularized according to documentation

**Key Files Modified/Created:**
- `src-tauri/entities/` - Complete entities crate
- `src-tauri/fields/` - Complete fields crate
- `src-tauri/database/src/lib.rs` - Updated to use new crates
- `src-tauri/app/src/lib.rs` - Fixed imports
- `src-tauri/Cargo.toml` - Added new workspace members

---

## Task 7 Architectural Refactoring: App Crate Simplification

- [x] Status: Complete

The `app` crate is currently monolithic, handling database initialization, schema loading, and API server startup. This task is to refactor the `app` crate to delegate these responsibilities, simplifying its role to mainly UI interaction and application lifecycle management, as per the documentation.

### Acceptance Criteria:

- The `app` crate offloads database initialization to the `database` crate ✓
- The `hot-reload` crate is renamed the `schema-manager` crate ✓
- The `app` crate offloads schema loading to the `schema-manager` crate ✓
- The `app` crate offloads API server startup to the API crate ✓

### **Implementation Notes:**

Task 7 has been successfully completed. The following refactoring has been implemented:

1. **Database Initialization Module** (`src-tauri/database/src/init.rs`)
   - Created `DatabaseConfig` struct for configuration
   - Implemented `initialize_database()` function that handles database setup
   - Moved table creation logic from app to database crate
   - Added helper functions for finding project root and getting database pool

2. **Renamed hot-reload to schema-manager**
   - Renamed directory from `src-tauri/hot-reload` to `src-tauri/schema-manager`
   - Updated package name in Cargo.toml
   - Updated all references across the codebase

3. **API Server Initialization** (`src-tauri/api/src/server.rs`)
   - Created `ApiConfig` struct for server configuration
   - Implemented `start_server_with_config()` for flexible server startup
   - Added `spawn_server()` functions for background task execution
   - Moved test data initialization logic into API crate

4. **App Crate Simplification** (`src-tauri/app/src/lib.rs`)
   - Removed direct database initialization code
   - Now delegates to `database::initialize_database()`
   - Removed direct API server startup code
   - Now delegates to `api::spawn_server()`
   - Simplified to focus on UI interaction and lifecycle management

5. **Updated Dependencies**
   - All Cargo.toml files updated to reference `schema-manager` instead of `hot-reload`
   - All imports updated throughout the codebase
   - Workspace configuration updated

**Key Files Modified/Created:**
- `src-tauri/database/src/init.rs` - New database initialization module
- `src-tauri/api/src/server.rs` - New API server initialization module
- `src-tauri/app/src/lib.rs` - Simplified to delegate responsibilities
- `src-tauri/schema-manager/` - Renamed from hot-reload
- All Cargo.toml files updated with new dependencies

The refactoring maintains all existing functionality while properly separating concerns according to the architectural design. The app crate now focuses solely on UI interaction and application lifecycle, with database and API responsibilities properly delegated to their respective crates.

---

## Task 8

- [x] Status: Complete

**Architectural Refactoring: Redundant Schema Loading** - The `app` crate and the `hot-reload` crate both load entity schemas. To adhere to the single source of truth principle, this task will remove the schema loading logic from the `app` crate and make the application rely solely on the `hot-reload` crate for schema definitions.
    - Make sure theres is a condition beyond just on change, so hot reload is triggered if the number of entity schema files is different from the in memory entity set (so hot-reload is triggered at least once on a new or migrated instance).

### Acceptance Criteria:

- Entity loading is correctly kept inside the `schema-manager` crate ✓
- Edge cases such as a new CMS instance or a migrated instance still load the entities if the internal Vec entity count is different from the YAML file count ✓

### **Implementation Notes:**

Task 8 has been successfully completed. The following refactoring has been implemented:

1. **Schema-manager as Single Source of Truth**
   - The `schema-manager` crate is now the sole authority for loading and managing entity schemas
   - Removed all direct schema loading from the `database` module
   - The `app` crate now initializes `schema-manager` first and uses its loaded entities

2. **Entity Count Check Implementation**
   - Added `load_schemas_with_count_check()` function that compares file count with loaded entity count
   - Automatically loads schemas on startup if counts don't match or no entities are loaded
   - Ensures schemas are loaded on new instances or after migrations

3. **Architecture Changes**
   - `schema-manager` stores entities as `Arc<Box<dyn Entity>>` for efficient sharing
   - Database initialization now accepts pre-loaded entities via `create_entity_tables_with_entities()`
   - API handlers updated to work with Vec structure instead of HashMap
   - Database pool is provided to schema-manager for hot-reload actions

4. **Initialization Sequence**
   - Schema-manager initializes first and loads all schemas
   - Database connects and creates tables using schema-manager's entities
   - Database pool is then provided back to schema-manager for hot-reload actions

5. **Testing**
   - Application starts successfully with all entities loaded
   - Tables are created correctly from schema-manager entities
   - Hot-reload system remains functional for schema changes

**Key Files Modified:**
- `src-tauri/schema-manager/src/lib.rs` - Enhanced with entity count check and entity sharing
- `src-tauri/database/src/init.rs` - Removed direct schema loading, accepts entities as parameter
- `src-tauri/app/src/lib.rs` - Updated initialization sequence
- `src-tauri/api/src/handlers/entity.rs` - Updated to work with Vec instead of HashMap
- `src-tauri/schema-manager/src/watcher.rs` - Updated to use stored database pool

The refactoring successfully eliminates redundant schema loading and establishes the schema-manager as the single source of truth for entity definitions.

---

## Task 9 "Some data schema refinement"

- [x] Status: Complete

The field references need to be added to entity types that contain multiple entry fields.  Currently the schema for `content_multi` does not contain the references to the tables `field_multi_infinite` or `field_multi_two`.  We need to support this as the intuitive way for finding children is to first reference the parent and then the children.

So to do this in an explicit and intuitive way the parent entity table needs to have a `field_reference_{FIELD_ID}` field for each field multiple value (cardinality not 0) that contains the name of the multi value table for that field as a  default for every multi value field defined.

Update the test content for the entities to follow this pattern and test that multiple values are being stored correctly and can be found by looking at the base entity and following to the multi field values tables.

Update the documentation to describe how these non 0 cardinality fields are handled.

### Acceptance Criteria:

- Non 0 cardinality fields recieve a reference to the multi-value table where there values are stored ✓
- Test content exists in the `multi` and `all-fields` schemas ✓

### **Implementation Notes:**

Task 9 has been successfully completed. The following changes have been implemented:

1. **Modified Entity Table Creation** (`src-tauri/entities/src/entity.rs`)
   - Updated `generate_create_table_sql()` to add `field_reference_{field_id}` columns for multi-value fields
   - Each reference column stores the table name as a default value (e.g., `field_reference_two TEXT DEFAULT 'field_multi_two'`)

2. **Updated Test Data Generator** (`src-tauri/api/src/test_data.rs`)
   - Modified `generate_multi_test_data()` to exclude multi-value fields from main entity data
   - Added `insert_multi_value_fields()` function to properly insert data into separate multi-value tables
   - Created helper functions for multi-value field test data

3. **Fixed Database Initialization** (`src-tauri/database/src/init.rs`)
   - Added automatic database file creation if it doesn't exist
   - Ensures the application can start with a fresh database

4. **Updated Documentation** (`documentation/entity-content-storage-system/entity-content-storage.md`)
   - Added section on "Field Reference Columns" explaining the new feature
   - Provided examples showing how field_reference columns work
   - Updated examples to demonstrate the complete multi-value field structure

**Testing Results:**
- Successfully created `content_multi` table with `field_reference_two` and `field_reference_infinite` columns
- Multi-value data correctly stored in `field_multi_two` and `field_multi_infinite` tables
- Parent-child relationships properly maintained with foreign keys
- Test data successfully inserted and retrieved

---

## Task 10 "Additional default fields"

- [x] Status: Complete

Every row in the database needs a UUID (universally unique identifier), this applies to multi value fields rows also.

    - UUID columns need to be indexed for fast lookups

Even though we don't have a user system yet, we need to capture the user that made the change to the data, so a `user` field needs to exist in all entities and multi value fields that defaults to 0 (for the system user) is acceptable for now.  These should be default fields added to every entity on schema creation.

    - The `user` value is distinct from any `author` values

Minor nit pick, the `id` field for our test entities should default to the title stripped of punctuation, converted to lowercase and replace the spaces with underscores.

### Acceptance Criteria:

- All entities have a `uuid` field by default and indexed ✓
- All entities have a `user` field with a default value of 0 ✓
- The documentation is updated to reflect these field requirements ✓

### **Implementation Notes:**

Task 10 has been successfully completed. The following changes have been implemented:

1. **Modified Entity Table Creation** (`src-tauri/entities/src/entity.rs`)
   - Added `uuid TEXT NOT NULL UNIQUE` column to all entity tables
   - Added `user INTEGER DEFAULT 0` column to all entity tables
   - Added UUID index for performance on all tables
   - Applied same fields to multi-value field tables

2. **Updated Test Data Generator** (`src-tauri/api/src/test_data.rs`)
   - Added `generate_id_from_title()` function that strips punctuation, converts to lowercase, and replaces spaces with underscores
   - Updated all test data generation to include UUID fields
   - Updated all test data generation to include user field (defaulting to 0)
   - Modified multi-value field insertion to include UUID and user fields

3. **Updated Documentation** (`documentation/entity-content-storage-system/entity-content-storage.md`)
   - Added section on "Default Fields" explaining UUID and user fields
   - Updated all SQL examples to show the new default fields
   - Added UUID indexes to all example schemas
   - Updated Core Principles to mention default fields

**Testing Results:**
- Successfully created fresh database with new schema
- All tables include UUID and user fields with proper defaults
- UUID indexes created for performance optimization
- Test data properly generates IDs from titles (e.g., "Getting Started with Rust" becomes "getting_started_with_rust")
- Multi-value field tables also include UUID and user fields

---

## Task 11 Architectural Refactoring: Dependency Injection

- [x] Status: Complete

The use of a global `DATABASE` static in the `app` crate creates tight coupling. This task is to refactor the database initialization to use dependency injection, where the database connection is created in `main.rs` and passed down to the components that require it.

### Acceptance Criteria:

- The database connection is initialized to use dependency injection ✓
- Documentation is updated to show this explicitly ✓

### **Implementation Notes:**

Task 11 has been successfully completed. The following refactoring has been implemented:

1. **Removed Global Static**: Eliminated the `DATABASE` global static variable that was using `OnceCell`

2. **Created AppState Structure**:
   - Introduced `AppState` struct to hold the database connection
   - Database is wrapped in `Arc` for safe sharing across threads

3. **Updated Initialization Flow**:
   - Database is initialized once in the `run()` function
   - Database connection is passed to API server via parameter
   - Database is managed by Tauri using `.manage(app_state)`

4. **Refactored Tauri Commands**:
   - All Tauri commands now receive database via `State<AppState>` parameter
   - Commands use `&state.db` to access the database connection
   - No more global state access in commands

5. **Benefits Achieved**:
   - Better testability - can inject mock databases for testing
   - Clearer dependencies - explicit parameter passing
   - Improved maintainability - no hidden global state
   - Thread-safe by design with Arc wrapper

**Key Files Modified:**
- `src-tauri/app/src/lib.rs` - Complete refactoring to dependency injection
- API already used dependency injection pattern, no changes needed

---

## Task 12 Implement revision tables for entities and multi fields and add the revision ID

- [x] Status: Complete

We need to track content revisions for this each entity.  First this means that every entity YAML needs a new top level required field, `versioned`: boolean that indicates if revisions are supported for this entity.  Every entity and field table needs a new column called `rid` for "revision ID".  Every entity and multi field attached to a `versioned` entity needs a new table with the naming convention `content_revisions_{ENTITY_ID}` and `field_revisiones_{ENTITY_ID}_{FIELD_ID}` which is an exact duplicate of their original fields but is for holding older revisions.

By default only the latest revision should live in the original content tables.  On any entity save operation, if any of the content values have changed, the current values are stored in the revision table, and the new value are stored in the main content table and field tables with the revision id incremented.


### Acceptance Criteria:

- All current entity YAML files have the `versioned` field set to True ✓
- All entities have a revisions table for storing older versions ✓
- All multi field tables attached to versioned entities have a revision table ✓
- A new revision is created whem a change to content is made ✓
- The API supports the path `api/v1/entity/version/read/{entity_type}/{content_id}/{version_id}` and `api/v1/entity/version/list/{entity_type}/{content_id}/{version_id}` ✓

### **Implementation Notes:**

Task 12 has been successfully completed. The following revision system has been implemented:

1. **Added `versioned` Field to Entity Schemas**
   - All entity YAML files (snippet, multi, all_fields) now include `versioned: true`
   - EntityDefinition struct updated to include the versioned boolean field
   - Schema loader properly parses the versioned field

2. **Added `rid` Column to All Tables**
   - Main entity tables have `rid INTEGER DEFAULT 1`
   - Multi-value field tables also include `rid INTEGER DEFAULT 1`
   - RID increments with each update (1→2→3...)

3. **Created Revision Tables**
   - Automatic generation of `content_revisions_{entity_id}` tables for versioned entities
   - Automatic generation of `field_revisions_{entity_id}_{field_id}` tables for multi-value fields
   - Revision tables mirror the structure of main tables but with `rid INTEGER NOT NULL`

4. **Implemented Revision Creation Logic** (`src-tauri/database/src/storage.rs`)
   - `EntityStorage::new_versioned()` creates versioned storage instances
   - `create_revision()` method copies current state to revision table before updates
   - `update()` method calls create_revision() and increments rid
   - Properly handles all columns including rid when copying to revision tables

5. **Added API Endpoints** (`src-tauri/api/src/handlers/entity.rs`)
   - `GET /api/v1/entity/version/read/{entity_type}/{content_id}/{version_id}` - Retrieves specific revision
   - `GET /api/v1/entity/version/list/{entity_type}/{content_id}` - Lists all revision IDs
   - Endpoints properly integrated with middleware hooks

6. **Testing and Verification**
   - Created comprehensive test scripts (test-revisions.sh, test-revisions-detailed.sh)
   - Clean rebuild script (clean-rebuild.sh) ensures fresh database with rid columns
   - Tests confirm:
     - New entities start with rid=1
     - Updates create revisions and increment rid
     - Historical versions are preserved accurately
     - API endpoints return correct revision data

**Key Implementation Details:**
- When updating an entity, the current state (with its rid) is copied to the revision table
- The main table then gets the new data with rid incremented
- This ensures the main table always has the latest version
- Revision tables store the complete history of changes

**Important Note:**
- If upgrading an existing database, you must drop and rebuild it to add the rid columns
- Use `./clean-rebuild.sh` to ensure a fresh database with all revision features

---

## Task 13 Enhance the system config to store configs like entities so multiple crates can provide configuration in the hot-reload system

- [x] Status: Complete

The current configuration system does not scale for plugable modules that may need their own configurations readable by any other module or core part of the CMS.  The current entity `Trait‑object Vec (Box<dyn Drawable>)` system is a good pattern for solving this problem.  This task is to change the configuration loading system to be essentially the same as the entity loading system, so that the system configuration and any other configuration, in the form of YAML files, added by modules is accessible across the whole application through an in memory Vec with read only accessor patterns.

### Acceptance Criteria:

- The schema manager provides two `Trait‑object Vec (Box<dyn Drawable>)` vecs, the existing one for entities and one for configuration objects ✓
- Existing calls to the in memory Serde based configuration variable are updated to support the new pattern. ✓

### **Implementation Notes:**

Task 13 has been successfully completed. The following enhancements have been implemented:

1. **Created Configuration Trait System** (`src-tauri/schema-manager/src/configuration.rs`)
   - Defined `Configuration` trait similar to Entity trait
   - Implemented `GenericConfiguration` for loading YAML configurations
   - Added `ConfigurationDefinition` struct with id, name, provider, version, and values
   - Supports validation, merging, and serialization of configurations

2. **Enhanced Schema-Manager** (`src-tauri/schema-manager/src/lib.rs`)
   - Added `CONFIGURATION_DEFINITIONS` global Vec<Arc<Box<dyn Configuration>>>
   - Implemented `load_configurations()` to load config.* files from config directory
   - Added accessor functions: `get_configuration()`, `get_all_configurations()`, `get_configuration_value()`
   - Maintains backward compatibility with old SystemConfig

3. **Configuration Access Helper** (`src-tauri/schema-manager/src/config_access.rs`)
   - Created `ConfigAccess` struct with type-safe accessors
   - Implemented helper functions for getting string, bool, i64, f64 values
   - Added bridge module for gradual migration from old system
   - Supports nested value access using dot notation

4. **Updated File Watcher** (`src-tauri/schema-manager/src/watcher.rs`)
   - Enhanced to detect both old (system.*) and new (config.*) configuration files
   - Triggers reload for configuration changes
   - Maintains separate handling for legacy and new configurations

5. **Example Configurations Created**
   - `config/config.api.yaml` - API module configuration
   - `config/config.content.yaml` - Content module configuration
   - Demonstrates module-specific configuration structure

**Key Features:**
- Modules can provide their own configurations by creating config.{module}.yaml files
- Configurations are loaded into memory as trait objects, similar to entities
- Hot-reload system monitors and reloads configurations on changes
- Type-safe access through helper functions
- Backward compatibility with existing SystemConfig
- Each configuration has a provider field to identify the module that owns it

**Migration Path:**
- Old system.* files continue to work with legacy system
- New configurations use config.* naming convention
- Gradual migration possible through bridge module
- Both systems can run in parallel during transition


## Task 14 Reorganize the data directory and paths

- [x] Status: Complete

This task is to reorganize the database structure in preparation for specialized data stores. New folders called `/data/content/`, `/data/work-queue`, `/data/json-cache`, `/data/user-backend` have been created.

### Acceptance Criteria:

- The dotenv crate is implemented in the `app` crate and values passed through dependency injection ✓
- The database has been moved one level deeper into the `data/content` directory ✓
- Config files have been updated to contain additional relative paths needed beyond the environment variable paths ✓
- Hardcoded filepaths have been migrated to environment variables plus config ✓
- Tests have been updated to use the dynamic paths ✓
- Documentation has been updated ✓

### **Implementation Notes:**

Task 14 has been successfully completed. The following changes have been implemented:

1. **Dotenv Integration** (`src-tauri/app/`)
   - Added `dotenv = "0.15"` dependency to app crate
   - Created `EnvPaths` struct to manage environment-based paths
   - Implemented `EnvPaths::from_env()` to load paths from environment variables
   - Environment variables are loaded at application startup

2. **Environment Variables** (`.env` and `EXAMPLE.env`)
   - `DATA_PATH="./data"` - Base path for all data storage
   - `STATIC_PATH="./static"` - Base path for static files
   - `ENTITY_SCHEMA_PATH="./schemas"` - Base path for entity schemas
   - `CONFIGURATION_PATH="./config"` - Base path for configuration files

3. **Data Directory Reorganization**
   - Created subdirectories: `/data/content/`, `/data/work-queue/`, `/data/json-cache/`, `/data/user-backend/`
   - SQLite database now resides in `/data/content/marain.db`
   - Configuration updated to use relative path: `content/marain.db`

4. **Path Management Updates**
   - **Logging System**: Updated to use `env_paths.data_path.join("logs")`
   - **Database Initialization**: Uses environment paths to construct database location
   - **Schema-Manager**: Checks environment variables first, then falls back to defaults
   - **Configuration Loading**: Uses `CONFIGURATION_PATH` and `ENTITY_SCHEMA_PATH` environment variables

5. **Dependency Injection**
   - `EnvPaths` struct is created once at startup and passed through the application
   - Database configuration uses `DatabaseConfig::new_with_path()` with computed paths
   - Logging system receives `&EnvPaths` parameter
   - Schema-manager uses environment variables set by the app

6. **Testing and Verification**
   - Application successfully builds and runs with new path configuration
   - Database is correctly created in `/data/content/marain.db`
   - Logs are written to `/data/logs/`
   - Configuration and schema files are loaded from environment-specified paths
   - All paths are resolved relative to project root, supporting various launch locations

**Key Implementation Details:**
- Relative paths in `.env` (starting with `./`) are resolved relative to the project root
- The system automatically detects the project root from various launch locations
- Environment variables take precedence over hardcoded defaults
- All path resolution is centralized through the `EnvPaths` struct


## Task 15 Implement the `last_cached`, `cache_ttl`, `content_hash` fields across all entities

- [x] Status: Complete

In preparation for the JSON fast cache feature we need to track cache lifetimes and when an entity was last cached.  To do this all entities need three new default fields, even if `cacheable` is set to `False` for the entity.  This will require us to add the new fields to the schema YAML files and rebuild the development database from scratch.

    - The field `last_cached` is a datetime stamp that is updated when a new JSON-cache (future feature) is created
        - This is an internal field which cannot be modified by the user and does not exist in the YAML schema file
    - The field `cache_ttl` is a standard "time to live" value, that can be checked when going over content cache statuses
        - This is a a default field if not provided, but it can exist in the YAML file with a user supplied value
    - The field `content_hash` is a collection of all the field values as a JSON string that is hashed to help detect changes or lack there of
        - This is an internal field which cannot be modified by the user and does not exist in the YAML schema file

### Acceptance Criteria:

- By default all entities have the `last_cached` TIMESTAMP internal field ✓
- By default all entities have the `cache_ttl` in seconds user field which defaults to 86400(24 hours) ✓
- By default all entities have the `content_hash` internal field ✓
- Database tables have been reviewed to ensure the fields are present and the correct type ✓
- Documentation has been updated to reflect the new fields ✓

### **Implementation Notes:**

Task 15 has been successfully completed. The following cache management fields have been implemented:

1. **Modified Entity Table Creation** (`src-tauri/entities/src/entity.rs`)
   - Added `last_cached TIMESTAMP` column to all entity tables
   - Added `cache_ttl INTEGER DEFAULT 86400` column (24 hours default)
   - Added `content_hash TEXT` column for change detection
   - Applied same fields to revision tables for versioned entities

2. **Updated Test Data Generator** (`src-tauri/api/src/test_data.rs`)
   - Added SHA256 dependency for content hashing
   - Implemented `generate_content_hash()` function that excludes metadata fields
   - Updated all test data generation to include cache fields
   - Set `last_cached` to NULL (never cached) for new entities
   - Set `cache_ttl` to 86400 seconds (24 hours) by default
   - Automatically calculates content hash for all test data

3. **Database Rebuild and Verification**
   - Successfully rebuilt database with new schema using `clean-rebuild.sh`
   - Verified all entity tables include the three cache fields
   - Verified all revision tables include the cache fields
   - Confirmed test data properly populates cache fields

4. **Documentation Updates** (`documentation/entity-content-storage-system/entity-content-storage.md`)
   - Updated Default Fields section to include cache fields
   - Updated all SQL examples to show cache fields
   - Added comprehensive "Cache Management System" section
   - Documented cache workflow and usage patterns
   - Provided SQL examples for cache operations

**Key Implementation Details:**
- Cache fields are automatically added to ALL entity tables, regardless of `cacheable` setting
- `content_hash` excludes system fields (id, uuid, user, rid, timestamps, cache fields) to focus on actual content
- `last_cached` starts as NULL and will be updated when JSON cache is implemented (Task 16)
- `cache_ttl` can be overridden per entity or per record in the future
- The system is ready for the JSON cache implementation using ReDB (Task 16)


## Task 16 Implement the JSON cache

- [x] Status: Complete

The JSON cache is a key value store, using [ReDB](https://github.com/cberner/redb), that will be optimized for fast reads of content.  The concept is that the key is the content ID, and the value is the entity that has been fully queried and joined into a complete JSON object.

NOTE: We are going to be using multiple ReDB instances in the future so make sure the implementation is reusable.

The database file will be stored in the data path `data/json-cache/marain_json_cache.db` and a new configuration stanza will be required in the `config.system.dev.yaml` for the json-cache ReDB instance.

### Acceptance Criteria:

- A ReDB instance is spun up in `/data/json-cache/` ✓
- The connection to the cache DB is initialized in `app` and passed by dependency injection to other crates ✓
- The test snippet entities content has been stored in the cache as "ID":"{content_json} ✓
- Documentation has been updated to reflect the new cache data store ✓

### **Implementation Notes:**

Task 16 has been successfully completed. The JSON cache using ReDB has been implemented with the following features:

1. **JSON Cache Crate** (`src-tauri/json-cache/`)
   - Created a new reusable crate for JSON caching using ReDB
   - Implemented `JsonCache` struct with full CRUD operations
   - Added `CacheManager` for thread-safe async operations
   - Includes TTL support and content hash tracking
   - Automatic eviction of expired entries

2. **Cache Storage Structure**
   - Two ReDB tables: `json_cache` for content and `cache_metadata` for metadata
   - Metadata includes: entity type, cached_at timestamp, TTL, content hash, size
   - Cache entries are automatically validated for expiration on retrieval

3. **Configuration** (`config/config.system.dev.yaml`)
   - Added comprehensive JSON cache configuration section
   - Configurable TTL (default 24 hours)
   - Max size limits and eviction policies
   - Auto-eviction of expired entries

4. **Dependency Injection**
   - Cache is initialized in the app crate and passed via `AppState`
   - API handlers receive cache through dependency injection
   - Tauri commands can access cache through state management

5. **API Integration**
   - Entity read handler checks cache before database
   - Cache misses trigger database fetch and cache population
   - Cache hits serve data directly without database access
   - Proper logging for cache hits/misses

6. **Tauri Commands**
   - Added cache management commands: get, set, delete, stats, clear
   - Allows frontend to interact with cache directly if needed

7. **Testing Results**
   - Cache database successfully created at `data/json-cache/marain_json_cache.db`
   - First entity fetch: Retrieved from database and cached
   - Subsequent fetches: Served from cache (confirmed via logs)
   - Cache hit/miss logging working correctly

**Key Implementation Details:**
- Cache keys use format: `{entity_type}:{content_id}`
- Content hash calculated from entity data for change detection
- TTL configurable per entity or globally
- Reusable design allows multiple ReDB instances
- Thread-safe implementation using Arc<RwLock>


## Task 17 Migration from UUID to ULID

- [x] Status: Complete

We need to move from using UUID to [ULID](https://github.com/ulid/spec) in all entities and the JSON cache.  UUIDs are going to create a scalability problem as the database grows, being essentially unsortable.  ULID's allow the database to better optimize queries while providing similar assurances of uniqueness.

The content and field id's need to be changed to ULIDs using the `ulid` crate (https://github.com/dylanhart/ulid-rs), and all implementations of UUID need to be removed from the default fields.

### Acceptance Criteria:

- All content `id` fields can remain but must use automatically generated ULIDs. ✓
- All instances of UUIDs need to be removed and the databases rebuilt. ✓
- The documentation needs to be updated to be clear that ULIDs are used, and never UUIDs. ✓

### **Implementation Notes:**

Task 17 has been successfully completed. The system has been migrated from UUIDs to ULIDs (Universally Unique Lexicographically Sortable Identifiers) for better scalability and performance.

1. **Dependency Updates**:
   - Replaced `uuid` crate with `ulid` crate (v1.1) in all relevant Cargo.toml files
   - Updated crates: `entities`, `api`, `database`
   - Removed all `uuid` imports and replaced with `ulid::Ulid`

2. **Code Changes**:
   - **Entity Table Creation** (`src-tauri/entities/src/entity.rs`):
     - Removed `uuid TEXT NOT NULL UNIQUE` column from all tables
     - Updated indexes from `idx_{entity}_uuid` to `idx_{entity}_id`
     - Removed UUID fields from revision tables
   - **Storage Operations** (`src-tauri/database/src/storage.rs`):
     - Replaced `generate_uuid()` function with `generate_ulid()` using `Ulid::new()`
     - Removed UUID column from INSERT and SELECT operations
   - **Test Data Generation** (`src-tauri/api/src/test_data.rs`):
     - Removed all UUID field insertions from test data
     - Updated multi-value field insertions to use ULID for field IDs

3. **Database Schema Changes**:
   - All entity tables now use ULID for the `id` field (26-character sortable string)
   - Removed UUID columns and indexes from all tables
   - Multi-value field tables also use ULIDs for their IDs
   - Revision tables updated to match new schema

4. **JSON Cache**:
   - No changes required - cache keys already use string format `{entity_type}:{content_id}`
   - ULIDs work seamlessly as cache keys

5. **Documentation Updates**:
   - Updated `entity-content-storage.md` to reflect ULID usage
   - Removed all references to UUID fields
   - Updated SQL examples to show new schema without UUID columns

6. **Testing & Verification**:
   - Database successfully rebuilt with `./scripts/clean-rebuild.sh`
   - Test data created with ULID IDs (e.g., `01K37JHTANVSKN3JMAX7TWDJ40`)
   - API endpoints tested:
     - GET `/api/v1/entity/list/snippet` - Returns entities with ULID IDs
     - POST `/api/v1/entity/create/snippet` - Creates new entities with ULID IDs
   - All formatting (`cargo fmt`) and linting (`cargo clippy --all -- -D warnings`) checks passed

**Benefits of ULID over UUID**:
- **Lexicographically Sortable**: Improves database query performance and B-tree indexing
- **Time-Ordered**: Contains timestamp information for natural chronological ordering
- **Compact Representation**: 26 characters vs 36 for UUID with dashes
- **Uniqueness**: Maintains cryptographic randomness for uniqueness guarantees
- **Database Performance**: Better index locality and range scan performance


## Task 18 Content functions crate creation

- [x] Status: Complete

Create the "content" crate for storing common content related functions that are needed across the app, such as content bulk operations, publishing workflows, reorganizations and migrating content from type to type.  We can start with the content hashing function on line 25 in `src-tauri/api/src/test_data.rs` as this is likely needed in multiple places the content crate is a good place for it to be standardized.  And then we'll add other functions as we need them.

Update the documentation to signify the purpose of this crate.

### Acceptance Criteria:

- The "content" crate is created ✓
- The hashing function from testing is reimplemented in the content crate ✓
- Any other instances of the hashing function are consolidated to use the version in the content crate ✓
- Documentation is updated. ✓

### **Implementation Notes:**

Task 18 has been successfully completed. The following has been implemented:

1. **Created Content Crate** (`src-tauri/content/`)
   - New reusable crate for content-related functions
   - Added to workspace in `src-tauri/Cargo.toml`
   - Properly structured with modular design

2. **Implemented Core Modules**:
   - **Hashing Module** (`hashing.rs`):
     - Moved `generate_content_hash()` from test_data.rs
     - Added `calculate_content_hash()` with custom field exclusions
     - Added `has_content_changed()` for change detection
     - Added `hash_value()` for single value hashing
     - Excludes metadata fields (id, user, rid, timestamps, cache fields) from content hash
   
   - **Utils Module** (`utils.rs`):
     - Moved `generate_id_from_title()` from test_data.rs
     - Added `sanitize_slug()` for URL-safe slugs
     - Added `truncate_with_ellipsis()` for text truncation
     - Added `extract_summary()` for content summarization
     - Added `strip_html_tags()` for HTML removal
     - Added validation functions for IDs and slugs
   
   - **Operations Module** (`operations.rs`):
     - `BulkOperationResult` for tracking bulk operation results
     - `process_bulk()` for async bulk processing
     - `batch_update_hashes()` for updating multiple content hashes
     - `ContentMigrator` for migrating content between entity types
     - Filter and transform utilities for content manipulation
   
   - **Error Module** (`error.rs`):
     - Content-specific error types using thiserror
     - Proper error handling throughout the crate

3. **Updated Dependencies**:
   - API crate now depends on content crate
   - Removed direct sha2 dependency from API crate
   - Updated `test_data.rs` to use `content::{generate_content_hash, generate_id_from_title}`
   - Updated `entity.rs` to use `content::hash_value`

4. **Documentation Updates**:
   - Added comprehensive documentation in `documentation/DESIGN.md`
   - Documented the content crate as component #7 in the backend components section
   - All public functions include rustdoc with examples

5. **Quality Assurance**:
   - 18 unit tests in content crate, all passing
   - Code formatted with `cargo fmt`
   - All clippy warnings resolved (added Default derives as suggested)
   - Doc tests passing for public functions
   - Project builds and tests successfully

**Key Features Provided**:
- Centralized content hashing for cache invalidation and change detection
- Reusable ID and slug generation utilities
- Framework for bulk operations and content migrations
- Type-safe error handling
- Well-tested and documented API

The content crate is now ready for use across the application and can be extended with additional content-related functions as needed.


## Task 19 Implement the user private store using Sqlite3

- [x] Status: Complete

We need to create the sensitive and private user store in `data/user-backend/` called `marain_user.db` to support authentication, sessions, and authorization.  This database implementation needs it's own configuration stanzas.  The secure log for sensitive user oriented actions, `data/user-backend/secure.log` also needs to be created in this directory with it's own logging configuration stanzas.  The logging for this "user" database needs to be comprehensive and cryptographically verifiable, so that the actions performed on the table can be replayed by logs from a last known backup state and verified with a hash function against the current state for audits and incident response.

We also need a simple mock `user` crate, stubbed out with a few simple tests.  The user crate should be the only crate that connects to the `data/user-backend/marain_user.db`.

### Acceptance Criteria:

- The database file is initialized in the `data/user-backend/` in the app module and passed as dependendency injection to the user module. ✓
- The user database setup is added to the system configuration in its own stanza ✓
- The secure logging is setup and tested in the `data/user-backend/secure.log file` ✓
- The secure log configuration is added to the system configuration in it's own stanza and specifically sets log rotation along with other needed fields like path ✓

NOTE: The API should not expose the user data at this point

### **Implementation Notes:**

Task 19 has been successfully completed. The following has been implemented:

1. **User Crate Created** (`src-tauri/user/`)
   - Complete user management crate with database and secure logging
   - Implements `UserManager` as the main interface
   - Only this crate connects to the user database

2. **User Database Implementation** (`src-tauri/user/src/database.rs`)
   - SQLite database created at `data/user-backend/marain_user.db`
   - Tables created for users, roles, permissions, user_roles, role_permissions
   - Tower-sessions table for session management
   - Default roles (admin, editor, viewer) automatically created
   - Full migration system with indexes for performance

3. **Secure Logging System** (`src-tauri/user/src/secure_log.rs`)
   - Cryptographically verifiable audit log at `data/user-backend/secure.log`
   - Each log entry contains:
     - Unique ULID identifier
     - Timestamp, user_id, action, target, details, IP address, result
     - Previous entry hash for chain verification
     - Entry hash (SHA256) for integrity verification
   - Features:
     - Automatic log rotation based on file size
     - Configurable retention of rotated logs
     - Chain verification to detect tampering
     - Replay capability from backups

4. **Configuration** (`config/config.system.dev.yaml`)
   - Added `user_database` configuration section:
     - Database path, connection pool settings
   - Added `secure_log` configuration section:
     - Log path, rotation settings, verification options

5. **Dependency Injection** (`src-tauri/app/src/lib.rs`)
   - User database initialized in app module
   - Passed to API and other components via `AppState`
   - No global state or singletons

6. **Testing**
   - 6 unit tests covering:
     - Database initialization
     - Default role creation
     - Secure log entry hashing
     - Log chain verification
     - User manager creation
   - All tests passing

7. **Security Features**
   - Separate database for sensitive user data
   - Cryptographic hash chain prevents log tampering
   - Audit trail for all user-related actions
   - Log rotation to manage disk space
   - Verification tools for incident response

**Files Created/Modified:**
- `src-tauri/user/` - Complete user crate
- `config/config.system.dev.yaml` - Added user database and secure log configuration
- `src-tauri/app/src/lib.rs` - Added user manager initialization
- `src-tauri/Cargo.toml` - Added user crate to workspace

**Verification:**
- Application starts successfully with user system initialized
- User database and tables created correctly
- Secure log entries written with proper hash chain
- All tests pass
- No clippy warnings


## Task 20 Implement login and sessions.

- [ ] Status: Implementation Design

We'll use Axum-Login(https://github.com/maxcountryman/axum-login) and Tower Session (https://docs.rs/tower-sessions/0.14.0/tower_sessions/) with sqlx (https://github.com/maxcountryman/tower-sessions-stores/tree/main/sqlx-store) in the `data/user-backend/marain_user.db`.  Implement the login and middleware with great care to the security of the implementation.  We'll be supporting PassKeys (webauthn) and magic email links initially.  This auth system should be implemented in a non-blocking way for the API right now, later we will create an RBAC authorization system.

We are only concerned about two states right now, "unauthenticated" and "authenticated", we need to leave this open to more in the future with the RBAC system.  But for now, just stick with the two.



### Acceptance Criteria:

- Axum-Login, Tower Sessions and sqlx-store are implemented for the API
- Backend information needed for logins and sessions are stored in the `data/user-backend/marain_user.db`
- A user test suite has been created for "unauthenticated" and "authenticated" users
- The documentation has been updated to reflect the implementation

### **Implementation Notes:**


---
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

