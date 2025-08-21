# File Management Specification

## Overview

This specification defines how Marain CMS manages configuration through YAML files as the source of truth, with automatic application state synchronization and hot-reload capabilities.

## Core Principles

1. **YAML as Source of Truth**: YAML files are the authoritative source for configuration.
2. **Automatic Synchronization**: Application state automatically updates when YAML files change.
3. **Zero Downtime**: Changes can be applied without restarting the application.
4. **Non-Breaking Changes**: System handles configuration evolution gracefully.
5. **Audit Trail**: All changes are tracked and reversible.

## Architecture Components

### 1. File Watcher System

The file watcher monitors specified directories for changes (e.g., `schemas/`, `config/`):

```rust
// Watches for:
- File creation (new configuration)
- File modification (configuration updates)
- File deletion (configuration removal)
- File moves/renames
```

**Key Features:**
- Debouncing to handle rapid file changes
- Validation before processing changes
- Error recovery for invalid YAML files
- Event queuing for ordered processing

### 2. Diff Engine

Compares current in-memory state with new YAML definition:

```rust
pub struct ConfigDiff {
    pub file_path: String,
    pub added_keys: Vec<String>,
    pub removed_keys: Vec<String>,
    pub modified_values: Vec<(String, Value)>,
}
```

**Diff Categories:**
- Safe changes (adding optional keys, changing comments)
- Warning changes (value type modifications, removing keys)
- Breaking changes (changing required keys, structural changes)

### 3. Action Generator

Generates actions based on configuration diffs:

```sql
-- Example generated action for a config change
-- This is now an abstract action, not necessarily SQL.
-- It could be an API call, a function call, or a state update.
-- For example, for entity changes, it would generate SQL.
-- For config changes, it might update a global config object.
Action::UpdateGlobalConfig("log.level", "debug");
```

**Action Types:**
- Database schema migration (for entities)
- In-memory config update
- Cache invalidation
- Service restart/reinitialization

### 4. State Mapping

Each YAML file maps to a part of the application state. For entities, this means database tables. For configuration, it means in-memory structs.

```sql
-- Generated from article.schema.yaml
CREATE TABLE content_article (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    body TEXT,
    author_id TEXT REFERENCES content_user(id),
    published_at TIMESTAMP,
    status TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_article_slug ON content_article(slug);
CREATE INDEX idx_article_author ON content_article(author_id);
CREATE INDEX idx_article_status ON content_article(status);
```

### 5. Hot-Reload Mechanism

Process for applying changes without downtime:

1. **Detection Phase**
   - File watcher detects YAML change
   - Load and validate new YAML file
   - Generate diff against current state

2. **Planning Phase**
   - Analyze diff for required actions
   - Check for breaking changes
   - Generate actions
   - Create rollback plan

3. **Execution Phase**
   - Begin database transaction
   - Execute actions
   - Update in-memory state
   - Notify connected clients
   - Commit or rollback based on success

4. **Verification Phase**
   - Validate application state matches YAML
   - Run integrity checks
   - Update configuration version

## Version Management

Track configuration versions and action history:

```sql
CREATE TABLE file_versions (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    version INTEGER NOT NULL,
    file_hash TEXT NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    actions_executed TEXT,
    rollback_actions TEXT,
    UNIQUE(file_path, version)
);
```

## API Endpoints

### Hot-Reload Endpoints

```yaml
/api/v1/hot-reload/reload:
  post:
    description: Manually trigger a hot-reload of all monitored files
    responses:
      200:
        description: Reload successful
        content:
          changes: Array of applied changes
          
/api/v1/hot-reload/status:
  get:
    description: Get current hot-reload synchronization status
    responses:
      200:
        description: Status information
        content:
          in_sync: Boolean
          pending_changes: Array of detected changes
          last_sync: Timestamp
          
/api/v1/hot-reload/history:
  get:
    description: Get hot-reload change history
    parameters:
      file_path: Optional file path filter
    responses:
      200:
        description: Action history
```

## Error Handling

### Validation Errors
- Invalid YAML syntax
- Missing required keys
- Invalid value types
- Circular references

### Migration Errors
- Action execution errors
- State inconsistencies
- Missing dependencies
- Insufficient permissions

### Recovery Strategies
- Automatic rollback on failure
- State backup before changes
- Manual intervention options
- Detailed error logging

## Security Considerations

1. **File System Security**
   - Restrict write access to monitored directories
   - Validate file ownership and permissions
   - Audit all configuration changes

2. **Migration Safety**
   - Dry-run mode for testing actions
   - Approval workflow for production changes
   - Backup before applying actions

3. **Access Control**
   - Role-based file modification
   - API authentication for reload endpoints
   - Change attribution and logging

## Implementation Phases

### Phase 1: Basic File Watching
- Implement file watcher for monitored directories
- Detect and log changes
- Basic validation of YAML files

### Phase 2: Diff Engine
- Compare old vs new file states
- Categorize changes by impact
- Generate change reports

### Phase 3: Action Generator
- Generate appropriate actions based on file type (e.g., SQL for entities)

### Phase 4: Hot-Reload System
- Apply migrations without restart
- Update in-memory state/cache
- Notify connected clients

### Phase 5: Advanced Features
- Versioning
- Action rollback
- Conflict resolution
- Performance optimization

## Testing Strategy

### Unit Tests
- YAML parser validation
- Diff engine accuracy
- Action generation logic

### Integration Tests
- File watcher reliability
- Action execution
- Rollback scenarios
- Inter-file dependencies

### End-to-End Tests
- Complete reload workflow
- Concurrent file changes
- Error recovery paths
- Performance under load

## Monitoring and Observability

### Metrics
- Reload frequency
- Action execution time
- Failed action count
- State drift detection

### Logging
- All file changes
- Action execution details
- Error conditions
- Performance metrics

### Alerts
- Failed actions
- File validation errors
- File system issues
- Database connection problems