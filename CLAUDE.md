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
| `shortcut.rs` | Key name strings → `uinput::event::keyboard::Key` mapping |
| `input/mod.rs` | Platform dispatch: re-exports `linux::run` on Linux |
| `input/linux.rs` | evdev grab loop → gesture engine → uinput shortcut/passthrough injection |

The input thread is a plain `std::thread` (evdev is blocking). Config updates are sent via `std::sync::mpsc::SyncSender<InputMessage>`. The `Arc<Mutex<Config>>` is shared between the Tauri state and the input thread.

### Svelte (`src/`)

| File | Purpose |
|---|---|
| `routes/+page.svelte` | Main page: loads config via `invoke`, renders wheel, threshold slider, save |
| `lib/GestroWheel.svelte` | SVG annular wheel with 8 sectors; emits `onSelect(dir)` on click |
| `lib/ShortcutRecorder.svelte` | Modal dialog; captures keydown → key combo; emits `onConfirm(keys)` |
| `lib/types.ts` | `Config`, `Direction`, `Shortcut` types + `formatShortcut` helper |

## Linux setup (required)

User must be in the `input` group and `/dev/uinput` must be accessible:

```sh
sudo usermod -a -G input $USER
# udev rule at /etc/udev/rules.d/99-gestro.rules:
# KERNEL=="uinput", MODE="0660", GROUP="input"
# SUBSYSTEM=="input", GROUP="input", MODE="0660"
sudo udevadm control --reload-rules && sudo udevadm trigger
# then log out and back in
```
