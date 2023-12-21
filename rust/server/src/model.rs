use std::collections::HashMap;

use deno_core::{serde_v8, v8};
use serde::{Deserialize, Serialize};
use zbus::zvariant::Value;

use common::dbus::{DBusUiPropertyContainer, DBusUiPropertyValueType, DBusUiWidget};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsUiEvent {
    ViewCreated {
        #[serde(rename = "reconcilerMode")]
        reconciler_mode: String,
        #[serde(rename = "viewName")]
        view_name: String
    },
    ViewDestroyed,
    ViewEvent {
        widget_id: UiWidgetId,
        #[serde(rename = "eventName")]
        event_name: String,
    },
    PluginCommand {
        #[serde(rename = "commandType")]
        command_type: String,
    }
}

pub type JsUiWidgetId = u32;
pub type JsUiEventName = String;

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
pub struct IntermediateUiWidget {
    pub widget_id: UiWidgetId,
    pub widget_type: String,
    pub widget_properties: HashMap<String, IntermediatePropertyValue>,
    pub widget_children: Vec<IntermediateUiWidget>,
}

#[derive(Debug, Clone)]
pub enum IntermediatePropertyValue {
    Function(v8::Global<v8::Function>),
    String(String),
    Number(f64),
    Bool(bool),
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

fn from_intermediate_to_dbus_properties(value: HashMap<String, IntermediatePropertyValue>) -> DBusUiPropertyContainer {
    let properties: HashMap<_, _> = value.iter()
        .filter_map(|(key, value)| {
            match value {
                IntermediatePropertyValue::Function(_) => Some((key.to_owned(), (DBusUiPropertyValueType::Function, Value::U8(0).to_owned()))),
                IntermediatePropertyValue::String(value) => Some((key.to_owned(), (DBusUiPropertyValueType::String, Value::Str(value.into()).to_owned()))),
                IntermediatePropertyValue::Number(value) => Some((key.to_owned(), (DBusUiPropertyValueType::Number, Value::F64(value.to_owned()).to_owned()))),
                IntermediatePropertyValue::Bool(value) => Some((key.to_owned(), (DBusUiPropertyValueType::Bool, Value::Bool(value.to_owned()).to_owned()))),
            }
        })
        .collect();

    DBusUiPropertyContainer(properties)
}
