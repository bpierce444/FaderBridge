# Project Charter: FaderBridge

## Mission Statement
Build a **rock-solid, low-latency bridge** between MIDI controllers and PreSonus UCNet devices. Prioritize **stability over features** and **simplicity over flexibility**.

## What This Project IS
- A translation layer between MIDI and UCNet protocols
- A visual mapping tool for creating controller assignments
- A preset library for common hardware combinations
- A professional tool for live sound and studio environments

## What This Project IS NOT
- A full DAW controller (no transport control, no plugin automation)
- A MIDI router or general-purpose MIDI processor
- A UCNet protocol analyzer or debugging tool
- A replacement for Universal Control software

## Hard Constraints (Non-Negotiable)
1. **Latency:** MIDI-to-UCNet round trip must be < 10ms on standard hardware
2. **Stability:** Zero crashes during a 4-hour live show scenario
3. **Compatibility:** Must work with Series III (32S, 32SC, 64S) and Quantum HD interfaces
4. **Simplicity:** A new user with a FaderPort should be controlling their mixer within 60 seconds

## Scope Boundaries

### Phase 1 (MVP) - IN SCOPE
- Connect to ONE mixer via UCNet (network or USB)
- Connect to ONE MIDI device
- Map faders (volume), mutes, and pans bidirectionally
- Save/Load mapping projects
- Basic visual feedback (fader movement on screen)

### Phase 1 - OUT OF SCOPE
- Multiple simultaneous mixers
- Fat Channel / EQ / Dynamics control
- Macros or scripting
- MIDI learn for complex Sysex controllers
- OSC support
- Plugin control

### Phase 2 - CONDITIONAL (Only if Phase 1 succeeds)
- Device library with presets
- Motorized fader feedback
- Quantum HD interface support
- Touch-sensitive fader detection

### Phase 3 - FUTURE (No promises)
- Advanced macros
- OSC bridge
- Multiple mixer support
- Fat Channel control

## Success Criteria (How We Know We're Done)
### Phase 1 MVP is complete when:
1. A user can connect a generic MIDI controller to a StudioLive 32S
2. Map 8 faders to channels 1-8 using MIDI Learn
3. Move a physical fader and see the mixer respond within 10ms
4. Move a mixer fader (in Universal Control) and see the MIDI controller respond
5. Save the mapping and reload it after restarting the app
6. The app runs for 4 hours without crashing or memory leaks

## Decision-Making Framework
When evaluating new features or changes, ask:
1. **Does this serve the core mission?** (MIDI ↔ UCNet translation)
2. **Does this compromise stability?** (If yes, reject)
3. **Does this add complexity to the user experience?** (If yes, needs strong justification)
4. **Can this wait until Phase 2?** (If yes, defer it)

## The "No" List (Common Feature Requests We Will Reject)
- ❌ "Can it control my DAW transport?"
- ❌ "Can it route MIDI between devices?"
- ❌ "Can it record automation?"
- ❌ "Can it work with non-PreSonus mixers?"
- ❌ "Can it support MIDI 2.0?"

## Quality Gates (Must Pass Before Release)
- [ ] All critical path code has 90%+ test coverage
- [ ] App launches in < 2 seconds on a 2019 MacBook Pro
- [ ] Memory usage stays below 150MB after 1 hour of operation
- [ ] No console errors or warnings during normal operation
- [ ] Works on macOS 12+ (Monterey and later)
- [ ] Passes accessibility audit (keyboard navigation works)

## When to Pivot or Kill the Project
**Stop development if:**
- We cannot achieve < 20ms latency after 2 months of optimization
- PreSonus changes UCNet protocol in a way that breaks our approach
- A competing solution emerges that solves the problem better
- User testing reveals the core concept is flawed
