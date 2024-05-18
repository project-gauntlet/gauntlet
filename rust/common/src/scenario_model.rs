use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::model::UiWidgetId;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum ScenarioUiRenderLocation {
    InlineView,
    View
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioUiWidget {
    pub widget_id: UiWidgetId,
    pub widget_type: String,
    pub widget_properties: HashMap<String, ScenarioUiPropertyValue>,
    pub widget_children: Vec<ScenarioUiWidget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScenarioUiPropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Object(HashMap<String, ScenarioUiPropertyValue>),
    Undefined,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScenarioFrontendEvent {
    ReplaceView {
        entrypoint_id: String,
        render_location: ScenarioUiRenderLocation,
        top_level_view: bool,
        container: ScenarioUiWidget,
    },
    ShowPreferenceRequiredView {
        entrypoint_id: String,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    ShowPluginErrorView {
        entrypoint_id: String,
        render_location: ScenarioUiRenderLocation,
    },
}
