# Task: Save/Load Projects

**ID:** TASK-006  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

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

*No work log entries yet*

---

## Related Documents
- PRD: Section 4.3 - Mapping & Profiles
- ADR: ADR-002 (SQLite for Persistence)
