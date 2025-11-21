# Style Guide: FaderBridge

## 1. Design Philosophy
**"The Dark Room Standard"**
Audio professionals often work in low-light environments (FOH, Theater, Studios).
*   **Rule 1:** Dark Mode is the default and primary mode.
*   **Rule 2:** High Contrast for active elements (Active Faders, Signal Meters).
*   **Rule 3:** Low Contrast for static containers (Backgrounds, Panels).

## 2. Color Palette (Tailwind Token Names)

| Role | Color Name | Hex Code | Usage |
| :--- | :--- | :--- | :--- |
| **Background** | `slate-950` | `#020617` | Main app background |
| **Panel** | `slate-900` | `#0f172a` | Device lists, sidebars |
| **Surface** | `slate-800` | `#1e293b` | Cards, list items |
| **Primary** | `cyan-500` | `#06b6d4` | Active Links, "Connected" status |
| **Accent** | `amber-500` | `#f59e0b` | "Touch" Mode, Warnings |
| **Signal** | `emerald-500` | `#10b981` | Signal presence, Good state |
| **Danger** | `rose-600` | `#e11d48` | Mute active, Disconnected, Clipping |

## 3. UI Component Standards

### 3.1. Faders & Knobs
*   **Skeuomorphism:** Minimal. Do not try to look exactly like a 1980s console.
*   **Visibility:** The "Cap" of the fader must be large and touch-friendly (even if using a mouse).
*   **Feedback:** When a physical MIDI fader is moved, the UI fader should "glow" or highlight to indicate it is being driven by hardware.

### 3.2. Text & Typography
*   **Font:** Inter or system-ui (San Francisco on Mac).
*   **Monospace:** JetBrains Mono or equivalent for:
    *   MIDI Addresses (`CC#7`)
    *   UCNet Strings (`line/ch1/vol`)
    *   IP Addresses
*   **Readability:** Minimum font size 12px. Labels for faders should be legible at a glance.

### 3.3. Layout (The Patch Bay)
*   **Flow:** Left-to-Right.
    *   **Left:** Inputs (MIDI Controllers)
    *   **Center:** Processing (Mapping/Translation)
    *   **Right:** Outputs (UCNet Mixers)
*   **Connections:** Use Bezier curves (SVG lines) to draw connections between devices, similar to node-based editors (Blender/Unreal).

## 4. React/Code Style
*   **Component Structure:**
    ```tsx
    // Prefer Function Declarations
    export function Fader({ value, onChange }: FaderProps) { ... }
    ```
*   **Imports:**
    *   Group 1: React / External Libraries
    *   Group 2: Internal Components
    *   Group 3: Utils / Types / Styles
*   **Naming:**
    *   Components: PascalCase (`MixerStrip.tsx`)
    *   Hooks: camelCase (`useMidiDevice.ts`)
    *   Constants: UPPER_SNAKE_CASE (`MAX_FADER_VALUE`)

## 5. Accessibility (A11y)
*   All interactive elements must be keyboard navigable (Tab index).
*   Color alone should not be the only indicator of state (use Icons + Color).
