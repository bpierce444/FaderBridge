# Task: Integrate Visual Feedback Components

**ID:** TASK-015  
**Status:** 🔴 Not Started  
**Priority:** Critical (P0)  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-24  
**Updated:** 2025-11-24  

---

## Description
Integrate the existing visual feedback components (`MixerStrip`, `Fader`, `MuteButton`, `PanKnob`) into the main application UI. These components exist and are tested but are not rendered anywhere in the app.

## Problem Statement
The following components exist but are never imported or used:
- `src/components/MixerStrip.tsx` - Not imported anywhere
- `src/components/Fader.tsx` - Only imported by MixerStrip
- `src/components/MuteButton.tsx` - Only imported by MixerStrip
- `src/components/PanKnob.tsx` - Only imported by MixerStrip
- `src/components/ActivityLight.tsx` - Used in StatusBar but hardcoded to `active={false}`

Users need to see real-time visual feedback when:
1. They move a physical MIDI fader
2. Someone changes a parameter on the mixer (via Universal Control)
3. Sync is active and parameters are being translated

## Acceptance Criteria
- [ ] MixerStrip components displayed in the center panel for mapped channels
- [ ] Faders move in real-time when MIDI messages are received
- [ ] Mute buttons toggle when MIDI note messages are received
- [ ] Pan knobs rotate when MIDI CC messages are received
- [ ] Visual feedback updates when UCNet parameters change
- [ ] Activity lights in StatusBar pulse when MIDI/UCNet activity occurs
- [ ] Performance: 60fps animation even with 8+ channels displayed
- [ ] Accessible: Keyboard navigation works for all controls

## Dependencies
- **Depends On:** TASK-009 (Active Sync Integration - for events)
- **Blocks:** None (but completes MVP user experience)

## Technical Notes

### Components to Integrate

1. **MixerStrip** (`src/components/MixerStrip.tsx`)
   - Combines Fader, MuteButton, PanKnob
   - Uses `useParameterValue` hook for real-time updates
   - Needs to be rendered for each mapped UCNet channel

2. **ActivityLight** (`src/components/ActivityLight.tsx`)
   - Currently hardcoded to `active={false}` in StatusBar
   - Needs to be wired to actual MIDI/UCNet events

### Integration Points

**Option A: Add to MappingManager**
- Show MixerStrip for each unique UCNet channel in mappings
- Pros: Directly related to mappings
- Cons: May clutter the mapping interface

**Option B: Create new MixerView feature**
- Dedicated view showing all mapped channels as mixer strips
- Pros: Clean separation, familiar mixer layout
- Cons: More work, need to add navigation

**Option C: Add to Dashboard center panel**
- Show MixerStrips alongside MappingManager
- Pros: Everything visible at once
- Cons: May be too busy

### Recommended Approach (Option A + Activity Lights)
1. Add a "Visual Feedback" section to MappingManager
2. For each unique UCNet device+channel in mappings, render a MixerStrip
3. Wire ActivityLights in StatusBar to sync events

### Files to Modify
- `src/features/MappingManager.tsx` - Add MixerStrip rendering
- `src/features/Dashboard.tsx` - May need layout adjustments
- `src/components/StatusBar.tsx` - Wire activity lights to events
- `src/hooks/useParameterValue.ts` - Ensure it receives sync events

### Event Flow for Real-Time Updates
1. Sync engine emits `sync:parameter-synced` event
2. `useParameterValue` hook listens for events matching its parameter
3. Hook updates local state, triggering re-render
4. Fader/Knob/Button animates to new position

### Activity Light Wiring
```tsx
// In StatusBar.tsx, replace:
<ActivityLight active={false} size={8} />

// With:
<ActivityLight active={midiActivity} size={8} />
```

Need to add state tracking for recent MIDI/UCNet activity (e.g., true for 500ms after last event).

## Testing Requirements
- [ ] Component renders MixerStrips for mapped channels
- [ ] Fader updates when sync event received
- [ ] MuteButton updates when sync event received
- [ ] PanKnob updates when sync event received
- [ ] Activity lights pulse on MIDI activity
- [ ] Activity lights pulse on UCNet activity
- [ ] Performance test: 60fps with 8 channels
- [ ] Accessibility: Keyboard navigation works

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (60%+ coverage for UI)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] No TypeScript `any` types
- [ ] All components have proper TypeScript interfaces
- [ ] Accessibility requirements met
- [ ] Visual feedback visible and responsive

---

## Work Log

*(No work started yet)*

---

## Related Documents
- TASK-007: Visual Feedback (component implementation)
- TASK-010: End-to-End Integration
- PRD: Section 5 - User Interface (UX) Guidelines
- STYLE_GUIDE: Section 3 - UI Component Standards
