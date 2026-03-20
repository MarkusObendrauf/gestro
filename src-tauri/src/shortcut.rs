use uinput::event::keyboard::Key;

/// Map a human-readable key name to a uinput Key.
/// Supports modifier names and common keys.
pub fn key_from_name(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        // Modifiers
        "ctrl" | "control" => Some(Key::LeftControl),
        "lctrl" => Some(Key::LeftControl),
        "rctrl" => Some(Key::RightControl),
        "shift" => Some(Key::LeftShift),
        "lshift" => Some(Key::LeftShift),
        "rshift" => Some(Key::RightShift),
        "alt" => Some(Key::LeftAlt),
        "lalt" => Some(Key::LeftAlt),
        "ralt" | "altgr" => Some(Key::RightAlt),
        "super" | "meta" | "win" => Some(Key::LeftMeta),
        "lsuper" => Some(Key::LeftMeta),
        "rsuper" => Some(Key::RightMeta),

        // Letters
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),

        // Numbers
        "0" => Some(Key::_0),
        "1" => Some(Key::_1),
        "2" => Some(Key::_2),
        "3" => Some(Key::_3),
        "4" => Some(Key::_4),
        "5" => Some(Key::_5),
        "6" => Some(Key::_6),
        "7" => Some(Key::_7),
        "8" => Some(Key::_8),
        "9" => Some(Key::_9),

        // Function keys
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),

        // Special
        "enter" | "return" => Some(Key::Enter),
        "space" => Some(Key::Space),
        "tab" => Some(Key::Tab),
        "esc" | "escape" => Some(Key::Esc),
        "backspace" => Some(Key::BackSpace),
        "delete" | "del" => Some(Key::Delete),
        "insert" | "ins" => Some(Key::Insert),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),
        "up" => Some(Key::Up),
        "down" => Some(Key::Down),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "print" | "printscreen" => Some(Key::SysRq),
        "pause" => None, // Pause is in uinput::event::keyboard::Misc, not Key
        "capslock" => Some(Key::CapsLock),
        "numlock" => Some(Key::NumLock),
        "scrolllock" => Some(Key::ScrollLock),

        _ => {
            log::warn!("pie: unknown key name '{name}'");
            None
        }
    }
}

/// Convert a list of key name strings to uinput Keys, dropping unknowns.
pub fn keys_to_uinput(names: &[String]) -> Vec<Key> {
    names.iter().filter_map(|n| key_from_name(n)).collect()
}
