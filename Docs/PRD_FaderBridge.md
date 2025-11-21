# Product Requirements Document (PRD): FaderBridge
**Version:** 1.0  
**Status:** Draft  
**Date:** November 20, 2025  
**Author:** Senior Software Designer (Cascade)

## 1. Executive Summary
**FaderBridge** is a middleware application designed to bridge the gap between standard MIDI hardware and PreSonus UCNet-enabled devices (Series III Mixers, Quantum HD Interfaces). It functions as a universal translator, allowing users to control complex digital mixer parameters using physical faders, knobs, and buttons from any MIDI controller. The product prioritizes low-latency performance, rock-solid stability for live environments, and a "zero-configuration" experience for supported hardware.

## 2. Problem Statement
PreSonus hardware uses the proprietary, high-resolution UCNet protocol over Ethernet/WiFi. While powerful, this locks out the vast ecosystem of standard MIDI controllers. Users currently cannot use their favorite third-party surfaces (e.g., Korg nanoKONTROL, Behringer X-Touch in generic mode) or specialized accessibility tools to control their mixers. Existing solutions (if any) are often command-line based, unstable, or lack visual feedback.

## 3. Target Audience
1.  **Live Sound Engineers:** Need physical faders for tactile control during shows.
2.  **Home Studio Owners:** Want to use existing MIDI controllers to control their Quantum HD interface monitor mixes.
3.  **Broadcast/Church Volunteers:** Need simplified control surfaces (limiting access to critical functions) via custom MIDI mappings.

## 4. Core Functional Requirements

### 4.1. Connectivity & Protocol Handling
*   **UCNet Client:**
    *   Auto-discovery of PreSonus devices on the local network (UDP/TCP) and direct USB connection.
    *   Support for Keep-Alive heartbeats to maintain connection.
    *   Bi-directional communication (Mixer changes update the MIDI controller; MIDI controller updates the Mixer).
    *   Handling of specific device profiles (Series III Console vs. Rack vs. Quantum Interface).
*   **MIDI Engine:**
    *   Support for USB MIDI, Bluetooth MIDI, and Virtual MIDI ports.
    *   High-resolution MIDI (14-bit CC) support for smoother fader travel.
    *   Sysex support for advanced controller feedback (screen updates on controllers that support it).

### 4.2. The Translation Engine
*   **Parameter Normalization:** Convert MIDI (0-127 or 0-16383) to UCNet Floats (0.0 - 1.0) and Decibels.
    *   *Requirement:* Must include customizable "Taper" curves (Linear, Logarithmic, Audio Taper) so the physical fader feels natural.
*   **Address Mapping:** Translate MIDI Note/CC to UCNet Strings (e.g., `CC#7 Ch1` -> `line/ch1/volume`).
*   **State Tracking:** The app must maintain a "Shadow State" of the mixer to ensure that when a MIDI device is plugged in, it immediately snaps to the mixer's current values (sending MIDI updates to motorized faders).

### 4.3. Mapping & Profiles
*   **Device Library (The "Smart" Layer):**
    *   Pre-loaded JSON definitions for PreSonus devices (defining what parameters exist: Faders, Mutes, Pans, Fat Channel).
    *   Pre-loaded definitions for popular MIDI Controllers (FaderPort, X-Touch, Launch Control).
*   **Preset Mappings:** "One-click" setup. Example: Select "StudioLive 32S" and "FaderPort 8" -> Automatically maps the first 8 faders.
*   **Custom Mapping:** Users can build their own maps.

### 4.4. "Intuitive" Learn Modes
*   **Standard MIDI Learn:** Click a UI element representing a mixer parameter -> Move a physical knob -> Mapping created.
*   **"Touch-and-Go" (Novel Feature):**
    1.  User enables "Touch Mode".
    2.  User touches the *actual fader* on the PreSonus mixer (if hardware allows) or clicks the element in Universal Control.
    3.  FaderBridge detects the incoming UCNet value change.
    4.  User moves the MIDI controller.
    5.  Link is established.

## 5. User Interface (UX) Guidelines
*   **Philosophy:** "Visual Signal Flow."
*   **Dashboard:**
    *   Left Panel: Detected MIDI Devices (Green indicator for active data).
    *   Right Panel: Detected UCNet Devices (Signal strength/Latency meter).
    *   Center: The "Patch Bay" visualization showing active links.
*   **Visual Feedback:**
    *   Faders on screen should move in real-time with hardware.
    *   "Activity Lights" next to parameters to debug connection issues easily.

## 6. Technical Architecture
*   **Backend (Core Logic):** Rust (Tauri).
    *   *Reasoning:* Zero garbage collection pauses (critical for MIDI timing), memory safety, and high performance. Handles all MIDI I/O and UCNet socket communication.
*   **Frontend (UI):** React + TypeScript.
    *   *Reasoning:* Modern, component-based UI development with type safety. Communicates with the Rust backend via Tauri's IPC bridge.
*   **Data Store:** SQLite.
    *   Stores mappings, device definitions, and user preferences.

## 7. Roadmap
*   **Phase 1 (MVP):**
    *   Connect to 1 Series III Mixer.
    *   Connect 1 Generic MIDI Device.
    *   Map Volume and Mute bidirectional.
    *   Save/Load Projects.
*   **Phase 2:**
    *   "Device Library" implementation.
    *   Motorized Fader calibration/feedback support.
    *   Quantum HD support.
*   **Phase 3:**
    *   Advanced Macros (One MIDI button triggers multiple UCNet commands).
    *   OSC (Open Sound Control) Support.

## 8. Open Questions / Risks
*   **Firmware Versioning:** PreSonus updates UCNet occasionally. How do we handle protocol changes? *Mitigation: Externalize device definitions to JSON files that can be updated without recompiling the app.*
*   **Resolution:** MIDI 127 steps vs. 32-bit floating point. *Mitigation: Implement smoothing/interpolation algorithms to prevent "zipper noise" on audio.*
