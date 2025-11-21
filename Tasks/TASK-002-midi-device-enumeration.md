# Task: MIDI Device Enumeration

**ID:** TASK-002  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

---

## Description
Implement detection and enumeration of all available MIDI devices (USB MIDI, Bluetooth MIDI, Virtual MIDI ports). Provide a clean interface for the UI to display available controllers.

## Acceptance Criteria
- [ ] App detects all USB MIDI devices connected to the system
- [ ] App detects Bluetooth MIDI devices (if supported by OS)
- [ ] App detects Virtual MIDI ports (e.g., IAC Driver on macOS)
- [ ] Device information extracted (name, manufacturer, port IDs)
- [ ] Hot-plug detection (devices added/removed while app is running)
- [ ] UI displays list of available MIDI devices with connection status
- [ ] Device selection persists across app restarts

## Dependencies
- **Depends On:** None (foundational task)
- **Blocks:** TASK-003 (Basic Parameter Mapping), TASK-005 (MIDI Learn)

## Technical Notes
- Consider using `midir` crate for cross-platform MIDI support
- macOS: CoreMIDI framework
- Need to handle MIDI device naming inconsistencies across platforms
- Virtual MIDI ports may have different behavior than physical devices
- Hot-plug detection requires OS-specific event monitoring

## Files Affected
- `src-tauri/src/midi/enumeration.rs` (to be created)
- `src-tauri/src/midi/device.rs` (to be created)
- `src-tauri/src/midi/mod.rs` (to be created)
- `src/features/MidiDeviceList.tsx` (to be created)

## Testing Requirements
- [ ] Unit tests for device info parsing
- [ ] Mock tests for device enumeration
- [ ] Integration test for hot-plug detection
- [ ] Manual testing with multiple MIDI controllers
- [ ] Manual testing with virtual MIDI ports
- [ ] Manual testing of device connect/disconnect events

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments

---

## Work Log

*No work log entries yet*

---

## Related Documents
- PRD: Section 4.1 - MIDI Engine
- ADR: ADR-004 (Mock Hardware I/O Using Traits)
