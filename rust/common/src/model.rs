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
pub struct ActionShortcut {
    pub key: String,
    pub kind: ActionShortcutKind,
}

#[derive(Debug, Clone)]
pub enum ActionShortcutKind {
    Main,
    Alternative
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
    Bytes(Vec<u8>),
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
