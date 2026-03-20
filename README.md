# gestro

An OS-wide mouse gesture launcher for Linux. Hold right-click, move in one of 8 compass directions, release — a configured keyboard shortcut fires. Completely invisible during use; configured through a system tray settings window.

## How it works

1. Hold right-click anywhere on screen
2. Move the mouse in a direction (N, NE, E, SE, S, SW, W, NW)
3. Release — gestro fires the keyboard shortcut you assigned to that direction

If you don't move far enough (configurable threshold), right-click passes through normally.

## Requirements

- Linux (X11 or Wayland)
- Rust toolchain + Cargo
- Node.js + npm
- User in the `input` group with `/dev/uinput` access (see setup below)

## Setup

### 1. Input permissions (required)

gestro needs access to raw input devices and the uinput kernel module:

```sh
sudo usermod -a -G input $USER
```

Create `/etc/udev/rules.d/99-gestro.rules`:

```
KERNEL=="uinput", MODE="0660", GROUP="input"
SUBSYSTEM=="input", GROUP="input", MODE="0660"
```

Then reload rules and log out/in:

```sh
sudo udevadm control --reload-rules && sudo udevadm trigger
# log out and back in for group membership to take effect
```

### 2. Install dependencies

```sh
npm install
```

### 3. Run

```sh
npm run tauri dev
```

gestro starts in the background with a system tray icon. Open the tray menu to show the settings window.

## Configuration

The settings window shows an 8-sector wheel. Click any sector to assign a keyboard shortcut to that direction. Adjust the movement threshold slider to control how far you must drag before a gesture registers.

Config is saved to `~/.config/gestro/config.json`.

## Building

```sh
npm run tauri build
```

The compiled binary will be in `src-tauri/target/release/gestro`.

## Development

```sh
# Run with hot reload
npm run tauri dev

# Type-check the Svelte frontend
npx svelte-check

# Check the Rust backend
cd src-tauri && cargo check
```

## Tech stack

- **Backend:** Rust, [Tauri 2](https://tauri.app/), [evdev](https://crates.io/crates/evdev), [uinput](https://crates.io/crates/uinput)
- **Frontend:** SvelteKit, Svelte 5 (runes), TypeScript
- **Input:** evdev grab loop for raw mouse events; uinput virtual device for shortcut injection
