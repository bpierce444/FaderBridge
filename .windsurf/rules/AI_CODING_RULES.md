# FaderBridge Coding Standards

## Technology Stack (CRITICAL - Verify Before Coding)
- You are building FaderBridge, a MIDI-to-UCNet bridge for PreSonus hardware
- Use Tauri v2.x as the application framework (NOT v1 - the APIs are different)
- Use Rust Edition 2021 or later for all backend code
- Use TypeScript 5.x+ with strict mode enabled for all frontend code
- Use React 18+ with functional components and hooks only (NO class components)
- Use Vite as the build tool
- Use TailwindCSS v3+ for styling
- Use SQLite for data persistence

## Knowledge Verification Protocol
- NEVER guess Tauri API signatures - the v1 to v2 migration changed many IPC patterns
- Before writing Tauri command handlers or IPC bridges, use search_web or mcp0_get-library-docs to verify current API syntax
- If you're unsure about a Tauri API, explicitly state "I need to verify the Tauri v2 API for [feature]" and use tools to check
- Flag any deprecated methods from the last 2 years (e.g., tauri.conf.json vs tauri.conf.json5)

## TypeScript Standards
- The `any` type is strictly forbidden - use `unknown` if necessary, but prefer defined interfaces
- All React props must be typed with explicit interfaces
- Use Zod or similar validation for all data crossing the Rust<->TypeScript bridge
- Prefer type inference where clear, but always type function parameters and return values

## Rust Standards
- Avoid `.unwrap()` in production code - use `match` or the `?` operator with proper error propagation
- Use `tokio` as the async runtime
- Use `thiserror` for error handling and propagation
- All public functions must have doc comments

## Code Documentation
- Do NOT comment what code does (the code should be self-documenting)
- DO comment WHY a specific decision was made, especially for:
  - MIDI timing and thread management
  - UCNet protocol quirks
  - Performance optimizations
  - Workarounds for hardware limitations

## Architecture Patterns
- React Context + Hooks manage UI state
- Rust backend is the single source of truth for hardware connection state
- IPC Flow: Frontend invokes command → Backend performs action → Backend emits event → Frontend listens
- NEVER poll from the frontend - always use event-driven architecture
- Use Tauri's event system for all backend-to-frontend communication

## File Structure
- `/src-tauri` - Rust backend code only
- `/src` - React frontend code only
- `/src/components` - Reusable UI atoms (Buttons, Faders, Knobs)
- `/src/features` - Complex business logic components (MixerStrip, PatchBay, DeviceManager)
- `/src/hooks` - Custom React hooks
- `/src/types` - TypeScript type definitions
- `/src/lib` - Utility functions and helpers

## Testing Requirements
- Write unit tests for all protocol translation logic (Rust)
- Write component tests for all UI components (Vitest + React Testing Library)
- Mock hardware I/O using traits in Rust tests
- Maintain 90%+ coverage for critical protocol logic
- Maintain 60%+ coverage for UI components

## Project Journal (MANDATORY)
- You MUST update PROJECT_JOURNAL.md at the end of every work session
- Update the journal when:
  - Completing a major milestone or feature
  - Making an architectural decision (link to the ADR)
  - Encountering a significant blocker or bug
  - Changing scope or priorities
  - At the end of ANY coding session (even if incomplete)
- Use the provided template in the journal file
- Update the "Phase 1 Locked Features" progress tracker
- Update the metrics table if you have new measurements
- Be honest about blockers and status (On Track / Blocked / At Risk)

## Before You Start Coding
- Read the PRD at Docs/PRD_FaderBridge.md to understand the product vision
- Read the STYLE_GUIDE.md for UI/UX standards
- Read the TESTING_STRATEGY.md for testing approach
- Read the PROJECT_JOURNAL.md to understand current progress and context
- If implementing Tauri features, verify the API syntax using available tools
