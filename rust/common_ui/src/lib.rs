use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use iced::Element;
use iced::Padding;
use iced::Pixels;
use iced::border::Radius;
use iced::keyboard::Modifiers;
use iced::widget::text;
use iced_fonts::lucide::arrow_big_up;
use iced_fonts::lucide::chevron_up;
use iced_fonts::lucide::command;
use iced_fonts::lucide::corner_down_left;
use iced_fonts::lucide::option;

pub fn padding(
    top: impl Into<Pixels>,
    right: impl Into<Pixels>,
    bottom: impl Into<Pixels>,
    left: impl Into<Pixels>,
) -> Padding {
    Padding {
        top: top.into().0,
        right: right.into().0,
        bottom: bottom.into().0,
        left: left.into().0,
    }
}
pub fn radius(
    top_left: impl Into<Pixels>,
    top_right: impl Into<Pixels>,
    bottom_right: impl Into<Pixels>,
    bottom_left: impl Into<Pixels>,
) -> Radius {
    Radius {
        top_left: top_left.into().0,
        top_right: top_right.into().0,
        bottom_right: bottom_right.into().0,
        bottom_left: bottom_left.into().0,
    }
}

pub fn shortcut_to_text<'a, Message, Theme: text::Catalog + 'a>(
    shortcut: &PhysicalShortcut,
) -> (
    Element<'a, Message, Theme>,
    Option<Element<'a, Message, Theme>>,
    Option<Element<'a, Message, Theme>>,
    Option<Element<'a, Message, Theme>>,
    Option<Element<'a, Message, Theme>>,
) {
    let (key_name, show_shift) = match shortcut.physical_key {
        PhysicalKey::Enter => {
            let key_name = if cfg!(target_os = "macos") {
                corner_down_left().size(14).into()
            } else {
                text("Enter").size(15).into()
            };

            (key_name, shortcut.modifier_shift)
        }
        _ => {
            let (key_name, show_shift) = physical_key_name(&shortcut.physical_key, shortcut.modifier_shift);

            let key_name: Element<_, _> = text(key_name).size(15).into();

            (key_name, show_shift)
        }
    };

    let alt_modifier_text = if shortcut.modifier_alt {
        if cfg!(target_os = "macos") {
            Some(option().size(15).into())
        } else {
            Some(text("Alt").size(15).into())
        }
    } else {
        None
    };

    let meta_modifier_text = if shortcut.modifier_meta {
        if cfg!(target_os = "macos") {
            Some(command().size(13).into())
        } else if cfg!(target_os = "windows") {
            Some(
                text("Win") // is it possible to have shortcuts that use win?
                    .size(15)
                    .into(),
            )
        } else {
            Some(text("Super").size(15).into())
        }
    } else {
        None
    };

    let control_modifier_text = if shortcut.modifier_control {
        if cfg!(target_os = "macos") {
            Some(chevron_up().size(15).into())
        } else {
            Some(text("Ctrl").size(15).into())
        }
    } else {
        None
    };

    let shift_modifier_text = if show_shift && shortcut.modifier_shift {
        if cfg!(target_os = "macos") {
            Some(arrow_big_up().size(17).into())
        } else {
            Some(text("Shift").size(15).into())
        }
    } else {
        None
    };

    (
        key_name,
        alt_modifier_text,
        meta_modifier_text,
        control_modifier_text,
        shift_modifier_text,
    )
}

pub fn physical_key_model(key: iced::keyboard::key::Code, modifiers: Modifiers) -> Option<PhysicalShortcut> {
    let model_key = match key {
        iced::keyboard::key::Code::Backquote => PhysicalKey::Backquote,
        iced::keyboard::key::Code::Backslash => PhysicalKey::Backslash,
        iced::keyboard::key::Code::BracketLeft => PhysicalKey::BracketLeft,
        iced::keyboard::key::Code::BracketRight => PhysicalKey::BracketRight,
        iced::keyboard::key::Code::Comma => PhysicalKey::Comma,
        iced::keyboard::key::Code::Digit0 => PhysicalKey::Digit0,
        iced::keyboard::key::Code::Digit1 => PhysicalKey::Digit1,
        iced::keyboard::key::Code::Digit2 => PhysicalKey::Digit2,
        iced::keyboard::key::Code::Digit3 => PhysicalKey::Digit3,
        iced::keyboard::key::Code::Digit4 => PhysicalKey::Digit4,
        iced::keyboard::key::Code::Digit5 => PhysicalKey::Digit5,
        iced::keyboard::key::Code::Digit6 => PhysicalKey::Digit6,
        iced::keyboard::key::Code::Digit7 => PhysicalKey::Digit7,
        iced::keyboard::key::Code::Digit8 => PhysicalKey::Digit8,
        iced::keyboard::key::Code::Digit9 => PhysicalKey::Digit9,
        iced::keyboard::key::Code::Equal => PhysicalKey::Equal,
        iced::keyboard::key::Code::IntlBackslash => PhysicalKey::IntlBackslash,
        iced::keyboard::key::Code::IntlRo => PhysicalKey::IntlRo,
        iced::keyboard::key::Code::IntlYen => PhysicalKey::IntlYen,
        iced::keyboard::key::Code::KeyA => PhysicalKey::KeyA,
        iced::keyboard::key::Code::KeyB => PhysicalKey::KeyB,
        iced::keyboard::key::Code::KeyC => PhysicalKey::KeyC,
        iced::keyboard::key::Code::KeyD => PhysicalKey::KeyD,
        iced::keyboard::key::Code::KeyE => PhysicalKey::KeyE,
        iced::keyboard::key::Code::KeyF => PhysicalKey::KeyF,
        iced::keyboard::key::Code::KeyG => PhysicalKey::KeyG,
        iced::keyboard::key::Code::KeyH => PhysicalKey::KeyH,
        iced::keyboard::key::Code::KeyI => PhysicalKey::KeyI,
        iced::keyboard::key::Code::KeyJ => PhysicalKey::KeyJ,
        iced::keyboard::key::Code::KeyK => PhysicalKey::KeyK,
        iced::keyboard::key::Code::KeyL => PhysicalKey::KeyL,
        iced::keyboard::key::Code::KeyM => PhysicalKey::KeyM,
        iced::keyboard::key::Code::KeyN => PhysicalKey::KeyN,
        iced::keyboard::key::Code::KeyO => PhysicalKey::KeyO,
        iced::keyboard::key::Code::KeyP => PhysicalKey::KeyP,
        iced::keyboard::key::Code::KeyQ => PhysicalKey::KeyQ,
        iced::keyboard::key::Code::KeyR => PhysicalKey::KeyR,
        iced::keyboard::key::Code::KeyS => PhysicalKey::KeyS,
        iced::keyboard::key::Code::KeyT => PhysicalKey::KeyT,
        iced::keyboard::key::Code::KeyU => PhysicalKey::KeyU,
        iced::keyboard::key::Code::KeyV => PhysicalKey::KeyV,
        iced::keyboard::key::Code::KeyW => PhysicalKey::KeyW,
        iced::keyboard::key::Code::KeyX => PhysicalKey::KeyX,
        iced::keyboard::key::Code::KeyY => PhysicalKey::KeyY,
        iced::keyboard::key::Code::KeyZ => PhysicalKey::KeyZ,
        iced::keyboard::key::Code::Minus => PhysicalKey::Minus,
        iced::keyboard::key::Code::Period => PhysicalKey::Period,
        iced::keyboard::key::Code::Quote => PhysicalKey::Quote,
        iced::keyboard::key::Code::Semicolon => PhysicalKey::Semicolon,
        iced::keyboard::key::Code::Slash => PhysicalKey::Slash,
        iced::keyboard::key::Code::Backspace => PhysicalKey::Backspace,
        iced::keyboard::key::Code::CapsLock => PhysicalKey::CapsLock,
        iced::keyboard::key::Code::ContextMenu => PhysicalKey::ContextMenu,
        iced::keyboard::key::Code::Enter => PhysicalKey::Enter,
        iced::keyboard::key::Code::Space => PhysicalKey::Space,
        iced::keyboard::key::Code::Tab => PhysicalKey::Tab,
        iced::keyboard::key::Code::Convert => PhysicalKey::Convert,
        iced::keyboard::key::Code::KanaMode => PhysicalKey::KanaMode,
        iced::keyboard::key::Code::Lang1 => PhysicalKey::Lang1,
        iced::keyboard::key::Code::Lang2 => PhysicalKey::Lang2,
        iced::keyboard::key::Code::Lang3 => PhysicalKey::Lang3,
        iced::keyboard::key::Code::Lang4 => PhysicalKey::Lang4,
        iced::keyboard::key::Code::Lang5 => PhysicalKey::Lang5,
        iced::keyboard::key::Code::NonConvert => PhysicalKey::NonConvert,
        iced::keyboard::key::Code::Delete => PhysicalKey::Delete,
        iced::keyboard::key::Code::End => PhysicalKey::End,
        iced::keyboard::key::Code::Help => PhysicalKey::Help,
        iced::keyboard::key::Code::Home => PhysicalKey::Home,
        iced::keyboard::key::Code::Insert => PhysicalKey::Insert,
        iced::keyboard::key::Code::PageDown => PhysicalKey::PageDown,
        iced::keyboard::key::Code::PageUp => PhysicalKey::PageUp,
        iced::keyboard::key::Code::ArrowDown => PhysicalKey::ArrowDown,
        iced::keyboard::key::Code::ArrowLeft => PhysicalKey::ArrowLeft,
        iced::keyboard::key::Code::ArrowRight => PhysicalKey::ArrowRight,
        iced::keyboard::key::Code::ArrowUp => PhysicalKey::ArrowUp,
        iced::keyboard::key::Code::NumLock => PhysicalKey::NumLock,
        iced::keyboard::key::Code::Numpad0 => PhysicalKey::Numpad0,
        iced::keyboard::key::Code::Numpad1 => PhysicalKey::Numpad1,
        iced::keyboard::key::Code::Numpad2 => PhysicalKey::Numpad2,
        iced::keyboard::key::Code::Numpad3 => PhysicalKey::Numpad3,
        iced::keyboard::key::Code::Numpad4 => PhysicalKey::Numpad4,
        iced::keyboard::key::Code::Numpad5 => PhysicalKey::Numpad5,
        iced::keyboard::key::Code::Numpad6 => PhysicalKey::Numpad6,
        iced::keyboard::key::Code::Numpad7 => PhysicalKey::Numpad7,
        iced::keyboard::key::Code::Numpad8 => PhysicalKey::Numpad8,
        iced::keyboard::key::Code::Numpad9 => PhysicalKey::Numpad9,
        iced::keyboard::key::Code::NumpadAdd => PhysicalKey::NumpadAdd,
        iced::keyboard::key::Code::NumpadBackspace => PhysicalKey::NumpadBackspace,
        iced::keyboard::key::Code::NumpadClear => PhysicalKey::NumpadClear,
        iced::keyboard::key::Code::NumpadClearEntry => PhysicalKey::NumpadClearEntry,
        iced::keyboard::key::Code::NumpadComma => PhysicalKey::NumpadComma,
        iced::keyboard::key::Code::NumpadDecimal => PhysicalKey::NumpadDecimal,
        iced::keyboard::key::Code::NumpadDivide => PhysicalKey::NumpadDivide,
        iced::keyboard::key::Code::NumpadEnter => PhysicalKey::NumpadEnter,
        iced::keyboard::key::Code::NumpadEqual => PhysicalKey::NumpadEqual,
        iced::keyboard::key::Code::NumpadHash => PhysicalKey::NumpadHash,
        iced::keyboard::key::Code::NumpadMemoryAdd => PhysicalKey::NumpadMemoryAdd,
        iced::keyboard::key::Code::NumpadMemoryClear => PhysicalKey::NumpadMemoryClear,
        iced::keyboard::key::Code::NumpadMemoryRecall => PhysicalKey::NumpadMemoryRecall,
        iced::keyboard::key::Code::NumpadMemoryStore => PhysicalKey::NumpadMemoryStore,
        iced::keyboard::key::Code::NumpadMemorySubtract => PhysicalKey::NumpadMemorySubtract,
        iced::keyboard::key::Code::NumpadMultiply => PhysicalKey::NumpadMultiply,
        iced::keyboard::key::Code::NumpadParenLeft => PhysicalKey::NumpadParenLeft,
        iced::keyboard::key::Code::NumpadParenRight => PhysicalKey::NumpadParenRight,
        iced::keyboard::key::Code::NumpadStar => PhysicalKey::NumpadStar,
        iced::keyboard::key::Code::NumpadSubtract => PhysicalKey::NumpadSubtract,
        iced::keyboard::key::Code::Escape => PhysicalKey::Escape,
        iced::keyboard::key::Code::Fn => PhysicalKey::Fn,
        iced::keyboard::key::Code::FnLock => PhysicalKey::FnLock,
        iced::keyboard::key::Code::PrintScreen => PhysicalKey::PrintScreen,
        iced::keyboard::key::Code::ScrollLock => PhysicalKey::ScrollLock,
        iced::keyboard::key::Code::Pause => PhysicalKey::Pause,
        iced::keyboard::key::Code::BrowserBack => PhysicalKey::BrowserBack,
        iced::keyboard::key::Code::BrowserFavorites => PhysicalKey::BrowserFavorites,
        iced::keyboard::key::Code::BrowserForward => PhysicalKey::BrowserForward,
        iced::keyboard::key::Code::BrowserHome => PhysicalKey::BrowserHome,
        iced::keyboard::key::Code::BrowserRefresh => PhysicalKey::BrowserRefresh,
        iced::keyboard::key::Code::BrowserSearch => PhysicalKey::BrowserSearch,
        iced::keyboard::key::Code::BrowserStop => PhysicalKey::BrowserStop,
        iced::keyboard::key::Code::Eject => PhysicalKey::Eject,
        iced::keyboard::key::Code::LaunchApp1 => PhysicalKey::LaunchApp1,
        iced::keyboard::key::Code::LaunchApp2 => PhysicalKey::LaunchApp2,
        iced::keyboard::key::Code::LaunchMail => PhysicalKey::LaunchMail,
        iced::keyboard::key::Code::MediaPlayPause => PhysicalKey::MediaPlayPause,
        iced::keyboard::key::Code::MediaSelect => PhysicalKey::MediaSelect,
        iced::keyboard::key::Code::MediaStop => PhysicalKey::MediaStop,
        iced::keyboard::key::Code::MediaTrackNext => PhysicalKey::MediaTrackNext,
        iced::keyboard::key::Code::MediaTrackPrevious => PhysicalKey::MediaTrackPrevious,
        iced::keyboard::key::Code::Power => PhysicalKey::Power,
        iced::keyboard::key::Code::Sleep => PhysicalKey::Sleep,
        iced::keyboard::key::Code::AudioVolumeDown => PhysicalKey::AudioVolumeDown,
        iced::keyboard::key::Code::AudioVolumeMute => PhysicalKey::AudioVolumeMute,
        iced::keyboard::key::Code::AudioVolumeUp => PhysicalKey::AudioVolumeUp,
        iced::keyboard::key::Code::WakeUp => PhysicalKey::WakeUp,
        iced::keyboard::key::Code::Abort => PhysicalKey::Abort,
        iced::keyboard::key::Code::Resume => PhysicalKey::Resume,
        iced::keyboard::key::Code::Suspend => PhysicalKey::Suspend,
        iced::keyboard::key::Code::Again => PhysicalKey::Again,
        iced::keyboard::key::Code::Copy => PhysicalKey::Copy,
        iced::keyboard::key::Code::Cut => PhysicalKey::Cut,
        iced::keyboard::key::Code::Find => PhysicalKey::Find,
        iced::keyboard::key::Code::Open => PhysicalKey::Open,
        iced::keyboard::key::Code::Paste => PhysicalKey::Paste,
        iced::keyboard::key::Code::Props => PhysicalKey::Props,
        iced::keyboard::key::Code::Select => PhysicalKey::Select,
        iced::keyboard::key::Code::Undo => PhysicalKey::Undo,
        iced::keyboard::key::Code::Hiragana => PhysicalKey::Hiragana,
        iced::keyboard::key::Code::Katakana => PhysicalKey::Katakana,
        iced::keyboard::key::Code::F1 => PhysicalKey::F1,
        iced::keyboard::key::Code::F2 => PhysicalKey::F2,
        iced::keyboard::key::Code::F3 => PhysicalKey::F3,
        iced::keyboard::key::Code::F4 => PhysicalKey::F4,
        iced::keyboard::key::Code::F5 => PhysicalKey::F5,
        iced::keyboard::key::Code::F6 => PhysicalKey::F6,
        iced::keyboard::key::Code::F7 => PhysicalKey::F7,
        iced::keyboard::key::Code::F8 => PhysicalKey::F8,
        iced::keyboard::key::Code::F9 => PhysicalKey::F9,
        iced::keyboard::key::Code::F10 => PhysicalKey::F10,
        iced::keyboard::key::Code::F11 => PhysicalKey::F11,
        iced::keyboard::key::Code::F12 => PhysicalKey::F12,
        iced::keyboard::key::Code::F13 => PhysicalKey::F13,
        iced::keyboard::key::Code::F14 => PhysicalKey::F14,
        iced::keyboard::key::Code::F15 => PhysicalKey::F15,
        iced::keyboard::key::Code::F16 => PhysicalKey::F16,
        iced::keyboard::key::Code::F17 => PhysicalKey::F17,
        iced::keyboard::key::Code::F18 => PhysicalKey::F18,
        iced::keyboard::key::Code::F19 => PhysicalKey::F19,
        iced::keyboard::key::Code::F20 => PhysicalKey::F20,
        iced::keyboard::key::Code::F21 => PhysicalKey::F21,
        iced::keyboard::key::Code::F22 => PhysicalKey::F22,
        iced::keyboard::key::Code::F23 => PhysicalKey::F23,
        iced::keyboard::key::Code::F24 => PhysicalKey::F24,
        iced::keyboard::key::Code::F25 => PhysicalKey::F25,
        iced::keyboard::key::Code::F26 => PhysicalKey::F26,
        iced::keyboard::key::Code::F27 => PhysicalKey::F27,
        iced::keyboard::key::Code::F28 => PhysicalKey::F28,
        iced::keyboard::key::Code::F29 => PhysicalKey::F29,
        iced::keyboard::key::Code::F30 => PhysicalKey::F30,
        iced::keyboard::key::Code::F31 => PhysicalKey::F31,
        iced::keyboard::key::Code::F32 => PhysicalKey::F32,
        iced::keyboard::key::Code::F33 => PhysicalKey::F33,
        iced::keyboard::key::Code::F34 => PhysicalKey::F34,
        iced::keyboard::key::Code::F35 => PhysicalKey::F35,
        iced::keyboard::key::Code::Meta
        | iced::keyboard::key::Code::AltLeft
        | iced::keyboard::key::Code::AltRight
        | iced::keyboard::key::Code::ControlLeft
        | iced::keyboard::key::Code::ControlRight
        | iced::keyboard::key::Code::ShiftRight
        | iced::keyboard::key::Code::ShiftLeft
        | iced::keyboard::key::Code::SuperLeft
        | iced::keyboard::key::Code::SuperRight
        | iced::keyboard::key::Code::Hyper
        | iced::keyboard::key::Code::Turbo
        | _ => return None,
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
