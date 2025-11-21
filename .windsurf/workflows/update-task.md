# Update Task

This workflow guides you through updating an existing task's progress and status.

## Steps

1. **Identify the task:**
   - Ask the user which task to update (by ID or title)
   - If they don't know, run `/list-tasks` first
   - Read the task file from `Tasks/TASK-XXX-*.md`

2. **Show current status:**
   - Display the task ID, title, and current status
   - Show the last work log entry (if any)
   - Show incomplete acceptance criteria

3. **Ask for updates:**
   - What work was done in this session?
   - How much time was spent? (duration)
   - Were any blockers encountered?
   - What are the next steps?
   - Should the status change? (🔴 → ⏳ → ✅ or ❌)

4. **Update the task file:**
   - Change the "Updated" date to today
   - Update the "Status" field if changed
   - Add a new entry to the "Work Log" section with:
     - Today's date
     - Session title (brief summary)
     - Duration
     - Progress made
     - Blockers (if any)
     - Next steps
   - Check off any completed acceptance criteria

5. **Update related files:**
   - If any files were modified, add them to "Files Affected" section
   - If technical notes were discovered, add them to "Technical Notes"
   - If dependencies were identified, update the "Dependencies" section

6. **Check Definition of Done:**
   - If status is being changed to ✅ Complete, verify ALL items in "Definition of Done Checklist" are checked
   - If not all items are checked, warn the user and ask for confirmation
   - Remind them to update PROJECT_JOURNAL.md (mandatory)

7. **Update PROJECT_JOURNAL.md (if task completed):**
   - If the task status changed to ✅, ask if they want to update the journal now
   - If yes, add an entry to PROJECT_JOURNAL.md with:
     - The task that was completed
     - What was learned
     - Any blockers encountered
     - Next task to work on

8. **Show summary:**
   - Display the updated task status
   - Show remaining acceptance criteria (if any)
   - Show next steps
   - If there are blockers, highlight them in red

## Important Rules
- Always update the "Updated" date field
- Never mark a task ✅ Complete without verifying Definition of Done
- Always add a work log entry, even for small updates
- If a task is blocked (❌), clearly document WHY in the work log
- Remind the user about PROJECT_JOURNAL.md update requirement
