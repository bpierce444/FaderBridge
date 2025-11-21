---
trigger: always_on
---

# Definition of Done: FaderBridge

## Purpose
This checklist ensures that every feature, bug fix, and component meets our quality standards **before** it is considered complete. No exceptions.

---

## For Every Code Change

### Code Quality
- [ ] Code follows the standards in `AI_CODING_RULES.md`
- [ ] No TypeScript `any` types (use `unknown` or proper interfaces)
- [ ] No Rust `.unwrap()` in production code paths
- [ ] All public functions have doc comments
- [ ] No compiler warnings
- [ ] No linter errors or warnings

### Testing
- [ ] Unit tests written and passing
- [ ] Test coverage meets minimums:
  - Protocol logic (Rust): 90%+
  - UI components (React): 60%+
- [ ] Manual testing performed on target hardware (if applicable)
- [ ] No regressions in existing functionality

### Documentation
- [ ] Code comments explain "why", not "what"
- [ ] Complex algorithms have explanatory comments
- [ ] If adding a new feature, update relevant docs in `/Docs`
- [ ] If changing an API, update type definitions
- [ ] **PROJECT_JOURNAL.md updated with session summary** (MANDATORY)

### Performance
- [ ] No memory leaks (verified with profiler if touching state management)
- [ ] MIDI-to-UCNet latency < 10ms (if touching protocol layer)
- [ ] UI remains responsive during heavy I/O operations

### Git Hygiene
- [ ] Commit message follows format: `[type]: brief description`
  - Types: `feat`, `fix`, `refactor`, `test`, `docs`, `perf`
  - Example: `feat: add MIDI learn for fader mapping`
- [ ] No commented-out code blocks
- [ ] No debug `console.log` or `println!` statements

---

## For New Features

### Design Validation
- [ ] Feature aligns with Project Charter scope
- [ ] Feature does not compromise core stability
- [ ] UX follows patterns in `STYLE_GUIDE.md`
- [ ] Feature has been discussed and approved (not a surprise)

### Implementation
- [ ] Feature works with both network and USB UCNet connections
- [ ] Feature works with generic MIDI controllers (not just FaderPort)
- [ ] Error handling covers edge cases:
  - Device disconnects mid-operation
  - Invalid MIDI messages
  - Malformed UCNet packets
- [ ] Feature degrades gracefully if hardware is unavailable

### User Experience
- [ ] Feature is discoverable (user doesn't need to read docs)
- [ ] Feature has visual feedback (user knows it's working)
- [ ] Feature has error feedback (user knows when it fails)
- [ ] Keyboard shortcuts work (if applicable)

---

## For Bug Fixes

### Root Cause Analysis
- [ ] Bug is reproducible with clear steps
- [ ] Root cause identified (not just symptoms)
- [ ] Fix addresses root cause, not just the symptom

### Verification
- [ ] Bug is fixed and verified manually
- [ ] Regression test added to prevent recurrence
- [ ] Related edge cases checked and tested
- [ ] Fix does not introduce new bugs (smoke test entire app)

---

## For Releases (MVP and beyond)

### Pre-Release Checklist
- [ ] All Phase 1 success criteria met (see `PROJECT_CHARTER.md`)
- [ ] All quality gates passed
- [ ] App tested on clean macOS installation
- [ ] App tested with at least 3 different MIDI controllers
- [ ] App tested with at least 2 different PreSonus devices
- [ ] Memory usage profiled (< 150MB after 1 hour)
- [ ] No crashes during 4-hour stress test

### Documentation
- [ ] README.md updated with installation instructions
- [ ] User guide created (if first release)
- [ ] Known issues documented
- [ ] Changelog updated

### Distribution
- [ ] App signed and notarized (macOS)
- [ ] Installer tested on clean system
- [ ] Version number updated in all relevant files

---

## Red Flags (Automatic Rejection)
If any of these are true, the work is **not done**:
- ❌ "It works on my machine" (but not tested elsewhere)
- ❌ "I'll add tests later"
- ❌ "It's just a quick hack"
- ❌ "The user probably won't do that"
- ❌ "I'm not sure why it works, but it does"

---

## When in Doubt
Ask yourself: **"Would I trust this code in a live concert?"**
If the answer is no, it's not done.
