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
pub enum ScenarioUiPropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    #[serde(with="base64")]
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

mod base64 {
    use serde::{Serialize, Deserialize};
    use serde::{Deserializer, Serializer};
    use base64::Engine;

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::engine::general_purpose::STANDARD.encode(v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD.decode(base64.as_bytes())
            .map_err(|e| serde::de::Error::custom(e))
    }
}