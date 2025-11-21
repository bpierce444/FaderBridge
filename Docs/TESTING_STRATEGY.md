# Testing Strategy: FaderBridge

## 1. Philosophy
"Test the Logic, Mock the Hardware."
Since we cannot easily plug in physical mixers and MIDI controllers during CI/CD, our tests must rely on robust mocking of the I/O layers.

## 2. The Testing Pyramid

### Tier 1: Unit Tests (The Foundation)
*   **Backend (Rust):** `cargo test`
    *   **Scope:** Protocol translation logic.
    *   **Example:** Function `midi_to_ucnet(0x7F)` returns `1.0`.
    *   **Example:** Function `ucnet_string_to_midi("/line/ch1/volume")` parses correctly.
    *   **Mocking:** We will create a `MockMidiDevice` struct that implements our internal `MidiIO` trait.
*   **Frontend (TypeScript):** `vitest`
    *   **Scope:** UI Component rendering and State logic.
    *   **Example:** A `<Fader />` component updates its internal visual state when receiving a new prop.

### Tier 2: Integration Tests
*   **Frontend-Backend Bridge:**
    *   **Tool:** `vitest` with mocked Tauri IPC.
    *   **Strategy:** We will mock the `invoke` function from `@tauri-apps/api`.
    *   **Scenario:** When Frontend calls `invoke('connect_device')`, ensure the UI transitions to "Connected" state based on the mocked response.

### Tier 3: End-to-End (E2E)
*   **Tool:** Playwright (configured for Electron/Tauri) or WebDriver.
*   **Note:** E2E for desktop apps is expensive. We will limit this to "Smoke Tests":
    *   App launches successfully.
    *   Navigation between "Dashboard" and "Settings" works.

## 3. Toolchain Setup

### Frontend (Vitest)
```json
// package.json
"scripts": {
  "test": "vitest",
  "coverage": "vitest run --coverage"
}
```

### Backend (Cargo)
```toml
# Cargo.toml
[dev-dependencies]
mockall = "0.11" # For mocking traits
```

## 4. Continuous Integration (CI) Rules
*   **Pre-Commit:** Husky hook runs `tsc` (Type Check) and `cargo check`.
*   **PR Check:** GitHub Actions runs `cargo test` and `npm run test`.
*   **Coverage:** Critical logic (Protocol Translation) requires 90%+ coverage. UI components require 60%+.

## 5. Test Data
*   **Fixtures:** We will maintain a folder `tests/fixtures/` containing:
    *   Sample UCNet JSON packets.
    *   Recorded MIDI Sysex dumps for regression testing.
