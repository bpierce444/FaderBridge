# Task: UCNet Protocol Implementation

**ID:** TASK-013  
**Status:** 🔴 Not Started  
**Priority:** Critical (P0)  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-24  
**Updated:** 2025-11-24  

---

## Description
Implement the actual UCNet protocol for communication with PreSonus Series III mixers. The current implementation has placeholder code that cannot communicate with real hardware. This is the most critical blocker for MVP.

## Problem Statement
The following files contain placeholder implementations that need real protocol code:
- `src-tauri/src/ucnet/discovery.rs` - Discovery packet format and response parsing
- `src-tauri/src/ucnet/connection.rs` - Connection handshake, keep-alive, and parameter control

## Acceptance Criteria
- [ ] Discovery packets follow actual UCNet protocol specification
- [ ] Response parsing extracts real device info (model, firmware, device ID)
- [ ] Network connection handshake establishes valid session with mixer
- [ ] USB connection opens device and establishes communication channel
- [ ] Keep-alive packets maintain connection (5-second interval)
- [ ] Parameter read: Can query current fader/mute/pan values from mixer
- [ ] Parameter write: Can set fader/mute/pan values on mixer
- [ ] Connection timeout and error handling works correctly
- [ ] Works with StudioLive 32S, 32SC, and 64S (Series III)

## Dependencies
- **Depends On:** None (foundational)
- **Blocks:** TASK-014 (Sync Integration), TASK-012 (Hardware Validation)

## Technical Notes

### Current Placeholder Locations
1. `discovery.rs:170-179` - `create_discovery_packet()` sends fake "UCNET_DISCOVER" bytes
2. `discovery.rs:187-205` - `parse_discovery_response()` returns hardcoded device info
3. `connection.rs:246-255` - `send_keepalive_packet()` does nothing
4. `connection.rs:259-269` - `connect_network()` doesn't perform handshake
5. `connection.rs:273-279` - `connect_usb()` doesn't open USB device

### Protocol Research Required
- UCNet uses UDP port 47809 for discovery
- TCP connection for control (port TBD)
- Need to capture/analyze packets from Universal Control software
- PreSonus vendor ID: 0x194f
- Consider using Wireshark to capture UCNet traffic

### Suggested Implementation Approach
1. Use Wireshark to capture discovery broadcast from Universal Control
2. Analyze discovery response packet structure
3. Implement discovery packet creation and parsing
4. Capture connection handshake sequence
5. Implement parameter get/set commands
6. Test with real hardware

### Files to Modify
- `src-tauri/src/ucnet/discovery.rs`
- `src-tauri/src/ucnet/connection.rs`
- `src-tauri/src/ucnet/types.rs` (may need new packet types)
- `src-tauri/src/ucnet/mod.rs` (may need new modules)

### New Files to Create
- `src-tauri/src/ucnet/protocol.rs` - Protocol constants and packet structures
- `src-tauri/src/ucnet/commands.rs` - Parameter get/set command implementations

## Testing Requirements
- [ ] Unit tests for packet creation/parsing
- [ ] Unit tests for parameter value encoding/decoding
- [ ] Integration test with mock UCNet server
- [ ] Manual testing with real Series III mixer (network)
- [ ] Manual testing with real Series III mixer (USB)

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments
- [ ] Successfully connects to real mixer
- [ ] Can read/write at least one parameter type

---

## Work Log

*(No work started yet)*

---

## Related Documents
- PRD: Section 4.1 - Connectivity & Protocol Handling
- ADR: ADR-004 (Mock Hardware I/O Using Traits)
- PROJECT_CHARTER: Hard Constraint #3 (Compatibility with Series III)
