# List Tasks

This workflow displays all tasks with their current status, organized by phase and priority.

## Steps

1. **Scan the Tasks folder:**
   - Read all `TASK-*.md` files in the `Tasks/` directory
   - Extract the following from each file:
     - Task ID
     - Title (from the markdown header)
     - Status (🔴 ⏳ ✅ ❌)
     - Priority (High/Medium/Low)
     - Phase (Phase 1 MVP / Phase 2 / Phase 3)
     - Last updated date

2. **Organize tasks by phase:**
   - Group tasks into Phase 1, Phase 2, Phase 3
   - Within each phase, sort by priority (High → Medium → Low)
   - Within each priority, sort by task ID (ascending)

3. **Display Phase 1 MVP tasks:**
   - Show a header: "## Phase 1 MVP Tasks (7 Required)"
   - For each Phase 1 task, display:
     ```
     [Status] TASK-XXX: [Title]
     Priority: [High/Medium/Low] | Updated: [Date]
     ```
   - Show progress: "X/7 Complete"
   - Highlight any blocked tasks in red

4. **Display Phase 2 tasks (if any):**
   - Show a header: "## Phase 2 Tasks"
   - Use the same format as Phase 1
   - Note: These should only exist if Phase 1 is complete

5. **Display Phase 3 tasks (if any):**
   - Show a header: "## Phase 3 Tasks"
   - Use the same format

6. **Show summary statistics:**
   - Total tasks: X
   - Not Started: X (🔴)
   - In Progress: X (⏳)
   - Complete: X (✅)
   - Blocked: X (❌)

7. **Highlight important information:**
   - If there are blocked tasks, list them separately with their blocker reasons
   - If Phase 1 is not complete, remind the user of the "One Feature Rule"
   - If multiple tasks are "In Progress", warn about violating the "One Feature Rule"

8. **Suggest next actions:**
   - If no tasks are in progress, suggest starting the highest priority incomplete task
   - If a task is blocked, suggest addressing the blocker
   - If Phase 1 is complete, suggest reviewing FEATURE_PRIORITIZATION.md for Phase 2

## Important Rules
- Always show Phase 1 tasks first (they are the priority)
- Highlight blocked tasks prominently
- Warn if more than one task is "In Progress" (violates One Feature Rule)
- Show the most recently updated tasks at the top of each priority group
