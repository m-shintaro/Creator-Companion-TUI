# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run

# Test
cargo test
cargo test <test_name>   # Run a single test

# Check (faster than build, no output binary)
cargo check

# Lint
cargo clippy
```

## Architecture

This is a Rust TUI app (`vcc-tui`) for managing VRChat VPM workflows, built on **ratatui** + **tokio**.

### State management (Redux-like)

The app follows a unidirectional data flow:

1. **`src/events/mod.rs`** — async event loop, converts crossterm key events and ticks into `Action` values sent over a channel
2. **`src/app/action.rs`** — `Action` enum: all user inputs and async results (key events, config loaded, task output, etc.)
3. **`src/app/reducer.rs`** — pure function: `(AppState, Action) → (AppState, Vec<Effect>)`. Contains all keybinding logic.
4. **`src/app/effect.rs`** — `Effect` enum: async side effects (load config, run vpm command, scan folder, etc.)
5. **`src/main.rs`** — orchestrates the loop: dispatches effects via `handle_effect()`, drives renders

### Key data types (`src/app/state.rs`)

- `Screen` enum — the 5 nav screens: New, Add, Projects, Manage, Settings
- `AppState` — all UI state (selected items, input buffers, search queries, task logs, system checks)
- `TaskRecord` / `TaskState` — tracks async vpm subprocess output and status
- `AppConfig` — persisted config (project list), serialized to `~/.config/vcc-tui/config.json`

### Services (`src/services/`)

- **`fs.rs`** — config/cache I/O; reads `Packages/vpm-manifest.json` for projects; loads available packages from VCC's cache at `~/.local/share/VRChatCreatorCompanion/Repos/`
- **`vpm.rs`** — `VpmClient` spawns the `vpm` CLI as a subprocess, streams stdout/stderr lines back as `Action::TaskOutput`, supports cancellation via `CancellationToken`

### UI (`src/ui/`)

- **`mod.rs`** — top-level layout: 1-line header, left nav (22 cols) + main area, 10-line scrollable log pane at bottom
- **`screens/`** — one file per screen; `dashboard.rs` is legacy/unused

### Async task model

Long-running vpm commands are tracked as `TaskRecord` entries in `AppState`. Each task has an ID, label, streamed output lines, and a `TaskState` (Running/Success/Failed/Cancelled). Cancellation is wired through `tokio_util::sync::CancellationToken`.

## External dependency

The app shells out to the `vpm` CLI (VRChat Package Manager). It must be installed and on `PATH` for package operations to work.
