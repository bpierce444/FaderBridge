# Task: Basic Parameter Mapping (Volume, Mute, Pan)

**ID:** TASK-003  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

---

## Description
Implement the core translation engine that maps MIDI messages (CC, Note) to UCNet parameters. Focus on the three essential parameters: Volume (faders), Mute (buttons), and Pan (knobs). This is the heart of the application.

## Acceptance Criteria
- [ ] MIDI CC messages (0-127) correctly translated to UCNet volume (0.0-1.0 or dB)
- [ ] MIDI Note On/Off messages correctly mapped to UCNet mute state (on/off)
- [ ] MIDI CC messages correctly translated to UCNet pan (-1.0 to 1.0)
- [ ] Customizable taper curves (Linear, Logarithmic, Audio Taper) for faders
- [ ] Parameter normalization prevents "zipper noise" on audio
- [ ] Support for 14-bit MIDI CC (high-resolution faders)
- [ ] Mapping data structure supports saving/loading (JSON serializable)

## Dependencies
- **Depends On:** TASK-001 (UCNet Device Discovery), TASK-002 (MIDI Device Enumeration)
- **Blocks:** TASK-004 (Bidirectional Sync), TASK-005 (MIDI Learn), TASK-006 (Save/Load Projects)

## Technical Notes
- UCNet volume: 0.0 to 1.0 float OR -∞ to +10 dB (need to verify protocol)
- UCNet mute: boolean (true/false)
- UCNet pan: -1.0 (left) to +1.0 (right), 0.0 = center
- MIDI CC: 0-127 (7-bit) or 0-16383 (14-bit for MSB/LSB pairs)
- Audio taper formula: `output = input^2.5` (approximation)
- Logarithmic taper: `output = log(input + 1) / log(2)`
- Consider using `serde` for JSON serialization

## Files Affected
- `src-tauri/src/translation/mapper.rs` (to be created)
- `src-tauri/src/translation/taper.rs` (to be created)
- `src-tauri/src/translation/types.rs` (to be created)
- `src-tauri/src/translation/mod.rs` (to be created)
- `src/types/mapping.ts` (to be created)

## Testing Requirements
- [ ] Unit tests for MIDI to UCNet conversion (all parameter types)
- [ ] Unit tests for each taper curve algorithm
- [ ] Unit tests for 14-bit MIDI CC handling
- [ ] Unit tests for edge cases (0, 127, overflow)
- [ ] Property-based tests for normalization (no zipper noise)
- [ ] Integration test with mock MIDI and UCNet devices
- [ ] Manual testing with real hardware (verify audio quality)

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] No audible zipper noise on fader movement

---

## Work Log

*No work log entries yet*

---

## Related Documents
- PRD: Section 4.2 - The Translation Engine
- ADR: ADR-002 (SQLite for Persistence)
