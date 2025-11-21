# Task: Save/Load Projects

**ID:** TASK-006  
**Status:** 🟡 In Progress  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** Cascade AI  
**Created:** 2025-11-20  
**Updated:** 2025-11-21  

---

## Description
Implement project persistence using SQLite to save and load mapping configurations. Users should be able to create multiple projects for different setups (e.g., "Studio Setup", "Live Show", "Church Mix").

## Acceptance Criteria
- [ ] User can save current mapping configuration as a named project
- [ ] User can load a previously saved project
- [ ] Projects stored in SQLite database with ACID guarantees
- [ ] Project includes: device IDs, parameter mappings, taper curves, user preferences
- [ ] Recent projects list shown on app launch
- [ ] Auto-save functionality (every 30 seconds if changes detected)
- [ ] Export/Import projects as JSON files for backup/sharing
- [ ] Database migration strategy in place for future schema changes

## Dependencies
- **Depends On:** TASK-003 (Basic Parameter Mapping), TASK-005 (MIDI Learn)
- **Blocks:** None (but enables user workflow)

## Technical Notes
- Use `rusqlite` crate for SQLite access
- Database location: `~/Library/Application Support/FaderBridge/projects.db` (macOS)
- Schema should include: projects table, mappings table, devices table
- Consider using `sqlx` for compile-time SQL verification
- JSON export format should be human-readable and version-tagged
- Auto-save should not block UI or cause performance issues

## Files Affected
- `src-tauri/src/db/schema.sql` (to be created)
- `src-tauri/src/db/projects.rs` (to be created)
- `src-tauri/src/db/migrations.rs` (to be created)
- `src-tauri/src/db/mod.rs` (to be created)
- `src-tauri/src/commands/projects.rs` (to be created)
- `src/features/ProjectManager.tsx` (to be created)
- `src/hooks/useProjects.ts` (to be created)

## Testing Requirements
- [ ] Unit tests for database CRUD operations
- [ ] Unit tests for JSON export/import
- [ ] Integration test for save/load workflow
- [ ] Test database corruption recovery
- [ ] Test migration from v1 to v2 schema (future-proofing)
- [ ] Manual testing of auto-save functionality
- [ ] Manual testing with large projects (100+ mappings)

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] Database migrations tested

---

## Work Log

### 2025-11-21 - Initial Implementation
**Completed:**
- ✅ Created SQLite schema with projects, devices, mappings, and preferences tables
- ✅ Implemented database connection and migration system
- ✅ Created Rust database layer with full CRUD operations for:
  - Projects (create, read, update, delete, set active)
  - Devices (create, read, update, delete)
  - Mappings (create, read, update, delete, find by MIDI CC)
- ✅ Implemented JSON export/import functionality with version tagging
- ✅ Created Tauri commands for all database operations
- ✅ Integrated database initialization into main.rs
- ✅ Created TypeScript types matching Rust types
- ✅ Implemented useProjects React hook with full CRUD operations
- ✅ Created ProjectManager React component with UI for:
  - Creating new projects
  - Loading/activating projects
  - Viewing recent and all projects
  - Deleting projects
  - Exporting/importing projects

**In Progress:**
- Auto-save functionality (not yet implemented)
- File dialog integration for export/import (using placeholder paths)

**Next Steps:**
- Implement auto-save with debouncing
- Add file dialog support using Tauri's dialog plugin
- Write comprehensive tests for database layer
- Add integration tests for project workflows
- Update documentation

---

## Related Documents
- PRD: Section 4.3 - Mapping & Profiles
- ADR: ADR-002 (SQLite for Persistence)
