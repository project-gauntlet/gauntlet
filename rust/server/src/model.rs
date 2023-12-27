use std::collections::HashMap;

use deno_core::serde_v8;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Value;

use common::dbus::{DBusUiPropertyContainer, DBusUiPropertyValue, DBusUiPropertyValueType, DBusUiWidget, value_bool_to_dbus, value_number_to_dbus, value_string_to_dbus};

#[derive(Debug)]
pub enum JsUiResponseData {
    Nothing
}

#[derive(Debug)]
pub enum JsUiRequestData {
    ReplaceContainerChildren {
        container: IntermediateUiWidget,
        new_children: Vec<IntermediateUiWidget>,
    },
}

pub type UiWidgetId = u32;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsUiEvent {
    OpenView {
        #[serde(rename = "frontend")]
        frontend: String,
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    RunCommand {
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
    PluginCommand {
        #[serde(rename = "commandType")]
        command_type: String,
    }
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
        frontend: String,
        entrypoint_id: String
    },
    RunCommand {
        entrypoint_id: String
    },
    ViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<IntermediatePropertyValue>,
    },
    PluginCommand {
        command_type: String,
    }
}

#[derive(Debug)]
pub struct IntermediateUiWidget {
    pub widget_id: UiWidgetId,
    pub widget_type: String,
    pub widget_properties: HashMap<String, IntermediatePropertyValue>,
    pub widget_children: Vec<IntermediateUiWidget>,
}

#[derive(Debug)]
pub enum IntermediatePropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    Undefined,
}


impl From<IntermediateUiWidget> for DBusUiWidget {
    fn from(value: IntermediateUiWidget) -> Self {
        let children = value.widget_children.into_iter()
            .map(|child| child.into())
            .collect::<Vec<DBusUiWidget>>();

        Self {
            widget_id: value.widget_id,
            widget_type: value.widget_type,
            widget_properties: from_intermediate_to_dbus_properties(value.widget_properties),
            widget_children: children
        }
    }
}

pub fn from_dbus_to_intermediate_value(value: DBusUiPropertyValue) -> anyhow::Result<IntermediatePropertyValue> {
    match value {
        DBusUiPropertyValue(DBusUiPropertyValueType::Undefined, _) => Ok(IntermediatePropertyValue::Undefined),
        DBusUiPropertyValue(DBusUiPropertyValueType::String, value) => {
            match value.into() {
                Value::Str(value) => Ok(IntermediatePropertyValue::String(value.to_string())),
                value @ _ => Err(anyhow::anyhow!("invalid dbus value {:?}, string expected", value))
            }
        }
        DBusUiPropertyValue(DBusUiPropertyValueType::Number, value) => {
            match value.into() {
                Value::F64(value) => Ok(IntermediatePropertyValue::Number(value)),
                value @ _ => Err(anyhow::anyhow!("invalid dbus value {:?}, number expected", value))
            }
        }
        DBusUiPropertyValue(DBusUiPropertyValueType::Bool, value) => {
            match value.into() {
                Value::Bool(value) => Ok(IntermediatePropertyValue::Bool(value)),
                value @ _ => Err(anyhow::anyhow!("invalid dbus value {:?}, bool expected", value))
            }
        }
    }
}


fn from_intermediate_to_dbus_properties(value: HashMap<String, IntermediatePropertyValue>) -> DBusUiPropertyContainer {
    let properties: HashMap<_, _> = value.iter()
        .filter_map(|(key, value)| {
            match value {
                IntermediatePropertyValue::String(value) => Some((key.to_owned(), value_string_to_dbus(value.to_owned()))),
                IntermediatePropertyValue::Number(value) => Some((key.to_owned(), value_number_to_dbus(value.to_owned()))),
                IntermediatePropertyValue::Bool(value) => Some((key.to_owned(), value_bool_to_dbus(value.to_owned()))),
                IntermediatePropertyValue::Undefined => None
            }
        })
        .collect();

    DBusUiPropertyContainer(properties)
}

#[derive(Debug, Clone)]
pub enum PluginEntrypointType {
    Command,
    View,
}

pub fn entrypoint_to_str(value: PluginEntrypointType) -> &'static str {
    match value {
        PluginEntrypointType::Command => "command",
        PluginEntrypointType::View => "view",
    }
}

pub fn entrypoint_from_str(value: &str) -> PluginEntrypointType {
    match value {
        "command" => PluginEntrypointType::Command,
        "view" => PluginEntrypointType::View,
        _ => {
            panic!("index contains illegal entrypoint_type: {}", value)
        }
    }
}