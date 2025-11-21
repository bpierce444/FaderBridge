# Task: MIDI Learn Functionality

**ID:** TASK-005  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

---

## Description
Implement the MIDI Learn feature that allows users to quickly map MIDI controllers to mixer parameters by clicking a parameter and moving a physical control. This is essential for supporting generic MIDI controllers.

## Acceptance Criteria
- [ ] User can enter "MIDI Learn Mode" via UI button
- [ ] User clicks on a mixer parameter (volume/mute/pan)
- [ ] App listens for the next MIDI message from the controller
- [ ] Mapping is created automatically and displayed in UI
- [ ] User can cancel MIDI Learn mode (ESC key or Cancel button)
- [ ] Visual feedback shows which parameter is waiting for MIDI input
- [ ] Duplicate mappings are detected and user is warned
- [ ] MIDI Learn works for CC, Note On/Off, and Pitch Bend messages

## Dependencies
- **Depends On:** TASK-002 (MIDI Device Enumeration), TASK-003 (Basic Parameter Mapping)
- **Blocks:** TASK-006 (Save/Load Projects)

## Technical Notes
- MIDI Learn mode should timeout after 10 seconds of inactivity
- Need to filter out MIDI clock and active sensing messages
- Consider highlighting the UI element that's in "learn mode"
- Store learned mappings in the same format as manual mappings
- May need to debounce rapid MIDI messages during learning

## Files Affected
- `src-tauri/src/midi/learn.rs` (to be created)
- `src-tauri/src/commands/learn.rs` (to be created)
- `src/features/MidiLearn.tsx` (to be created)
- `src/hooks/useMidiLearn.ts` (to be created)
- `src/components/LearnButton.tsx` (to be created)

## Testing Requirements
- [ ] Unit tests for MIDI message filtering
- [ ] Unit tests for duplicate detection
- [ ] Mock tests for learn mode state machine
- [ ] Integration test for full learn workflow
- [ ] Manual testing with various MIDI controllers
- [ ] Manual testing of timeout behavior
- [ ] Manual testing of cancel functionality

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] Keyboard shortcuts work (ESC to cancel)

---

## Work Log

*No work log entries yet*

---

## Related Documents
- PRD: Section 4.4 - "Intuitive" Learn Modes
- STYLE_GUIDE: Section 3.2 - User Experience
