# Task: Hardware Validation & Performance Testing

**ID:** TASK-012  
**Status:** � In Progress  
**Priority:** High  
**Phase:** Phase 1 MVP (Validation)  
**Assigned:** Cascade AI  
**Created:** 2025-11-23  
**Updated:** 2025-11-24  

---

## Description
Perform comprehensive testing with real hardware (MIDI controllers and UCNet devices) to validate all Phase 1 features work correctly in production. This includes performance testing, stability testing, and user acceptance testing.

## Acceptance Criteria
- [ ] App successfully connects to a real Series III mixer (network and USB)
- [ ] App successfully connects to at least 3 different MIDI controllers
- [ ] MIDI → UCNet latency measured at < 10ms average
- [ ] UCNet → MIDI latency measured at < 10ms average
- [ ] Motorized faders update smoothly (if controller supports them)
- [ ] No audio glitches or zipper noise during parameter changes
- [ ] App runs for 4+ hours without crashes or memory leaks
- [ ] Memory usage stays below 150MB after 1 hour of operation
- [ ] App launches in < 2 seconds on target hardware
- [ ] All keyboard shortcuts work as expected
- [ ] Project save/load works reliably with 50+ mappings
- [ ] Auto-save does not cause UI lag or audio glitches

## Dependencies
- **Depends On:** TASK-010 (End-to-End Integration)
- **Blocks:** Phase 1 MVP Release

## Technical Notes
- Requires access to:
  - PreSonus Series III mixer (32S, 32SC, or 64S)
  - Multiple MIDI controllers (FaderPort, X-Touch, nanoKONTROL, etc.)
  - Universal Control software (for UCNet parameter changes)
- Use `tokio::time::Instant` for latency measurements
- Use Activity Monitor (macOS) for memory profiling
- Consider using Instruments.app for detailed performance profiling
- Document all hardware configurations tested

## Testing Checklist

### Connection Testing
- [ ] Network discovery finds mixer within 2 seconds
- [ ] USB discovery finds mixer within 2 seconds
- [ ] MIDI hot-plug detection works (connect/disconnect during runtime)
- [ ] UCNet hot-plug detection works (power on/off mixer during runtime)
- [ ] Keep-alive maintains connection for 4+ hours
- [ ] Graceful handling of network interruptions

### Mapping Testing
- [ ] MIDI Learn works with all tested controllers
- [ ] Volume mappings work with all taper curves (Linear, Log, Audio)
- [ ] Mute mappings work with Note On/Off and CC
- [ ] Pan mappings work with rotary knobs and faders
- [ ] 14-bit MIDI CC works with high-resolution faders
- [ ] Multiple mappings to the same parameter work correctly

### Sync Testing
- [ ] MIDI → UCNet sync works with < 10ms latency
- [ ] UCNet → MIDI sync works with < 10ms latency
- [ ] Shadow state prevents feedback loops
- [ ] Rapid parameter changes (100+/sec) handled without issues
- [ ] Motorized faders update smoothly (if available)
- [ ] No audio glitches during parameter changes

### Persistence Testing
- [ ] Save project with 50+ mappings
- [ ] Load project and verify all mappings restored
- [ ] Auto-save works every 30 seconds
- [ ] Export/Import projects as JSON
- [ ] Project persists across app restarts

### Performance Testing
- [ ] App launches in < 2 seconds
- [ ] Memory usage < 150MB after 1 hour
- [ ] No memory leaks during 4-hour stress test
- [ ] CPU usage < 10% during normal operation
- [ ] UI remains responsive during heavy I/O

### Stability Testing
- [ ] No crashes during 4-hour stress test
- [ ] Graceful error handling for all failure modes
- [ ] No console errors or warnings during normal operation
- [ ] App recovers from device disconnections

## Files Affected
- `Docs/HARDWARE_TEST_REPORT.md` (to be created)
- `Docs/PERFORMANCE_METRICS.md` (to be created)

## Definition of Done Checklist
- [ ] All testing checklist items completed
- [ ] Hardware test report documented
- [ ] Performance metrics documented
- [ ] All critical bugs fixed
- [ ] PROJECT_JOURNAL.md updated
- [ ] Known issues documented (if any)

---

## Work Log

### 2025-11-24 - Test Infrastructure Setup

**Documentation Created:**
1. **HARDWARE_TEST_REPORT.md** - Comprehensive test report template with:
   - Hardware configuration tables
   - Connection testing checklists
   - Mapping testing procedures
   - Sync testing with latency measurement tables
   - Persistence testing checklists
   - Performance metrics tables
   - Stability testing procedures
   - Test procedures appendix

2. **PERFORMANCE_METRICS.md** - Performance tracking document with:
   - Target metrics from PROJECT_CHARTER.md
   - Latency measurement breakdown by stage
   - Memory usage tracking tables
   - CPU usage benchmarks
   - Startup performance metrics
   - Throughput metrics
   - UI performance metrics
   - Profiling tool instructions

**Existing Infrastructure Verified:**
- `SyncEngine` already has latency measurement (records samples, calculates avg/min/max)
- `get_latency_stats` Tauri command exposes metrics to frontend
- `SyncStatusIndicator` component displays latency with color-coded status:
  - Green (< 10ms): Target met
  - Amber (10-20ms): Slow warning
  - Red (> 20ms): Lagging alert
- `useActiveSync` hook polls latency stats every 1 second when active

**Next Steps:**
- Manual testing with real hardware required
- Fill in test report with actual measurements
- Document any issues found during testing

---

## Related Documents
- PROJECT_CHARTER: Quality Gates
- PROJECT_CHARTER: Success Criteria (Phase 1 MVP)
- TESTING_STRATEGY: Manual Testing Guidelines
