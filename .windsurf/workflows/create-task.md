# Create Task

This workflow guides you through creating a new task file for FaderBridge.

## Steps

1. **Ask the user for task details:**
   - Task title (short, descriptive)
   - Description (what needs to be done)
   - Priority (High/Medium/Low)
   - Phase (Phase 1 MVP / Phase 2 / Phase 3)
   - Acceptance criteria (at least 3 items)

2. **Generate task ID:**
   - Look in the `Tasks/` folder to find the highest existing TASK-XXX number
   - Increment by 1 for the new task ID
   - Format: `TASK-XXX` (e.g., TASK-001, TASK-002)

3. **Create the task file:**
   - Use the template from `Tasks/TASK_TEMPLATE.md`
   - File name format: `TASK-XXX-short-description.md`
   - Example: `TASK-001-ucnet-device-discovery.md`
   - Fill in all required fields:
     - ID, Status (🔴 Not Started), Priority, Phase
     - Created date (today's date)
     - Updated date (today's date)
     - Description
     - Acceptance criteria
     - Leave Dependencies, Technical Notes, and Files Affected empty (to be filled later)

4. **Verify against PROJECT_CHARTER.md:**
   - Check if this task aligns with the current phase scope
   - If it's a Phase 1 task, verify it's one of the 7 locked features
   - If it's not in scope, warn the user and ask for confirmation

5. **Update Tasks/README.md:**
   - If this is a Phase 1 task, add it to the "Phase 1 MVP Tasks" section
   - Ensure the task list is up to date

6. **Confirm with user:**
   - Show the created task file path
   - Display the task ID and title
   - Ask if they want to start working on it now (if yes, run `/update-task`)

## Important Rules
- Always use the TASK_TEMPLATE.md as the base
- Never skip the PROJECT_CHARTER.md verification step
- Task IDs must be sequential and never reused
- All dates must be in YYYY-MM-DD format
- Status emoji must be one of: 🔴 ⏳ ✅ ❌
