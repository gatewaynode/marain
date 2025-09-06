# Hot-Reload System Summary

## Overview

This document summarizes the planning work completed for implementing a generic, file-based hot-reload system for YAML files in Marain CMS.

## Problem Statement

The current Marain CMS implementation:
- Loads some YAML files into memory at startup
- State is not consistently synchronized with file contents
- Requires application restart for configuration changes
- Does not synchronize application state with YAML definitions

## Solution Design

We've designed a comprehensive system that:
1. **Treats YAML files as the source of truth** for configuration and state
2. **Automatically synchronizes** application state when YAML files change
3. **Requires no recompilation** or restart for configuration updates
4. **Generates appropriate actions** based on file type
5. **Provides rollback capabilities** for failed actions

## Key Components

### 1. File Watcher System
- Monitors configured directories (e.g. `schemas/`, `config/`) for YAML changes
- Detects file creation, modification, and deletion
- Implements debouncing for rapid changes
- Queues events for ordered processing

### 2. Diff Engine
- Compares old and new file states
- Categorizes changes as safe, warning, or breaking
- Generates detailed change reports
- Enables informed decision-making

### 3. Action Generator
- Creates actions from diffs based on file type
- Generates database migrations for entity schemas
- Generates state updates for configuration files
- Is extensible for other file types

### 4. Hot-Reload Coordinator
- Orchestrates the entire reload process
- Validates files before applying changes
- Executes actions within transactions where applicable
- Updates in-memory application state
- Notifies connected clients of changes

### 5. Version Tracking
- Maintains history of all file changes
- Stores actions and rollback information
- Enables point-in-time recovery
- Provides audit trail

## State Management Changes

### From (Current):
```sql
-- Generic JSON storage
CREATE TABLE content (
    id TEXT PRIMARY KEY,
    entity_type TEXT,
    data JSON,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);
```

### To (New):
```sql
-- Entity-specific tables with typed columns
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
```

This dynamic schema generation aligns with the entity modeling in [DEVELOPER-GUIDE.md](../DEVELOPER-GUIDE.md#data-modeling--storage) and [entity-management.md](../entity-management-system/entity-management.md) for hot-reloaded YAML definitions.

## Implementation Plan

The work has been broken down into 11 main tasks that will:
1. Build the foundational components
2. Implement the action generation and execution system
3. Integrate with the application state
4. Add UI controls for hot-reload management
5. Provide comprehensive testing and monitoring

**Estimated Timeline**: 25-35 days (5-7 weeks)

## Benefits

1. **Developer Experience**
   - No restart required for configuration changes
   - Immediate feedback on YAML file modifications
   - Clear action history and rollback options

2. **Performance**
   - Proper database indexes and constraints
   - Typed columns instead of JSON parsing
   - Optimized queries with real SQL

3. **Reliability**
   - Automatic validation before applying changes
   - Transaction-based actions (where applicable) with rollback
   - Comprehensive error handling

4. **Maintainability**
   - YAML files as single source of truth
   - Clear audit trail of all changes
   - Automated application state synchronization

## Next Steps

1. **Review and approve** the specification and implementation plan
2. **Prioritize** which components to build first
3. **Set up** development environment for hot-reload feature
4. **Begin implementation** with the Action Generator

## Related Documents

- [File Management Specification](./file-management-spec.md) - Detailed technical specification
- [Hot-Reload Implementation Plan](./hot-reload-implementation-plan.md) - Complete implementation breakdown

## Questions to Consider

1. Should we support gradual state migration (keeping both old and new state management during transition)?
2. What level of breaking changes should require manual approval?
3. How should we handle file conflicts in multi-instance deployments?
4. Should we implement change notifications via webhooks?

## Success Metrics

- ✅ Zero downtime for configuration changes
- ✅ < 100ms reload time for typical changes
- ✅ 100% backward compatibility during state migration
- ✅ Zero data loss guarantee
- ✅ Complete audit trail of all changes

## Cross-References and Best Practices

For integration with core systems:
- See [DEVELOPER-GUIDE.md](../DEVELOPER-GUIDE.md#system-architecture) for overall architecture and API flow affected by hot-reloads.
- Refer to [entity-management-system/entity-management.md](../entity-management-system/entity-management.md) for schema loading details using trait-objects.

### Security Best Practices for Hot-Reload
- **File Validation:** Parse and validate YAML inputs before applying changes to prevent malicious schemas (e.g., injection via fields).
- **Access Control:** Restrict watched directories to config/schemas; log all reload events to secure.log.
- **Rollback Security:** Ensure rollbacks don't expose sensitive state; test for race conditions in multi-instance setups.
- **Dependencies:** Use secure file watching (e.g., notify crate); update for vulnerabilities.
- **Testing:** Unit test action generators; E2E for reload scenarios without data loss.