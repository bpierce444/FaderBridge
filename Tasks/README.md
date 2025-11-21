# FaderBridge Task Management

## Overview
This folder contains all task files for the FaderBridge project. Each task is a standalone markdown file that tracks progress, dependencies, and completion status.

## Task Naming Convention
- **Format:** `TASK-XXX-short-description.md`
- **Examples:**
  - `TASK-001-ucnet-device-discovery.md`
  - `TASK-002-midi-device-enumeration.md`
  - `TASK-003-basic-parameter-mapping.md`

## Task Statuses
- 🔴 **Not Started** - Task has been defined but work hasn't begun
- ⏳ **In Progress** - Currently being worked on
- ✅ **Complete** - All acceptance criteria met, Definition of Done satisfied
- ❌ **Blocked** - Cannot proceed due to dependencies or external issues

## Priority Levels
- **High** - Critical path for Phase 1 MVP
- **Medium** - Important but not blocking
- **Low** - Nice to have, can be deferred

## Workflow Commands
Use these Windsurf workflows to manage tasks:

- `/create-task` - Create a new task file
- `/update-task` - Update an existing task's status and progress
- `/list-tasks` - Show all tasks with their current status
- `/task-dependencies` - Visualize task dependency graph

## Task Lifecycle

```
1. Create Task (/create-task)
   ↓
2. Work on Task (update status to ⏳)
   ↓
3. Update Progress (/update-task after each session)
   ↓
4. Complete Task (verify Definition of Done)
   ↓
5. Mark Complete (status → ✅)
   ↓
6. Update PROJECT_JOURNAL.md
```

## Phase 1 MVP Tasks
The following tasks are locked for Phase 1 (see PROJECT_CHARTER.md):

1. **TASK-001:** UCNet device discovery (network + USB)
   - Status: 🔴 Not Started
   - File: `TASK-001-ucnet-device-discovery.md`
   
2. **TASK-002:** MIDI device enumeration
   - Status: 🔴 Not Started
   - File: `TASK-002-midi-device-enumeration.md`
   
3. **TASK-003:** Basic parameter mapping (volume, mute, pan)
   - Status: 🔴 Not Started
   - File: `TASK-003-basic-parameter-mapping.md`
   - Depends on: TASK-001, TASK-002
   
4. **TASK-004:** Bidirectional sync (< 10ms latency)
   - Status: 🔴 Not Started
   - File: `TASK-004-bidirectional-sync.md`
   - Depends on: TASK-001, TASK-002, TASK-003
   
5. **TASK-005:** MIDI Learn functionality
   - Status: 🔴 Not Started
   - File: `TASK-005-midi-learn.md`
   - Depends on: TASK-002, TASK-003
   
6. **TASK-006:** Save/Load projects
   - Status: 🔴 Not Started
   - File: `TASK-006-save-load-projects.md`
   - Depends on: TASK-003, TASK-005
   
7. **TASK-007:** Visual feedback (on-screen faders)
   - Status: 🔴 Not Started
   - File: `TASK-007-visual-feedback.md`
   - Depends on: TASK-003, TASK-004

**Current Progress:** 0/7 Complete

## Best Practices
- **One Task at a Time:** Follow the "One Feature Rule" from FEATURE_PRIORITIZATION.md
- **Update Frequently:** Update task files after every work session
- **Be Honest:** If blocked, mark it immediately and document why
- **Link Everything:** Reference ADRs, PRD sections, and related tasks
- **Test Before Complete:** Never mark a task complete without passing tests

## Quick Reference
- **Template:** `TASK_TEMPLATE.md`
- **Rules:** `.windsurf/rules/AI_CODING_RULES.md`
- **Definition of Done:** `.windsurf/rules/DEFINITION_OF_DONE.md`
- **Project Journal:** `PROJECT_JOURNAL.md`
