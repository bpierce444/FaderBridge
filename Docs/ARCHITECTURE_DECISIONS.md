# Architecture Decision Records (ADR): FaderBridge

## What is an ADR?
An Architecture Decision Record documents **why** we made a significant technical choice. This prevents future developers (including yourself) from asking "Why did we do it this way?" or worse, undoing a decision that was made for good reasons.

---

## ADR Template

When making a significant architectural decision, copy this template into a new section below:

```markdown
## ADR-XXX: [Title]
**Date:** YYYY-MM-DD
**Status:** [Proposed / Accepted / Deprecated / Superseded by ADR-YYY]

### Context
What is the issue we're trying to solve? What constraints exist?

### Decision
What did we decide to do?

### Consequences
What are the positive and negative outcomes of this decision?

### Alternatives Considered
What other options did we evaluate and why did we reject them?
```

---

## ADR-001: Use Tauri v2 Instead of Electron
**Date:** 2025-11-20
**Status:** Accepted

### Context
We need a cross-platform desktop framework that can:
- Access low-level MIDI and network APIs
- Provide a modern UI with React
- Minimize latency for real-time audio control
- Keep the app size small for distribution

### Decision
Use **Tauri v2** with Rust backend and React (TypeScript) frontend.

### Consequences
**Positive:**
- Native performance for MIDI/Network I/O (Rust has no GC pauses)
- Small app size (~5-10MB vs 100MB+ for Electron)
- Memory safety guarantees from Rust
- Active community and good documentation

**Negative:**
- Smaller ecosystem than Electron (fewer libraries)
- Team needs Rust knowledge (steeper learning curve than Node.js)
- Some npm packages may not work in Tauri's environment

### Alternatives Considered
- **Electron:** Rejected due to large bundle size and potential GC pauses affecting MIDI timing
- **Native Swift (macOS only):** Rejected because we want cross-platform potential in the future
- **Qt/C++:** Rejected due to complex UI development compared to React

---

## ADR-002: SQLite for Persistence Instead of JSON Files
**Date:** 2025-11-20
**Status:** Accepted

### Context
We need to store:
- User-created mapping projects
- Device profiles (MIDI controllers and UCNet devices)
- User preferences

### Decision
Use **SQLite** as the embedded database.

### Consequences
**Positive:**
- Structured queries for complex lookups (e.g., "Find all mappings for FaderPort 8")
- ACID transactions prevent corruption if app crashes during save
- Easy to add search/filter features later
- Single-file database is easy to backup

**Negative:**
- Slightly more complex than JSON files
- Requires migration strategy if schema changes

### Alternatives Considered
- **JSON files:** Rejected because they don't scale well for complex queries and are prone to corruption
- **Cloud database:** Rejected because this is a local-first app (no internet required)

---

## ADR-003: Event-Driven IPC Instead of Polling
**Date:** 2025-11-20
**Status:** Accepted

### Context
The frontend needs to know when hardware state changes (e.g., a fader moves on the mixer). We could either:
1. Poll the backend every 50ms asking "Did anything change?"
2. Have the backend emit events when changes occur

### Decision
Use **Tauri's event system** for all backend-to-frontend communication. The backend emits events, and the frontend listens.

### Consequences
**Positive:**
- Lower latency (no polling delay)
- Lower CPU usage (no unnecessary checks)
- Scales better with multiple devices
- Follows Tauri best practices

**Negative:**
- Slightly more complex to set up than polling
- Requires careful event naming conventions to avoid conflicts

### Alternatives Considered
- **Polling:** Rejected due to latency and CPU overhead
- **WebSockets:** Rejected because Tauri's IPC is faster and simpler

---

## ADR-004: Mock Hardware I/O Using Traits (Rust)
**Date:** 2025-11-20
**Status:** Accepted

### Context
We need to test protocol translation logic without physical MIDI controllers or mixers. Directly calling hardware APIs in tests is:
- Slow (real I/O takes time)
- Unreliable (hardware may not be connected)
- Difficult to simulate edge cases (e.g., device disconnects)

### Decision
Define Rust **traits** for `MidiIO` and `UCNetIO`, then implement:
- `RealMidiDevice` (production code)
- `MockMidiDevice` (test code)

### Consequences
**Positive:**
- Tests run fast (no real I/O)
- Tests are deterministic (no flaky failures)
- Easy to simulate error conditions
- Follows Rust best practices for testability

**Negative:**
- Requires upfront design of trait interfaces
- Mocks must be kept in sync with real implementations

### Alternatives Considered
- **Integration tests only:** Rejected because they're too slow and require hardware
- **Dependency injection (DI container):** Rejected as overkill for this project size

---

## ADR-005: TailwindCSS for Styling Instead of CSS-in-JS
**Date:** 2025-11-20
**Status:** Accepted

### Context
We need a styling solution that:
- Works well with React
- Allows rapid UI iteration
- Produces small bundle sizes
- Supports dark mode easily

### Decision
Use **TailwindCSS v3+** with utility classes.

### Consequences
**Positive:**
- Fast development (no context switching to CSS files)
- Consistent design system (spacing, colors defined in config)
- Built-in dark mode support
- Purges unused styles automatically (small bundle)

**Negative:**
- HTML can look verbose with many utility classes
- Learning curve for developers unfamiliar with utility-first CSS

### Alternatives Considered
- **Styled Components:** Rejected due to runtime overhead
- **CSS Modules:** Rejected because it's slower to iterate than utilities
- **Plain CSS:** Rejected because it's harder to maintain consistency

---

## ADR-006: Vitest Instead of Jest for Frontend Testing
**Date:** 2025-11-20
**Status:** Accepted

### Context
We need a test runner for React components and TypeScript utilities. It must:
- Work with Vite (our build tool)
- Support TypeScript without extra config
- Be fast

### Decision
Use **Vitest** as the test runner.

### Consequences
**Positive:**
- Native Vite integration (no config needed)
- Faster than Jest (uses Vite's transform pipeline)
- Jest-compatible API (easy migration if needed)
- Built-in TypeScript support

**Negative:**
- Smaller ecosystem than Jest (fewer plugins)
- Relatively new (less battle-tested)

### Alternatives Considered
- **Jest:** Rejected because it requires extra config for Vite and is slower
- **Cypress Component Testing:** Rejected because it's overkill for unit tests

---

## How to Add a New ADR

1. Copy the template above
2. Assign the next sequential number (ADR-007, ADR-008, etc.)
3. Fill out all sections honestly (including negatives)
4. Discuss with team (if applicable) before marking as "Accepted"
5. Add to this file in chronological order

**When to create an ADR:**
- Choosing a major library or framework
- Deciding on a data structure or algorithm
- Establishing a coding pattern that will be used throughout the codebase
- Making a trade-off between competing concerns (e.g., performance vs. simplicity)

**When NOT to create an ADR:**
- Trivial decisions (e.g., "Should this variable be named `count` or `counter`?")
- Decisions that are easily reversible
- Implementation details that don't affect the overall architecture
