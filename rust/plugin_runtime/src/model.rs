use crate::JsEvent;
use gauntlet_common::model::{EntrypointId, Icons, PluginId, RootWidget};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum JsMessage {
    Event(JsEvent),
    Response(Result<JsResponse, String>),
    Stop,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub enum JsUiRenderLocation {
    InlineView,
    View
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
    Request(JsRequest),
}

#[derive(Debug, Encode, Decode)]
pub enum JsResponse {
    Nothing,
    AssetData {
        data: Vec<u8>
    },
    EntrypointGeneratorEntrypointIds {
        data: Vec<String>
    },
    PluginPreferences {
        data: HashMap<String, JsPreferenceUserData>
    },
    EntrypointPreferences {
        data: HashMap<String, JsPreferenceUserData>
    },
    PluginPreferencesRequired {
        data: bool
    },
    EntrypointPreferencesRequired {
        data: bool
    },
    ClipboardRead {
        data: JsClipboardData
    },
    ClipboardReadText {
        data: Option<String>
    },
    ActionIdForShortcut {
        data: Option<String>
    },
}

#[derive(Debug, Encode, Decode)]
pub enum JsRequest {
    Render {
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: JsUiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    },
    ClearInlineView,
    ShowPluginErrorView {
        entrypoint_id: EntrypointId,
        render_location: JsUiRenderLocation,
    },
    ShowPreferenceRequiredView {
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    },
    ShowHud {
        display: String
    },
    UpdateLoadingBar {
        entrypoint_id: EntrypointId,
        show: bool
    },
    ReloadSearchIndex {
        generated_commands: Vec<JsGeneratedSearchItem>,
        refresh_search_list: bool
    },
    GetAssetData {
        path: String,
    },
    GetEntrypointGeneratorEntrypointIds,
    GetPluginPreferences,
    GetEntrypointPreferences {
        entrypoint_id: EntrypointId,
    },
    PluginPreferencesRequired,
    EntrypointPreferencesRequired {
        entrypoint_id: EntrypointId,
    },
    ClipboardRead,
    ClipboardReadText,
    ClipboardWrite {
        data: JsClipboardData
    },
    ClipboardWriteText {
        data: String
    },
    ClipboardClear,
    GetActionIdForShortcut {
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    },
}

#[derive(Deserialize, Serialize, Encode, Decode)]
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
        tooltip: Option<String>
    },
    IconAccessory {
        icon: Icons,
        tooltip: Option<String>
    },
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct JsClipboardData {
    pub text_data: Option<String>,
    pub png_data: Option<Vec<u8>>
}