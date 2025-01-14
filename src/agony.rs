// ASDIUHIH9UPOAFHDASUIYHGAADSFIHAFDIGJ IO HATE THIS.
use global_hotkey::hotkey::Code;
use macroquad::input::KeyCode;

pub fn mq_key_to_global_hotkey(k: KeyCode) -> Code {
    match k {
        KeyCode::Space => Code::Space,
        KeyCode::Apostrophe => Code::Quote,
        KeyCode::Comma => Code::Comma,
        KeyCode::Minus => Code::Minus,
        KeyCode::Period => Code::Period,
        KeyCode::Slash => Code::Slash,
        KeyCode::Key0 => Code::Digit0,
        KeyCode::Key1 => Code::Digit1,
        KeyCode::Key2 => Code::Digit2,
        KeyCode::Key3 => Code::Digit3,
        KeyCode::Key4 => Code::Digit4,
        KeyCode::Key5 => Code::Digit5,
        KeyCode::Key6 => Code::Digit6,
        KeyCode::Key7 => Code::Digit7,
        KeyCode::Key8 => Code::Digit8,
        KeyCode::Key9 => Code::Digit9,
        KeyCode::Semicolon => Code::Semicolon,
        KeyCode::Equal => Code::Equal,
        KeyCode::A => Code::KeyA,
        KeyCode::B => Code::KeyB,
        KeyCode::C => Code::KeyC,
        KeyCode::D => Code::KeyD,
        KeyCode::E => Code::KeyE,
        KeyCode::F => Code::KeyF,
        KeyCode::G => Code::KeyG,
        KeyCode::H => Code::KeyH,
        KeyCode::I => Code::KeyI,
        KeyCode::J => Code::KeyJ,
        KeyCode::K => Code::KeyK,
        KeyCode::L => Code::KeyL,
        KeyCode::M => Code::KeyM,
        KeyCode::N => Code::KeyN,
        KeyCode::O => Code::KeyO,
        KeyCode::P => Code::KeyP,
        KeyCode::Q => Code::KeyQ,
        KeyCode::R => Code::KeyR,
        KeyCode::S => Code::KeyS,
        KeyCode::T => Code::KeyT,
        KeyCode::U => Code::KeyU,
        KeyCode::V => Code::KeyV,
        KeyCode::W => Code::KeyW,
        KeyCode::X => Code::KeyX,
        KeyCode::Y => Code::KeyY,
        KeyCode::Z => Code::KeyZ,
        KeyCode::LeftBracket => Code::BracketLeft,
        KeyCode::Backslash => Code::Backslash,
        KeyCode::RightBracket => Code::BracketRight,
        KeyCode::GraveAccent => Code::Backquote,
        KeyCode::World1 => Code::Unidentified, // ??
        KeyCode::World2 => Code::Unidentified, // ????
        KeyCode::Escape => Code::Escape,
        KeyCode::Enter => Code::Enter,
        KeyCode::Tab => Code::Tab,
        KeyCode::Backspace => Code::Backspace,
        KeyCode::Insert => Code::Insert,
        KeyCode::Delete => Code::Delete,
        KeyCode::Right => Code::ArrowRight,
        KeyCode::Left => Code::ArrowLeft,
        KeyCode::Down => Code::ArrowDown,
        KeyCode::Up => Code::ArrowUp,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::PageDown => Code::PageDown,
        KeyCode::Home => Code::Home,
        KeyCode::End => Code::End,
        KeyCode::CapsLock => Code::CapsLock,
        KeyCode::ScrollLock => Code::ScrollLock,
        KeyCode::NumLock => Code::NumLock,
        KeyCode::PrintScreen => Code::PrintScreen,
        KeyCode::Pause => Code::Pause,
        KeyCode::F1 => Code::F1,
        KeyCode::F2 => Code::F2,
        KeyCode::F3 => Code::F3,
        KeyCode::F4 => Code::F4,
        KeyCode::F5 => Code::F5,
        KeyCode::F6 => Code::F6,
        KeyCode::F7 => Code::F7,
        KeyCode::F8 => Code::F8,
        KeyCode::F9 => Code::F9,
        KeyCode::F10 => Code::F10,
        KeyCode::F11 => Code::F11,
        KeyCode::F12 => Code::F12,
        KeyCode::F13 => Code::F13,
        KeyCode::F14 => Code::F14,
        KeyCode::F15 => Code::F15,
        KeyCode::F16 => Code::F16,
        KeyCode::F17 => Code::F17,
        KeyCode::F18 => Code::F18,
        KeyCode::F19 => Code::F19,
        KeyCode::F20 => Code::F20,
        KeyCode::F21 => Code::F21,
        KeyCode::F22 => Code::F22,
        KeyCode::F23 => Code::F23,
        KeyCode::F24 => Code::F24,
        KeyCode::F25 => Code::F25,
        KeyCode::Kp0 => Code::Numpad0,
        KeyCode::Kp1 => Code::Numpad1,
        KeyCode::Kp2 => Code::Numpad2,
        KeyCode::Kp3 => Code::Numpad3,
        KeyCode::Kp4 => Code::Numpad4,
        KeyCode::Kp5 => Code::Numpad5,
        KeyCode::Kp6 => Code::Numpad6,
        KeyCode::Kp7 => Code::Numpad7,
        KeyCode::Kp8 => Code::Numpad8,
        KeyCode::Kp9 => Code::Numpad9,
        KeyCode::KpDecimal => Code::NumpadDecimal,
        KeyCode::KpDivide => Code::NumpadDivide,
        KeyCode::KpMultiply => Code::NumpadMultiply,
        KeyCode::KpSubtract => Code::NumpadSubtract,
        KeyCode::KpAdd => Code::NumpadAdd,
        KeyCode::KpEnter => Code::NumpadEnter,
        KeyCode::KpEqual => Code::NumpadEqual,
        KeyCode::LeftShift => Code::ShiftLeft,
        KeyCode::LeftControl => Code::ControlLeft,
        KeyCode::LeftAlt => Code::AltLeft,
        KeyCode::LeftSuper => Code::Super,
        KeyCode::RightShift => Code::ShiftRight,
        KeyCode::RightControl => Code::ControlRight,
        KeyCode::RightAlt => Code::AltRight,
        KeyCode::RightSuper => Code::Super,
        KeyCode::Menu => Code::ContextMenu,
        KeyCode::Unknown => Code::Unidentified,
    }
}
