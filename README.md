# FaderBridge

**A professional MIDI-to-UCNet bridge for PreSonus hardware**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Status: Planning](https://img.shields.io/badge/Status-Planning-blue.svg)]()

## Overview

FaderBridge is a middleware application that bridges the gap between standard MIDI controllers and PreSonus UCNet-enabled devices (Series III Mixers, Quantum HD Interfaces). It provides low-latency, bidirectional control with an intuitive mapping interface.

### Key Features

- 🎛️ **Bidirectional Control** - MIDI ↔ UCNet translation with < 10ms latency
- 🔌 **Universal Connectivity** - Network (UDP/TCP) and USB support
- 🎯 **Intuitive Mapping** - MIDI Learn and "Touch-and-Go" modes
- 💾 **Preset Library** - Pre-configured mappings for popular controllers
- 🎨 **Professional UI** - Dark mode optimized for live environments
- 🔒 **Rock Solid** - Built for stability in live performance scenarios

## Supported Hardware

### PreSonus Devices
- Series III Mixers (32S, 32SC, 64S)
- Quantum HD Interfaces (planned Phase 2)

### MIDI Controllers
- Any USB MIDI controller
- Bluetooth MIDI devices
- Virtual MIDI ports
- Pre-configured support for FaderPort, X-Touch, and more

## Project Status

**Current Phase:** Phase 0 - Planning & Documentation  
**Phase 1 MVP Progress:** 0/7 features complete

See [PROJECT_JOURNAL.md](PROJECT_JOURNAL.md) for detailed progress tracking.

## Technology Stack

- **Framework:** Tauri v2 (Rust + React + TypeScript)
- **Backend:** Rust Edition 2021+
- **Frontend:** React 18+ with TypeScript 5+
- **Styling:** TailwindCSS v3+
- **Database:** SQLite
- **Testing:** Vitest (Frontend) + Cargo Test (Backend)

## Documentation

### For Users
- [Installation Guide](Docs/INSTALLATION.md) *(coming soon)*
- [User Manual](Docs/USER_MANUAL.md) *(coming soon)*

### For Developers
- [Product Requirements Document](Docs/PRD_FaderBridge.md)
- [Project Charter](Docs/PROJECT_CHARTER.md)
- [Architecture Decisions](Docs/ARCHITECTURE_DECISIONS.md)
- [Style Guide](Docs/STYLE_GUIDE.md)
- [Testing Strategy](Docs/TESTING_STRATEGY.md)
- [Task Management Rules](Docs/TASK_MANAGEMENT_RULES.md)

### For Contributors
- [Coding Standards](.windsurf/rules/AI_CODING_RULES.md)
- [Definition of Done](.windsurf/rules/DEFINITION_OF_DONE.md)
- [Feature Prioritization](Docs/FEATURE_PRIORITIZATION.md)

## Development Setup

### Prerequisites
- Rust 1.70+ (Edition 2021)
- Node.js 18+
- Tauri CLI
- macOS 12+ (Monterey or later)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/bpierce444/FaderBridge.git
cd FaderBridge

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Run tests
npm test                  # Frontend tests
cargo test --manifest-path src-tauri/Cargo.toml  # Backend tests
```

*(Note: Project scaffold not yet created - coming in Phase 1)*

## Project Structure

```
FaderBridge/
├── .windsurf/
│   ├── rules/              # AI agent coding standards
│   └── workflows/          # Task management workflows
├── Docs/                   # Project documentation
├── Tasks/                  # Task tracking files
├── src/                    # React frontend (coming soon)
├── src-tauri/              # Rust backend (coming soon)
└── PROJECT_JOURNAL.md      # Development progress log
```

## Contributing

This project follows strict quality standards to ensure reliability in live performance environments. Before contributing:

1. Read the [Project Charter](Docs/PROJECT_CHARTER.md) to understand scope
2. Review [Coding Standards](.windsurf/rules/AI_CODING_RULES.md)
3. Follow the [Task Management Rules](Docs/TASK_MANAGEMENT_RULES.md)
4. Ensure all changes pass the [Definition of Done](.windsurf/rules/DEFINITION_OF_DONE.md)

### The "Live Concert" Test
> **"Would I trust this code in a live concert?"**  
> If the answer is no, it's not ready to merge.

## Roadmap

### Phase 1 (MVP) - In Planning
- [ ] UCNet device discovery (network + USB)
- [ ] MIDI device enumeration
- [ ] Basic parameter mapping (volume, mute, pan)
- [ ] Bidirectional sync (< 10ms latency)
- [ ] MIDI Learn functionality
- [ ] Save/Load projects
- [ ] Visual feedback (on-screen faders)

### Phase 2 - Future
- Device library with presets
- Motorized fader feedback
- Quantum HD support
- Touch-and-Go mapping

### Phase 3 - Speculative
- Advanced macros
- OSC support
- Fat Channel control

See [Feature Prioritization](Docs/FEATURE_PRIORITIZATION.md) for details.

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- PreSonus for the UCNet protocol
- The Tauri team for the excellent framework
- The Rust and React communities

## Contact

- **GitHub Issues:** [Report bugs or request features](https://github.com/bpierce444/FaderBridge/issues)
- **Discussions:** [Ask questions or share ideas](https://github.com/bpierce444/FaderBridge/discussions)

---

**Status:** 🚧 Under active development - Not yet ready for production use
