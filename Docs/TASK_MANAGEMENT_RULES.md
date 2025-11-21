# Task Management Rules for FaderBridge

> **CRITICAL:** All AI agents working on this project MUST follow these task management rules.

## Task Workflow Requirements (MANDATORY)

### When Starting Work
1. **Before coding anything**, run `/list-tasks` to see current task status
2. **Check for in-progress tasks** - if one exists, continue it (One Feature Rule)
3. **If no task exists** for your work, run `/create-task` first
4. **Never start coding** without an associated task file

### During Work Sessions
1. **Update task progress** using `/update-task` after significant progress
2. **Document blockers immediately** - don't wait until the end of the session
3. **Track time spent** - include duration in work log entries
4. **Update acceptance criteria** - check off items as you complete them

### When Completing Work
1. **Verify Definition of Done** - ALL checklist items must be complete
2. **Run `/update-task`** to mark status as ✅ Complete
3. **Update PROJECT_JOURNAL.md** (mandatory - this is in DEFINITION_OF_DONE)
4. **Check task dependencies** - run `/task-dependencies` to see what unblocks

### Task File Updates (MANDATORY)
You MUST update the task file when:
- Starting work on a task (status 🔴 → ⏳)
- After each work session (add work log entry)
- Encountering a blocker (status → ❌, document why)
- Completing acceptance criteria (check off items)
- Finishing the task (status → ✅, verify Definition of Done)

## Available Workflows

### `/create-task`
Creates a new task file with proper ID, template, and validation.

**When to use:**
- Starting work on a new feature
- Breaking down a large feature into subtasks
- Planning future work

**What it does:**
- Generates sequential task ID
- Creates task file from template
- Validates against PROJECT_CHARTER.md
- Updates Tasks/README.md

### `/update-task`
Updates an existing task's progress, status, and work log.

**When to use:**
- After every work session (even if incomplete)
- When changing task status
- When encountering blockers
- When completing acceptance criteria

**What it does:**
- Updates task status and dates
- Adds work log entry
- Checks Definition of Done (if marking complete)
- Reminds about PROJECT_JOURNAL.md update

### `/list-tasks`
Displays all tasks organized by phase and priority.

**When to use:**
- At the start of a work session
- To check overall project progress
- To find the next task to work on
- To see blocked tasks

**What it does:**
- Shows all tasks with status
- Highlights blocked tasks
- Shows Phase 1 progress (X/7 complete)
- Warns about One Feature Rule violations

### `/task-dependencies`
Visualizes task dependencies and critical path.

**When to use:**
- Planning work order
- Understanding task relationships
- Finding tasks ready to start
- Identifying blockers

**What it does:**
- Shows dependency graph
- Identifies critical path
- Lists ready-to-start tasks
- Validates dependency integrity

## Task Status Meanings

- 🔴 **Not Started** - Task defined but no work begun
- ⏳ **In Progress** - Currently being worked on (should only be ONE at a time)
- ✅ **Complete** - All acceptance criteria met, Definition of Done satisfied
- ❌ **Blocked** - Cannot proceed due to dependencies or external issues

## The "One Feature Rule"

**CRITICAL:** Only ONE task should be ⏳ In Progress at any time.

**Why?**
- Prevents context switching
- Ensures focused, quality work
- Avoids half-finished features
- Maintains project momentum

**Enforcement:**
- `/list-tasks` will warn if multiple tasks are in progress
- Before starting a new task, complete or block the current one
- If you must switch tasks, document why in the work log

## Task Priority Guidelines

### High Priority
- Phase 1 MVP tasks (the 7 locked features)
- Blockers preventing other work
- Critical bugs affecting core functionality

### Medium Priority
- Phase 2 features (only after Phase 1 complete)
- Improvements to existing features
- Non-critical bug fixes

### Low Priority
- Phase 3 speculative features
- Nice-to-have improvements
- Documentation enhancements

## Integration with Other Systems

### PROJECT_JOURNAL.md
- When marking a task ✅ Complete, you MUST update the journal
- Reference the completed task ID in the journal entry
- Update the Phase 1 progress tracker

### DEFINITION_OF_DONE.md
- Every task has a "Definition of Done Checklist"
- ALL items must be checked before marking ✅ Complete
- This includes: tests, documentation, journal update, no warnings

### PROJECT_CHARTER.md
- All tasks must align with the current phase scope
- Phase 1 tasks are locked (no additions without approval)
- `/create-task` validates against the charter automatically

### FEATURE_PRIORITIZATION.md
- Use the Impact/Effort matrix when creating tasks
- High Impact + Low Effort tasks should be High Priority
- Low Impact + High Effort tasks should be rejected or deferred

## Task File Best Practices

### Good Task Titles
- ✅ "Implement UCNet device discovery over UDP"
- ✅ "Add MIDI Learn for fader mapping"
- ❌ "Fix stuff" (too vague)
- ❌ "Implement everything for Phase 1" (too broad)

### Good Acceptance Criteria
- ✅ "App detects Series III mixer on local network within 2 seconds"
- ✅ "MIDI CC messages are translated to UCNet with < 10ms latency"
- ❌ "Make it work" (not measurable)
- ❌ "Do the thing" (not specific)

### Good Work Log Entries
- ✅ Include duration, progress, blockers, next steps
- ✅ Be specific about what was accomplished
- ✅ Document why decisions were made
- ❌ "Worked on stuff for a while" (too vague)
- ❌ Empty entries (always document something)

## Enforcement

These rules are **mandatory** and enforced through:
1. **Workflow validation** - workflows check for rule compliance
2. **Definition of Done** - task updates required for completion
3. **Code review** - no PR without associated task file
4. **Project Journal** - task completion tracked in journal

## Quick Reference Card

```
Before Coding:  /list-tasks → Check for in-progress tasks
Start New Work: /create-task → Create task file first
During Work:    /update-task → Update after each session
Check Progress: /list-tasks → See overall status
Plan Work:      /task-dependencies → See what's ready
Complete Task:  /update-task (✅) → Verify Definition of Done
                PROJECT_JOURNAL.md → Update journal (mandatory)
```
