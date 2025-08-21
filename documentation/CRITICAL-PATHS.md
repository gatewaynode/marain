# Critical Path Configuration

This document outlines critical path configurations that MUST be followed to prevent system issues.

## Environment-Based Path Management

**NEW IN v2.0**: All paths are now managed through environment variables defined in `.env` file:
- `DATA_PATH` - Base path for all data storage (default: `./data`)
- `STATIC_PATH` - Base path for static files (default: `./static`)
- `ENTITY_SCHEMA_PATH` - Base path for entity schemas (default: `./schemas`)
- `CONFIGURATION_PATH` - Base path for configuration files (default: `./config`)

## Log File Location

**CRITICAL REQUIREMENT**: Log files MUST be stored in `{DATA_PATH}/logs/`.

### Why This Matters
- The hot-reload system watches the `src-tauri` directory for changes
- If logs are written anywhere inside `src-tauri/`, each log write triggers a file change event
- This creates an infinite loop: log write → file change → rebuild → log write → ...
- This will cause 100% CPU usage and continuous rebuilding

### Correct Configuration
```
marain-cms/                 # Project root
├── data/                   # {DATA_PATH}
│   ├── logs/              # ✅ CORRECT: Logs go here
│   │   └── marain-cms.YYYY-MM-DD.log
│   ├── content/           # Database storage
│   │   └── marain.db      # SQLite database
│   ├── work-queue/        # Future: Work queue storage
│   ├── json-cache/        # Future: JSON cache storage
│   └── user-backend/      # Future: User backend storage
├── src-tauri/
│   ├── app/
│   │   └── data/          # ❌ WRONG: Never put logs here
│   └── ...
└── ...
```

### Implementation Details
The logging system (`src-tauri/app/src/logging.rs`) uses the `EnvPaths` struct to determine log location:
- Loads environment variables from `.env` file using `dotenv`
- Constructs log path as `env_paths.data_path.join("logs")`
- Automatically creates the directory if it doesn't exist

## Database Location

**REQUIREMENT**: The SQLite database file is stored in `{DATA_PATH}/content/marain.db`.

### Why This Matters
- Keeps runtime data separate from source code
- Prevents accidental commits of database files
- Ensures consistent location across development environments
- Prepares for future specialized data stores

## Hot-Reload Watched Directories

The hot-reload system monitors these directories (configured via environment variables):
- `{CONFIGURATION_PATH}` - System configuration files (default: `/config/`)
- `{ENTITY_SCHEMA_PATH}` - Entity schema definitions (default: `/schemas/`)

**IMPORTANT**: Never place frequently-changing files (logs, temp files, build artifacts) in these directories.

## File System Rules

### DO NOT Place in Watched Directories:
- Log files
- Temporary files
- Build artifacts
- Cache files
- Database files
- Any files that change frequently during runtime

### Safe Locations for Runtime Files:
- `{DATA_PATH}/` - All runtime data (logs, database, uploads)
- `{DATA_PATH}/content/` - Database files
- `{DATA_PATH}/logs/` - Application logs
- `{DATA_PATH}/work-queue/` - Work queue data (future)
- `{DATA_PATH}/json-cache/` - JSON cache data (future)
- `{DATA_PATH}/user-backend/` - User backend data (future)
- System temp directory - For truly temporary files

## Troubleshooting

### Symptom: Continuous rebuilding
**Cause**: Files are being written to a watched directory
**Solution**: Check that logs and other runtime files are in `/data/`

### Symptom: Logs not being created
**Cause**: Incorrect environment variable configuration
**Solution**: Verify `.env` file exists and `DATA_PATH` is set correctly

### Symptom: "Failed to create logs directory" error
**Cause**: Permission issues or incorrect path
**Solution**: Ensure the application has write permissions to `{DATA_PATH}/logs/`

## Developer Checklist

Before adding any new file I/O operations:
- [ ] Is this a runtime-generated file?
- [ ] If yes, is it going to `{DATA_PATH}/` or another non-watched location?
- [ ] Have I documented the file location in this document?
- [ ] Have I tested that it doesn't trigger hot-reload?
- [ ] Have I updated the `.env` file if new paths are needed?

## Environment Variables Configuration

Example `.env` file:
```env
# Current Environment
ENVIRONMENT="dev"

# The base path for data
DATA_PATH="./data"

# The base path for static files
STATIC_PATH="./static"

# The base path for entity schemas
ENTITY_SCHEMA_PATH="./schemas"

# The base path for configurations
CONFIGURATION_PATH="./config"
```

## References

- Environment paths: `src-tauri/app/src/lib.rs` (EnvPaths struct)
- Logging implementation: `src-tauri/app/src/logging.rs`
- Schema-manager watcher: `src-tauri/schema-manager/src/watcher.rs`
- Database initialization: `src-tauri/database/src/init.rs`
- API server initialization: `src-tauri/api/src/server.rs`
- Environment configuration: `.env` and `EXAMPLE.env`