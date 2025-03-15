use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::model::ActionShortcutKey;


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Manifest structure for a plugin.")]
pub struct PluginManifest {
    #[serde(rename = "$schema")]
    schema: Option<String>,

    #[schemars(description = "Metadata about the plugin.")]
    pub gauntlet: PluginManifestMetadata,

    #[schemars(description = "Entrypoints for the plugin.")]
    pub entrypoint: Vec<PluginManifestEntrypoint>,

    #[serde(default)]
    #[schemars(description = "List of supported operating systems.")]
    pub supported_system: Vec<PluginManifestSupportedSystem>,

    #[serde(default)]
    #[schemars(description = "Permissions required by the plugin.")]
    pub permissions: PluginManifestPermissions,

    #[serde(default)]
    #[schemars(description = "Preferences that can be configured by the user in the settings view.")]
    pub preferences: Vec<PluginManifestPreference>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Metadata for the plugin manifest.")]
pub struct PluginManifestMetadata {
    #[schemars(description = "Name of the plugin.")]
    pub name: String,

    #[schemars(description = "Description of the plugin.")]
    pub description: String,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Action that can be performed by the plugin.")]
pub struct PluginManifestAction {
    #[schemars(description = "Unique identifier for the action.")]
    pub id: String,
    #[schemars(description = "Description of what the action does.")]
    pub description: String,
    #[schemars(description = "Keyboard shortcut to trigger the action.")]
    pub shortcut: PluginManifestActionShortcut,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "An entrypoint for the plugin.")]
pub struct PluginManifestEntrypoint {
    pub id: String,
    pub name: String,
    pub description: String,

    #[allow(unused)] // Used during plugin build
    pub path: String,

    pub icon: Option<String>,

    #[serde(rename = "type")]
    pub entrypoint_type: PluginManifestEntrypointTypes,

    #[serde(default)]
    pub preferences: Vec<PluginManifestPreference>,

    #[serde(default)]
    pub actions: Vec<PluginManifestAction>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(description = "User-configurable preference options.")]
pub enum PluginManifestPreference {
    #[serde(rename = "number")]
    #[schemars(description = "A numeric preference.")]
    Number {
        id: String,
        name: String,
        default: Option<f64>,
        description: String,
    },

    #[serde(rename = "string")]
    #[schemars(description = "A string preference.")]
    String {
        id: String,
        name: String,
        default: Option<String>,
        description: String,
    },

    #[serde(rename = "enum")]
    #[schemars(description = "An enum preference with selectable values.")]
    Enum {
        id: String,
        name: String,
        default: Option<String>,
        description: String,
        enum_values: Vec<PluginManifestPreferenceEnumValue>,
    },

    #[serde(rename = "bool")]
    #[schemars(description = "A boolean preference.")]
    Bool {
        id: String,
        name: String,
        default: Option<bool>,
        description: String,
    },

    #[serde(rename = "list_of_strings")]
    #[schemars(description = "A list of strings preference.")]
    ListOfStrings {
        id: String,
        name: String,
        // default: Option<Vec<String>>,
        description: String,
    },

    #[serde(rename = "list_of_numbers")]
    #[schemars(description = "A list of numbers preference.")]
    ListOfNumbers {
        id: String,
        name: String,
        // default: Option<Vec<f64>>,
        description: String,
    },
    #[serde(rename = "list_of_enums")]
    #[schemars(description = "A list of enumerated preference values.")]
    ListOfEnums {
        id: String,
        name: String,
        // default: Option<Vec<String>>,
        enum_values: Vec<PluginManifestPreferenceEnumValue>,
        description: String,
    },
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "An enumerated preference value.")]
pub struct PluginManifestPreferenceEnumValue {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Types of plugin entrypoints.")]
pub enum PluginManifestEntrypointTypes {
    #[serde(rename = "command")]
    #[schemars(description = "A command entrypoint.")]
    Command,

    #[serde(rename = "view")]
    #[schemars(description = "A view-based entrypoint.")]
    View,

    #[serde(rename = "inline-view")]
    #[schemars(description = "An inline view entrypoint.")]
    InlineView,

    #[serde(rename = "entrypoint-generator")]
    #[schemars(description = "Generates new entrypoints dynamically.")]
    EntrypointGenerator,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Keyboard shortcut configuration for a plugin action.")]
pub struct PluginManifestActionShortcut {
    #[schemars(description = "The key to be pressed for this shortcut.")]
    pub key: PluginManifestActionShortcutKey,
    #[schemars(description = "The type of shortcut.")]
    pub kind: PluginManifestActionShortcutKind,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "The type of shortcut.")]
pub enum PluginManifestActionShortcutKind {
    #[serde(rename = "main")]
    #[schemars(description = "Main shortcut for the action (e.g. cmd).")]
    Main,
    #[serde(rename = "alternative")]
    #[schemars(description = "Alternative shortcut for the action (e.g. opt).")]
    Alternative,
}

// only stuff that is present on 60% keyboard
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub enum PluginManifestActionShortcutKey {
    #[serde(rename = "0")]
    Num0,
    #[serde(rename = "1")]
    Num1,
    #[serde(rename = "2")]
    Num2,
    #[serde(rename = "3")]
    Num3,
    #[serde(rename = "4")]
    Num4,
    #[serde(rename = "5")]
    Num5,
    #[serde(rename = "6")]
    Num6,
    #[serde(rename = "7")]
    Num7,
    #[serde(rename = "8")]
    Num8,
    #[serde(rename = "9")]
    Num9,

    #[serde(rename = "!")]
    Exclamation,
    #[serde(rename = "@")]
    AtSign,
    #[serde(rename = "#")]
    Hash,
    #[serde(rename = "$")]
    Dollar,
    #[serde(rename = "%")]
    Percent,
    #[serde(rename = "^")]
    Caret,
    #[serde(rename = "&")]
    Ampersand,
    #[serde(rename = "*")]
    Star,
    #[serde(rename = "(")]
    LeftParenthesis,
    #[serde(rename = ")")]
    RightParenthesis,

    #[serde(rename = "a")]
    LowerA,
    #[serde(rename = "b")]
    LowerB,
    #[serde(rename = "c")]
    LowerC,
    #[serde(rename = "d")]
    LowerD,
    #[serde(rename = "e")]
    LowerE,
    #[serde(rename = "f")]
    LowerF,
    #[serde(rename = "g")]
    LowerG,
    #[serde(rename = "h")]
    LowerH,
    #[serde(rename = "i")]
    LowerI,
    #[serde(rename = "j")]
    LowerJ,
    #[serde(rename = "k")]
    LowerK,
    #[serde(rename = "l")]
    LowerL,
    #[serde(rename = "m")]
    LowerM,
    #[serde(rename = "n")]
    LowerN,
    #[serde(rename = "o")]
    LowerO,
    #[serde(rename = "p")]
    LowerP,
    #[serde(rename = "q")]
    LowerQ,
    #[serde(rename = "r")]
    LowerR,
    #[serde(rename = "s")]
    LowerS,
    #[serde(rename = "t")]
    LowerT,
    #[serde(rename = "u")]
    LowerU,
    #[serde(rename = "v")]
    LowerV,
    #[serde(rename = "w")]
    LowerW,
    #[serde(rename = "x")]
    LowerX,
    #[serde(rename = "y")]
    LowerY,
    #[serde(rename = "z")]
    LowerZ,

    #[serde(rename = "A")]
    UpperA,
    #[serde(rename = "B")]
    UpperB,
    #[serde(rename = "C")]
    UpperC,
    #[serde(rename = "D")]
    UpperD,
    #[serde(rename = "E")]
    UpperE,
    #[serde(rename = "F")]
    UpperF,
    #[serde(rename = "G")]
    UpperG,
    #[serde(rename = "H")]
    UpperH,
    #[serde(rename = "I")]
    UpperI,
    #[serde(rename = "J")]
    UpperJ,
    #[serde(rename = "K")]
    UpperK,
    #[serde(rename = "L")]
    UpperL,
    #[serde(rename = "M")]
    UpperM,
    #[serde(rename = "N")]
    UpperN,
    #[serde(rename = "O")]
    UpperO,
    #[serde(rename = "P")]
    UpperP,
    #[serde(rename = "Q")]
    UpperQ,
    #[serde(rename = "R")]
    UpperR,
    #[serde(rename = "S")]
    UpperS,
    #[serde(rename = "T")]
    UpperT,
    #[serde(rename = "U")]
    UpperU,
    #[serde(rename = "V")]
    UpperV,
    #[serde(rename = "W")]
    UpperW,
    #[serde(rename = "X")]
    UpperX,
    #[serde(rename = "Y")]
    UpperY,
    #[serde(rename = "Z")]
    UpperZ,

    #[serde(rename = "-")]
    Minus,
    #[serde(rename = "=")]
    Equals,
    #[serde(rename = ",")]
    Comma,
    #[serde(rename = ".")]
    Dot,
    #[serde(rename = "/")]
    Slash,
    #[serde(rename = "[")]
    OpenSquareBracket,
    #[serde(rename = "]")]
    CloseSquareBracket,
    #[serde(rename = ";")]
    Semicolon,
    #[serde(rename = "'")]
    Quote,
    #[serde(rename = "\\")]
    Backslash,

    #[serde(rename = "_")]
    Underscore,
    #[serde(rename = "+")]
    Plus,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "?")]
    QuestionMark,
    #[serde(rename = "{")]
    LeftBrace,
    #[serde(rename = "}")]
    RightBrace,
    #[serde(rename = ":")]
    Colon,
    #[serde(rename = "\"")]
    DoubleQuotes,
    #[serde(rename = "|")]
    Pipe,
}

impl PluginManifestActionShortcutKey {
    pub fn to_model(self) -> ActionShortcutKey {
        match self {
            PluginManifestActionShortcutKey::Num0 => ActionShortcutKey::Num0,
            PluginManifestActionShortcutKey::Num1 => ActionShortcutKey::Num1,
            PluginManifestActionShortcutKey::Num2 => ActionShortcutKey::Num2,
            PluginManifestActionShortcutKey::Num3 => ActionShortcutKey::Num3,
            PluginManifestActionShortcutKey::Num4 => ActionShortcutKey::Num4,
            PluginManifestActionShortcutKey::Num5 => ActionShortcutKey::Num5,
            PluginManifestActionShortcutKey::Num6 => ActionShortcutKey::Num6,
            PluginManifestActionShortcutKey::Num7 => ActionShortcutKey::Num7,
            PluginManifestActionShortcutKey::Num8 => ActionShortcutKey::Num8,
            PluginManifestActionShortcutKey::Num9 => ActionShortcutKey::Num9,
            PluginManifestActionShortcutKey::Exclamation => ActionShortcutKey::Exclamation,
            PluginManifestActionShortcutKey::AtSign => ActionShortcutKey::AtSign,
            PluginManifestActionShortcutKey::Hash => ActionShortcutKey::Hash,
            PluginManifestActionShortcutKey::Dollar => ActionShortcutKey::Dollar,
            PluginManifestActionShortcutKey::Percent => ActionShortcutKey::Percent,
            PluginManifestActionShortcutKey::Caret => ActionShortcutKey::Caret,
            PluginManifestActionShortcutKey::Ampersand => ActionShortcutKey::Ampersand,
            PluginManifestActionShortcutKey::Star => ActionShortcutKey::Star,
            PluginManifestActionShortcutKey::LeftParenthesis => ActionShortcutKey::LeftParenthesis,
            PluginManifestActionShortcutKey::RightParenthesis => ActionShortcutKey::RightParenthesis,
            PluginManifestActionShortcutKey::LowerA => ActionShortcutKey::LowerA,
            PluginManifestActionShortcutKey::LowerB => ActionShortcutKey::LowerB,
            PluginManifestActionShortcutKey::LowerC => ActionShortcutKey::LowerC,
            PluginManifestActionShortcutKey::LowerD => ActionShortcutKey::LowerD,
            PluginManifestActionShortcutKey::LowerE => ActionShortcutKey::LowerE,
            PluginManifestActionShortcutKey::LowerF => ActionShortcutKey::LowerF,
            PluginManifestActionShortcutKey::LowerG => ActionShortcutKey::LowerG,
            PluginManifestActionShortcutKey::LowerH => ActionShortcutKey::LowerH,
            PluginManifestActionShortcutKey::LowerI => ActionShortcutKey::LowerI,
            PluginManifestActionShortcutKey::LowerJ => ActionShortcutKey::LowerJ,
            PluginManifestActionShortcutKey::LowerK => ActionShortcutKey::LowerK,
            PluginManifestActionShortcutKey::LowerL => ActionShortcutKey::LowerL,
            PluginManifestActionShortcutKey::LowerM => ActionShortcutKey::LowerM,
            PluginManifestActionShortcutKey::LowerN => ActionShortcutKey::LowerN,
            PluginManifestActionShortcutKey::LowerO => ActionShortcutKey::LowerO,
            PluginManifestActionShortcutKey::LowerP => ActionShortcutKey::LowerP,
            PluginManifestActionShortcutKey::LowerQ => ActionShortcutKey::LowerQ,
            PluginManifestActionShortcutKey::LowerR => ActionShortcutKey::LowerR,
            PluginManifestActionShortcutKey::LowerS => ActionShortcutKey::LowerS,
            PluginManifestActionShortcutKey::LowerT => ActionShortcutKey::LowerT,
            PluginManifestActionShortcutKey::LowerU => ActionShortcutKey::LowerU,
            PluginManifestActionShortcutKey::LowerV => ActionShortcutKey::LowerV,
            PluginManifestActionShortcutKey::LowerW => ActionShortcutKey::LowerW,
            PluginManifestActionShortcutKey::LowerX => ActionShortcutKey::LowerX,
            PluginManifestActionShortcutKey::LowerY => ActionShortcutKey::LowerY,
            PluginManifestActionShortcutKey::LowerZ => ActionShortcutKey::LowerZ,
            PluginManifestActionShortcutKey::UpperA => ActionShortcutKey::UpperA,
            PluginManifestActionShortcutKey::UpperB => ActionShortcutKey::UpperB,
            PluginManifestActionShortcutKey::UpperC => ActionShortcutKey::UpperC,
            PluginManifestActionShortcutKey::UpperD => ActionShortcutKey::UpperD,
            PluginManifestActionShortcutKey::UpperE => ActionShortcutKey::UpperE,
            PluginManifestActionShortcutKey::UpperF => ActionShortcutKey::UpperF,
            PluginManifestActionShortcutKey::UpperG => ActionShortcutKey::UpperG,
            PluginManifestActionShortcutKey::UpperH => ActionShortcutKey::UpperH,
            PluginManifestActionShortcutKey::UpperI => ActionShortcutKey::UpperI,
            PluginManifestActionShortcutKey::UpperJ => ActionShortcutKey::UpperJ,
            PluginManifestActionShortcutKey::UpperK => ActionShortcutKey::UpperK,
            PluginManifestActionShortcutKey::UpperL => ActionShortcutKey::UpperL,
            PluginManifestActionShortcutKey::UpperM => ActionShortcutKey::UpperM,
            PluginManifestActionShortcutKey::UpperN => ActionShortcutKey::UpperN,
            PluginManifestActionShortcutKey::UpperO => ActionShortcutKey::UpperO,
            PluginManifestActionShortcutKey::UpperP => ActionShortcutKey::UpperP,
            PluginManifestActionShortcutKey::UpperQ => ActionShortcutKey::UpperQ,
            PluginManifestActionShortcutKey::UpperR => ActionShortcutKey::UpperR,
            PluginManifestActionShortcutKey::UpperS => ActionShortcutKey::UpperS,
            PluginManifestActionShortcutKey::UpperT => ActionShortcutKey::UpperT,
            PluginManifestActionShortcutKey::UpperU => ActionShortcutKey::UpperU,
            PluginManifestActionShortcutKey::UpperV => ActionShortcutKey::UpperV,
            PluginManifestActionShortcutKey::UpperW => ActionShortcutKey::UpperW,
            PluginManifestActionShortcutKey::UpperX => ActionShortcutKey::UpperX,
            PluginManifestActionShortcutKey::UpperY => ActionShortcutKey::UpperY,
            PluginManifestActionShortcutKey::UpperZ => ActionShortcutKey::UpperZ,
            PluginManifestActionShortcutKey::Minus => ActionShortcutKey::Minus,
            PluginManifestActionShortcutKey::Equals => ActionShortcutKey::Equals,
            PluginManifestActionShortcutKey::Comma => ActionShortcutKey::Comma,
            PluginManifestActionShortcutKey::Dot => ActionShortcutKey::Dot,
            PluginManifestActionShortcutKey::Slash => ActionShortcutKey::Slash,
            PluginManifestActionShortcutKey::OpenSquareBracket => ActionShortcutKey::OpenSquareBracket,
            PluginManifestActionShortcutKey::CloseSquareBracket => ActionShortcutKey::CloseSquareBracket,
            PluginManifestActionShortcutKey::Semicolon => ActionShortcutKey::Semicolon,
            PluginManifestActionShortcutKey::Quote => ActionShortcutKey::Quote,
            PluginManifestActionShortcutKey::Backslash => ActionShortcutKey::Backslash,
            PluginManifestActionShortcutKey::Underscore => ActionShortcutKey::Underscore,
            PluginManifestActionShortcutKey::Plus => ActionShortcutKey::Plus,
            PluginManifestActionShortcutKey::LessThan => ActionShortcutKey::LessThan,
            PluginManifestActionShortcutKey::GreaterThan => ActionShortcutKey::GreaterThan,
            PluginManifestActionShortcutKey::QuestionMark => ActionShortcutKey::QuestionMark,
            PluginManifestActionShortcutKey::LeftBrace => ActionShortcutKey::LeftBrace,
            PluginManifestActionShortcutKey::RightBrace => ActionShortcutKey::RightBrace,
            PluginManifestActionShortcutKey::Colon => ActionShortcutKey::Colon,
            PluginManifestActionShortcutKey::DoubleQuotes => ActionShortcutKey::DoubleQuotes,
            PluginManifestActionShortcutKey::Pipe => ActionShortcutKey::Pipe,
        }
    }
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[schemars(description = "Permissions required by the plugin.")]
pub struct PluginManifestPermissions {
    #[serde(default)]
    #[schemars(description = "Environment variables that the plugin can access.")]
    pub environment: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Network domains that the plugin can access.")]
    pub network: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Filesystem permissions for the plugin.")]
    pub filesystem: PluginManifestPermissionsFileSystem,
    #[serde(default)]
    #[schemars(description = "Execution permissions for the plugin.")]
    pub exec: PluginManifestPermissionsExec,
    #[serde(default)]
    #[schemars(description = "System permissions for the plugin.")]
    pub system: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Clipboard permissions for the plugin.")]
    pub clipboard: Vec<PluginManifestClipboardPermissions>,
    #[serde(default)]
    #[schemars(description = "Permissions for the main search bar.")]
    pub main_search_bar: Vec<PluginManifestMainSearchBarPermissions>,
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[schemars(description = "Filesystem permissions for the plugin.")]
pub struct PluginManifestPermissionsFileSystem {
    #[serde(default)]
    #[schemars(description = "Paths that the plugin can read from.")]
    pub read: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Paths that the plugin can write to.")]
    pub write: Vec<String>,
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[schemars(description = "Execution permissions for the plugin.")]
pub struct PluginManifestPermissionsExec {
    #[serde(default)]
    #[schemars(description = "Commands that the plugin can execute.")]
    pub command: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Executables that the plugin can run.")]
    pub executable: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Clipboard permissions for the plugin.")]
pub enum PluginManifestClipboardPermissions {
    #[serde(rename = "read")]
    #[schemars(description = "Allows the plugin to read from the clipboard.")]
    Read,
    #[serde(rename = "write")]
    #[schemars(description = "Allows the plugin to write to the clipboard.")]
    Write,
    #[serde(rename = "clear")]
    #[schemars(description = "Allows the plugin to clear the clipboard contents.")]
    Clear,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub enum PluginManifestMainSearchBarPermissions {
    #[serde(rename = "read")]
    #[schemars(description = "Allows the plugin to read the main search bar")]
    Read,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "os")]
pub enum PluginManifestSupportedSystem {
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "macos")]
    MacOS,
}

impl std::fmt::Display for PluginManifestSupportedSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PluginManifestSupportedSystem::Linux => write!(f, "Linux"),
            PluginManifestSupportedSystem::Windows => write!(f, "Windows"),
            PluginManifestSupportedSystem::MacOS => write!(f, "MacOS"),
        }
    }
}
