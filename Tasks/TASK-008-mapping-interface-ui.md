# Task: Mapping Interface UI

**ID:** TASK-008  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP (Integration)  
**Assigned:** TBD  
**Created:** 2025-11-23  
**Updated:** 2025-11-23  

---

## Description
Create the central UI that allows users to create, view, edit, and delete parameter mappings between connected MIDI controllers and UCNet devices. This is the "patch bay" visualization mentioned in the PRD.

## Acceptance Criteria
- [ ] UI displays all available UCNet parameters (volume, mute, pan) for connected devices
- [ ] UI displays all available MIDI controls (CC, Note, Pitch Bend) from connected controllers
- [ ] User can create a mapping by selecting a MIDI control and a UCNet parameter
- [ ] User can delete existing mappings
- [ ] User can edit mapping properties (taper curve, MIDI channel)
- [ ] Mappings are visually represented as connections/links
- [ ] UI shows which parameters are currently mapped
- [ ] UI integrates with MIDI Learn feature (TASK-005)
- [ ] UI updates in real-time when mappings are created/deleted
- [ ] Empty state shown when no devices are connected

## Dependencies
- **Depends On:** TASK-001 (UCNet Discovery), TASK-002 (MIDI Enumeration), TASK-003 (Parameter Mapping), TASK-005 (MIDI Learn)
- **Blocks:** TASK-010 (End-to-End Integration)

## Technical Notes
- Should use the existing `ParameterMapping` types from `src/types/mapping.ts`
- Integrate with `useMidiLearn` hook from TASK-005
- Consider a table/list view for mappings (not a visual patch bay for MVP)
- Need to call Tauri commands to create/delete mappings in the backend
- Should display parameter names in human-readable format (e.g., "Channel 1 Volume" not "line/ch1/volume")

## Files Affected
- `src/features/MappingManager.tsx` (to be created)
- `src/components/MappingRow.tsx` (to be created)
- `src/components/ParameterSelector.tsx` (to be created)
- `src/hooks/useMappings.ts` (to be created)
- `src-tauri/src/commands/mapping.rs` (to be created)

## Testing Requirements
- [ ] Component tests for MappingManager
- [ ] Component tests for MappingRow (create, edit, delete)
- [ ] Component tests for ParameterSelector
- [ ] Hook tests for useMappings
- [ ] Integration test: Create mapping → Save → Reload → Verify persistence
- [ ] Manual testing with real devices

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (60%+ coverage for UI)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] No TypeScript `any` types
- [ ] All components have proper TypeScript interfaces
- [ ] Accessibility requirements met (keyboard navigation)

---

## Work Log

*(No work started yet)*

---

## Related Documents
- PRD: Section 4.3 - Mapping & Profiles
- PRD: Section 5 - User Interface (UX) Guidelines
- STYLE_GUIDE: Section 3 - UI Component Standards
