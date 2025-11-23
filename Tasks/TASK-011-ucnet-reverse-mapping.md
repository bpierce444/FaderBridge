# Task: UCNet → MIDI Reverse Mapping

**ID:** TASK-011  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP (Integration)  
**Assigned:** TBD  
**Created:** 2025-11-23  
**Updated:** 2025-11-23  

---

## Description
Implement the reverse mapping functionality (UCNet parameter changes → MIDI controller updates) that was deferred in TASK-004. This is required for true bidirectional sync and motorized fader support.

## Acceptance Criteria
- [ ] When a UCNet parameter changes, the system can look up the corresponding MIDI mapping
- [ ] MIDI output messages are sent to the controller with the correct CC/Note/Pitch Bend
- [ ] Reverse mapping supports all parameter types (volume, mute, pan)
- [ ] Reverse mapping respects taper curves (converts UCNet float back to MIDI value)
- [ ] Shadow state prevents feedback loops (UCNet → MIDI → UCNet → ...)
- [ ] Motorized faders update smoothly (if controller supports them)
- [ ] Latency is < 10ms for reverse mapping
- [ ] Works with both 7-bit and 14-bit MIDI CC

## Dependencies
- **Depends On:** TASK-003 (Basic Parameter Mapping), TASK-004 (Bidirectional Sync)
- **Blocks:** TASK-009 (Active Sync Integration)

## Technical Notes
- This addresses the TODO in `src-tauri/src/sync/engine.rs` (line ~80)
- Need to create a reverse lookup table: `HashMap<UcNetAddress, Vec<ParameterMapping>>`
- Reverse taper curve: Given UCNet value (0.0-1.0), compute MIDI value (0-127 or 0-16383)
- For 14-bit CC, need to split value into MSB and LSB and send both messages
- Consider caching the reverse lookup table for performance
- May need to debounce rapid UCNet changes to avoid flooding MIDI output

## Files Affected
- `src-tauri/src/translation/mapper.rs` (add reverse mapping methods)
- `src-tauri/src/sync/engine.rs` (implement `sync_ucnet_to_midi`)
- `src-tauri/src/midi/connection.rs` (add MIDI output methods)

## Testing Requirements
- [ ] Unit tests for reverse taper curve calculations
- [ ] Unit tests for UCNet float → MIDI value conversion
- [ ] Unit tests for 14-bit MIDI CC splitting (MSB/LSB)
- [ ] Unit tests for reverse lookup table
- [ ] Integration test: UCNet change → MIDI output
- [ ] Integration test: Feedback loop prevention
- [ ] Manual testing with motorized faders (if available)

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met (< 10ms latency)
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments

---

## Work Log

*(No work started yet)*

---

## Related Documents
- PRD: Section 4.2 - The Translation Engine
- TASK-004: Bidirectional Sync (TODO comment)
- PROJECT_JOURNAL: Known Issues #1 (UCNet → MIDI Reverse Mapping)
