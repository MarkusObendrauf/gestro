use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};

use crate::config::Shortcut;

/// Map a key name string to a Windows Virtual Key code.
fn vk_for(name: &str) -> Option<u16> {
    Some(match name {
        // Letters
        "a" => 0x41,
        "b" => 0x42,
        "c" => 0x43,
        "d" => 0x44,
        "e" => 0x45,
        "f" => 0x46,
        "g" => 0x47,
        "h" => 0x48,
        "i" => 0x49,
        "j" => 0x4A,
        "k" => 0x4B,
        "l" => 0x4C,
        "m" => 0x4D,
        "n" => 0x4E,
        "o" => 0x4F,
        "p" => 0x50,
        "q" => 0x51,
        "r" => 0x52,
        "s" => 0x53,
        "t" => 0x54,
        "u" => 0x55,
        "v" => 0x56,
        "w" => 0x57,
        "x" => 0x58,
        "y" => 0x59,
        "z" => 0x5A,
        // Digits
        "0" => 0x30,
        "1" => 0x31,
        "2" => 0x32,
        "3" => 0x33,
        "4" => 0x34,
        "5" => 0x35,
        "6" => 0x36,
        "7" => 0x37,
        "8" => 0x38,
        "9" => 0x39,
        // Function keys
        "f1" => 0x70,
        "f2" => 0x71,
        "f3" => 0x72,
        "f4" => 0x73,
        "f5" => 0x74,
        "f6" => 0x75,
        "f7" => 0x76,
        "f8" => 0x77,
        "f9" => 0x78,
        "f10" => 0x79,
        "f11" => 0x7A,
        "f12" => 0x7B,
        // Special keys
        "tab" => 0x09,
        "space" => 0x20,
        "enter" | "return" => 0x0D,
        "esc" | "escape" => 0x1B,
        "backspace" => 0x08,
        "delete" | "del" => 0x2E,
        "home" => 0x24,
        "end" => 0x23,
        "pageup" | "pgup" => 0x21,
        "pagedown" | "pgdn" => 0x22,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        // Modifiers
        "ctrl" | "control" => 0x11,
        "shift" => 0x10,
        "alt" | "option" => 0x12,
        "win" | "super" | "meta" | "cmd" | "command" => 0x5B,
        _ => {
            log::warn!("gestro: unknown key '{name}' for Windows");
            return None;
        }
    })
}

/// Keys that require KEYEVENTF_EXTENDEDKEY to distinguish from numpad variants.
fn is_extended(vk: u16) -> bool {
    matches!(
        vk,
        0x21  // VK_PRIOR (PageUp)
        | 0x22  // VK_NEXT (PageDown)
        | 0x23  // VK_END
        | 0x24  // VK_HOME
        | 0x25  // VK_LEFT
        | 0x26  // VK_UP
        | 0x27  // VK_RIGHT
        | 0x28  // VK_DOWN
        | 0x2E  // VK_DELETE
        | 0x5B  // VK_LWIN
        | 0x5C  // VK_RWIN
        | 0x70..=0x7B // F1-F12
    )
}

fn make_key_input(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Emit a keyboard shortcut via `SendInput`.
pub fn emit_shortcut(shortcut: &Shortcut) {
    let mut modifiers: Vec<u16> = Vec::new();
    let mut primary: Option<u16> = None;

    for k in &shortcut.keys {
        let name = k.to_lowercase();
        match name.as_str() {
            "ctrl" | "control" => modifiers.push(0x11),
            "shift" => modifiers.push(0x10),
            "alt" | "option" => modifiers.push(0x12),
            "win" | "super" | "meta" | "cmd" | "command" => modifiers.push(0x5B),
            s => {
                if primary.is_none() {
                    primary = vk_for(s);
                }
            }
        }
    }

    let Some(prim) = primary else {
        log::warn!("gestro: shortcut has no primary key, skipping");
        return;
    };

    let mut inputs: Vec<INPUT> = Vec::new();

    // Press all modifiers
    for &m in &modifiers {
        let ext = if is_extended(m) { KEYEVENTF_EXTENDEDKEY } else { KEYBD_EVENT_FLAGS(0) };
        inputs.push(make_key_input(m, ext));
    }

    // Press primary key
    let prim_ext = if is_extended(prim) { KEYEVENTF_EXTENDEDKEY } else { KEYBD_EVENT_FLAGS(0) };
    inputs.push(make_key_input(prim, prim_ext));

    // Release primary key
    inputs.push(make_key_input(prim, prim_ext | KEYEVENTF_KEYUP));

    // Release modifiers in reverse order
    for &m in modifiers.iter().rev() {
        let ext = if is_extended(m) { KEYEVENTF_EXTENDEDKEY } else { KEYBD_EVENT_FLAGS(0) };
        inputs.push(make_key_input(m, ext | KEYEVENTF_KEYUP));
    }

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
