# Task: Integrate Visual Feedback Components

**ID:** TASK-015  
**Status:** ✅ Complete  
**Priority:** Critical (P0)  
**Phase:** Phase 1 MVP  
**Assigned:** Cascade  
**Created:** 2025-11-24  
**Updated:** 2025-11-26  

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
- [x] MixerStrip components displayed in the center panel for mapped channels
- [x] Faders move in real-time when MIDI messages are received
- [x] Mute buttons toggle when MIDI note messages are received
- [x] Pan knobs rotate when MIDI CC messages are received
- [x] Visual feedback updates when UCNet parameters change
- [x] Activity lights in StatusBar pulse when MIDI/UCNet activity occurs
- [ ] Performance: 60fps animation even with 8+ channels displayed (requires hardware testing)
- [x] Accessible: Keyboard navigation works for all controls

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
- [x] Component renders MixerStrips for mapped channels
- [x] Fader updates when sync event received
- [x] MuteButton updates when sync event received
- [x] PanKnob updates when sync event received
- [x] Activity lights pulse on MIDI activity
- [x] Activity lights pulse on UCNet activity
- [ ] Performance test: 60fps with 8 channels (requires hardware testing)
- [x] Accessibility: Keyboard navigation works

## Definition of Done Checklist
- [x] Code follows AI_CODING_RULES.md
- [x] Tests written and passing (60%+ coverage for UI)
- [x] Documentation updated
- [x] PROJECT_JOURNAL.md updated
- [x] No compiler warnings
- [x] No TypeScript `any` types
- [x] All components have proper TypeScript interfaces
- [x] Accessibility requirements met
- [x] Visual feedback visible and responsive

---

## Work Log

### 2025-11-26 - Implementation Complete
**Duration:** ~1 hour

**Changes Made:**
1. Created `useActivityIndicator` hook (`src/hooks/useActivityIndicator.ts`)
   - Tracks activity from Tauri backend events with 500ms auto-timeout
   - Listens for configurable event names
   - Provides `triggerActivity()` and `reset()` methods

2. Updated `StatusBar` component (`src/components/StatusBar.tsx`)
   - Wired MIDI activity light to: `midi:message-received`, `parameter-update`, `sync:midi-to-ucnet`
   - Wired UCNet activity light to: `ucnet:parameter-changed`, `sync:ucnet-to-midi`, `sync:parameter-synced`
   - Added proper ARIA labels for accessibility

3. Updated `MappingManager` component (`src/features/MappingManager.tsx`)
   - Added `VisualFeedbackSection` component
   - Added `extractUniqueChannels()` helper function
   - Renders MixerStrips for each unique UCNet channel in mappings
   - Horizontal scrollable layout for multiple channels

4. Created tests (`src/hooks/useActivityIndicator.test.ts`)
   - 8 tests covering initialization, event handling, manual triggers, and cleanup

5. Updated `StatusBar.test.tsx` with 3 new tests for activity indicators

6. Updated `MappingManager.test.tsx` with Tauri event API mock

**Files Created:**
- `src/hooks/useActivityIndicator.ts`
- `src/hooks/useActivityIndicator.test.ts`

**Files Modified:**
- `src/components/StatusBar.tsx`
- `src/components/StatusBar.test.tsx`
- `src/features/MappingManager.tsx`
- `src/features/MappingManager.test.tsx`
- `PROJECT_JOURNAL.md`

---

## Related Documents
- TASK-007: Visual Feedback (component implementation)
- TASK-010: End-to-End Integration
- PRD: Section 5 - User Interface (UX) Guidelines
- STYLE_GUIDE: Section 3 - UI Component Standards
