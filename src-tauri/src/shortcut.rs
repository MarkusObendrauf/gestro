use uinput::event::keyboard::{Key, Misc};

/// A key that can be pressed/released via uinput — either a standard Key or a Misc key.
pub enum AnyKey {
    Key(Key),
    Misc(Misc),
}

/// Map a human-readable key name to a uinput key.
pub fn key_from_name(name: &str) -> Option<AnyKey> {
    match name.to_lowercase().as_str() {
        // Modifiers
        "ctrl" | "control" => Some(AnyKey::Key(Key::LeftControl)),
        "lctrl" => Some(AnyKey::Key(Key::LeftControl)),
        "rctrl" => Some(AnyKey::Key(Key::RightControl)),
        "shift" => Some(AnyKey::Key(Key::LeftShift)),
        "lshift" => Some(AnyKey::Key(Key::LeftShift)),
        "rshift" => Some(AnyKey::Key(Key::RightShift)),
        "alt" => Some(AnyKey::Key(Key::LeftAlt)),
        "lalt" => Some(AnyKey::Key(Key::LeftAlt)),
        "ralt" | "altgr" => Some(AnyKey::Key(Key::RightAlt)),
        "super" | "meta" | "win" => Some(AnyKey::Key(Key::LeftMeta)),
        "lsuper" => Some(AnyKey::Key(Key::LeftMeta)),
        "rsuper" => Some(AnyKey::Key(Key::RightMeta)),

        // Letters
        "a" => Some(AnyKey::Key(Key::A)),
        "b" => Some(AnyKey::Key(Key::B)),
        "c" => Some(AnyKey::Key(Key::C)),
        "d" => Some(AnyKey::Key(Key::D)),
        "e" => Some(AnyKey::Key(Key::E)),
        "f" => Some(AnyKey::Key(Key::F)),
        "g" => Some(AnyKey::Key(Key::G)),
        "h" => Some(AnyKey::Key(Key::H)),
        "i" => Some(AnyKey::Key(Key::I)),
        "j" => Some(AnyKey::Key(Key::J)),
        "k" => Some(AnyKey::Key(Key::K)),
        "l" => Some(AnyKey::Key(Key::L)),
        "m" => Some(AnyKey::Key(Key::M)),
        "n" => Some(AnyKey::Key(Key::N)),
        "o" => Some(AnyKey::Key(Key::O)),
        "p" => Some(AnyKey::Key(Key::P)),
        "q" => Some(AnyKey::Key(Key::Q)),
        "r" => Some(AnyKey::Key(Key::R)),
        "s" => Some(AnyKey::Key(Key::S)),
        "t" => Some(AnyKey::Key(Key::T)),
        "u" => Some(AnyKey::Key(Key::U)),
        "v" => Some(AnyKey::Key(Key::V)),
        "w" => Some(AnyKey::Key(Key::W)),
        "x" => Some(AnyKey::Key(Key::X)),
        "y" => Some(AnyKey::Key(Key::Y)),
        "z" => Some(AnyKey::Key(Key::Z)),

        // Numbers
        "0" => Some(AnyKey::Key(Key::_0)),
        "1" => Some(AnyKey::Key(Key::_1)),
        "2" => Some(AnyKey::Key(Key::_2)),
        "3" => Some(AnyKey::Key(Key::_3)),
        "4" => Some(AnyKey::Key(Key::_4)),
        "5" => Some(AnyKey::Key(Key::_5)),
        "6" => Some(AnyKey::Key(Key::_6)),
        "7" => Some(AnyKey::Key(Key::_7)),
        "8" => Some(AnyKey::Key(Key::_8)),
        "9" => Some(AnyKey::Key(Key::_9)),

        // Function keys
        "f1" => Some(AnyKey::Key(Key::F1)),
        "f2" => Some(AnyKey::Key(Key::F2)),
        "f3" => Some(AnyKey::Key(Key::F3)),
        "f4" => Some(AnyKey::Key(Key::F4)),
        "f5" => Some(AnyKey::Key(Key::F5)),
        "f6" => Some(AnyKey::Key(Key::F6)),
        "f7" => Some(AnyKey::Key(Key::F7)),
        "f8" => Some(AnyKey::Key(Key::F8)),
        "f9" => Some(AnyKey::Key(Key::F9)),
        "f10" => Some(AnyKey::Key(Key::F10)),
        "f11" => Some(AnyKey::Key(Key::F11)),
        "f12" => Some(AnyKey::Key(Key::F12)),

        // Special
        "enter" | "return" => Some(AnyKey::Key(Key::Enter)),
        "space" => Some(AnyKey::Key(Key::Space)),
        "tab" => Some(AnyKey::Key(Key::Tab)),
        "esc" | "escape" => Some(AnyKey::Key(Key::Esc)),
        "backspace" => Some(AnyKey::Key(Key::BackSpace)),
        "delete" | "del" => Some(AnyKey::Key(Key::Delete)),
        "insert" | "ins" => Some(AnyKey::Key(Key::Insert)),
        "home" => Some(AnyKey::Key(Key::Home)),
        "end" => Some(AnyKey::Key(Key::End)),
        "pageup" | "pgup" => Some(AnyKey::Key(Key::PageUp)),
        "pagedown" | "pgdn" => Some(AnyKey::Key(Key::PageDown)),
        "up" => Some(AnyKey::Key(Key::Up)),
        "down" => Some(AnyKey::Key(Key::Down)),
        "left" => Some(AnyKey::Key(Key::Left)),
        "right" => Some(AnyKey::Key(Key::Right)),
        "print" | "printscreen" => Some(AnyKey::Key(Key::SysRq)),
        "capslock" => Some(AnyKey::Key(Key::CapsLock)),
        "numlock" => Some(AnyKey::Key(Key::NumLock)),
        "scrolllock" => Some(AnyKey::Key(Key::ScrollLock)),

        // Browser / media keys (live in uinput::event::keyboard::Misc)
        "browserback" | "browser_back" => Some(AnyKey::Misc(Misc::Back)),
        "browserforward" | "browser_forward" => Some(AnyKey::Misc(Misc::Forward)),

        _ => {
            log::warn!("gestro: unknown key name '{name}'");
            None
        }
    }
}

/// Convert a list of key name strings to uinput AnyKeys, dropping unknowns.
pub fn keys_to_uinput(names: &[String]) -> Vec<AnyKey> {
    names.iter().filter_map(|n| key_from_name(n)).collect()
}
