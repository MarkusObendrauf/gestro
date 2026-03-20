# gestro

An OS-wide mouse gesture launcher for macOS, Windows, and Linux. Hold right-click, move in one of 8 compass directions, release — a configured keyboard shortcut fires. Completely invisible during use; configured through a system tray settings window.

## How it works

1. Hold right-click anywhere on screen
2. Move the mouse in a direction (N, NE, E, SE, S, SW, W, NW)
3. Release — gestro fires the keyboard shortcut you assigned to that direction

If you don't move far enough (configurable threshold), right-click passes through normally.

## Platform notes

### macOS

gestro requires Accessibility permission to intercept mouse events system-wide. On first launch macOS will prompt you; if it doesn't, go to **System Settings → Privacy & Security → Accessibility** and enable gestro.

### Linux (X11 / Wayland)

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

### Windows

No additional setup required.

## Configuration

The settings window shows an 8-sector wheel. Click any sector to assign a keyboard shortcut to that direction. Adjust the movement threshold slider to control how far you must drag before a gesture registers.

Config is saved to `~/.config/gestro/config.json`.

## Tech stack

- **Backend:** Rust, [Tauri 2](https://tauri.app/)
- **Frontend:** SvelteKit, Svelte 5 (runes), TypeScript
- **Input (Linux):** evdev grab loop + uinput virtual device
- **Input (macOS):** CGEventTap at HID level + CoreGraphics key injection
- **Input (Windows):** WH_MOUSE_LL low-level hook + SendInput key injection
