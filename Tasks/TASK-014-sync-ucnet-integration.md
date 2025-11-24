# Task: Wire Sync Engine to UCNet

**ID:** TASK-014  
**Status:** 🔴 Not Started  
**Priority:** Critical (P0)  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-24  
**Updated:** 2025-11-24  

---

## Description
Connect the sync engine to actually apply parameter changes to UCNet devices. Currently, MIDI messages are received and translated, but the resulting UCNet parameter changes are only logged, not applied.

## Problem Statement
In `src-tauri/src/commands/sync_integration.rs:67-72`, there's a TODO comment:
```rust
// TODO: Apply UCNet parameter changes
// This requires UCNet connection to be available
debug!(
    "Would apply UCNet change: {} ch{} {:?} = {:?}",
    result.device_id, result.channel, result.parameter_type, result.value
);
```

The sync engine produces `MappingResult` objects with UCNet parameter changes, but these are never sent to the mixer.

## Acceptance Criteria
- [ ] MIDI CC messages result in actual fader changes on the mixer
- [ ] MIDI Note On/Off messages result in actual mute changes on the mixer
- [ ] MIDI CC messages result in actual pan changes on the mixer
- [ ] UCNet parameter changes from mixer result in MIDI output to controllers
- [ ] Latency is measured end-to-end (MIDI in → UCNet out)
- [ ] Latency remains < 10ms average
- [ ] Error handling for UCNet communication failures
- [ ] Graceful degradation if UCNet device disconnects

## Dependencies
- **Depends On:** TASK-013 (UCNet Protocol Implementation)
- **Blocks:** TASK-012 (Hardware Validation)

## Technical Notes

### Current Flow (Broken)
1. MIDI message received → `sync_integration.rs:start_sync_integration()`
2. Message sent to sync engine → `engine.handle_midi_message()`
3. Sync engine returns `Vec<MappingResult>` with UCNet changes
4. **BROKEN:** Results are logged but not applied

### Required Flow
1. MIDI message received → `sync_integration.rs:start_sync_integration()`
2. Message sent to sync engine → `engine.handle_midi_message()`
3. Sync engine returns `Vec<MappingResult>` with UCNet changes
4. **NEW:** For each result, call UCNet connection to set parameter
5. Emit event to frontend for visual feedback

### Implementation Steps
1. Get reference to UCNet connection manager in sync integration
2. For each `MappingResult`, call appropriate UCNet set command
3. Handle errors (device not connected, communication failure)
4. Implement reverse direction (UCNet → MIDI):
   - Subscribe to UCNet parameter change events
   - Call `engine.handle_ucnet_change()` 
   - Send resulting MIDI messages to controllers

### Files to Modify
- `src-tauri/src/commands/sync_integration.rs` - Apply UCNet changes
- `src-tauri/src/commands/mod.rs` - May need to share UCNet state
- `src-tauri/src/main.rs` - Wire up UCNet events to sync engine

### Key Code Locations
- `sync_integration.rs:42-78` - MIDI message handler loop
- `engine.rs:139-201` - `handle_midi_message()` returns results
- `engine.rs:212-269` - `handle_ucnet_change()` for reverse direction

## Testing Requirements
- [ ] Integration test: MIDI CC → UCNet fader change
- [ ] Integration test: MIDI Note → UCNet mute change
- [ ] Integration test: UCNet change → MIDI output
- [ ] Latency test: End-to-end < 10ms
- [ ] Error handling test: UCNet disconnect during sync
- [ ] Manual testing with real hardware

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] End-to-end sync working with real hardware
- [ ] Latency < 10ms verified

---

## Work Log

*(No work started yet)*

---

## Related Documents
- TASK-009: Active Sync Integration (frontend side)
- TASK-004: Bidirectional Sync (sync engine implementation)
- PROJECT_CHARTER: Hard Constraint #1 (< 10ms latency)
