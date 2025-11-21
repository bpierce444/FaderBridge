# Task: UCNet Device Discovery

**ID:** TASK-001  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

---

## Description
Implement automatic discovery and connection to PreSonus UCNet devices on both network (UDP/TCP) and USB connections. This is the foundation for all mixer communication.

## Acceptance Criteria
- [ ] App detects Series III mixers on local network via UDP broadcast within 2 seconds
- [ ] App enumerates USB-connected UCNet devices using PreSonus VID (0x194f)
- [ ] Connection state is tracked and maintained (connected/disconnected)
- [ ] Keep-alive heartbeat packets sent every 5 seconds to maintain connection
- [ ] Device information extracted (model, firmware version, IP/USB identifier)
- [ ] UI displays discovered devices with connection status indicator
- [ ] Graceful handling of device disconnection and reconnection

## Dependencies
- **Depends On:** None (foundational task)
- **Blocks:** TASK-003 (Basic Parameter Mapping), TASK-004 (Bidirectional Sync)

## Technical Notes
- UCNet discovery uses UDP port 47809 for broadcast
- USB devices use PreSonus vendor ID: 0x194f
- Keep-alive packets required every 5 seconds or connection drops
- Need to handle both IPv4 and IPv6 networks
- Consider using `tokio::net::UdpSocket` for async network operations
- For USB, investigate `libusb` or `rusb` crate

## Files Affected
- `src-tauri/src/ucnet/discovery.rs` (to be created)
- `src-tauri/src/ucnet/connection.rs` (to be created)
- `src-tauri/src/ucnet/mod.rs` (to be created)
- `src/features/DeviceManager.tsx` (to be created)

## Testing Requirements
- [ ] Unit tests for UDP broadcast parsing
- [ ] Unit tests for USB device enumeration
- [ ] Mock tests for connection state management
- [ ] Integration test for keep-alive mechanism
- [ ] Manual testing with real Series III mixer (network)
- [ ] Manual testing with real Series III mixer (USB)

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (90%+ coverage)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met (< 2s discovery time)
- [ ] No `.unwrap()` in production code
- [ ] All public functions have doc comments

---

## Work Log

*No work log entries yet*

---

## Related Documents
- PRD: Section 4.1 - Connectivity & Protocol Handling
- ADR: ADR-003 (Event-Driven IPC)
- ADR: ADR-004 (Mock Hardware I/O Using Traits)
