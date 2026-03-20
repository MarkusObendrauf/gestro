# Contributing to gestro

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- Node.js + npm
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform

## Platform setup

### Linux

gestro reads raw input devices via evdev and injects events via uinput. Your user must be in the `input` group and uinput must be accessible:

```sh
sudo usermod -a -G input $USER
```

Create `/etc/udev/rules.d/99-gestro.rules`:

```
KERNEL=="uinput", MODE="0660", GROUP="input"
SUBSYSTEM=="input", GROUP="input", MODE="0660"
```

Reload and log out/in:

```sh
sudo udevadm control --reload-rules && sudo udevadm trigger
# log out and back in
```

### macOS

gestro uses CGEventTap which requires Accessibility permission. Grant it in **System Settings → Privacy & Security → Accessibility** before running.

### Windows

No additional setup required.

## Commands

```sh
# Install frontend dependencies
npm install

# Run in dev mode (Vite + Tauri with hot reload)
npm run tauri dev

# Type-check the Svelte frontend
npx svelte-check

# Check the Rust backend
cd src-tauri && cargo check

# Build for release
npm run tauri build
```

## Architecture

See [CLAUDE.md](CLAUDE.md) for a detailed breakdown of the codebase.
