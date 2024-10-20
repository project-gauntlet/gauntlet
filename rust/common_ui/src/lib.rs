use iced::advanced::widget::Text;
use iced::Element;
use iced::keyboard::Modifiers;
use iced::widget::text;
use iced_aw::core::icons;

use common::model::{PhysicalKey, PhysicalShortcut};

pub fn shortcut_to_text<'a, Message, Theme: text::StyleSheet + 'a>(shortcut: &PhysicalShortcut) -> (Element<'a, Message, Theme>, Option<Element<'a, Message, Theme>>, Option<Element<'a, Message, Theme>>, Option<Element<'a, Message, Theme>>, Option<Element<'a, Message, Theme>>) {
    let (key_name, show_shift) = match shortcut.physical_key {
        PhysicalKey::Enter => {
            let key_name = if cfg!(target_os = "macos") {
                text(icons::Bootstrap::ArrowReturnLeft)
                    .font(icons::BOOTSTRAP_FONT)
                    .into()
            } else {
                text("Enter")
                    .into()
            };

            (key_name, shortcut.modifier_shift)
        }
        _ => {
            let (key_name, show_shift) = physical_key_name(&shortcut.physical_key, shortcut.modifier_shift);

            let key_name: Element<_, _> = text(key_name)
                .into();

            (key_name, show_shift)
        }
    };

    let alt_modifier_text = if shortcut.modifier_alt {
        if cfg!(target_os = "macos") {
            Some(
                text(icons::Bootstrap::Option)
                    .font(icons::BOOTSTRAP_FONT)
                    .into()
            )
        } else {
            Some(
                text("Alt")
                    .into()
            )
        }
    } else {
        None
    };

    let meta_modifier_text = if shortcut.modifier_meta {
        if cfg!(target_os = "macos") {
            Some(
                text(icons::Bootstrap::Command)
                    .font(icons::BOOTSTRAP_FONT)
                    .into()
            )
        } else if cfg!(target_os = "windows") {
            Some(
                text("Win") // is it possible to have shortcuts that use win?
                    .into()
            )
        } else {
            Some(
                text("Super")
                    .into()
            )
        }
    } else {
        None
    };

    let control_modifier_text = if shortcut.modifier_control {
        if cfg!(target_os = "macos") {
            Some(
                text("^") // TODO bootstrap doesn't have proper macos ctrl icon
                    .font(icons::BOOTSTRAP_FONT)
                    .into()
            )
        } else {
            Some(
                text("Ctrl")
                    .into()
            )
        }
    } else {
        None
    };

    let shift_modifier_text = if show_shift && shortcut.modifier_shift {
        if cfg!(target_os = "macos") {
            Some(
                text(icons::Bootstrap::Shift)
                    .font(icons::BOOTSTRAP_FONT)
                    .into()
            )
        } else {
            Some(
                text("Shift")
                    .into()
            )
        }
    } else {
        None
    };

    (key_name, alt_modifier_text, meta_modifier_text, control_modifier_text, shift_modifier_text)
}

pub fn physical_key_model(key: iced::keyboard::key::PhysicalKey, modifiers: Modifiers) -> Option<PhysicalShortcut> {
    let model_key = match key {
        iced::keyboard::key::PhysicalKey::Backquote => PhysicalKey::Backquote,
        iced::keyboard::key::PhysicalKey::Backslash => PhysicalKey::Backslash,
        iced::keyboard::key::PhysicalKey::BracketLeft => PhysicalKey::BracketLeft,
        iced::keyboard::key::PhysicalKey::BracketRight => PhysicalKey::BracketRight,
        iced::keyboard::key::PhysicalKey::Comma => PhysicalKey::Comma,
        iced::keyboard::key::PhysicalKey::Digit0 => PhysicalKey::Digit0,
        iced::keyboard::key::PhysicalKey::Digit1 => PhysicalKey::Digit1,
        iced::keyboard::key::PhysicalKey::Digit2 => PhysicalKey::Digit2,
        iced::keyboard::key::PhysicalKey::Digit3 => PhysicalKey::Digit3,
        iced::keyboard::key::PhysicalKey::Digit4 => PhysicalKey::Digit4,
        iced::keyboard::key::PhysicalKey::Digit5 => PhysicalKey::Digit5,
        iced::keyboard::key::PhysicalKey::Digit6 => PhysicalKey::Digit6,
        iced::keyboard::key::PhysicalKey::Digit7 => PhysicalKey::Digit7,
        iced::keyboard::key::PhysicalKey::Digit8 => PhysicalKey::Digit8,
        iced::keyboard::key::PhysicalKey::Digit9 => PhysicalKey::Digit9,
        iced::keyboard::key::PhysicalKey::Equal => PhysicalKey::Equal,
        iced::keyboard::key::PhysicalKey::IntlBackslash => PhysicalKey::IntlBackslash,
        iced::keyboard::key::PhysicalKey::IntlRo => PhysicalKey::IntlRo,
        iced::keyboard::key::PhysicalKey::IntlYen => PhysicalKey::IntlYen,
        iced::keyboard::key::PhysicalKey::KeyA => PhysicalKey::KeyA,
        iced::keyboard::key::PhysicalKey::KeyB => PhysicalKey::KeyB,
        iced::keyboard::key::PhysicalKey::KeyC => PhysicalKey::KeyC,
        iced::keyboard::key::PhysicalKey::KeyD => PhysicalKey::KeyD,
        iced::keyboard::key::PhysicalKey::KeyE => PhysicalKey::KeyE,
        iced::keyboard::key::PhysicalKey::KeyF => PhysicalKey::KeyF,
        iced::keyboard::key::PhysicalKey::KeyG => PhysicalKey::KeyG,
        iced::keyboard::key::PhysicalKey::KeyH => PhysicalKey::KeyH,
        iced::keyboard::key::PhysicalKey::KeyI => PhysicalKey::KeyI,
        iced::keyboard::key::PhysicalKey::KeyJ => PhysicalKey::KeyJ,
        iced::keyboard::key::PhysicalKey::KeyK => PhysicalKey::KeyK,
        iced::keyboard::key::PhysicalKey::KeyL => PhysicalKey::KeyL,
        iced::keyboard::key::PhysicalKey::KeyM => PhysicalKey::KeyM,
        iced::keyboard::key::PhysicalKey::KeyN => PhysicalKey::KeyN,
        iced::keyboard::key::PhysicalKey::KeyO => PhysicalKey::KeyO,
        iced::keyboard::key::PhysicalKey::KeyP => PhysicalKey::KeyP,
        iced::keyboard::key::PhysicalKey::KeyQ => PhysicalKey::KeyQ,
        iced::keyboard::key::PhysicalKey::KeyR => PhysicalKey::KeyR,
        iced::keyboard::key::PhysicalKey::KeyS => PhysicalKey::KeyS,
        iced::keyboard::key::PhysicalKey::KeyT => PhysicalKey::KeyT,
        iced::keyboard::key::PhysicalKey::KeyU => PhysicalKey::KeyU,
        iced::keyboard::key::PhysicalKey::KeyV => PhysicalKey::KeyV,
        iced::keyboard::key::PhysicalKey::KeyW => PhysicalKey::KeyW,
        iced::keyboard::key::PhysicalKey::KeyX => PhysicalKey::KeyX,
        iced::keyboard::key::PhysicalKey::KeyY => PhysicalKey::KeyY,
        iced::keyboard::key::PhysicalKey::KeyZ => PhysicalKey::KeyZ,
        iced::keyboard::key::PhysicalKey::Minus => PhysicalKey::Minus,
        iced::keyboard::key::PhysicalKey::Period => PhysicalKey::Period,
        iced::keyboard::key::PhysicalKey::Quote => PhysicalKey::Quote,
        iced::keyboard::key::PhysicalKey::Semicolon => PhysicalKey::Semicolon,
        iced::keyboard::key::PhysicalKey::Slash => PhysicalKey::Slash,
        iced::keyboard::key::PhysicalKey::Backspace => PhysicalKey::Backspace,
        iced::keyboard::key::PhysicalKey::CapsLock => PhysicalKey::CapsLock,
        iced::keyboard::key::PhysicalKey::ContextMenu => PhysicalKey::ContextMenu,
        iced::keyboard::key::PhysicalKey::Enter => PhysicalKey::Enter,
        iced::keyboard::key::PhysicalKey::Space => PhysicalKey::Space,
        iced::keyboard::key::PhysicalKey::Tab => PhysicalKey::Tab,
        iced::keyboard::key::PhysicalKey::Convert => PhysicalKey::Convert,
        iced::keyboard::key::PhysicalKey::KanaMode => PhysicalKey::KanaMode,
        iced::keyboard::key::PhysicalKey::Lang1 => PhysicalKey::Lang1,
        iced::keyboard::key::PhysicalKey::Lang2 => PhysicalKey::Lang2,
        iced::keyboard::key::PhysicalKey::Lang3 => PhysicalKey::Lang3,
        iced::keyboard::key::PhysicalKey::Lang4 => PhysicalKey::Lang4,
        iced::keyboard::key::PhysicalKey::Lang5 => PhysicalKey::Lang5,
        iced::keyboard::key::PhysicalKey::NonConvert => PhysicalKey::NonConvert,
        iced::keyboard::key::PhysicalKey::Delete => PhysicalKey::Delete,
        iced::keyboard::key::PhysicalKey::End => PhysicalKey::End,
        iced::keyboard::key::PhysicalKey::Help => PhysicalKey::Help,
        iced::keyboard::key::PhysicalKey::Home => PhysicalKey::Home,
        iced::keyboard::key::PhysicalKey::Insert => PhysicalKey::Insert,
        iced::keyboard::key::PhysicalKey::PageDown => PhysicalKey::PageDown,
        iced::keyboard::key::PhysicalKey::PageUp => PhysicalKey::PageUp,
        iced::keyboard::key::PhysicalKey::ArrowDown => PhysicalKey::ArrowDown,
        iced::keyboard::key::PhysicalKey::ArrowLeft => PhysicalKey::ArrowLeft,
        iced::keyboard::key::PhysicalKey::ArrowRight => PhysicalKey::ArrowRight,
        iced::keyboard::key::PhysicalKey::ArrowUp => PhysicalKey::ArrowUp,
        iced::keyboard::key::PhysicalKey::NumLock => PhysicalKey::NumLock,
        iced::keyboard::key::PhysicalKey::Numpad0 => PhysicalKey::Numpad0,
        iced::keyboard::key::PhysicalKey::Numpad1 => PhysicalKey::Numpad1,
        iced::keyboard::key::PhysicalKey::Numpad2 => PhysicalKey::Numpad2,
        iced::keyboard::key::PhysicalKey::Numpad3 => PhysicalKey::Numpad3,
        iced::keyboard::key::PhysicalKey::Numpad4 => PhysicalKey::Numpad4,
        iced::keyboard::key::PhysicalKey::Numpad5 => PhysicalKey::Numpad5,
        iced::keyboard::key::PhysicalKey::Numpad6 => PhysicalKey::Numpad6,
        iced::keyboard::key::PhysicalKey::Numpad7 => PhysicalKey::Numpad7,
        iced::keyboard::key::PhysicalKey::Numpad8 => PhysicalKey::Numpad8,
        iced::keyboard::key::PhysicalKey::Numpad9 => PhysicalKey::Numpad9,
        iced::keyboard::key::PhysicalKey::NumpadAdd => PhysicalKey::NumpadAdd,
        iced::keyboard::key::PhysicalKey::NumpadBackspace => PhysicalKey::NumpadBackspace,
        iced::keyboard::key::PhysicalKey::NumpadClear => PhysicalKey::NumpadClear,
        iced::keyboard::key::PhysicalKey::NumpadClearEntry => PhysicalKey::NumpadClearEntry,
        iced::keyboard::key::PhysicalKey::NumpadComma => PhysicalKey::NumpadComma,
        iced::keyboard::key::PhysicalKey::NumpadDecimal => PhysicalKey::NumpadDecimal,
        iced::keyboard::key::PhysicalKey::NumpadDivide => PhysicalKey::NumpadDivide,
        iced::keyboard::key::PhysicalKey::NumpadEnter => PhysicalKey::NumpadEnter,
        iced::keyboard::key::PhysicalKey::NumpadEqual => PhysicalKey::NumpadEqual,
        iced::keyboard::key::PhysicalKey::NumpadHash => PhysicalKey::NumpadHash,
        iced::keyboard::key::PhysicalKey::NumpadMemoryAdd => PhysicalKey::NumpadMemoryAdd,
        iced::keyboard::key::PhysicalKey::NumpadMemoryClear => PhysicalKey::NumpadMemoryClear,
        iced::keyboard::key::PhysicalKey::NumpadMemoryRecall => PhysicalKey::NumpadMemoryRecall,
        iced::keyboard::key::PhysicalKey::NumpadMemoryStore => PhysicalKey::NumpadMemoryStore,
        iced::keyboard::key::PhysicalKey::NumpadMemorySubtract => PhysicalKey::NumpadMemorySubtract,
        iced::keyboard::key::PhysicalKey::NumpadMultiply => PhysicalKey::NumpadMultiply,
        iced::keyboard::key::PhysicalKey::NumpadParenLeft => PhysicalKey::NumpadParenLeft,
        iced::keyboard::key::PhysicalKey::NumpadParenRight => PhysicalKey::NumpadParenRight,
        iced::keyboard::key::PhysicalKey::NumpadStar => PhysicalKey::NumpadStar,
        iced::keyboard::key::PhysicalKey::NumpadSubtract => PhysicalKey::NumpadSubtract,
        iced::keyboard::key::PhysicalKey::Escape => PhysicalKey::Escape,
        iced::keyboard::key::PhysicalKey::Fn => PhysicalKey::Fn,
        iced::keyboard::key::PhysicalKey::FnLock => PhysicalKey::FnLock,
        iced::keyboard::key::PhysicalKey::PrintScreen => PhysicalKey::PrintScreen,
        iced::keyboard::key::PhysicalKey::ScrollLock => PhysicalKey::ScrollLock,
        iced::keyboard::key::PhysicalKey::Pause => PhysicalKey::Pause,
        iced::keyboard::key::PhysicalKey::BrowserBack => PhysicalKey::BrowserBack,
        iced::keyboard::key::PhysicalKey::BrowserFavorites => PhysicalKey::BrowserFavorites,
        iced::keyboard::key::PhysicalKey::BrowserForward => PhysicalKey::BrowserForward,
        iced::keyboard::key::PhysicalKey::BrowserHome => PhysicalKey::BrowserHome,
        iced::keyboard::key::PhysicalKey::BrowserRefresh => PhysicalKey::BrowserRefresh,
        iced::keyboard::key::PhysicalKey::BrowserSearch => PhysicalKey::BrowserSearch,
        iced::keyboard::key::PhysicalKey::BrowserStop => PhysicalKey::BrowserStop,
        iced::keyboard::key::PhysicalKey::Eject => PhysicalKey::Eject,
        iced::keyboard::key::PhysicalKey::LaunchApp1 => PhysicalKey::LaunchApp1,
        iced::keyboard::key::PhysicalKey::LaunchApp2 => PhysicalKey::LaunchApp2,
        iced::keyboard::key::PhysicalKey::LaunchMail => PhysicalKey::LaunchMail,
        iced::keyboard::key::PhysicalKey::MediaPlayPause => PhysicalKey::MediaPlayPause,
        iced::keyboard::key::PhysicalKey::MediaSelect => PhysicalKey::MediaSelect,
        iced::keyboard::key::PhysicalKey::MediaStop => PhysicalKey::MediaStop,
        iced::keyboard::key::PhysicalKey::MediaTrackNext => PhysicalKey::MediaTrackNext,
        iced::keyboard::key::PhysicalKey::MediaTrackPrevious => PhysicalKey::MediaTrackPrevious,
        iced::keyboard::key::PhysicalKey::Power => PhysicalKey::Power,
        iced::keyboard::key::PhysicalKey::Sleep => PhysicalKey::Sleep,
        iced::keyboard::key::PhysicalKey::AudioVolumeDown => PhysicalKey::AudioVolumeDown,
        iced::keyboard::key::PhysicalKey::AudioVolumeMute => PhysicalKey::AudioVolumeMute,
        iced::keyboard::key::PhysicalKey::AudioVolumeUp => PhysicalKey::AudioVolumeUp,
        iced::keyboard::key::PhysicalKey::WakeUp => PhysicalKey::WakeUp,
        iced::keyboard::key::PhysicalKey::Abort => PhysicalKey::Abort,
        iced::keyboard::key::PhysicalKey::Resume => PhysicalKey::Resume,
        iced::keyboard::key::PhysicalKey::Suspend => PhysicalKey::Suspend,
        iced::keyboard::key::PhysicalKey::Again => PhysicalKey::Again,
        iced::keyboard::key::PhysicalKey::Copy => PhysicalKey::Copy,
        iced::keyboard::key::PhysicalKey::Cut => PhysicalKey::Cut,
        iced::keyboard::key::PhysicalKey::Find => PhysicalKey::Find,
        iced::keyboard::key::PhysicalKey::Open => PhysicalKey::Open,
        iced::keyboard::key::PhysicalKey::Paste => PhysicalKey::Paste,
        iced::keyboard::key::PhysicalKey::Props => PhysicalKey::Props,
        iced::keyboard::key::PhysicalKey::Select => PhysicalKey::Select,
        iced::keyboard::key::PhysicalKey::Undo => PhysicalKey::Undo,
        iced::keyboard::key::PhysicalKey::Hiragana => PhysicalKey::Hiragana,
        iced::keyboard::key::PhysicalKey::Katakana => PhysicalKey::Katakana,
        iced::keyboard::key::PhysicalKey::F1 => PhysicalKey::F1,
        iced::keyboard::key::PhysicalKey::F2 => PhysicalKey::F2,
        iced::keyboard::key::PhysicalKey::F3 => PhysicalKey::F3,
        iced::keyboard::key::PhysicalKey::F4 => PhysicalKey::F4,
        iced::keyboard::key::PhysicalKey::F5 => PhysicalKey::F5,
        iced::keyboard::key::PhysicalKey::F6 => PhysicalKey::F6,
        iced::keyboard::key::PhysicalKey::F7 => PhysicalKey::F7,
        iced::keyboard::key::PhysicalKey::F8 => PhysicalKey::F8,
        iced::keyboard::key::PhysicalKey::F9 => PhysicalKey::F9,
        iced::keyboard::key::PhysicalKey::F10 => PhysicalKey::F10,
        iced::keyboard::key::PhysicalKey::F11 => PhysicalKey::F11,
        iced::keyboard::key::PhysicalKey::F12 => PhysicalKey::F12,
        iced::keyboard::key::PhysicalKey::F13 => PhysicalKey::F13,
        iced::keyboard::key::PhysicalKey::F14 => PhysicalKey::F14,
        iced::keyboard::key::PhysicalKey::F15 => PhysicalKey::F15,
        iced::keyboard::key::PhysicalKey::F16 => PhysicalKey::F16,
        iced::keyboard::key::PhysicalKey::F17 => PhysicalKey::F17,
        iced::keyboard::key::PhysicalKey::F18 => PhysicalKey::F18,
        iced::keyboard::key::PhysicalKey::F19 => PhysicalKey::F19,
        iced::keyboard::key::PhysicalKey::F20 => PhysicalKey::F20,
        iced::keyboard::key::PhysicalKey::F21 => PhysicalKey::F21,
        iced::keyboard::key::PhysicalKey::F22 => PhysicalKey::F22,
        iced::keyboard::key::PhysicalKey::F23 => PhysicalKey::F23,
        iced::keyboard::key::PhysicalKey::F24 => PhysicalKey::F24,
        iced::keyboard::key::PhysicalKey::F25 => PhysicalKey::F25,
        iced::keyboard::key::PhysicalKey::F26 => PhysicalKey::F26,
        iced::keyboard::key::PhysicalKey::F27 => PhysicalKey::F27,
        iced::keyboard::key::PhysicalKey::F28 => PhysicalKey::F28,
        iced::keyboard::key::PhysicalKey::F29 => PhysicalKey::F29,
        iced::keyboard::key::PhysicalKey::F30 => PhysicalKey::F30,
        iced::keyboard::key::PhysicalKey::F31 => PhysicalKey::F31,
        iced::keyboard::key::PhysicalKey::F32 => PhysicalKey::F32,
        iced::keyboard::key::PhysicalKey::F33 => PhysicalKey::F33,
        iced::keyboard::key::PhysicalKey::F34 => PhysicalKey::F34,
        iced::keyboard::key::PhysicalKey::F35 => PhysicalKey::F35,
        iced::keyboard::key::PhysicalKey::Meta
        | iced::keyboard::key::PhysicalKey::AltLeft
        | iced::keyboard::key::PhysicalKey::AltRight
        | iced::keyboard::key::PhysicalKey::ControlLeft
        | iced::keyboard::key::PhysicalKey::ControlRight
        | iced::keyboard::key::PhysicalKey::ShiftRight
        | iced::keyboard::key::PhysicalKey::ShiftLeft
        | iced::keyboard::key::PhysicalKey::SuperLeft
        | iced::keyboard::key::PhysicalKey::SuperRight
        | iced::keyboard::key::PhysicalKey::Hyper
        | iced::keyboard::key::PhysicalKey::Turbo
        | iced::keyboard::key::PhysicalKey::Unidentified
        | _ => {
            return None
        }
    };

    let modifier_shift = modifiers.shift();
    let modifier_control = modifiers.control();
    let modifier_alt = modifiers.alt();
    let modifier_meta = modifiers.logo();

    Some(PhysicalShortcut {
        physical_key: model_key,
        modifier_shift,
        modifier_control,
        modifier_alt,
        modifier_meta,
    })
}

pub fn physical_key_name(key: &PhysicalKey, modifier_shift: bool) -> (&'static str, bool) {
    let (name, show_shift) = match key {
        PhysicalKey::Backquote => (if modifier_shift { "~" } else { "`" }, false),
        PhysicalKey::Backslash => (if modifier_shift { "|" } else { "\\" }, false),
        PhysicalKey::IntlBackslash => (if modifier_shift { "Intl |" } else { "Intl \\" }, false),
        PhysicalKey::BracketLeft => (if modifier_shift { "{" } else { "[" }, false),
        PhysicalKey::BracketRight => (if modifier_shift { "}" } else { "]" }, false),
        PhysicalKey::Comma => (if modifier_shift { "<" } else { "," }, false),
        PhysicalKey::Digit1 => (if modifier_shift { "!" } else { "1" }, false),
        PhysicalKey::Digit2 => (if modifier_shift { "@" } else { "2" }, false),
        PhysicalKey::Digit3 => (if modifier_shift { "#" } else { "3" }, false),
        PhysicalKey::Digit4 => (if modifier_shift { "$" } else { "4" }, false),
        PhysicalKey::Digit5 => (if modifier_shift { "%" } else { "5" }, false),
        PhysicalKey::Digit6 => (if modifier_shift { "^" } else { "6" }, false),
        PhysicalKey::Digit7 => (if modifier_shift { "&" } else { "7" }, false),
        PhysicalKey::Digit8 => (if modifier_shift { "*" } else { "8" }, false),
        PhysicalKey::Digit9 => (if modifier_shift { "(" } else { "9" }, false),
        PhysicalKey::Digit0 => (if modifier_shift { ")" } else { "0" }, false),
        PhysicalKey::Equal => (if modifier_shift { "+" } else { "=" }, false),
        PhysicalKey::Minus => (if modifier_shift { "_" } else { "-" }, false),
        PhysicalKey::Period => (if modifier_shift { ">" } else { "." }, false),
        PhysicalKey::Quote => (if modifier_shift { "\"" } else { "'" }, false),
        PhysicalKey::Semicolon => (if modifier_shift { ":" } else { ";" }, false),
        PhysicalKey::Slash => (if modifier_shift { "?" } else { "/" }, false),
        PhysicalKey::IntlRo => ("IntlRo", true),
        PhysicalKey::IntlYen => ("IntlYen", true),
        PhysicalKey::KeyA => ("A", true),
        PhysicalKey::KeyB => ("B", true),
        PhysicalKey::KeyC => ("C", true),
        PhysicalKey::KeyD => ("D", true),
        PhysicalKey::KeyE => ("E", true),
        PhysicalKey::KeyF => ("F", true),
        PhysicalKey::KeyG => ("G", true),
        PhysicalKey::KeyH => ("H", true),
        PhysicalKey::KeyI => ("I", true),
        PhysicalKey::KeyJ => ("J", true),
        PhysicalKey::KeyK => ("K", true),
        PhysicalKey::KeyL => ("L", true),
        PhysicalKey::KeyM => ("M", true),
        PhysicalKey::KeyN => ("N", true),
        PhysicalKey::KeyO => ("O", true),
        PhysicalKey::KeyP => ("P", true),
        PhysicalKey::KeyQ => ("Q", true),
        PhysicalKey::KeyR => ("R", true),
        PhysicalKey::KeyS => ("S", true),
        PhysicalKey::KeyT => ("T", true),
        PhysicalKey::KeyU => ("U", true),
        PhysicalKey::KeyV => ("V", true),
        PhysicalKey::KeyW => ("W", true),
        PhysicalKey::KeyX => ("X", true),
        PhysicalKey::KeyY => ("Y", true),
        PhysicalKey::KeyZ => ("Z", true),
        PhysicalKey::Backspace => ("Backspace", true),
        PhysicalKey::CapsLock => ("CapsLock", true),
        PhysicalKey::ContextMenu => ("ContextMenu", true),
        PhysicalKey::Enter => ("Enter", true),
        PhysicalKey::Space => ("Space", true),
        PhysicalKey::Tab => ("Tab", true),
        PhysicalKey::Convert => ("Convert", true),
        PhysicalKey::KanaMode => ("KanaMode", true),
        PhysicalKey::Lang1 => ("Lang1", true),
        PhysicalKey::Lang2 => ("Lang2", true),
        PhysicalKey::Lang3 => ("Lang3", true),
        PhysicalKey::Lang4 => ("Lang4", true),
        PhysicalKey::Lang5 => ("Lang5", true),
        PhysicalKey::NonConvert => ("NonConvert", true),
        PhysicalKey::Delete => ("Delete", true),
        PhysicalKey::End => ("End", true),
        PhysicalKey::Help => ("Help", true),
        PhysicalKey::Home => ("Home", true),
        PhysicalKey::Insert => ("Insert", true),
        PhysicalKey::PageDown => ("PageDown", true),
        PhysicalKey::PageUp => ("PageUp", true),
        PhysicalKey::ArrowDown => ("ArrowDown", true),
        PhysicalKey::ArrowLeft => ("ArrowLeft", true),
        PhysicalKey::ArrowRight => ("ArrowRight", true),
        PhysicalKey::ArrowUp => ("ArrowUp", true),
        PhysicalKey::NumLock => ("NumLock", true),
        PhysicalKey::Numpad0 => ("Numpad 0", true),
        PhysicalKey::Numpad1 => ("Numpad 1", true),
        PhysicalKey::Numpad2 => ("Numpad 2", true),
        PhysicalKey::Numpad3 => ("Numpad 3", true),
        PhysicalKey::Numpad4 => ("Numpad 4", true),
        PhysicalKey::Numpad5 => ("Numpad 5", true),
        PhysicalKey::Numpad6 => ("Numpad 6", true),
        PhysicalKey::Numpad7 => ("Numpad 7", true),
        PhysicalKey::Numpad8 => ("Numpad 8", true),
        PhysicalKey::Numpad9 => ("Numpad 9", true),
        PhysicalKey::NumpadAdd => ("NumpadAdd", true),
        PhysicalKey::NumpadBackspace => ("NumpadBackspace", true),
        PhysicalKey::NumpadClear => ("NumpadClear", true),
        PhysicalKey::NumpadClearEntry => ("NumpadClearEntry", true),
        PhysicalKey::NumpadComma => ("NumpadComma", true),
        PhysicalKey::NumpadDecimal => ("NumpadDecimal", true),
        PhysicalKey::NumpadDivide => ("NumpadDivide", true),
        PhysicalKey::NumpadEnter => ("NumpadEnter", true),
        PhysicalKey::NumpadEqual => ("NumpadEqual", true),
        PhysicalKey::NumpadHash => ("NumpadHash", true),
        PhysicalKey::NumpadMemoryAdd => ("NumpadMemoryAdd", true),
        PhysicalKey::NumpadMemoryClear => ("NumpadMemoryClear", true),
        PhysicalKey::NumpadMemoryRecall => ("NumpadMemoryRecall", true),
        PhysicalKey::NumpadMemoryStore => ("NumpadMemoryStore", true),
        PhysicalKey::NumpadMemorySubtract => ("NumpadMemorySubtract", true),
        PhysicalKey::NumpadMultiply => ("NumpadMultiply", true),
        PhysicalKey::NumpadParenLeft => ("NumpadParenLeft", true),
        PhysicalKey::NumpadParenRight => ("NumpadParenRight", true),
        PhysicalKey::NumpadStar => ("NumpadStar", true),
        PhysicalKey::NumpadSubtract => ("NumpadSubtract", true),
        PhysicalKey::Escape => ("Escape", true),
        PhysicalKey::Fn => ("Fn", true),
        PhysicalKey::FnLock => ("FnLock", true),
        PhysicalKey::PrintScreen => ("PrintScreen", true),
        PhysicalKey::ScrollLock => ("ScrollLock", true),
        PhysicalKey::Pause => ("Pause", true),
        PhysicalKey::BrowserBack => ("BrowserBack", true),
        PhysicalKey::BrowserFavorites => ("BrowserFavorites", true),
        PhysicalKey::BrowserForward => ("BrowserForward", true),
        PhysicalKey::BrowserHome => ("BrowserHome", true),
        PhysicalKey::BrowserRefresh => ("BrowserRefresh", true),
        PhysicalKey::BrowserSearch => ("BrowserSearch", true),
        PhysicalKey::BrowserStop => ("BrowserStop", true),
        PhysicalKey::Eject => ("Eject", true),
        PhysicalKey::LaunchApp1 => ("LaunchApp1", true),
        PhysicalKey::LaunchApp2 => ("LaunchApp2", true),
        PhysicalKey::LaunchMail => ("LaunchMail", true),
        PhysicalKey::MediaPlayPause => ("MediaPlayPause", true),
        PhysicalKey::MediaSelect => ("MediaSelect", true),
        PhysicalKey::MediaStop => ("MediaStop", true),
        PhysicalKey::MediaTrackNext => ("MediaTrackNext", true),
        PhysicalKey::MediaTrackPrevious => ("MediaTrackPrevious", true),
        PhysicalKey::Power => ("Power", true),
        PhysicalKey::Sleep => ("Sleep", true),
        PhysicalKey::AudioVolumeDown => ("AudioVolumeDown", true),
        PhysicalKey::AudioVolumeMute => ("AudioVolumeMute", true),
        PhysicalKey::AudioVolumeUp => ("AudioVolumeUp", true),
        PhysicalKey::WakeUp => ("WakeUp", true),
        PhysicalKey::Abort => ("Abort", true),
        PhysicalKey::Resume => ("Resume", true),
        PhysicalKey::Suspend => ("Suspend", true),
        PhysicalKey::Again => ("Again", true),
        PhysicalKey::Copy => ("Copy", true),
        PhysicalKey::Cut => ("Cut", true),
        PhysicalKey::Find => ("Find", true),
        PhysicalKey::Open => ("Open", true),
        PhysicalKey::Paste => ("Paste", true),
        PhysicalKey::Props => ("Props", true),
        PhysicalKey::Select => ("Select", true),
        PhysicalKey::Undo => ("Undo", true),
        PhysicalKey::Hiragana => ("Hiragana", true),
        PhysicalKey::Katakana => ("Katakana", true),
        PhysicalKey::F1 => ("F1", true),
        PhysicalKey::F2 => ("F2", true),
        PhysicalKey::F3 => ("F3", true),
        PhysicalKey::F4 => ("F4", true),
        PhysicalKey::F5 => ("F5", true),
        PhysicalKey::F6 => ("F6", true),
        PhysicalKey::F7 => ("F7", true),
        PhysicalKey::F8 => ("F8", true),
        PhysicalKey::F9 => ("F9", true),
        PhysicalKey::F10 => ("F10", true),
        PhysicalKey::F11 => ("F11", true),
        PhysicalKey::F12 => ("F12", true),
        PhysicalKey::F13 => ("F13", true),
        PhysicalKey::F14 => ("F14", true),
        PhysicalKey::F15 => ("F15", true),
        PhysicalKey::F16 => ("F16", true),
        PhysicalKey::F17 => ("F17", true),
        PhysicalKey::F18 => ("F18", true),
        PhysicalKey::F19 => ("F19", true),
        PhysicalKey::F20 => ("F20", true),
        PhysicalKey::F21 => ("F21", true),
        PhysicalKey::F22 => ("F22", true),
        PhysicalKey::F23 => ("F23", true),
        PhysicalKey::F24 => ("F24", true),
        PhysicalKey::F25 => ("F25", true),
        PhysicalKey::F26 => ("F26", true),
        PhysicalKey::F27 => ("F27", true),
        PhysicalKey::F28 => ("F28", true),
        PhysicalKey::F29 => ("F29", true),
        PhysicalKey::F30 => ("F30", true),
        PhysicalKey::F31 => ("F31", true),
        PhysicalKey::F32 => ("F32", true),
        PhysicalKey::F33 => ("F33", true),
        PhysicalKey::F34 => ("F34", true),
        PhysicalKey::F35 => ("F35", true),
    };

    (name, show_shift)
}
