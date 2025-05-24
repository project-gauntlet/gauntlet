use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use global_hotkey::hotkey::Code;
use global_hotkey::hotkey::HotKey;
use global_hotkey::hotkey::Modifiers;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use tokio::runtime::Handle;

use crate::ui::AppMsg;

pub fn register_listener(msg_sender: Sender<AppMsg>) {
    let handle = Handle::current();

    global_hotkey::GlobalHotKeyEvent::set_event_handler(Some(move |e: global_hotkey::GlobalHotKeyEvent| {
        let mut msg_sender = msg_sender.clone();

        if let global_hotkey::HotKeyState::Pressed = e.state() {
            handle.spawn(async move {
                if let Err(err) = msg_sender.send(AppMsg::HandleGlobalShortcut(e.id)).await {
                    tracing::warn!(target = "rpc", "error occurred when receiving shortcut event {:?}", err)
                }
            });
        }
    }));
}

pub fn convert_physical_shortcut_to_hotkey(shortcut: PhysicalShortcut) -> HotKey {
    let modifiers: Modifiers = {
        let mut modifiers = Modifiers::empty();

        if shortcut.modifier_alt {
            modifiers.insert(Modifiers::ALT);
        }

        if shortcut.modifier_control {
            modifiers.insert(Modifiers::CONTROL);
        }

        if shortcut.modifier_meta {
            modifiers.insert(Modifiers::META);
        }

        if shortcut.modifier_shift {
            modifiers.insert(Modifiers::SHIFT);
        }

        modifiers
    };

    let code = match shortcut.physical_key {
        PhysicalKey::Backquote => Code::Backquote,
        PhysicalKey::Backslash => Code::Backslash,
        PhysicalKey::BracketLeft => Code::BracketLeft,
        PhysicalKey::BracketRight => Code::BracketRight,
        PhysicalKey::Comma => Code::Comma,
        PhysicalKey::Digit1 => Code::Digit1,
        PhysicalKey::Digit2 => Code::Digit2,
        PhysicalKey::Digit3 => Code::Digit3,
        PhysicalKey::Digit4 => Code::Digit4,
        PhysicalKey::Digit5 => Code::Digit5,
        PhysicalKey::Digit6 => Code::Digit6,
        PhysicalKey::Digit7 => Code::Digit7,
        PhysicalKey::Digit8 => Code::Digit8,
        PhysicalKey::Digit9 => Code::Digit9,
        PhysicalKey::Digit0 => Code::Digit0,
        PhysicalKey::Equal => Code::Equal,
        PhysicalKey::IntlBackslash => Code::IntlBackslash,
        PhysicalKey::IntlRo => Code::IntlRo,
        PhysicalKey::IntlYen => Code::IntlYen,
        PhysicalKey::KeyA => Code::KeyA,
        PhysicalKey::KeyB => Code::KeyB,
        PhysicalKey::KeyC => Code::KeyC,
        PhysicalKey::KeyD => Code::KeyD,
        PhysicalKey::KeyE => Code::KeyE,
        PhysicalKey::KeyF => Code::KeyF,
        PhysicalKey::KeyG => Code::KeyG,
        PhysicalKey::KeyH => Code::KeyH,
        PhysicalKey::KeyI => Code::KeyI,
        PhysicalKey::KeyJ => Code::KeyJ,
        PhysicalKey::KeyK => Code::KeyK,
        PhysicalKey::KeyL => Code::KeyL,
        PhysicalKey::KeyM => Code::KeyM,
        PhysicalKey::KeyN => Code::KeyN,
        PhysicalKey::KeyO => Code::KeyO,
        PhysicalKey::KeyP => Code::KeyP,
        PhysicalKey::KeyQ => Code::KeyQ,
        PhysicalKey::KeyR => Code::KeyR,
        PhysicalKey::KeyS => Code::KeyS,
        PhysicalKey::KeyT => Code::KeyT,
        PhysicalKey::KeyU => Code::KeyU,
        PhysicalKey::KeyV => Code::KeyV,
        PhysicalKey::KeyW => Code::KeyW,
        PhysicalKey::KeyX => Code::KeyX,
        PhysicalKey::KeyY => Code::KeyY,
        PhysicalKey::KeyZ => Code::KeyZ,
        PhysicalKey::Minus => Code::Minus,
        PhysicalKey::Period => Code::Period,
        PhysicalKey::Quote => Code::Quote,
        PhysicalKey::Semicolon => Code::Semicolon,
        PhysicalKey::Slash => Code::Slash,
        PhysicalKey::Backspace => Code::Backspace,
        PhysicalKey::CapsLock => Code::CapsLock,
        PhysicalKey::ContextMenu => Code::ContextMenu,
        PhysicalKey::Enter => Code::Enter,
        PhysicalKey::Space => Code::Space,
        PhysicalKey::Tab => Code::Tab,
        PhysicalKey::Convert => Code::Convert,
        PhysicalKey::KanaMode => Code::KanaMode,
        PhysicalKey::Lang1 => Code::Lang1,
        PhysicalKey::Lang2 => Code::Lang2,
        PhysicalKey::Lang3 => Code::Lang3,
        PhysicalKey::Lang4 => Code::Lang4,
        PhysicalKey::Lang5 => Code::Lang5,
        PhysicalKey::NonConvert => Code::NonConvert,
        PhysicalKey::Delete => Code::Delete,
        PhysicalKey::End => Code::End,
        PhysicalKey::Help => Code::Help,
        PhysicalKey::Home => Code::Home,
        PhysicalKey::Insert => Code::Insert,
        PhysicalKey::PageDown => Code::PageDown,
        PhysicalKey::PageUp => Code::PageUp,
        PhysicalKey::ArrowDown => Code::ArrowDown,
        PhysicalKey::ArrowLeft => Code::ArrowLeft,
        PhysicalKey::ArrowRight => Code::ArrowRight,
        PhysicalKey::ArrowUp => Code::ArrowUp,
        PhysicalKey::NumLock => Code::NumLock,
        PhysicalKey::Numpad0 => Code::Numpad0,
        PhysicalKey::Numpad1 => Code::Numpad1,
        PhysicalKey::Numpad2 => Code::Numpad2,
        PhysicalKey::Numpad3 => Code::Numpad3,
        PhysicalKey::Numpad4 => Code::Numpad4,
        PhysicalKey::Numpad5 => Code::Numpad5,
        PhysicalKey::Numpad6 => Code::Numpad6,
        PhysicalKey::Numpad7 => Code::Numpad7,
        PhysicalKey::Numpad8 => Code::Numpad8,
        PhysicalKey::Numpad9 => Code::Numpad9,
        PhysicalKey::NumpadAdd => Code::NumpadAdd,
        PhysicalKey::NumpadBackspace => Code::NumpadBackspace,
        PhysicalKey::NumpadClear => Code::NumpadClear,
        PhysicalKey::NumpadClearEntry => Code::NumpadClearEntry,
        PhysicalKey::NumpadComma => Code::NumpadComma,
        PhysicalKey::NumpadDecimal => Code::NumpadDecimal,
        PhysicalKey::NumpadDivide => Code::NumpadDivide,
        PhysicalKey::NumpadEnter => Code::NumpadEnter,
        PhysicalKey::NumpadEqual => Code::NumpadEqual,
        PhysicalKey::NumpadHash => Code::NumpadHash,
        PhysicalKey::NumpadMemoryAdd => Code::NumpadMemoryAdd,
        PhysicalKey::NumpadMemoryClear => Code::NumpadMemoryClear,
        PhysicalKey::NumpadMemoryRecall => Code::NumpadMemoryRecall,
        PhysicalKey::NumpadMemoryStore => Code::NumpadMemoryStore,
        PhysicalKey::NumpadMemorySubtract => Code::NumpadMemorySubtract,
        PhysicalKey::NumpadMultiply => Code::NumpadMultiply,
        PhysicalKey::NumpadParenLeft => Code::NumpadParenLeft,
        PhysicalKey::NumpadParenRight => Code::NumpadParenRight,
        PhysicalKey::NumpadStar => Code::NumpadStar,
        PhysicalKey::NumpadSubtract => Code::NumpadSubtract,
        PhysicalKey::Escape => Code::Escape,
        PhysicalKey::Fn => Code::Fn,
        PhysicalKey::FnLock => Code::FnLock,
        PhysicalKey::PrintScreen => Code::PrintScreen,
        PhysicalKey::ScrollLock => Code::ScrollLock,
        PhysicalKey::Pause => Code::Pause,
        PhysicalKey::BrowserBack => Code::BrowserBack,
        PhysicalKey::BrowserFavorites => Code::BrowserFavorites,
        PhysicalKey::BrowserForward => Code::BrowserForward,
        PhysicalKey::BrowserHome => Code::BrowserHome,
        PhysicalKey::BrowserRefresh => Code::BrowserRefresh,
        PhysicalKey::BrowserSearch => Code::BrowserSearch,
        PhysicalKey::BrowserStop => Code::BrowserStop,
        PhysicalKey::Eject => Code::Eject,
        PhysicalKey::LaunchApp1 => Code::LaunchApp1,
        PhysicalKey::LaunchApp2 => Code::LaunchApp2,
        PhysicalKey::LaunchMail => Code::LaunchMail,
        PhysicalKey::MediaPlayPause => Code::MediaPlayPause,
        PhysicalKey::MediaSelect => Code::MediaSelect,
        PhysicalKey::MediaStop => Code::MediaStop,
        PhysicalKey::MediaTrackNext => Code::MediaTrackNext,
        PhysicalKey::MediaTrackPrevious => Code::MediaTrackPrevious,
        PhysicalKey::Power => Code::Power,
        PhysicalKey::Sleep => Code::Sleep,
        PhysicalKey::AudioVolumeDown => Code::AudioVolumeDown,
        PhysicalKey::AudioVolumeMute => Code::AudioVolumeMute,
        PhysicalKey::AudioVolumeUp => Code::AudioVolumeUp,
        PhysicalKey::WakeUp => Code::WakeUp,
        PhysicalKey::Abort => Code::Abort,
        PhysicalKey::Resume => Code::Resume,
        PhysicalKey::Suspend => Code::Suspend,
        PhysicalKey::Again => Code::Again,
        PhysicalKey::Copy => Code::Copy,
        PhysicalKey::Cut => Code::Cut,
        PhysicalKey::Find => Code::Find,
        PhysicalKey::Open => Code::Open,
        PhysicalKey::Paste => Code::Paste,
        PhysicalKey::Props => Code::Props,
        PhysicalKey::Select => Code::Select,
        PhysicalKey::Undo => Code::Undo,
        PhysicalKey::Hiragana => Code::Hiragana,
        PhysicalKey::Katakana => Code::Katakana,
        PhysicalKey::F1 => Code::F1,
        PhysicalKey::F2 => Code::F2,
        PhysicalKey::F3 => Code::F3,
        PhysicalKey::F4 => Code::F4,
        PhysicalKey::F5 => Code::F5,
        PhysicalKey::F6 => Code::F6,
        PhysicalKey::F7 => Code::F7,
        PhysicalKey::F8 => Code::F8,
        PhysicalKey::F9 => Code::F9,
        PhysicalKey::F10 => Code::F10,
        PhysicalKey::F11 => Code::F11,
        PhysicalKey::F12 => Code::F12,
        PhysicalKey::F13 => Code::F13,
        PhysicalKey::F14 => Code::F14,
        PhysicalKey::F15 => Code::F15,
        PhysicalKey::F16 => Code::F16,
        PhysicalKey::F17 => Code::F17,
        PhysicalKey::F18 => Code::F18,
        PhysicalKey::F19 => Code::F19,
        PhysicalKey::F20 => Code::F20,
        PhysicalKey::F21 => Code::F21,
        PhysicalKey::F22 => Code::F22,
        PhysicalKey::F23 => Code::F23,
        PhysicalKey::F24 => Code::F24,
        PhysicalKey::F25 => Code::F25,
        PhysicalKey::F26 => Code::F26,
        PhysicalKey::F27 => Code::F27,
        PhysicalKey::F28 => Code::F28,
        PhysicalKey::F29 => Code::F29,
        PhysicalKey::F30 => Code::F30,
        PhysicalKey::F31 => Code::F31,
        PhysicalKey::F32 => Code::F32,
        PhysicalKey::F33 => Code::F33,
        PhysicalKey::F34 => Code::F34,
        PhysicalKey::F35 => Code::F35,
    };

    HotKey::new(Some(modifiers), code)
}
