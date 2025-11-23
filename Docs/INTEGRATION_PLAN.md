# FaderBridge Integration Plan

**Date:** 2025-11-23  
**Status:** Planning Complete  
**Phase:** Phase 1 MVP (Integration)

---

## Executive Summary

All 7 core features for Phase 1 MVP are **complete ✅**, but they exist as **isolated modules**. The app can discover devices and has all the building blocks, but there's no way for users to actually **use** the connected devices together.

**What's Missing:** The integration layer that connects all features into a cohesive, functional application.

---

## Current State Analysis

### What Works ✅
- **Device Discovery:** MIDI controllers and UCNet devices can be discovered and connected
- **Parameter Mapping:** Translation engine can convert MIDI ↔ UCNet values
- **Bidirectional Sync:** Sync engine can maintain shadow state and prevent feedback loops
- **MIDI Learn:** Users can capture MIDI messages and create mappings
- **Project Persistence:** Mappings can be saved/loaded from SQLite database
- **Visual Feedback:** UI components (Fader, MuteButton, PanKnob) exist and work in isolation

### What's Missing 🔴
1. **No Mapping Interface:** Users can't create/view/edit mappings between connected devices
2. **No Active Sync:** Sync engine is not wired up to real MIDI/UCNet events
3. **No Integrated UI:** All features are scattered, not presented in a cohesive dashboard
4. **No Reverse Mapping:** UCNet → MIDI sync is incomplete (TODO in code)
5. **No Hardware Testing:** Everything is unit tested but not validated with real devices

---

## Post-Connection User Flow (Target)

According to the **PRD Section 5** and **PROJECT_CHARTER Success Criteria**, after connecting devices:

1. **User connects MIDI controller** → Device appears in left panel with "Connected" status
2. **User connects UCNet mixer** → Device appears in right panel with "Connected" status
3. **User sees mapping interface** → Center panel shows available parameters and controls
4. **User creates mappings** → Either manually or via MIDI Learn
5. **Sync activates automatically** → Parameter changes flow bidirectionally
6. **Visual feedback updates** → On-screen faders/buttons move with hardware
7. **User saves project** → Configuration persists for next session

**Current Reality:** Steps 1-2 work. Steps 3-7 are missing.

---

## Integration Tasks Created

### TASK-008: Mapping Interface UI
**Priority:** High  
**Dependencies:** TASK-001, TASK-002, TASK-003, TASK-005 (all complete)  
**Estimated Duration:** 1-2 weeks

**What It Does:**
- Creates the central UI for managing parameter mappings
- Displays available UCNet parameters and MIDI controls
- Allows users to create, edit, and delete mappings
- Integrates with MIDI Learn feature

**Key Files:**
- `src/features/MappingManager.tsx`
- `src/components/MappingRow.tsx`
- `src/hooks/useMappings.ts`
- `src-tauri/src/commands/mapping.rs`

---

### TASK-009: Active Sync Integration
**Priority:** High  
**Dependencies:** TASK-004, TASK-008, TASK-011  
**Estimated Duration:** 1 week

**What It Does:**
- Wires up the sync engine to real MIDI/UCNet events
- Makes mappings "live" so parameter changes actually flow
- Adds sync status indicators to the UI
- Implements error handling for device disconnections

**Key Files:**
- `src-tauri/src/commands/sync.rs` (extend)
- `src-tauri/src/main.rs` (event handlers)
- `src/hooks/useActiveSync.ts`
- `src/features/SyncStatusIndicator.tsx`

---

### TASK-010: End-to-End Integration & Main UI Layout
**Priority:** High  
**Dependencies:** TASK-007, TASK-008, TASK-009  
**Estimated Duration:** 1-2 weeks

**What It Does:**
- Refactors `App.tsx` into proper dashboard layout
- Integrates all features (devices, mappings, visual feedback, projects)
- Implements top bar (project management) and status bar (sync/latency)
- Ensures responsive design and accessibility

**Key Files:**
- `src/App.tsx` (major refactor)
- `src/components/Layout.tsx`
- `src/components/TopBar.tsx`
- `src/components/StatusBar.tsx`

---

### TASK-011: UCNet → MIDI Reverse Mapping
**Priority:** High  
**Dependencies:** TASK-003, TASK-004 (all complete)  
**Estimated Duration:** 1 week

**What It Does:**
- Implements the reverse lookup (UCNet parameter → MIDI mapping)
- Adds reverse taper curve calculations
- Enables motorized fader support
- Addresses the TODO in `sync/engine.rs`

**Key Files:**
- `src-tauri/src/translation/mapper.rs` (add reverse methods)
- `src-tauri/src/sync/engine.rs` (implement `sync_ucnet_to_midi`)
- `src-tauri/src/midi/connection.rs` (MIDI output)

---

### TASK-012: Hardware Validation & Performance Testing
**Priority:** High  
**Dependencies:** TASK-010  
**Estimated Duration:** 2 weeks

**What It Does:**
- Tests with real PreSonus hardware (Series III mixer)
- Tests with multiple MIDI controllers
- Validates < 10ms latency requirement
- Performs 4-hour stability test
- Measures memory usage and performance

**Deliverables:**
- `Docs/HARDWARE_TEST_REPORT.md`
- `Docs/PERFORMANCE_METRICS.md`

---

## Recommended Work Order

### Sprint 5: Integration Layer (Weeks 9-10)
**Parallel Work:**
1. **TASK-008:** Mapping Interface UI (Priority 1)
2. **TASK-011:** UCNet → MIDI Reverse Mapping (Can work in parallel)

**Rationale:** Both tasks depend only on completed features and can be developed independently.

### Sprint 6: Active Sync (Week 11)
3. **TASK-009:** Active Sync Integration

**Rationale:** Requires both TASK-008 and TASK-011 to be complete.

### Sprint 7: Final Integration (Week 12)
4. **TASK-010:** End-to-End Integration & Main UI Layout

**Rationale:** Brings everything together into cohesive user experience.

### Sprint 8: Validation & Release (Weeks 13-14)
5. **TASK-012:** Hardware Validation & Performance Testing

**Rationale:** Final validation before MVP release.

---

## Critical Path

```
TASK-011 → TASK-009 → TASK-010 → TASK-012
(Reverse)  (Active)   (End-to-End) (Hardware)
(Mapping)  (Sync)     (Integration) (Validation)
```

**Total Duration:** 6 weeks (assuming full-time work)

---

## Risks & Mitigation

### High Risk
- **TASK-009 (Active Sync):** Complex state management with real-time events
  - *Mitigation:* Use existing event-driven architecture (ADR-003), comprehensive testing
  
- **TASK-012 (Hardware Validation):** Cannot test without physical devices
  - *Mitigation:* Ensure all unit/integration tests pass first, have hardware ready

### Medium Risk
- **TASK-011 (Reverse Mapping):** Edge cases in taper curve inversion
  - *Mitigation:* Property-based testing, manual validation with known values

- **TASK-010 (End-to-End Integration):** May reveal architectural issues
  - *Mitigation:* Incremental integration, frequent testing

### Low Risk
- **TASK-008 (Mapping Interface):** Standard CRUD UI
  - *Mitigation:* Follow existing patterns from TASK-007

---

## Success Criteria

Phase 1 MVP is complete when:
1. ✅ User can connect a MIDI controller to a Series III mixer
2. ✅ User can create 8 fader mappings using MIDI Learn
3. ✅ Moving a physical fader updates the mixer within 10ms
4. ✅ Moving a mixer fader updates the MIDI controller
5. ✅ User can save and reload the mapping
6. ✅ App runs for 4 hours without crashes or memory leaks

**Current Status:** 0/6 criteria met (all require integration tasks)

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Start TASK-008** (Mapping Interface UI) immediately
3. **Start TASK-011** (Reverse Mapping) in parallel if resources allow
4. **Update PROJECT_JOURNAL.md** after each work session
5. **Track progress** in Tasks/README.md and DEPENDENCY_GRAPH.md

---

## References
- **PRD:** `Docs/PRD_FaderBridge.md` (Section 4.2, 4.3, 5)
- **Project Charter:** `Docs/PROJECT_CHARTER.md` (Success Criteria)
- **Dependency Graph:** `Tasks/DEPENDENCY_GRAPH.md`
- **Task Files:** `Tasks/TASK-008.md` through `TASK-012.md`
