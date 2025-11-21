# Task: Bidirectional Sync (< 10ms Latency)

**ID:** TASK-004  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

---

## Description
Implement bidirectional synchronization between MIDI controllers and UCNet devices with < 10ms round-trip latency. Changes on the mixer must update the MIDI controller, and vice versa. This is critical for the "live feel" of the application.

## Acceptance Criteria
- [ ] MIDI controller changes reflected on UCNet mixer within 10ms
- [ ] UCNet mixer changes (via Universal Control or hardware) reflected on MIDI controller within 10ms
- [ ] Shadow state maintained to prevent feedback loops
- [ ] Motorized faders on MIDI controllers update smoothly (if supported)
- [ ] No race conditions or state conflicts
- [ ] Latency measured and logged (must be < 10ms average)
- [ ] Event-driven architecture (no polling from frontend)

## Dependencies
- **Depends On:** TASK-001 (UCNet Device Discovery), TASK-002 (MIDI Device Enumeration), TASK-003 (Basic Parameter Mapping)
- **Blocks:** TASK-007 (Visual Feedback)

## Technical Notes
- Use Tauri's event system for backend-to-frontend communication (ADR-003)
- Shadow state prevents infinite feedback loops (A→B→A→B...)
- Latency target: < 10ms round-trip (MIDI → UCNet → MIDI)
- Consider using `tokio::time::Instant` for latency measurement
- Thread priority may need adjustment for real-time performance
- Debouncing may be needed for rapid parameter changes

## Files Affected
- `src-tauri/src/sync/engine.rs` (to be created)
- `src-tauri/src/sync/shadow_state.rs` (to be created)
- `src-tauri/src/sync/mod.rs` (to be created)
- `src-tauri/src/commands/sync.rs` (to be created)
- `src/hooks/useSync.ts` (to be created)

## Testing Requirements
- [ ] Unit tests for shadow state management
- [ ] Unit tests for feedback loop prevention
- [ ] Integration test for MIDI → UCNet → MIDI round-trip
- [ ] Latency measurement tests (must average < 10ms)
- [ ] Stress test with rapid parameter changes (100+ updates/sec)
- [ ] Manual testing with real hardware (verify no audio glitches)
- [ ] Manual testing of motorized fader response

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] **Performance requirements met (< 10ms latency)**
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] Latency metrics logged and verified

---

## Work Log

*No work log entries yet*

---

## Related Documents
- PRD: Section 4.2 - State Tracking
- ADR: ADR-003 (Event-Driven IPC)
- PROJECT_CHARTER: Hard Constraint #1 (< 10ms latency)
