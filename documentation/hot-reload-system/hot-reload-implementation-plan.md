# Hot-Reload Implementation Plan

## Overview

This plan outlines the implementation steps to create a fully synchronized, file-based configuration management system with hot-reload capabilities for Marain CMS.

## Current State Analysis

### What We Have
- ✅ YAML files in `schemas/` and `config/` directories
- ✅ Rudimentary loading of some YAML files at startup
- ✅ Some state managed in memory
- ✅ Basic application functionality

### What's Missing
- ❌ File watching for YAML changes
- ❌ Automatic state synchronization based on file content
- ❌ Diff detection for configuration changes
- ❌ Action generation and execution
- ❌ Hot-reload without restart
- ❌ Versioning for all monitored files
- ❌ Rollback capabilities

## Implementation Tasks

### Task 1: Create Action Generator
**Priority: High**
**Estimated Effort: 2-3 days**

Create a system that generates actions based on the type of YAML file that has changed:

```rust
// src-tauri/src/hot_reload/action_generator.rs
pub struct ActionGenerator {
    pub fn generate_actions(diff: &ConfigDiff) -> Vec<Action>
}
```

**Subtasks:**
1. Determine file type (e.g., entity schema, system config)
2. For entities, generate SQL migration actions.
3. For system config, generate in-memory config update actions.
4. For other types, define and generate appropriate actions (e.g., cache invalidation).
5. Ensure actions are idempotent where possible.

### Task 2: Implement File Watcher System
**Priority: High**
**Estimated Effort: 1-2 days**

Add file watching capabilities to detect YAML file changes in monitored directories:

```rust
// src-tauri/src/hot_reload/file_watcher.rs
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    monitored_dirs: Vec<PathBuf>,
    event_sender: mpsc::Sender<FileChangeEvent>,
}

pub enum FileChangeEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}
```

**Subtasks:**
1. Integrate `notify` crate for cross-platform file watching
2. Implement debouncing for rapid file changes
3. Filter for .yaml/.yml files only
4. Queue events for ordered processing
5. Add error handling for file access issues

### Task 3: Build Diff Engine
**Priority: High**
**Estimated Effort: 2-3 days**

Create a system to detect differences between YAML file states:

```rust
// src-tauri/src/hot_reload/diff_engine.rs
pub struct DiffEngine {
    pub fn compare(
        old_state: &Value,
        new_state: &Value
    ) -> ConfigDiff
    
    pub fn categorize_changes(diff: &ConfigDiff) -> ChangeCategory
}

pub enum ChangeCategory {
    Safe,      // Non-breaking changes
    Warning,   // Potentially breaking changes
    Breaking,  // Definitely breaking changes
}
```

**Subtasks:**
1. Detect added keys
2. Detect removed keys
3. Detect value type changes
4. Detect structural changes
5. Categorize changes by impact

### Task 4: Create Action Executor
**Priority: High**
**Estimated Effort: 3-4 days**

Execute actions generated from configuration diffs:

```rust
// src-tauri/src/hot_reload/action_executor.rs
pub struct ActionExecutor {
    pub async fn execute_actions(
        actions: Vec<Action>
    ) -> Result<()>
    
    pub async fn rollback_actions(
        actions: Vec<Action>
    ) -> Result<()>
}

pub enum Action {
    DatabaseMigration(Migration),
    UpdateConfig(String, Value),
    InvalidateCache(String),
    // ... other actions
}
```

**Subtasks:**
1. Execute database migrations within a transaction.
2. Update the in-memory configuration state.
3. Invalidate relevant caches.
4. Implement rollback logic for each action type.
5. Support dry-run mode.

### Task 5: Create Version Tracking
**Priority: Medium**
**Estimated Effort: 1-2 days**

Track file versions and action history:

```sql
-- Migration to add version tracking
CREATE TABLE IF NOT EXISTS file_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    version INTEGER NOT NULL,
    file_hash TEXT NOT NULL,
    action_id TEXT NOT NULL,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    applied_by TEXT,
    actions_executed TEXT,
    rollback_actions TEXT,
    status TEXT CHECK(status IN ('pending', 'applied', 'rolled_back')),
    UNIQUE(file_path, version)
);

CREATE INDEX idx_file_versions_path ON file_versions(file_path);
CREATE INDEX idx_file_versions_status ON file_versions(status);
```


### Task 6: Implement Hot-Reload Coordinator
**Priority: High**
**Estimated Effort: 2-3 days**

Coordinate the hot-reload process:

```rust
// src-tauri/src/hot_reload/coordinator.rs
pub struct HotReloadCoordinator {
    file_watcher: FileWatcher,
    diff_engine: DiffEngine,
    action_generator: ActionGenerator,
    action_executor: ActionExecutor,
    app_state: Arc<RwLock<AppState>>,
}

impl HotReloadCoordinator {
    pub async fn start(&self) -> Result<()>
    pub async fn process_file_change(&self, event: FileChangeEvent) -> Result<()>
    pub async fn reload_all_files(&self) -> Result<ReloadReport>
}
```

**Subtasks:**
1. Listen for file change events
2. Load and validate changed YAML files
3. Generate and categorize diffs
4. Generate and execute actions
5. Update in-memory application state
6. Broadcast changes to connected clients or other system components

### Task 7: Add API Endpoints for Hot-Reload Management
**Priority: Medium**
**Estimated Effort: 1 day**

Create API endpoints for manual control:

```yaml
# Add to openapi.yaml
/api/v1/hot-reload/reload:
  post:
    summary: Manually trigger a hot-reload of all monitored files
    
/api/v1/hot-reload/status:
  get:
    summary: Get current hot-reload synchronization status
    
/api/v1/hot-reload/history:
  get:
    summary: Get hot-reload change history
    
/api/v1/hot-reload/diff:
  post:
    summary: Preview changes without applying them
```

### Task 8: Update Application State to Use Hot-Reloaded Configuration
**Priority: High**
**Estimated Effort: 2-3 days**

Modify application logic to read from the hot-reloaded state:

```rust
// Example: Accessing a hot-reloaded configuration value
let log_level = APP_STATE.get_config_value("log.level").unwrap_or("info");

// Example: Using a hot-reloaded entity definition
let entity_def = APP_STATE.get_entity("article").unwrap();
let table_name = entity_def.table_name();
// ... use table_name in a query
```

### Task 9: Create Admin UI for Hot-Reload Management
**Priority: Medium**
**Estimated Effort: 2-3 days**

Add UI components for hot-reload management:

1. Hot-reload sync status indicator
2. Action history viewer
3. Manual reload button
4. Diff preview before applying changes
5. Rollback controls
6. YAML validation errors display

### Task 10: Implement Comprehensive Testing
**Priority: High**
**Estimated Effort: 3-4 days**

Create tests for all components:

**Unit Tests:**
- Action generator logic
- Diff engine accuracy
- Action executor correctness
- Version tracking logic

**Integration Tests:**
- File watcher reliability
- Action execution and rollback
- Hot-reload workflow
- Concurrent file changes

**End-to-End Tests:**
- Complete file change workflow
- UI interaction with hot-reload
- Performance under multiple changes
- Error recovery scenarios

### Task 11: Add Monitoring and Logging
**Priority: Medium**
**Estimated Effort: 1-2 days**

Implement observability:

1. Log all file changes with details
2. Track action execution times
3. Monitor file watcher health
4. Alert on failed actions
5. Performance metrics for reload operations

## Data and State Migration Strategy

### Phase 1: Parallel Operation
1. Keep existing state management
2. Implement new hot-reloadable state management in parallel
3. Sync state between old and new systems
4. Verify state integrity

### Phase 2: Cutover
1. Switch reads to new state management system
2. Continue dual writes
3. Monitor for issues
4. Performance comparison

### Phase 3: Cleanup
1. Stop updates to old state management system
2. Final state verification
3. Remove old state management system
4. Update all code references

## Risk Mitigation

### Technical Risks
1. **Data Loss**: Implement comprehensive backups before actions are executed.
2. **Performance**: Test with large datasets before production
3. **Compatibility**: Ensure both SQLite and PostgreSQL support
4. **Concurrency**: Handle multiple simultaneous file changes

### Operational Risks
1. **Unauthorized Changes**: Implement file system permissions
2. **Invalid YAML**: Validate before processing
3. **Action Failures**: Automatic rollback and alerting
4. **Version Conflicts**: Implement proper locking

## Success Criteria

1. ✅ YAML changes automatically sync to application state
2. ✅ No application restart required
3. ✅ Zero data loss during state changes
4. ✅ Rollback capability for all actions
5. ✅ Performance impact < 100ms for reload
6. ✅ Full audit trail of all changes
7. ✅ Works with both SQLite and PostgreSQL
8. ✅ Comprehensive error handling

## Timeline Estimate

**Total Estimated Effort: 25-35 days**

### Sprint 1 (Week 1-2): Foundation
- Task 1: Action Generator
- Task 2: File Watcher System
- Task 3: Diff Engine

### Sprint 2 (Week 3-4): Migration System
- Task 4: Action Executor
- Task 5: Version Tracking

### Sprint 3 (Week 5-6): Integration
- Task 6: Hot-Reload Coordinator
- Task 7: API Endpoints
- Task 8: Update Application State

### Sprint 4 (Week 7-8): Polish
- Task 9: Admin UI
- Task 10: Testing
- Task 11: Monitoring

## Next Steps

1. Review and approve this implementation plan
2. Create detailed technical designs for each component
3. Set up development branch for hot-reload feature
4. Begin implementation with Task 1 (Action Generator)
5. Regular progress reviews and adjustments