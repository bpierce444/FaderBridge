# Task: Active Sync Integration

**ID:** TASK-009  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP (Integration)  
**Assigned:** TBD  
**Created:** 2025-11-23  
**Updated:** 2025-11-23  

---

## Description
Integrate the bidirectional sync engine (TASK-004) with the mapping interface (TASK-008) so that parameter changes actually flow between connected devices. This makes the mappings "live" and functional.

## Acceptance Criteria
- [ ] When a MIDI controller sends a message, the corresponding UCNet parameter updates
- [ ] When a UCNet parameter changes (via Universal Control), the MIDI controller updates
- [ ] Sync engine starts automatically when both devices are connected and mappings exist
- [ ] Sync engine stops when devices are disconnected
- [ ] Latency is measured and logged (must be < 10ms)
- [ ] Visual feedback shows sync is active (activity lights on parameters)
- [ ] User can enable/disable sync without disconnecting devices
- [ ] Sync state persists across app restarts (if enabled)
- [ ] Error handling for sync failures (device disconnects mid-operation)

## Dependencies
- **Depends On:** TASK-004 (Bidirectional Sync), TASK-008 (Mapping Interface UI)
- **Blocks:** TASK-010 (End-to-End Integration)

## Technical Notes
- Use the `SyncEngine` from `src-tauri/src/sync/engine.rs`
- Need to wire up MIDI input callbacks to call `sync_midi_to_ucnet`
- Need to wire up UCNet value change events to call `sync_ucnet_to_midi`
- Should use Tauri events for real-time updates to the UI
- Consider adding a "Sync Active" indicator in the UI
- May need to implement the reverse mapping (UCNet → MIDI) that was deferred in TASK-004

## Files Affected
- `src-tauri/src/commands/sync.rs` (extend existing)
- `src-tauri/src/main.rs` (wire up event handlers)
- `src/hooks/useActiveSync.ts` (to be created)
- `src/features/SyncStatusIndicator.tsx` (to be created)
- `src-tauri/src/sync/engine.rs` (implement reverse mapping)

## Testing Requirements
- [ ] Integration test: MIDI CC → UCNet parameter update
- [ ] Integration test: UCNet parameter → MIDI CC update
- [ ] Integration test: Feedback loop prevention (shadow state)
- [ ] Latency measurement test (must average < 10ms)
- [ ] Stress test: 100+ parameter changes per second
- [ ] Manual testing with real hardware (verify no audio glitches)
- [ ] Manual testing with motorized faders (if available)

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage for sync logic)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] **Performance requirements met (< 10ms latency)**
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] Latency metrics logged and verified

---

## Work Log

*(No work started yet)*

---

## Related Documents
- PRD: Section 4.2 - The Translation Engine (State Tracking)
- PROJECT_CHARTER: Hard Constraint #1 (< 10ms latency)
- ADR: ADR-003 (Event-Driven IPC)
- TASK-004: Bidirectional Sync implementation
