use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::model::ActionShortcutKey;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Plugin Manifest definition")]
pub struct PluginManifest {
    #[serde(rename = "$schema")]
    #[allow(unused)]
    schema: Option<String>,
    #[schemars(description = "General plugin metadata")]
    pub gauntlet: PluginManifestMetadata,
    #[schemars(description = "Plugin entrypoints, all plugin will have at least one entrypoint")]
    pub entrypoint: Vec<PluginManifestEntrypoint>,
    #[serde(default)]
    #[schemars(description = "List of supported operating systems")]
    pub supported_system: Vec<PluginManifestSupportedSystem>,
    #[serde(default)]
    #[schemars(description = "Permissions required by the plugin")]
    pub permissions: PluginManifestPermissions,
    #[serde(default)]
    #[schemars(description = "Preferences that can be configured by the user in the settings view")]
    pub preferences: Vec<PluginManifestPreference>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Plugin entrypoint definition")]
pub struct PluginManifestEntrypoint {
    #[schemars(description = "Unique identifier of the entrypoint, can only contain small letters, numbers and dash")]
    pub id: String,
    #[schemars(description = "Entrypoint name")]
    pub name: String,
    #[schemars(description = "Entrypoint description")]
    pub description: String,
    #[allow(unused)] // Used during plugin build
    #[schemars(description = "Path to TypeScript file relative to package directory")]
    path: String,
    #[schemars(description = "Entrypoint icon, path to file in assets relative to it")]
    pub icon: Option<String>,
    #[serde(rename = "type")]
    #[schemars(description = "Type of the entrypoint")]
    pub entrypoint_type: PluginManifestEntrypointTypes,
    #[serde(default)]
    #[schemars(description = "List of definitions of plugin preferences")]
    pub preferences: Vec<PluginManifestPreference>,
    #[serde(default)]
    #[schemars(description = "List of definitions of plugin actions")]
    pub actions: Vec<PluginManifestAction>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(description = "User-configurable preference options")]
pub enum PluginManifestPreference {
    #[serde(rename = "number")]
    #[schemars(description = "A numeric preference")]
    Number {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        #[schemars(description = "Default value")]
        default: Option<f64>,
        #[schemars(description = "Description of the preference")]
        description: String,
    },
    #[serde(rename = "string")]
    #[schemars(description = "A string preference")]
    String {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        #[schemars(description = "Default value")]
        default: Option<String>,
        #[schemars(description = "Description of the preference")]
        description: String,
    },
    #[serde(rename = "enum")]
    #[schemars(description = "An enum preference with selectable values")]
    Enum {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        #[schemars(description = "Default value")]
        default: Option<String>,
        #[schemars(description = "Description of the preference")]
        description: String,
        #[schemars(description = "List of allowed enum values")]
        enum_values: Vec<PluginManifestPreferenceEnumValue>,
    },
    #[serde(rename = "bool")]
    #[schemars(description = "A boolean preference")]
    Bool {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        #[schemars(description = "Default value")]
        default: Option<bool>,
        #[schemars(description = "Description of the preference")]
        description: String,
    },
    #[serde(rename = "list_of_strings")]
    #[schemars(description = "A list of strings preference")]
    ListOfStrings {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        // default: Option<Vec<String>>,
        #[schemars(description = "Description of the preference")]
        description: String,
    },
    #[serde(rename = "list_of_numbers")]
    #[schemars(description = "A list of numbers preference")]
    ListOfNumbers {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        // default: Option<Vec<f64>>,
        #[schemars(description = "Description of the preference")]
        description: String,
    },
    #[serde(rename = "list_of_enums")]
    #[schemars(description = "A list of enumerated preference values")]
    ListOfEnums {
        #[schemars(description = "Unique identifier of the preference, can only contain letters and numbers")]
        id: String,
        #[schemars(description = "Display name of the preference")]
        name: String,
        // default: Option<Vec<String>>,
        #[schemars(description = "List of allowed enum values")]
        enum_values: Vec<PluginManifestPreferenceEnumValue>,
        #[schemars(description = "Description of the preference")]
        description: String,
    },
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Definition of the values available in enumerated preference")]
pub struct PluginManifestPreferenceEnumValue {
    #[schemars(description = "Displayed name")]
    pub label: String,
    #[schemars(description = "Internal enum value")]
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Types of plugin entrypoints")]
pub enum PluginManifestEntrypointTypes {
    #[serde(rename = "command")]
    #[schemars(description = "A function-based entrypoint")]
    Command,
    #[serde(rename = "view")]
    #[schemars(description = "A view-based entrypoint")]
    View,
    #[serde(rename = "inline-view")]
    #[schemars(description = "A view-based entrypoint displayed under main search bar")]
    InlineView,
    #[serde(rename = "entrypoint-generator")]
    #[schemars(description = "Entrypoint that dynamically generatepointsd")]
    EntrypointGenerator,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Action definition")]
pub struct PluginManifestAction {
    #[schemars(description = "Unique identifier for the action, can only contain letters and numbers")]
    pub id: String,
    #[schemars(description = "Description of what the action does")]
    pub description: String,
    #[schemars(description = "Default keyboard shortcut to trigger the action")]
    pub shortcut: PluginManifestActionShortcut,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Keyboard shortcut for a plugin action")]
pub struct PluginManifestActionShortcut {
    #[schemars(description = "The main key to be pressed for this shortcut")]
    pub key: PluginManifestActionShortcutKey,
    #[schemars(description = "The kind of shortcut, defines required modifiers")]
    pub kind: PluginManifestActionShortcutKind,
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

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "The kind of shortcut")]
pub enum PluginManifestActionShortcutKind {
    #[serde(rename = "main")]
    #[schemars(description = "Main kind shortcuts require Ctrl modifier on Windows/Linux or Cmd on macOS")]
    Main,
    #[serde(rename = "alternative")]
    #[schemars(description = "Alternative kind shortcuts require Alt modifier on Windows/Linux or Opt on macOS")]
    Alternative,
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

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "General plugin metadata")]
pub struct PluginManifestMetadata {
    #[schemars(description = "Name of the plugin")]
    pub name: String,
    #[schemars(description = "Description of the plugin")]
    pub description: String,
    #[schemars(description = "Description of the plugin")]
    #[serde(default)]
    pub authors: Vec<PluginManifestMetadataAuthor>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Plugin author")]
pub struct PluginManifestMetadataAuthor {
    #[schemars(description = "Author name")]
    pub name: String,
    #[schemars(
        description = "URIs that identify the author. Can be a link to social media page or an email (if email it should begin with mailto: schema)"
    )]
    #[serde(default)]
    pub uris: Vec<String>,
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[schemars(description = "Permissions required by the plugin")]
pub struct PluginManifestPermissions {
    #[serde(default)]
    #[schemars(description = "Environment variables that the plugin can access")]
    pub environment: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Network address (domain or ip address + optional port) that the plugin can access")]
    pub network: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Filesystem permissions for the plugin")]
    pub filesystem: PluginManifestPermissionsFileSystem,
    #[serde(default)]
    #[schemars(description = "Execution permissions for the plugin")]
    pub exec: PluginManifestPermissionsExec,
    #[serde(default)]
    #[schemars(description = "Deno system permissions for the plugin")]
    pub system: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Clipboard permissions for the plugin")]
    pub clipboard: Vec<PluginManifestClipboardPermissions>,
    #[serde(default)]
    #[schemars(description = "Permissions for the main search bar")]
    pub main_search_bar: Vec<PluginManifestMainSearchBarPermissions>,
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[schemars(description = "Filesystem permissions for the plugin")]
pub struct PluginManifestPermissionsFileSystem {
    #[serde(default)]
    #[schemars(description = "Paths that the plugin can read from")]
    pub read: Vec<String>,
    #[serde(default)]
    #[schemars(description = "Paths that the plugin can write to")]
    pub write: Vec<String>,
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
#[schemars(description = "Execution permissions for the plugin")]
pub struct PluginManifestPermissionsExec {
    #[serde(default)]
    #[schemars(description = "List of commands on PATH that the plugin can execute")]
    pub command: Vec<String>,
    #[serde(default)]
    #[schemars(description = "List of paths to executables that the plugin can run")]
    pub executable: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Clipboard permissions for the plugin")]
pub enum PluginManifestClipboardPermissions {
    #[serde(rename = "read")]
    #[schemars(description = "Allows the plugin to read from the clipboard")]
    Read,
    #[serde(rename = "write")]
    #[schemars(description = "Allows the plugin to write to the clipboard")]
    Write,
    #[serde(rename = "clear")]
    #[schemars(description = "Allows the plugin to clear the clipboard contents")]
    Clear,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub enum PluginManifestMainSearchBarPermissions {
    #[serde(rename = "read")]
    #[schemars(description = "Allows the plugin to read the main search bar")]
    Read,
}
