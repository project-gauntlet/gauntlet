use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use common::model::{UiPropertyValue, UiRenderLocation, UiWidget, UiWidgetId};

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
    ClearInlineView,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScenarioBackendEvent {
    Search {
        text: String
    },
    RequestViewRender,
}


pub fn ui_render_location_to_scenario(render_location: UiRenderLocation) -> ScenarioUiRenderLocation {
    match render_location {
        UiRenderLocation::InlineView => ScenarioUiRenderLocation::InlineView,
        UiRenderLocation::View => ScenarioUiRenderLocation::View,
    }
}

pub fn ui_widget_to_scenario(value: UiWidget) -> ScenarioUiWidget {
    let children = value.widget_children.into_iter()
        .map(|child| ui_widget_to_scenario(child))
        .collect::<Vec<_>>();

    ScenarioUiWidget {
        widget_id: value.widget_id,
        widget_type: value.widget_type,
        widget_properties: ui_property_values_to_scenario(value.widget_properties),
        widget_children: children
    }
}

fn ui_property_values_to_scenario(value: HashMap<String, UiPropertyValue>) -> HashMap<String, ScenarioUiPropertyValue> {
    value.into_iter()
        .map(|(key, value)| (key, ui_property_value_to_scenario(value)))
        .collect()
}

fn ui_property_value_to_scenario(value: UiPropertyValue) -> ScenarioUiPropertyValue {
    match value {
        UiPropertyValue::String(value) => ScenarioUiPropertyValue::String(value),
        UiPropertyValue::Number(value) => ScenarioUiPropertyValue::Number(value),
        UiPropertyValue::Bool(value) => ScenarioUiPropertyValue::Bool(value),
        UiPropertyValue::Bytes(value) => ScenarioUiPropertyValue::Bytes(value),
        UiPropertyValue::Object(value) => {
            let value: HashMap<String, _> = value.into_iter()
                .map(|(name, value)| (name, ui_property_value_to_scenario(value)))
                .collect();

            ScenarioUiPropertyValue::Object(value)
        }
        UiPropertyValue::Undefined => ScenarioUiPropertyValue::Undefined,
    }
}