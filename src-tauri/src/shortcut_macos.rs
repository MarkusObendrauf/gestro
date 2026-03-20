use std::ffi::c_void;

use crate::config::Shortcut;

type CGKeyCode = u16;
type CGEventFlags = u64;

const FLAG_CMD: CGEventFlags = 0x00100000;
const FLAG_SHIFT: CGEventFlags = 0x00020000;
const FLAG_ALT: CGEventFlags = 0x00080000;
const FLAG_CTRL: CGEventFlags = 0x00040000;

/// kCGSessionEventTap — posts events after the HID tap level, so they won't
/// loop back through our kCGHIDEventTap interceptor.
const SESSION_TAP: u32 = 1;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateKeyboardEvent(
        source: *mut c_void,
        keycode: CGKeyCode,
        down: bool,
    ) -> *mut c_void;
    fn CGEventSetFlags(event: *mut c_void, flags: CGEventFlags);
    fn CGEventPost(tap_location: u32, event: *mut c_void);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

fn keycode_for(name: &str) -> Option<CGKeyCode> {
    // Hardware VK codes — layout-independent (US ANSI positions).
    Some(match name {
        "a" => 0x00,
        "s" => 0x01,
        "d" => 0x02,
        "f" => 0x03,
        "h" => 0x04,
        "g" => 0x05,
        "z" => 0x06,
        "x" => 0x07,
        "c" => 0x08,
        "v" => 0x09,
        "b" => 0x0B,
        "q" => 0x0C,
        "w" => 0x0D,
        "e" => 0x0E,
        "r" => 0x0F,
        "y" => 0x10,
        "t" => 0x11,
        "1" => 0x12,
        "2" => 0x13,
        "3" => 0x14,
        "4" => 0x15,
        "6" => 0x16,
        "5" => 0x17,
        "7" => 0x1A,
        "8" => 0x1C,
        "9" => 0x19,
        "0" => 0x1D,
        "o" => 0x1F,
        "u" => 0x20,
        "i" => 0x22,
        "p" => 0x23,
        "l" => 0x25,
        "j" => 0x26,
        "k" => 0x28,
        "n" => 0x2D,
        "m" => 0x2E,
        "tab" => 0x30,
        "space" => 0x31,
        "esc" | "escape" => 0x35,
        "delete" | "backspace" => 0x33,
        "del" | "forwarddelete" => 0x75,
        "enter" | "return" => 0x24,
        "capslock" => 0x39,
        "home" => 0x73,
        "end" => 0x77,
        "pageup" | "pgup" => 0x74,
        "pagedown" | "pgdn" => 0x79,
        "left" => 0x7B,
        "right" => 0x7C,
        "down" => 0x7D,
        "up" => 0x7E,
        "f1" => 0x7A,
        "f2" => 0x78,
        "f3" => 0x63,
        "f4" => 0x76,
        "f5" => 0x60,
        "f6" => 0x61,
        "f7" => 0x62,
        "f8" => 0x64,
        "f9" => 0x65,
        "f10" => 0x6D,
        "f11" => 0x67,
        "f12" => 0x6F,
        _ => {
            log::warn!("gestro: unknown key '{name}' for macOS");
            return None;
        }
    })
}

/// Parse a `Shortcut` into `(CGKeyCode, modifier_flags)`.
/// Returns `None` if no non-modifier key is present.
fn parse_shortcut(shortcut: &Shortcut) -> Option<(CGKeyCode, CGEventFlags)> {
    let mut flags: CGEventFlags = 0;
    let mut keycode: Option<CGKeyCode> = None;
    for k in &shortcut.keys {
        match k.to_lowercase().as_str() {
            "cmd" | "command" | "super" | "meta" => flags |= FLAG_CMD,
            "ctrl" | "control" => flags |= FLAG_CTRL,
            "alt" | "option" => flags |= FLAG_ALT,
            "shift" => flags |= FLAG_SHIFT,
            s => {
                if keycode.is_none() {
                    keycode = keycode_for(s);
                }
            }
        }
    }
    keycode.map(|k| (k, flags))
}

/// Emit a keyboard shortcut (key-down then key-up) via `CGEventPost`.
///
/// # Safety
/// Must be called from a thread with an active `CFRunLoop` (i.e. from inside the
/// event-tap callback or the input-thread run loop).
pub unsafe fn emit_shortcut(shortcut: &Shortcut) {
    let Some((kc, flags)) = parse_shortcut(shortcut) else {
        return;
    };

    let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), kc, true);
    if !down.is_null() {
        CGEventSetFlags(down, flags);
        CGEventPost(SESSION_TAP, down);
        CFRelease(down as *const c_void);
    }

    let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), kc, false);
    if !up.is_null() {
        CGEventSetFlags(up, flags);
        CGEventPost(SESSION_TAP, up);
        CFRelease(up as *const c_void);
    }
}
