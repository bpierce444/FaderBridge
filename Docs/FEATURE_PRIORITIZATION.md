# Feature Prioritization Matrix: FaderBridge

## How to Use This Document
When evaluating a new feature request, score it on two axes:
1. **Impact** (How much does this improve the core mission?)
2. **Effort** (How much work is required?)

Then place it in the appropriate quadrant.

---

## The Priority Quadrants

```
High Impact, Low Effort          |  High Impact, High Effort
(DO FIRST)                       |  (PLAN CAREFULLY)
---------------------------------+---------------------------------
✅ Basic MIDI Learn              |  🔶 Motorized fader feedback
✅ Save/Load projects            |  🔶 Fat Channel control
✅ Visual fader feedback         |  🔶 Multi-device support
✅ Connection status indicators  |  🔶 Advanced macro system
                                 |
---------------------------------+---------------------------------
Low Impact, Low Effort           |  Low Impact, High Effort
(DO IF TIME PERMITS)             |  (AVOID)
                                 |
🟡 Dark/Light theme toggle       |  ❌ Plugin automation
🟡 Keyboard shortcuts            |  ❌ MIDI routing
🟡 Window size persistence       |  ❌ Built-in UCNet analyzer
🟡 Recent projects list          |  ❌ Custom scripting language
```

---

## Phase 1 (MVP) - Locked Features
These are **non-negotiable** for the first release. Do not add anything else until these are complete.

| Feature | Status | Acceptance Criteria |
|---------|--------|---------------------|
| UCNet device discovery | 🔴 Not Started | App detects Series III mixer on network |
| MIDI device enumeration | 🔴 Not Started | App lists all connected MIDI devices |
| Basic parameter mapping | 🔴 Not Started | Map MIDI CC to UCNet volume/mute/pan |
| Bidirectional sync | 🔴 Not Started | Changes flow both directions < 10ms |
| MIDI Learn | 🔴 Not Started | Click parameter → move controller → linked |
| Save/Load projects | 🔴 Not Started | Mappings persist across app restarts |
| Visual feedback | 🔴 Not Started | On-screen faders move with hardware |

---

## Phase 2 - Candidate Features
These are **allowed** after Phase 1 ships, but must be individually approved.

### High Priority (Likely to be included)
- **Device Library:** Pre-built mappings for FaderPort, X-Touch, etc.
  - *Justification:* Dramatically improves first-time user experience
  - *Effort:* Medium (requires JSON schema + UI for selection)
  
- **Motorized Fader Feedback:** Send position updates to controllers with motors
  - *Justification:* Critical for professional controllers
  - *Effort:* High (requires per-device calibration)

- **Quantum HD Support:** Extend beyond Series III to interfaces
  - *Justification:* Expands addressable market
  - *Effort:* Medium (UCNet is same, but parameter addresses differ)

### Medium Priority (Needs strong justification)
- **Touch-and-Go Mapping:** Novel alternative to MIDI Learn
  - *Justification:* More intuitive than traditional MIDI Learn
  - *Effort:* Medium (requires UCNet message monitoring)

- **Multiple MIDI Devices:** Control one mixer with multiple controllers
  - *Justification:* Useful for complex setups
  - *Effort:* Medium (requires conflict resolution logic)

### Low Priority (Defer unless user demand is overwhelming)
- **OSC Support:** Control mixers via Open Sound Control
  - *Justification:* Niche use case (TouchOSC users)
  - *Effort:* High (new protocol layer)

- **Macro System:** One button triggers multiple actions
  - *Justification:* Power user feature
  - *Effort:* High (requires scripting engine)

---

## Phase 3 - Speculative Features
These are **ideas only**. Do not work on these until Phase 2 is complete and validated.

- Advanced Fat Channel control (EQ, Compressor, Gate)
- Scene recall integration
- MIDI 2.0 support
- Windows/Linux ports
- Plugin automation (VST control)

---

## Feature Request Evaluation Template

When someone (including yourself) suggests a new feature, fill this out:

```markdown
### Feature: [Name]

**Description:** [1-2 sentences]

**User Story:** As a [type of user], I want [feature] so that [benefit].

**Impact Score (1-10):**
- Improves core mission: [ /10]
- User demand: [ /10]
- Competitive advantage: [ /10]
**Total Impact: [ /30]**

**Effort Score (1-10):**
- Backend complexity: [ /10]
- Frontend complexity: [ /10]
- Testing burden: [ /10]
**Total Effort: [ /30]**

**Risk Assessment:**
- Does this compromise stability? [Yes/No]
- Does this add significant complexity? [Yes/No]
- Does this require new dependencies? [Yes/No]

**Decision:** [Approve for Phase X / Defer / Reject]
**Reasoning:** [Why?]
```

---

## The "One Feature Rule"
**At any given time, only ONE new feature is in active development.**

This prevents:
- Context switching
- Half-finished features
- Integration nightmares
- Scope creep

If you want to start a new feature, finish the current one first.

---

## How to Say No
When rejecting a feature request, use this template:

> "Thanks for the suggestion! [Feature] is interesting, but it doesn't align with our Phase 1 goals of [core mission]. We're focused on [current priority] right now. If there's strong user demand after launch, we can revisit this in Phase 2."

Be firm but respectful. Saying no is a feature.
