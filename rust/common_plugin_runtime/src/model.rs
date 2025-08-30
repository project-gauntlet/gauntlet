use std::collections::HashMap;
use std::fmt;

use bincode::Decode;
use bincode::Encode;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::Icons;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::UiWidgetId;
use serde::Deserialize;
use serde::Serialize;

use crate::api::BackendForPluginRuntimeApiRequestData;
use crate::api::BackendForPluginRuntimeApiResponseData;

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(tag = "type")]
pub enum JsEvent {
    OpenView {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
    },
    CloseView,
    PopView {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
    },
    RunCommand {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
    },
    RunGeneratedEntrypoint {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
        #[serde(rename = "actionIndex")]
        action_index: usize,
    },
    ViewEvent {
        #[serde(rename = "widgetId")]
        widget_id: UiWidgetId,
        #[serde(rename = "eventName")]
        event_name: String,
        #[serde(rename = "eventArguments")]
        event_arguments: Vec<JsUiPropertyValue>,
    },
    KeyboardEvent {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
        origin: JsKeyboardEventOrigin,
        key: String,
        #[serde(rename = "modifierShift")]
        modifier_shift: bool,
        #[serde(rename = "modifierControl")]
        modifier_control: bool,
        #[serde(rename = "modifierAlt")]
        modifier_alt: bool,
        #[serde(rename = "modifierMeta")]
        modifier_meta: bool,
    },
    OpenInlineView {
        #[serde(rename = "text")]
        text: String,
    },
    RefreshSearchIndex,
    MacosWindowTrackingEvent {
        event: JsMacosApplicationEvent,
    },
}

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(tag = "type")]
pub enum JsMacosApplicationEvent {
    WindowOpened {
        window_id: String,
        bundle_path: Option<String>,
        title: Option<String>,
    },
    WindowClosed {
        window_id: String,
    },
    WindowTitleChanged {
        window_id: String,
        title: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, Encode, Decode)]
pub enum JsKeyboardEventOrigin {
    MainView,
    PluginView,
}

// FIXME this could have been serde_v8::AnyValue but it doesn't support undefined, make a pr?
#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(tag = "type")]
pub enum JsUiPropertyValue {
    String { value: String },
    Number { value: f64 },
    Bool { value: bool },
    Undefined,
    Null,
}

#[derive(Debug, Encode, Decode)]
pub enum JsMessage {
    Event(JsEvent),
    Response(Result<BackendForPluginRuntimeApiResponseData, String>),
    Stop,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub enum JsUiRenderLocation {
    InlineView,
    View,
}

#[derive(Debug, Encode, Decode)]
pub struct JsPluginCode {
    pub js: HashMap<String, String>,
}

#[derive(Debug, Encode, Decode)]
pub struct JsInit {
    pub plugin_id: PluginId,
    pub plugin_uuid: String,
    pub code: JsPluginCode,
    pub permissions: JsPluginPermissions,
    pub inline_view_entrypoint_id: Option<String>,
    pub entrypoint_names: HashMap<EntrypointId, String>,
    pub dev_plugin: bool,
    pub home_dir: String,
    pub local_storage_dir: String,
    pub plugin_cache_dir: String,
    pub plugin_data_dir: String,
    pub stdout_file: Option<String>,
    pub stderr_file: Option<String>,
}

#[derive(Debug, Encode, Decode)]
pub struct JsPluginPermissions {
    pub environment: Vec<String>,
    pub network: Vec<String>,
    pub filesystem: JsPluginPermissionsFileSystem,
    pub exec: JsPluginPermissionsExec,
    pub system: Vec<String>,
    pub main_search_bar: Vec<JsPluginPermissionsMainSearchBar>,
}

#[derive(Debug, Encode, Decode)]
pub struct JsPluginPermissionsFileSystem {
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Debug, Encode, Decode)]
pub struct JsPluginPermissionsExec {
    pub command: Vec<String>,
    pub executable: Vec<String>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum JsPluginPermissionsMainSearchBar {
    Read,
}

#[derive(Debug, Encode, Decode)]
pub enum JsPluginRuntimeMessage {
    Stopped,
    Request(BackendForPluginRuntimeApiRequestData),
}

#[derive(Encode, Decode)]
pub struct JsGeneratedSearchItem {
    pub entrypoint_name: String,
    pub generator_entrypoint_id: String,
    pub entrypoint_id: String,
    pub entrypoint_uuid: String,
    pub entrypoint_icon: Option<Vec<u8>>,
    pub entrypoint_actions: Vec<JsGeneratedSearchItemAction>,
    pub entrypoint_accessories: Vec<JsGeneratedSearchItemAccessory>,
}

impl fmt::Debug for JsGeneratedSearchItem {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // exclude entrypoint_icon
        fmt.debug_struct("JsGeneratedSearchItem")
            .field("entrypoint_name", &self.entrypoint_name)
            .field("generator_entrypoint_id", &self.generator_entrypoint_id)
            .field("entrypoint_id", &self.entrypoint_id)
            .field("entrypoint_uuid", &self.entrypoint_uuid)
            .field("entrypoint_actions", &self.entrypoint_actions)
            .field("entrypoint_accessories", &self.entrypoint_accessories)
            .finish()
    }
}

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
pub struct JsGeneratedSearchItemAction {
    pub id: Option<String>,
    pub action_type: JsGeneratedSearchItemActionType,
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
pub enum JsGeneratedSearchItemActionType {
    View,
    Command,
}

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(untagged)]
pub enum JsPreferenceUserData {
    Number(f64),
    String(String),
    Bool(bool),
    ListOfStrings(Vec<String>),
    ListOfNumbers(Vec<f64>),
}

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(untagged)]
pub enum JsGeneratedSearchItemAccessory {
    TextAccessory {
        text: String,
        icon: Option<Icons>,
        tooltip: Option<String>,
    },
    IconAccessory {
        icon: Icons,
        tooltip: Option<String>,
    },
}

#[derive(Debug, Encode, Decode)]
pub struct JsClipboardData {
    pub text_data: Option<String>,
    pub png_data: Option<Vec<u8>>,
}
