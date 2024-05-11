use std::collections::HashMap;

use deno_core::serde_v8;
use serde::{Deserialize, Serialize};

use common::model::{EntrypointId, UiPropertyValue, UiWidget, UiWidgetId};

#[derive(Debug)]
pub enum JsUiResponseData {
    Nothing
}

#[derive(Debug)]
pub enum JsUiRequestData {
    ReplaceView {
        entrypoint_id: EntrypointId,
        render_location: JsRenderLocation,
        top_level_view: bool,
        container: UiWidget,
    },
    ClearInlineView,
    ShowPluginErrorView {
        entrypoint_id: EntrypointId,
        render_location: JsRenderLocation,
    },
    ShowPreferenceRequiredView {
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum JsRenderLocation {
    InlineView,
    View
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsUiEvent {
    OpenView {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    RunCommand {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    RunGeneratedCommand {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    ViewEvent {
        #[serde(rename = "widgetId")]
        widget_id: UiWidgetId,
        #[serde(rename = "eventName")]
        event_name: String,
        #[serde(rename = "eventArguments")]
        event_arguments: Vec<JsPropertyValue>,
    },
    KeyboardEvent {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
        key: String,
        #[serde(rename = "modifierShift")]
        modifier_shift: bool,
        #[serde(rename = "modifierControl")]
        modifier_control: bool,
        #[serde(rename = "modifierAlt")]
        modifier_alt: bool,
        #[serde(rename = "modifierMeta")]
        modifier_meta: bool
    },
    PluginCommand {
        #[serde(rename = "commandType")]
        command_type: String,
    },
    OpenInlineView {
        #[serde(rename = "text")]
        text: String,
    },
    ReloadSearchIndex,
}

// FIXME this could have been serde_v8::AnyValue but it doesn't support undefined, make a pr?
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsPropertyValue {
    String {
        value: String
    },
    Number {
        value: f64
    },
    Bool {
        value: bool
    },
    Undefined,
}

#[derive(Deserialize, Serialize)]
pub struct JsUiWidget<'a> {
    #[serde(rename = "widgetId")]
    pub widget_id: UiWidgetId,
    #[serde(rename = "widgetType")]
    pub widget_type: String,
    #[serde(rename = "widgetProperties")]
    pub widget_properties: HashMap<String, serde_v8::Value<'a>>,
    #[serde(rename = "widgetChildren")]
    pub widget_children: Vec<JsUiWidget<'a>>,
}

#[derive(Debug)]
pub enum IntermediateUiEvent {
    OpenView {
        entrypoint_id: EntrypointId
    },
    RunCommand {
        entrypoint_id: String
    },
    RunGeneratedCommand {
        entrypoint_id: String
    },
    HandleViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    HandleKeyboardEvent {
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    },
    PluginCommand {
        command_type: String,
    },
    OpenInlineView {
        text: String,
    },
    ReloadSearchIndex,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PreferenceUserData {
    Number(f64),
    String(String),
    Bool(bool),
    ListOfStrings(Vec<String>),
    ListOfNumbers(Vec<f64>),
}
