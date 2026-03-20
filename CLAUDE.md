# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**gestro** — an OS-wide mouse gesture launcher. Hold right-click, move in one of 8 compass directions, release → fires a keyboard shortcut. Invisible to the user; configured via a Tauri settings window.

## Commands

```sh
# Run in dev mode (starts Vite + Tauri with hot reload)
npm run tauri dev

# Type-check Svelte frontend
npx svelte-check

# Check Rust backend
cd src-tauri && cargo check

# Build for release
npm run tauri build
```

## Architecture

**Tech stack:** Tauri 2 (Rust backend + SvelteKit frontend), Svelte 5 (runes syntax).

### Rust (`src-tauri/src/`)

| File | Purpose |
|---|---|
| `lib.rs` | Tauri entry: system tray, background input thread, `get_config`/`save_config` commands |
| `config.rs` | `Config` struct, load/save JSON at `~/.config/gestro/config.json` |
| `gesture.rs` | State machine (Idle → Pressed → Gesturing) + `atan2` direction calc |
| `shortcut.rs` | (Linux) Key name strings → `uinput::event::keyboard::Key` mapping |
| `shortcut_macos.rs` | (macOS) Key name strings → CGKeyCode + CGEventFlags injection |
| `shortcut_windows.rs` | (Windows) Key name strings → Virtual Key codes + SendInput injection |
| `input/mod.rs` | Platform dispatch: `InputMessage` type, re-exports `run_platform` per OS |
| `input/linux.rs` | evdev grab loop → gesture engine → uinput shortcut/passthrough injection |
| `input/macos.rs` | CGEventTap at HID level → gesture engine → CoreGraphics key injection |
| `input/windows.rs` | WH_MOUSE_LL hook + message pump → gesture engine → SendInput key injection |

The input thread is a plain `std::thread`. Config updates are sent via `std::sync::mpsc::SyncSender<InputMessage>`. The `Arc<Mutex<Config>>` is shared between the Tauri state and the input thread.

### Svelte (`src/`)

| File | Purpose |
|---|---|
| `routes/+page.svelte` | Main page: loads config via `invoke`, renders wheel, threshold slider, save |
| `lib/GestroWheel.svelte` | SVG annular wheel with 8 sectors; emits `onSelect(dir)` on click |
| `lib/ShortcutRecorder.svelte` | Modal dialog; captures keydown → key combo; emits `onConfirm(keys)` |
| `lib/types.ts` | `Config`, `Direction`, `Shortcut` types + `formatShortcut` helper |

## Platform setup

See [CONTRIBUTING.md](CONTRIBUTING.md) for per-platform prerequisites (Linux input group/udev, macOS Accessibility permission).
