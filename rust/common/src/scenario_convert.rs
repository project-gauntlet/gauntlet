use std::collections::HashMap;
use crate::model::{UiPropertyValue, UiRenderLocation, UiWidget};
use crate::scenario_model::{ScenarioUiPropertyValue, ScenarioUiRenderLocation, ScenarioUiWidget};

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
        UiPropertyValue::Bytes(value) => ScenarioUiPropertyValue::Bytes(value.to_vec()),
        UiPropertyValue::Object(value) => {
            let value: HashMap<String, _> = value.into_iter()
                .map(|(name, value)| (name, ui_property_value_to_scenario(value)))
                .collect();

            ScenarioUiPropertyValue::Object(value)
        }
        UiPropertyValue::Undefined => ScenarioUiPropertyValue::Undefined,
    }
}

pub fn ui_widget_from_scenario(value: ScenarioUiWidget) -> UiWidget {
    let children = value.widget_children.into_iter()
        .map(|child| ui_widget_from_scenario(child))
        .collect::<Vec<_>>();

    UiWidget {
        widget_id: value.widget_id,
        widget_type: value.widget_type,
        widget_properties: ui_property_values_from_scenario(value.widget_properties),
        widget_children: children
    }
}

fn ui_property_values_from_scenario(value: HashMap<String, ScenarioUiPropertyValue>) -> HashMap<String, UiPropertyValue> {
    value.into_iter()
        .map(|(key, value)| (key, ui_property_value_from_scenario(value)))
        .collect()
}

fn ui_property_value_from_scenario(value: ScenarioUiPropertyValue) -> UiPropertyValue {
    match value {
        ScenarioUiPropertyValue::String(value) => UiPropertyValue::String(value),
        ScenarioUiPropertyValue::Number(value) => UiPropertyValue::Number(value),
        ScenarioUiPropertyValue::Bool(value) => UiPropertyValue::Bool(value),
        ScenarioUiPropertyValue::Bytes(value) => UiPropertyValue::Bytes(bytes::Bytes::from(value)),
        ScenarioUiPropertyValue::Object(value) => {
            let value: HashMap<String, _> = value.into_iter()
                .map(|(name, value)| (name, ui_property_value_from_scenario(value)))
                .collect();

            UiPropertyValue::Object(value)
        }
        ScenarioUiPropertyValue::Undefined => UiPropertyValue::Undefined,
    }
}

pub fn ui_render_location_from_scenario(render_location: ScenarioUiRenderLocation) -> UiRenderLocation {
    match render_location {
        ScenarioUiRenderLocation::InlineView => UiRenderLocation::InlineView,
        ScenarioUiRenderLocation::View => UiRenderLocation::View,
    }
}
