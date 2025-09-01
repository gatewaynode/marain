# MARC CLI Documentation

## Overview

`marc` is the command-line interface for the Marain CMS system. It provides direct access to system health checks, configuration management, and other administrative functions without needing to run the full application.

## Installation

### Building from Source

```bash
cd src-tauri
cargo build --package marc --release
```

The binary will be available at `src-tauri/target/release/marc`

### Adding to System PATH

To use `marc` from anywhere on your system:

```bash
# macOS/Linux
cp src-tauri/target/release/marc /usr/local/bin/
# Or add to your PATH
export PATH="$PATH:/path/to/marain/src-tauri/target/release"

# Windows
# Copy marc.exe to a directory in your PATH
```

## Usage Requirements

- **Must be run from the Marain project root or any subdirectory within it**
- The CLI will automatically detect the project root by looking for:
  - `src-tauri/` directory
  - `schemas/` directory  
  - `config/` directory
  - `package.json` file
  - `src-tauri/Cargo.toml` file

If run outside a Marain project, you'll see:
```
Error: Not in a Marain project directory. Please run marc from the project root.
```

## Commands

### Global Options

```bash
marc [OPTIONS] <COMMAND>
```

**Options:**
- `-v, --verbose` - Enable verbose output for debugging
- `-h, --help` - Print help information
- `-V, --version` - Print version information

### Health Check

Check the health status of various system components.

```bash
marc health [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Output format [default: text] [possible values: text, json]

**Example Output (text):**
```
=== Marain System Health Check ===

Overall Status: HEALTHY
Timestamp: 2025-01-09T10:30:00Z

Components:
──────────────────────────────────────────────────
✓ DATABASE (healthy)
  Database file exists and is accessible
  
✓ CONFIGURATION (healthy)
  All configuration files present and valid
  Loaded sections: system, api, content
  
✓ SCHEMAS (healthy)
  Found 3 schema file(s)
  Schemas: snippet.schema.yaml, multi.schema.yaml, all_fields.schema.yaml
  
○ API (offline)
  API server is not running or not reachable
```

**Example Output (json):**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-09T10:30:00Z",
  "components": {
    "database": {
      "status": "healthy",
      "message": "Database file exists and is accessible",
      "path": "./data/content/marain.db"
    },
    "configuration": {
      "status": "healthy",
      "message": "All configuration files present and valid",
      "loaded_sections": ["system", "api", "content"]
    },
    "schemas": {
      "status": "healthy",
      "message": "Found 3 schema file(s)",
      "count": 3,
      "schemas": ["snippet.schema.yaml", "multi.schema.yaml", "all_fields.schema.yaml"]
    },
    "api": {
      "status": "offline",
      "message": "API server is not running or not reachable",
      "endpoint": "http://localhost:3030"
    }
  }
}
```

### Configuration Management

#### List All Configurations

Display all loaded configuration sections.

```bash
marc config list [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Output format [default: text] [possible values: text, json, yaml]

**Example Output (text):**
```
=== Marain Configuration ===

[system]
  database: 
    path: data/marain.db
    max_connections: 10
  api: 
    host: 127.0.0.1
    port: 3030

[api]
  cors: 
    allowed_origins: 
      - http://localhost:3000
      - http://localhost:1420

[content]
  cache: 
    ttl: 3600
    max_size: 1000

Total sections: 3
```

#### Get Specific Configuration Value

Retrieve a specific configuration value using dot notation.

```bash
marc config get <SECTION> [OPTIONS]
```

**Arguments:**
- `<SECTION>` - Configuration path using dot notation (e.g., "system.database.path")

**Options:**
- `-f, --format <FORMAT>` - Output format [default: text] [possible values: text, json, yaml]

**Examples:**

```bash
# Get database path
marc config get system.database.path

# Output:
=== Configuration Value ===

Path: system.database.path
Type: string

Value:
data/marain.db
```

```bash
# Get API configuration in JSON
marc config get system.api --format json

# Output:
{
  "host": "127.0.0.1",
  "port": 3030
}
```

```bash
# Get CORS settings in YAML
marc config get api.cors --format yaml

# Output:
allowed_origins:
  - http://localhost:3000
  - http://localhost:1420
```

## Environment Variables

The CLI respects the following environment variables (typically set in `.env`):

- `ENVIRONMENT` - Current environment (dev, staging, production) [default: dev]
- `DATA_PATH` - Base path for data storage [default: ./data]
- `STATIC_PATH` - Base path for static files [default: ./static]
- `ENTITY_SCHEMA_PATH` - Base path for entity schemas [default: ./schemas]
- `CONFIGURATION_PATH` - Base path for configuration files [default: ./config]

## Exit Codes

- `0` - Success
- `1` - General error (not in project root, missing files, etc.)
- `101` - Cargo/compilation error

## Examples

### Common Workflows

#### 1. Quick System Check
```bash
# Check if everything is set up correctly
marc health

# Get detailed JSON output for scripting
marc health --format json | jq '.components.database.status'
```

#### 2. Configuration Inspection
```bash
# View all configurations
marc config list

# Check specific database settings
marc config get system.database

# Export configuration to file
marc config list --format yaml > current-config.yaml
```

#### 3. Debugging with Verbose Mode
```bash
# Run with verbose output to see detailed logs
marc --verbose health

# Combine with specific commands
marc -v config get system.api.port
```

#### 4. Integration with Scripts
```bash
#!/bin/bash
# Check if database is healthy before running migrations
if marc health --format json | jq -e '.components.database.status == "healthy"' > /dev/null; then
    echo "Database is healthy, proceeding with migrations..."
    # Run migrations
else
    echo "Database is not healthy, aborting."
    exit 1
fi
```

## Testing

The CLI includes comprehensive test coverage:

```bash
# Run all CLI tests
cargo test --package marc

# Run specific test file
cargo test --package marc --test cli_tests

# Run with output
cargo test --package marc -- --nocapture
```

## Development

### Project Structure
```
src-tauri/cli/
├── Cargo.toml           # Package configuration
├── src/
│   ├── main.rs         # Entry point and CLI setup
│   ├── commands/       # Command implementations
│   │   ├── mod.rs
│   │   ├── health.rs   # Health check command
│   │   └── config.rs   # Configuration commands
│   └── utils/          # Utility modules
│       ├── mod.rs
│       ├── env_paths.rs    # Environment path management
│       └── project_root.rs # Project root detection
└── tests/
    └── cli_tests.rs    # Integration tests
```

### Adding New Commands

1. Create a new module in `src/commands/`
2. Implement the command logic
3. Add the command to the CLI enum in `main.rs`
4. Update the match statement in `main()`
5. Add tests in `tests/cli_tests.rs`

### Error Handling

The CLI uses `anyhow` for error handling and provides user-friendly error messages with colored output using the `colored` crate.

## Troubleshooting

### "Not in a Marain project directory"
- Ensure you're running `marc` from within the Marain project directory tree
- Check that required directories (`src-tauri/`, `schemas/`, `config/`) exist

### "Failed to load configurations"
- Verify configuration files exist in the `config/` directory
- Check YAML syntax in configuration files
- Ensure environment variables are set correctly

### "Database not accessible"
- Check that `DATA_PATH` is set correctly
- Verify file permissions on the database file
- Ensure the database has been initialized

### Test Failures
- Clear environment variables that might interfere: `unset DATA_PATH STATIC_PATH ENTITY_SCHEMA_PATH CONFIGURATION_PATH`
- Ensure no other instances are accessing the same resources
- Run tests individually to isolate issues

## Future Enhancements

Planned features for future versions:
- Entity management commands (create, list, update, delete)
- Database migration commands
- Schema validation and generation
- User management operations
- Cache management utilities
- API server control (start, stop, status)
- Log viewing and analysis
- Performance metrics and monitoring