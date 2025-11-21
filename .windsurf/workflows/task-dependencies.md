# Task Dependencies

This workflow visualizes task dependencies and helps identify the critical path.

## Steps

1. **Scan all task files:**
   - Read all `TASK-*.md` files in the `Tasks/` directory
   - Extract for each task:
     - Task ID
     - Title
     - Status
     - "Depends On" field (list of task IDs)
     - "Blocks" field (list of task IDs)

2. **Build dependency graph:**
   - Create a visual representation of task dependencies
   - Use ASCII art or markdown formatting:
     ```
     TASK-001 (✅) → TASK-003 (⏳) → TASK-005 (🔴)
                  ↘ TASK-004 (🔴)
     
     TASK-002 (✅) → TASK-003 (⏳)
     ```

3. **Identify critical path:**
   - Find the longest chain of dependent tasks
   - Highlight this path as the "Critical Path"
   - Show estimated completion order

4. **Show ready-to-start tasks:**
   - List all tasks with status 🔴 that have NO incomplete dependencies
   - These are tasks that can be started immediately
   - Sort by priority (High → Medium → Low)

5. **Show blocked tasks:**
   - List all tasks with status ❌ or tasks that cannot start due to dependencies
   - For each blocked task, show:
     - What it's waiting for (specific task IDs)
     - Status of those dependencies

6. **Validate dependency integrity:**
   - Check for circular dependencies (A depends on B, B depends on A)
   - Check for orphaned references (task depends on non-existent task)
   - Warn about any issues found

7. **Suggest next task:**
   - Based on the critical path and priorities, suggest which task to work on next
   - Consider:
     - Is it on the critical path?
     - Is it high priority?
     - Are all dependencies complete?
     - Is it a Phase 1 task?

8. **Show Phase 1 completion path:**
   - Specifically for Phase 1 MVP, show the optimal order to complete all 7 tasks
   - Highlight any dependencies between Phase 1 tasks

## Important Rules
- Always validate dependency integrity before showing the graph
- Warn about circular dependencies immediately
- Prioritize Phase 1 tasks in all recommendations
- If a task is marked ⏳ In Progress, highlight it as the current focus
