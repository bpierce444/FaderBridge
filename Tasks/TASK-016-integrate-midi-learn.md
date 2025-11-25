# Task: Integrate MIDI Learn into UI

**ID:** TASK-016  
**Status:** 🟡 In Progress  
**Priority:** Critical (P0)  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-24  
**Updated:** 2025-11-25  

---

## Description
Connect the MIDI Learn feature to the mapping creation workflow. The MIDI Learn backend and UI components exist but are not integrated into the main application flow. Users should be able to click "MIDI Learn" and create mappings by moving physical controls.

## Problem Statement
The following components exist but are not connected:
- `src/features/MidiLearn.tsx` - Status overlay, not rendered anywhere
- `src/components/LearnButton.tsx` - Learn button component, not imported anywhere
- `src/hooks/useMidiLearn.ts` - Hook for MIDI learn state, not used in MappingManager
- Backend commands exist but aren't called from mapping creation flow

The MVP success criteria states: "Map 8 faders to channels 1-8 using MIDI Learn" - this workflow doesn't exist.

## Acceptance Criteria
- [x] "MIDI Learn" button visible in MappingManager
- [x] Clicking MIDI Learn enters learn mode with visual feedback
- [ ] Moving a MIDI control captures the CC/Note/Channel
- [x] User can select target UCNet parameter before or after learning
- [ ] Mapping is automatically created and saved to database
- [x] ESC key cancels learn mode
- [x] 10-second timeout with countdown displayed
- [ ] Duplicate mapping detection warns user
- [x] Learn mode status overlay visible during learning

## Dependencies
- **Depends On:** TASK-005 (MIDI Learn - backend), TASK-008 (Mapping Interface UI)
- **Blocks:** None (but critical for MVP UX)

## Technical Notes

### Current Components

1. **MidiLearn.tsx** - Status overlay
   - Shows learning state, countdown, device info
   - Has cancel button and ESC key support
   - Needs to be rendered in Dashboard or MappingManager

2. **LearnButton.tsx** - Trigger button
   - Starts learn mode when clicked
   - Shows learning state
   - Needs to be added to MappingManager

3. **useMidiLearn.ts** - React hook
   - Manages learn state
   - Calls backend commands
   - Returns `startLearn`, `cancelLearn`, `learnState`

### Integration Flow

**Desired User Flow:**
1. User clicks "New Mapping" in MappingManager
2. User selects UCNet parameter (e.g., "Channel 1 Volume")
3. User clicks "MIDI Learn" button
4. Status overlay appears: "Move a MIDI control..."
5. User moves physical fader
6. MIDI CC captured, mapping created automatically
7. Mapping appears in list

**Alternative Flow (Learn First):**
1. User clicks "MIDI Learn" in toolbar
2. Status overlay appears
3. User moves physical fader
4. MIDI CC captured
5. Dialog appears: "Select UCNet parameter to map to"
6. User selects parameter
7. Mapping created

### Implementation Steps

1. **Add MidiLearn overlay to Dashboard**
   ```tsx
   // In Dashboard.tsx
   import { MidiLearn } from './MidiLearn';
   
   // In render:
   <MidiLearn />
   ```

2. **Add LearnButton to MappingManager create form**
   ```tsx
   // In MappingManager.tsx
   import { LearnButton } from '../components/LearnButton';
   
   // In create form:
   <LearnButton 
     onLearnComplete={(midiInfo) => {
       setNewMapping({
         ...newMapping,
         midi_channel: midiInfo.channel,
         midi_cc: midiInfo.controller,
       });
     }}
   />
   ```

3. **Connect learn result to mapping creation**
   - When learn completes, auto-fill MIDI fields
   - If UCNet parameter already selected, create mapping immediately

4. **Fix failing tests**
   - `useMidiLearn.test.ts` has 9 failing tests (timeouts)
   - Need to fix mock setup or async handling

### Files to Modify
- `src/features/Dashboard.tsx` - Add MidiLearn overlay
- `src/features/MappingManager.tsx` - Add LearnButton, handle learn result
- `src/components/LearnButton.tsx` - May need props for callback
- `src/hooks/useMidiLearn.ts` - May need to expose more state
- `src/hooks/useMidiLearn.test.ts` - Fix failing tests

### Backend Commands Available
- `start_midi_learn` - Start learning for a parameter
- `cancel_midi_learn` - Cancel learn mode
- `get_midi_learn_state` - Get current state
- `is_midi_learning` - Check if in learn mode

## Testing Requirements
- [x] LearnButton renders in MappingManager
- [x] Clicking LearnButton starts learn mode
- [x] MidiLearn overlay appears during learning
- [x] ESC key cancels learn mode
- [x] Timeout after 10 seconds
- [ ] Learn result populates MIDI fields
- [ ] Mapping created after learn + parameter selection
- [x] Fix all 9 failing useMidiLearn tests

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (including fixed tests)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] No TypeScript `any` types
- [ ] MIDI Learn workflow functional end-to-end
- [ ] Can create mapping using MIDI Learn

---

## Work Log

### 2025-11-25 - UI Integration Complete
**Duration:** ~1 hour

**Completed:**
- Added `MidiLearn` overlay to `Dashboard.tsx` (renders as fixed overlay when learning)
- Added `useMidiLearn` hook to `MappingManager.tsx`
- Added "Learn" button next to MIDI CC input field with:
  - Loading state animation when learning
  - Disabled state until UCNet parameter selected
  - Cancel link during learn mode
- Fixed all 9 failing `useMidiLearn.test.ts` tests:
  - Root cause: Fake timers conflicting with React Testing Library's `waitFor`
  - Solution: Use real timers with appropriate poll intervals
- TypeScript compiles with no errors
- All useMidiLearn tests pass (9/9)

**Files Modified:**
- `src/features/Dashboard.tsx` - Added MidiLearn overlay
- `src/features/MappingManager.tsx` - Added Learn button and hook integration
- `src/hooks/useMidiLearn.test.ts` - Fixed all 9 failing tests

**Remaining Work:**
- Backend needs to route MIDI input to `MidiLearn.process_message()` for actual capture
- Learn result needs to populate MIDI fields in form
- Auto-create mapping when learn completes with parameter selected

---

## Related Documents
- TASK-005: MIDI Learn Functionality (backend)
- TASK-008: Mapping Interface UI
- PRD: Section 4.4 - "Intuitive" Learn Modes
- PROJECT_CHARTER: Success Criteria #2 (Map 8 faders using MIDI Learn)
