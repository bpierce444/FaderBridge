# Task: Fix Failing Tests

**ID:** TASK-017  
**Status:** 🔴 Not Started  
**Priority:** Critical (P0)  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-24  
**Updated:** 2025-11-24  

---

## Description
Fix the 17 failing frontend tests. Tests must pass before MVP release per the Definition of Done.

## Problem Statement
Running `npm test -- --run` shows:
- **4 test files failed**
- **17 tests failed** out of 154 total
- **137 tests passed**

### Failing Test Files
1. `src/hooks/useActiveSync.test.ts` - 1 failure
2. `src/hooks/useMidiLearn.test.ts` - 9 failures (all tests timing out)
3. Plus warnings about `act()` wrapping in multiple test files

## Acceptance Criteria
- [ ] All 154 tests pass
- [ ] No `act()` warnings in test output
- [ ] Tests complete in reasonable time (< 60 seconds total)
- [ ] No flaky tests (run 3x to verify)

## Dependencies
- **Depends On:** None
- **Blocks:** MVP Release

## Technical Notes

### Failure Details

#### 1. useActiveSync.test.ts (1 failure)
**Test:** "auto-initializes when starting sync if not initialized"
**Error:** `expected false to be true` at line 125
**Issue:** `status.active` is false when it should be true after `startSync()`

```typescript
// Line 123-126
await waitFor(() => {
  expect(result.current.status.initialized).toBe(true);
  expect(result.current.status.active).toBe(true);  // FAILS
});
```

**Likely Cause:** 
- Mock for `start_sync_integration` may not be updating state correctly
- Or `startSync()` isn't setting `active: true` in the hook

#### 2. useMidiLearn.test.ts (9 failures - ALL tests)
**Error:** `Test timed out in 5000ms`
**Issue:** All tests are timing out, suggesting:
- Infinite loop in hook
- Unresolved promise
- Missing mock causing hang
- Polling interval not being cleaned up

**Tests Failing:**
1. should initialize with idle state
2. should start learn mode
3. should cancel learn mode
4. should poll for state updates when learning
5. should handle ESC key to cancel learn mode
6. should handle errors when starting learn mode
7. should handle errors when cancelling learn mode
8. should refresh state manually
9. should not poll when not in learn mode

**Likely Cause:**
- The hook uses `setInterval` for polling that may not be cleaned up
- Mocks for Tauri `invoke` may not be resolving
- ESC key listener may be causing issues

### act() Warnings
Multiple warnings about state updates not wrapped in `act()`:
```
Warning: An update to TestComponent inside a test was not wrapped in act(...)
```

**Cause:** Async state updates happening outside of React's test utilities.

**Fix:** Wrap async operations in `act()` or use `waitFor()` properly.

### Investigation Steps

1. **For useActiveSync:**
   - Check mock implementation for `start_sync_integration`
   - Verify hook sets `active: true` after successful start
   - Check if there's a race condition

2. **For useMidiLearn:**
   - Check if `invoke` mock is properly set up
   - Look for uncleared intervals/timeouts
   - Check if ESC key listener is causing issues
   - Try increasing test timeout to see if it eventually completes

3. **For act() warnings:**
   - Identify which state updates are causing warnings
   - Wrap in `act()` or use `waitFor()` with proper assertions

### Files to Modify
- `src/hooks/useActiveSync.test.ts`
- `src/hooks/useMidiLearn.test.ts`
- `src/hooks/useMidiLearn.ts` (if hook has bugs)
- `src/hooks/useActiveSync.ts` (if hook has bugs)
- `src/test/setup.ts` (if mock setup is wrong)

### Test Setup Reference
Check `src/test/setup.ts` and `vitest.config.ts` for:
- Tauri mock configuration
- Global test setup
- Timeout settings

## Testing Requirements
- [ ] Run `npm test -- --run` - all tests pass
- [ ] Run tests 3 times to verify no flaky tests
- [ ] No warnings in test output
- [ ] Test duration < 60 seconds

## Definition of Done Checklist
- [ ] All 154 tests passing
- [ ] No act() warnings
- [ ] No test timeouts
- [ ] Tests are not flaky (verified with multiple runs)
- [ ] PROJECT_JOURNAL.md updated

---

## Work Log

*(No work started yet)*

---

## Related Documents
- DEFINITION_OF_DONE.md: Testing requirements
- vitest.config.ts: Test configuration
- src/test/setup.ts: Test setup and mocks
