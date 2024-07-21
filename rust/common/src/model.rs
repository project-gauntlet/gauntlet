use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use gix_url::Scheme;
use gix_url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(Arc<str>);

impl PluginId {
    pub fn from_string(plugin_id: impl ToString) -> Self {
        PluginId(plugin_id.to_string().into())
    }

    fn try_to_url(&self) -> anyhow::Result<Url> {
        let url = self.to_string();
        let url: &str = url.as_ref();
        let url = gix_url::parse(url.try_into()?)?;
        Ok(url)
    }

    pub fn try_to_git_url(&self) -> anyhow::Result<String> {
        let url = self.try_to_url()?;

        Ok(url.to_bstring().to_string())
    }

    pub fn try_to_path(&self) -> anyhow::Result<PathBuf> {
        let url = self.try_to_url()?;

        if url.scheme != Scheme::File {
            return Err(anyhow!("plugin id is expected to point to local file"))
        }

        let plugin_dir: String = url.path.try_into()?;
        let plugin_dir = PathBuf::from(plugin_dir);
        Ok(plugin_dir)
    }
}

impl ToString for PluginId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntrypointId(Arc<str>);

impl EntrypointId {
    pub fn from_string(entrypoint_id: impl ToString) -> Self {
        EntrypointId(entrypoint_id.to_string().into())
    }
}

impl ToString for EntrypointId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    InProgress,
    Done,
    Failed {
        message: String
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UiRenderLocation {
    InlineView,
    View
}

#[derive(Debug, Clone)]
pub struct PhysicalShortcut {
    pub physical_key: PhysicalKey,
    pub modifier_shift: bool,
    pub modifier_control: bool,
    pub modifier_alt: bool,
    pub modifier_meta: bool,
}

#[derive(Debug, Clone)]
pub struct LocalSaveData {
    pub stdout_file_path: String,
    pub stderr_file_path: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub entrypoint_icon: Option<String>,
    pub entrypoint_type: SearchResultEntrypointType,
}

#[derive(Debug, Clone)]
pub enum SearchResultEntrypointType {
    Command,
    View,
    GeneratedCommand,
}

#[derive(Debug)]
pub enum UiResponseData {
    Nothing,
}

#[derive(Debug)]
pub enum UiRequestData {
    ShowWindow,
    ClearInlineView {
        plugin_id: PluginId
    },
    ReplaceView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: UiWidget,
    },
    ShowPreferenceRequiredView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    ShowPluginErrorView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    },
    RequestSearchResultUpdate
}

#[derive(Debug)]
pub enum BackendResponseData {
    Nothing,
    Search {
        results: Vec<SearchResult>
    },
    RequestViewRender {
        shortcuts: HashMap<String, PhysicalShortcut>
    },
}

#[derive(Debug)]
pub enum BackendRequestData {
    Search {
        text: String,
        render_inline_view: bool
    },
    RequestViewRender {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    },
    RequestViewClose {
        plugin_id: PluginId,
    },
    RequestRunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    },
    RequestRunGeneratedCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    },
    SendViewEvent {
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>
    },
    SendKeyboardEvent {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    },
    SendOpenEvent {
        plugin_id: PluginId,
        href: String
    },
    OpenSettingsWindow,
    OpenSettingsWindowPreferences {
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>
    },
}

#[derive(Debug, Clone)]
pub struct UiWidget {
    pub widget_id: UiWidgetId,
    pub widget_type: String,
    pub widget_properties: HashMap<String, UiPropertyValue>,
    pub widget_children: Vec<UiWidget>,
}

#[derive(Debug, Clone)]
pub enum UiPropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    Bytes(bytes::Bytes),
    Object(HashMap<String, UiPropertyValue>),
    Undefined,
}

impl UiPropertyValue {
    pub fn as_string(&self) -> Option<&str> {
        if let UiPropertyValue::String(val) = self {
            Some(val)
        } else {
            None
        }
    }
    pub fn as_number(&self) -> Option<&f64> {
        if let UiPropertyValue::Number(val) = self {
            Some(val)
        } else {
            None
        }
    }
    pub fn as_bool(&self) -> Option<&bool> {
        if let UiPropertyValue::Bool(val) = self {
            Some(val)
        } else {
            None
        }
    }
    pub fn as_bytes(&self) -> Option<&[u8]> {
        if let UiPropertyValue::Bytes(val) = self {
            Some(val)
        } else {
            None
        }
    }
    pub fn as_object<T: UiPropertyValueToStruct>(&self) -> Option<T> {
        if let UiPropertyValue::Object(val) = self {
            Some(UiPropertyValueToStruct::convert(val).expect("invalid object"))
        } else {
            None
        }
    }
    pub fn as_union<T: UiPropertyValueToEnum>(&self) -> anyhow::Result<T> {
        UiPropertyValueToEnum::convert(self)
    }
}

pub trait UiPropertyValueToStruct {
    fn convert(value: &HashMap<String, UiPropertyValue>) -> anyhow::Result<Self> where Self: Sized;
}

pub trait UiPropertyValueToEnum {
    fn convert(value: &UiPropertyValue) -> anyhow::Result<Self> where Self: Sized;
}

pub type UiWidgetId = u32;

#[derive(Debug, Clone)]
pub struct SettingsEntrypoint {
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub entrypoint_description: String,
    pub entrypoint_type: SettingsEntrypointType,
    pub enabled: bool,
    pub preferences: HashMap<String, PluginPreference>,
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Clone)]
pub struct SettingsPlugin {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub plugin_description: String,
    pub enabled: bool,
    pub entrypoints: HashMap<EntrypointId, SettingsEntrypoint>,
    pub preferences: HashMap<String, PluginPreference>,
    pub preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Clone)]
pub enum SettingsEntrypointType {
    Command,
    View,
    InlineView,
    CommandGenerator,
}

#[derive(Debug, Clone)]
pub enum PluginPreferenceUserData {
    Number {
        value: Option<f64>,
    },
    String {
        value: Option<String>,
    },
    Enum {
        value: Option<String>,
    },
    Bool {
        value: Option<bool>,
    },
    ListOfStrings {
        value: Option<Vec<String>>,
    },
    ListOfNumbers {
        value: Option<Vec<f64>>,
    },
    ListOfEnums {
        value: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone)]
pub enum PluginPreference {
    Number {
        default: Option<f64>,
        description: String,
    },
    String {
        default: Option<String>,
        description: String,
    },
    Enum {
        default: Option<String>,
        description: String,
        enum_values: Vec<PreferenceEnumValue>,
    },
    Bool {
        default: Option<bool>,
        description: String,
    },
    ListOfStrings {
        default: Option<Vec<String>>,
        description: String,
    },
    ListOfNumbers {
        default: Option<Vec<f64>>,
        description: String,
    },
    ListOfEnums {
        default: Option<Vec<String>>,
        enum_values: Vec<PreferenceEnumValue>,
        description: String,
    },
}

#[derive(Debug, Clone)]
pub struct PreferenceEnumValue {
    pub label: String,
    pub value: String,
}


// copy of iced (currently fork) PhysicalKey but without modifiers
#[derive(Debug, Clone)]
pub enum PhysicalKey {
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    Backspace,
    CapsLock,
    ContextMenu,
    Enter,
    Space,
    Tab,
    Convert,
    KanaMode,
    Lang1,
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    Escape,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Abort,
    Resume,
    Suspend,
    Again,
    Copy,
    Cut,
    Find,
    Open,
    Paste,
    Props,
    Select,
    Undo,
    Hiragana,
    Katakana,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
}

impl PhysicalKey {
    pub fn from_value(key: String) -> PhysicalKey {
        match key.as_str() {
            "Backquote" => PhysicalKey::Backquote,
            "Backslash" => PhysicalKey::Backslash,
            "BracketLeft" => PhysicalKey::BracketLeft,
            "BracketRight" => PhysicalKey::BracketRight,
            "Comma" => PhysicalKey::Comma,
            "Digit0" => PhysicalKey::Digit0,
            "Digit1" => PhysicalKey::Digit1,
            "Digit2" => PhysicalKey::Digit2,
            "Digit3" => PhysicalKey::Digit3,
            "Digit4" => PhysicalKey::Digit4,
            "Digit5" => PhysicalKey::Digit5,
            "Digit6" => PhysicalKey::Digit6,
            "Digit7" => PhysicalKey::Digit7,
            "Digit8" => PhysicalKey::Digit8,
            "Digit9" => PhysicalKey::Digit9,
            "Equal" => PhysicalKey::Equal,
            "IntlBackslash" => PhysicalKey::IntlBackslash,
            "IntlRo" => PhysicalKey::IntlRo,
            "IntlYen" => PhysicalKey::IntlYen,
            "KeyA" => PhysicalKey::KeyA,
            "KeyB" => PhysicalKey::KeyB,
            "KeyC" => PhysicalKey::KeyC,
            "KeyD" => PhysicalKey::KeyD,
            "KeyE" => PhysicalKey::KeyE,
            "KeyF" => PhysicalKey::KeyF,
            "KeyG" => PhysicalKey::KeyG,
            "KeyH" => PhysicalKey::KeyH,
            "KeyI" => PhysicalKey::KeyI,
            "KeyJ" => PhysicalKey::KeyJ,
            "KeyK" => PhysicalKey::KeyK,
            "KeyL" => PhysicalKey::KeyL,
            "KeyM" => PhysicalKey::KeyM,
            "KeyN" => PhysicalKey::KeyN,
            "KeyO" => PhysicalKey::KeyO,
            "KeyP" => PhysicalKey::KeyP,
            "KeyQ" => PhysicalKey::KeyQ,
            "KeyR" => PhysicalKey::KeyR,
            "KeyS" => PhysicalKey::KeyS,
            "KeyT" => PhysicalKey::KeyT,
            "KeyU" => PhysicalKey::KeyU,
            "KeyV" => PhysicalKey::KeyV,
            "KeyW" => PhysicalKey::KeyW,
            "KeyX" => PhysicalKey::KeyX,
            "KeyY" => PhysicalKey::KeyY,
            "KeyZ" => PhysicalKey::KeyZ,
            "Minus" => PhysicalKey::Minus,
            "Period" => PhysicalKey::Period,
            "Quote" => PhysicalKey::Quote,
            "Semicolon" => PhysicalKey::Semicolon,
            "Slash" => PhysicalKey::Slash,
            "Backspace" => PhysicalKey::Backspace,
            "CapsLock" => PhysicalKey::CapsLock,
            "ContextMenu" => PhysicalKey::ContextMenu,
            "Enter" => PhysicalKey::Enter,
            "Space" => PhysicalKey::Space,
            "Tab" => PhysicalKey::Tab,
            "Convert" => PhysicalKey::Convert,
            "KanaMode" => PhysicalKey::KanaMode,
            "Lang1" => PhysicalKey::Lang1,
            "Lang2" => PhysicalKey::Lang2,
            "Lang3" => PhysicalKey::Lang3,
            "Lang4" => PhysicalKey::Lang4,
            "Lang5" => PhysicalKey::Lang5,
            "NonConvert" => PhysicalKey::NonConvert,
            "Delete" => PhysicalKey::Delete,
            "End" => PhysicalKey::End,
            "Help" => PhysicalKey::Help,
            "Home" => PhysicalKey::Home,
            "Insert" => PhysicalKey::Insert,
            "PageDown" => PhysicalKey::PageDown,
            "PageUp" => PhysicalKey::PageUp,
            "ArrowDown" => PhysicalKey::ArrowDown,
            "ArrowLeft" => PhysicalKey::ArrowLeft,
            "ArrowRight" => PhysicalKey::ArrowRight,
            "ArrowUp" => PhysicalKey::ArrowUp,
            "NumLock" => PhysicalKey::NumLock,
            "Numpad0" => PhysicalKey::Numpad0,
            "Numpad1" => PhysicalKey::Numpad1,
            "Numpad2" => PhysicalKey::Numpad2,
            "Numpad3" => PhysicalKey::Numpad3,
            "Numpad4" => PhysicalKey::Numpad4,
            "Numpad5" => PhysicalKey::Numpad5,
            "Numpad6" => PhysicalKey::Numpad6,
            "Numpad7" => PhysicalKey::Numpad7,
            "Numpad8" => PhysicalKey::Numpad8,
            "Numpad9" => PhysicalKey::Numpad9,
            "NumpadAdd" => PhysicalKey::NumpadAdd,
            "NumpadBackspace" => PhysicalKey::NumpadBackspace,
            "NumpadClear" => PhysicalKey::NumpadClear,
            "NumpadClearEntry" => PhysicalKey::NumpadClearEntry,
            "NumpadComma" => PhysicalKey::NumpadComma,
            "NumpadDecimal" => PhysicalKey::NumpadDecimal,
            "NumpadDivide" => PhysicalKey::NumpadDivide,
            "NumpadEnter" => PhysicalKey::NumpadEnter,
            "NumpadEqual" => PhysicalKey::NumpadEqual,
            "NumpadHash" => PhysicalKey::NumpadHash,
            "NumpadMemoryAdd" => PhysicalKey::NumpadMemoryAdd,
            "NumpadMemoryClear" => PhysicalKey::NumpadMemoryClear,
            "NumpadMemoryRecall" => PhysicalKey::NumpadMemoryRecall,
            "NumpadMemoryStore" => PhysicalKey::NumpadMemoryStore,
            "NumpadMemorySubtract" => PhysicalKey::NumpadMemorySubtract,
            "NumpadMultiply" => PhysicalKey::NumpadMultiply,
            "NumpadParenLeft" => PhysicalKey::NumpadParenLeft,
            "NumpadParenRight" => PhysicalKey::NumpadParenRight,
            "NumpadStar" => PhysicalKey::NumpadStar,
            "NumpadSubtract" => PhysicalKey::NumpadSubtract,
            "Escape" => PhysicalKey::Escape,
            "Fn" => PhysicalKey::Fn,
            "FnLock" => PhysicalKey::FnLock,
            "PrintScreen" => PhysicalKey::PrintScreen,
            "ScrollLock" => PhysicalKey::ScrollLock,
            "Pause" => PhysicalKey::Pause,
            "BrowserBack" => PhysicalKey::BrowserBack,
            "BrowserFavorites" => PhysicalKey::BrowserFavorites,
            "BrowserForward" => PhysicalKey::BrowserForward,
            "BrowserHome" => PhysicalKey::BrowserHome,
            "BrowserRefresh" => PhysicalKey::BrowserRefresh,
            "BrowserSearch" => PhysicalKey::BrowserSearch,
            "BrowserStop" => PhysicalKey::BrowserStop,
            "Eject" => PhysicalKey::Eject,
            "LaunchApp1" => PhysicalKey::LaunchApp1,
            "LaunchApp2" => PhysicalKey::LaunchApp2,
            "LaunchMail" => PhysicalKey::LaunchMail,
            "MediaPlayPause" => PhysicalKey::MediaPlayPause,
            "MediaSelect" => PhysicalKey::MediaSelect,
            "MediaStop" => PhysicalKey::MediaStop,
            "MediaTrackNext" => PhysicalKey::MediaTrackNext,
            "MediaTrackPrevious" => PhysicalKey::MediaTrackPrevious,
            "Power" => PhysicalKey::Power,
            "Sleep" => PhysicalKey::Sleep,
            "AudioVolumeDown" => PhysicalKey::AudioVolumeDown,
            "AudioVolumeMute" => PhysicalKey::AudioVolumeMute,
            "AudioVolumeUp" => PhysicalKey::AudioVolumeUp,
            "WakeUp" => PhysicalKey::WakeUp,
            "Abort" => PhysicalKey::Abort,
            "Resume" => PhysicalKey::Resume,
            "Suspend" => PhysicalKey::Suspend,
            "Again" => PhysicalKey::Again,
            "Copy" => PhysicalKey::Copy,
            "Cut" => PhysicalKey::Cut,
            "Find" => PhysicalKey::Find,
            "Open" => PhysicalKey::Open,
            "Paste" => PhysicalKey::Paste,
            "Props" => PhysicalKey::Props,
            "Select" => PhysicalKey::Select,
            "Undo" => PhysicalKey::Undo,
            "Hiragana" => PhysicalKey::Hiragana,
            "Katakana" => PhysicalKey::Katakana,
            "F1" => PhysicalKey::F1,
            "F2" => PhysicalKey::F2,
            "F3" => PhysicalKey::F3,
            "F4" => PhysicalKey::F4,
            "F5" => PhysicalKey::F5,
            "F6" => PhysicalKey::F6,
            "F7" => PhysicalKey::F7,
            "F8" => PhysicalKey::F8,
            "F9" => PhysicalKey::F9,
            "F10" => PhysicalKey::F10,
            "F11" => PhysicalKey::F11,
            "F12" => PhysicalKey::F12,
            "F13" => PhysicalKey::F13,
            "F14" => PhysicalKey::F14,
            "F15" => PhysicalKey::F15,
            "F16" => PhysicalKey::F16,
            "F17" => PhysicalKey::F17,
            "F18" => PhysicalKey::F18,
            "F19" => PhysicalKey::F19,
            "F20" => PhysicalKey::F20,
            "F21" => PhysicalKey::F21,
            "F22" => PhysicalKey::F22,
            "F23" => PhysicalKey::F23,
            "F24" => PhysicalKey::F24,
            "F25" => PhysicalKey::F25,
            "F26" => PhysicalKey::F26,
            "F27" => PhysicalKey::F27,
            "F28" => PhysicalKey::F28,
            "F29" => PhysicalKey::F29,
            "F30" => PhysicalKey::F30,
            "F31" => PhysicalKey::F31,
            "F32" => PhysicalKey::F32,
            "F33" => PhysicalKey::F33,
            "F34" => PhysicalKey::F34,
            "F35" => PhysicalKey::F35,
            _ => {
                panic!("unknown key: {}", key)
            }
        }
    }

    pub fn to_value(&self) -> String {
        match self {
            PhysicalKey::Backquote => "Backquote",
            PhysicalKey::Backslash => "Backslash",
            PhysicalKey::BracketLeft => "BracketLeft",
            PhysicalKey::BracketRight => "BracketRight",
            PhysicalKey::Comma => "Comma",
            PhysicalKey::Digit0 => "Digit0",
            PhysicalKey::Digit1 => "Digit1",
            PhysicalKey::Digit2 => "Digit2",
            PhysicalKey::Digit3 => "Digit3",
            PhysicalKey::Digit4 => "Digit4",
            PhysicalKey::Digit5 => "Digit5",
            PhysicalKey::Digit6 => "Digit6",
            PhysicalKey::Digit7 => "Digit7",
            PhysicalKey::Digit8 => "Digit8",
            PhysicalKey::Digit9 => "Digit9",
            PhysicalKey::Equal => "Equal",
            PhysicalKey::IntlBackslash => "IntlBackslash",
            PhysicalKey::IntlRo => "IntlRo",
            PhysicalKey::IntlYen => "IntlYen",
            PhysicalKey::KeyA => "KeyA",
            PhysicalKey::KeyB => "KeyB",
            PhysicalKey::KeyC => "KeyC",
            PhysicalKey::KeyD => "KeyD",
            PhysicalKey::KeyE => "KeyE",
            PhysicalKey::KeyF => "KeyF",
            PhysicalKey::KeyG => "KeyG",
            PhysicalKey::KeyH => "KeyH",
            PhysicalKey::KeyI => "KeyI",
            PhysicalKey::KeyJ => "KeyJ",
            PhysicalKey::KeyK => "KeyK",
            PhysicalKey::KeyL => "KeyL",
            PhysicalKey::KeyM => "KeyM",
            PhysicalKey::KeyN => "KeyN",
            PhysicalKey::KeyO => "KeyO",
            PhysicalKey::KeyP => "KeyP",
            PhysicalKey::KeyQ => "KeyQ",
            PhysicalKey::KeyR => "KeyR",
            PhysicalKey::KeyS => "KeyS",
            PhysicalKey::KeyT => "KeyT",
            PhysicalKey::KeyU => "KeyU",
            PhysicalKey::KeyV => "KeyV",
            PhysicalKey::KeyW => "KeyW",
            PhysicalKey::KeyX => "KeyX",
            PhysicalKey::KeyY => "KeyY",
            PhysicalKey::KeyZ => "KeyZ",
            PhysicalKey::Minus => "Minus",
            PhysicalKey::Period => "Period",
            PhysicalKey::Quote => "Quote",
            PhysicalKey::Semicolon => "Semicolon",
            PhysicalKey::Slash => "Slash",
            PhysicalKey::Backspace => "Backspace",
            PhysicalKey::CapsLock => "CapsLock",
            PhysicalKey::ContextMenu => "ContextMenu",
            PhysicalKey::Enter => "Enter",
            PhysicalKey::Space => "Space",
            PhysicalKey::Tab => "Tab",
            PhysicalKey::Convert => "Convert",
            PhysicalKey::KanaMode => "KanaMode",
            PhysicalKey::Lang1 => "Lang1",
            PhysicalKey::Lang2 => "Lang2",
            PhysicalKey::Lang3 => "Lang3",
            PhysicalKey::Lang4 => "Lang4",
            PhysicalKey::Lang5 => "Lang5",
            PhysicalKey::NonConvert => "NonConvert",
            PhysicalKey::Delete => "Delete",
            PhysicalKey::End => "End",
            PhysicalKey::Help => "Help",
            PhysicalKey::Home => "Home",
            PhysicalKey::Insert => "Insert",
            PhysicalKey::PageDown => "PageDown",
            PhysicalKey::PageUp => "PageUp",
            PhysicalKey::ArrowDown => "ArrowDown",
            PhysicalKey::ArrowLeft => "ArrowLeft",
            PhysicalKey::ArrowRight => "ArrowRight",
            PhysicalKey::ArrowUp => "ArrowUp",
            PhysicalKey::NumLock => "NumLock",
            PhysicalKey::Numpad0 => "Numpad0",
            PhysicalKey::Numpad1 => "Numpad1",
            PhysicalKey::Numpad2 => "Numpad2",
            PhysicalKey::Numpad3 => "Numpad3",
            PhysicalKey::Numpad4 => "Numpad4",
            PhysicalKey::Numpad5 => "Numpad5",
            PhysicalKey::Numpad6 => "Numpad6",
            PhysicalKey::Numpad7 => "Numpad7",
            PhysicalKey::Numpad8 => "Numpad8",
            PhysicalKey::Numpad9 => "Numpad9",
            PhysicalKey::NumpadAdd => "NumpadAdd",
            PhysicalKey::NumpadBackspace => "NumpadBackspace",
            PhysicalKey::NumpadClear => "NumpadClear",
            PhysicalKey::NumpadClearEntry => "NumpadClearEntry",
            PhysicalKey::NumpadComma => "NumpadComma",
            PhysicalKey::NumpadDecimal => "NumpadDecimal",
            PhysicalKey::NumpadDivide => "NumpadDivide",
            PhysicalKey::NumpadEnter => "NumpadEnter",
            PhysicalKey::NumpadEqual => "NumpadEqual",
            PhysicalKey::NumpadHash => "NumpadHash",
            PhysicalKey::NumpadMemoryAdd => "NumpadMemoryAdd",
            PhysicalKey::NumpadMemoryClear => "NumpadMemoryClear",
            PhysicalKey::NumpadMemoryRecall => "NumpadMemoryRecall",
            PhysicalKey::NumpadMemoryStore => "NumpadMemoryStore",
            PhysicalKey::NumpadMemorySubtract => "NumpadMemorySubtract",
            PhysicalKey::NumpadMultiply => "NumpadMultiply",
            PhysicalKey::NumpadParenLeft => "NumpadParenLeft",
            PhysicalKey::NumpadParenRight => "NumpadParenRight",
            PhysicalKey::NumpadStar => "NumpadStar",
            PhysicalKey::NumpadSubtract => "NumpadSubtract",
            PhysicalKey::Escape => "Escape",
            PhysicalKey::Fn => "Fn",
            PhysicalKey::FnLock => "FnLock",
            PhysicalKey::PrintScreen => "PrintScreen",
            PhysicalKey::ScrollLock => "ScrollLock",
            PhysicalKey::Pause => "Pause",
            PhysicalKey::BrowserBack => "BrowserBack",
            PhysicalKey::BrowserFavorites => "BrowserFavorites",
            PhysicalKey::BrowserForward => "BrowserForward",
            PhysicalKey::BrowserHome => "BrowserHome",
            PhysicalKey::BrowserRefresh => "BrowserRefresh",
            PhysicalKey::BrowserSearch => "BrowserSearch",
            PhysicalKey::BrowserStop => "BrowserStop",
            PhysicalKey::Eject => "Eject",
            PhysicalKey::LaunchApp1 => "LaunchApp1",
            PhysicalKey::LaunchApp2 => "LaunchApp2",
            PhysicalKey::LaunchMail => "LaunchMail",
            PhysicalKey::MediaPlayPause => "MediaPlayPause",
            PhysicalKey::MediaSelect => "MediaSelect",
            PhysicalKey::MediaStop => "MediaStop",
            PhysicalKey::MediaTrackNext => "MediaTrackNext",
            PhysicalKey::MediaTrackPrevious => "MediaTrackPrevious",
            PhysicalKey::Power => "Power",
            PhysicalKey::Sleep => "Sleep",
            PhysicalKey::AudioVolumeDown => "AudioVolumeDown",
            PhysicalKey::AudioVolumeMute => "AudioVolumeMute",
            PhysicalKey::AudioVolumeUp => "AudioVolumeUp",
            PhysicalKey::WakeUp => "WakeUp",
            PhysicalKey::Abort => "Abort",
            PhysicalKey::Resume => "Resume",
            PhysicalKey::Suspend => "Suspend",
            PhysicalKey::Again => "Again",
            PhysicalKey::Copy => "Copy",
            PhysicalKey::Cut => "Cut",
            PhysicalKey::Find => "Find",
            PhysicalKey::Open => "Open",
            PhysicalKey::Paste => "Paste",
            PhysicalKey::Props => "Props",
            PhysicalKey::Select => "Select",
            PhysicalKey::Undo => "Undo",
            PhysicalKey::Hiragana => "Hiragana",
            PhysicalKey::Katakana => "Katakana",
            PhysicalKey::F1 => "F1",
            PhysicalKey::F2 => "F2",
            PhysicalKey::F3 => "F3",
            PhysicalKey::F4 => "F4",
            PhysicalKey::F5 => "F5",
            PhysicalKey::F6 => "F6",
            PhysicalKey::F7 => "F7",
            PhysicalKey::F8 => "F8",
            PhysicalKey::F9 => "F9",
            PhysicalKey::F10 => "F10",
            PhysicalKey::F11 => "F11",
            PhysicalKey::F12 => "F12",
            PhysicalKey::F13 => "F13",
            PhysicalKey::F14 => "F14",
            PhysicalKey::F15 => "F15",
            PhysicalKey::F16 => "F16",
            PhysicalKey::F17 => "F17",
            PhysicalKey::F18 => "F18",
            PhysicalKey::F19 => "F19",
            PhysicalKey::F20 => "F20",
            PhysicalKey::F21 => "F21",
            PhysicalKey::F22 => "F22",
            PhysicalKey::F23 => "F23",
            PhysicalKey::F24 => "F24",
            PhysicalKey::F25 => "F25",
            PhysicalKey::F26 => "F26",
            PhysicalKey::F27 => "F27",
            PhysicalKey::F28 => "F28",
            PhysicalKey::F29 => "F29",
            PhysicalKey::F30 => "F30",
            PhysicalKey::F31 => "F31",
            PhysicalKey::F32 => "F32",
            PhysicalKey::F33 => "F33",
            PhysicalKey::F34 => "F34",
            PhysicalKey::F35 => "F35",
        }.to_string()
    }
}

