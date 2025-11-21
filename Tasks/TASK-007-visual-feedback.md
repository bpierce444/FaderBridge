# Task: Visual Feedback (On-Screen Faders)

**ID:** TASK-007  
**Status:** 🔴 Not Started  
**Priority:** High  
**Phase:** Phase 1 MVP  
**Assigned:** TBD  
**Created:** 2025-11-20  
**Updated:** 2025-11-20  

---

## Description
Implement the visual UI components that display real-time parameter values. On-screen faders, knobs, and buttons should move in sync with hardware, providing immediate visual feedback to the user.

## Acceptance Criteria
- [ ] On-screen faders move smoothly when hardware faders are moved
- [ ] On-screen mute buttons toggle when hardware buttons are pressed
- [ ] On-screen pan knobs rotate when hardware knobs are turned
- [ ] Visual feedback is smooth (60fps minimum)
- [ ] "Activity lights" show which parameters are receiving data
- [ ] UI follows STYLE_GUIDE.md (Dark Room Standard, Slate/Cyan palette)
- [ ] Fader caps are large and touch-friendly (even with mouse)
- [ ] Parameter values displayed numerically (dB for volume, % for pan)

## Dependencies
- **Depends On:** TASK-004 (Bidirectional Sync), TASK-003 (Basic Parameter Mapping)
- **Blocks:** None (but completes the MVP user experience)

## Technical Notes
- Use React 18+ with functional components and hooks
- Consider using `framer-motion` for smooth animations
- Fader component should support both mouse drag and click-to-set
- Activity lights should fade out after 500ms of no activity
- Use TailwindCSS for styling (slate-950 background, cyan-500 accents)
- Ensure accessibility (keyboard navigation, ARIA labels)

## Files Affected
- `src/components/Fader.tsx` (to be created)
- `src/components/MuteButton.tsx` (to be created)
- `src/components/PanKnob.tsx` (to be created)
- `src/components/ActivityLight.tsx` (to be created)
- `src/features/MixerStrip.tsx` (to be created)
- `src/hooks/useParameterValue.ts` (to be created)

## Testing Requirements
- [ ] Component tests for Fader (Vitest + React Testing Library)
- [ ] Component tests for MuteButton
- [ ] Component tests for PanKnob
- [ ] Visual regression tests (screenshot comparison)
- [ ] Performance test (60fps with 32 faders moving simultaneously)
- [ ] Accessibility audit (keyboard navigation, screen reader)
- [ ] Manual testing on different screen sizes

## Definition of Done Checklist
- [ ] Code follows AI_CODING_RULES.md
- [ ] Tests written and passing (60%+ coverage for UI)
- [ ] Documentation updated
- [ ] PROJECT_JOURNAL.md updated
- [ ] No compiler warnings
- [ ] Performance requirements met (60fps)
- [ ] No TypeScript `any` types
- [ ] All components have proper TypeScript interfaces
- [ ] Accessibility requirements met

---

## Work Log

*No work log entries yet*

---

## Related Documents
- PRD: Section 5 - User Interface (UX) Guidelines
- STYLE_GUIDE: Section 3 - UI Component Standards
- STYLE_GUIDE: Section 2 - Color Palette
