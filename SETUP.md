# FaderBridge Development Setup

## Prerequisites

### Required Software
- **Node.js** 18+ (currently using v22.13.1)
- **Rust** 1.70+ (currently using v1.88.0)
- **Cargo** (comes with Rust)
- **macOS** 12+ (Monterey or later)

### Verify Installation
```bash
node --version   # Should show v18+ 
rustc --version  # Should show 1.70+
cargo --version  # Should show 1.70+
```

## Initial Setup

### 1. Clone the Repository
```bash
git clone https://github.com/bpierce444/FaderBridge.git
cd FaderBridge
```

### 2. Install Dependencies
```bash
# Install Node.js dependencies
npm install

# Rust dependencies will be fetched automatically by Cargo
```

### 3. Verify Setup
```bash
# Check Rust backend compiles
cargo check --manifest-path src-tauri/Cargo.toml

# Check TypeScript compiles
npm run build
```

## Development Workflow

### Running in Development Mode
```bash
# Start the development server (hot-reload enabled)
npm run tauri:dev
```

This will:
1. Start Vite dev server on `http://localhost:1420`
2. Compile the Rust backend
3. Launch the Tauri window with hot-reload

### Building for Production
```bash
# Build optimized production bundle
npm run tauri:build
```

Output will be in `src-tauri/target/release/bundle/`

## Testing

### Frontend Tests (Vitest)
```bash
# Run tests once
npm test

# Run tests in watch mode
npm run test:ui

# Generate coverage report
npm run test:coverage
```

### Backend Tests (Cargo)
```bash
# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run with output
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture
```

## Code Quality

### Linting
```bash
# Lint TypeScript/React code
npm run lint

# Format code with Prettier
npm run format
```

### Rust Formatting
```bash
# Format Rust code
cargo fmt --manifest-path src-tauri/Cargo.toml

# Check formatting without applying
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

## Project Structure

```
FaderBridge/
├── src/                    # React frontend
│   ├── components/         # Reusable UI components
│   ├── features/           # Feature-specific components
│   ├── hooks/              # Custom React hooks
│   ├── types/              # TypeScript type definitions
│   ├── lib/                # Utility functions
│   ├── test/               # Test setup and utilities
│   ├── App.tsx             # Main App component
│   ├── main.tsx            # React entry point
│   └── index.css           # Global styles (TailwindCSS)
│
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── ucnet/          # UCNet protocol (TASK-001)
│   │   ├── midi/           # MIDI handling (TASK-002)
│   │   ├── translation/    # Parameter mapping (TASK-003)
│   │   ├── sync/           # Bidirectional sync (TASK-004)
│   │   ├── db/             # SQLite persistence (TASK-006)
│   │   ├── commands/       # Tauri command handlers
│   │   └── main.rs         # Rust entry point
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   └── build.rs            # Build script
│
├── Docs/                   # Project documentation
├── Tasks/                  # Task tracking files
├── .windsurf/              # Windsurf IDE configuration
│   ├── rules/              # AI coding standards
│   └── workflows/          # Task management workflows
│
├── package.json            # Node.js dependencies
├── tsconfig.json           # TypeScript configuration
├── vite.config.ts          # Vite bundler configuration
├── vitest.config.ts        # Vitest test configuration
├── tailwind.config.js      # TailwindCSS configuration
└── PROJECT_JOURNAL.md      # Development progress log
```

## Common Issues

### Issue: "Cannot find module '@tauri-apps/api'"
**Solution:** Run `npm install` to install dependencies

### Issue: Rust compilation errors
**Solution:** Ensure Rust 1.70+ is installed: `rustup update`

### Issue: Port 1420 already in use
**Solution:** Kill the process using the port:
```bash
lsof -ti:1420 | xargs kill -9
```

### Issue: TypeScript errors in IDE
**Solution:** Restart TypeScript server in your IDE, or run:
```bash
npm run build
```

## Environment Variables

Create a `.env` file in the project root for local configuration:

```env
# Development mode
TAURI_DEBUG=true

# Log level (trace, debug, info, warn, error)
RUST_LOG=info
```

## Debugging

### Frontend Debugging
- Open DevTools in the Tauri window: Right-click → Inspect Element
- Or use the keyboard shortcut: `Cmd+Option+I` (macOS)

### Backend Debugging
- Rust logs will appear in the terminal where you ran `npm run tauri:dev`
- Use `log::info!()`, `log::debug!()`, etc. in Rust code
- Set `RUST_LOG=debug` for verbose logging

## Next Steps

1. Review the [Project Charter](Docs/PROJECT_CHARTER.md) for scope
2. Read the [AI Coding Rules](.windsurf/rules/AI_CODING_RULES.md) for standards
3. Check the [Task List](Tasks/README.md) for Phase 1 MVP tasks
4. Start with TASK-001: UCNet Device Discovery

## Getting Help

- **Documentation:** See `/Docs` folder
- **Task Management:** Use `/list-tasks` workflow in Windsurf
- **GitHub Issues:** https://github.com/bpierce444/FaderBridge/issues
- **Project Journal:** Check `PROJECT_JOURNAL.md` for latest progress

---

**Last Updated:** 2025-11-20  
**Version:** 0.1.0 (Phase 1 MVP - Development)
