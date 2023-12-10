use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zbus::zvariant::Value;

use common::dbus::{DBusUiPropertyContainer, DBusUiPropertyValueType, DBusUiWidget};

#[derive(Debug)]
pub enum JsUiResponseData {
    GetRoot {
        container: JsUiWidget
    },
    CreateInstance {
        widget: JsUiWidget
    },
    CreateTextInstance {
        widget: JsUiWidget
    },
    CloneInstance {
        widget: JsUiWidget
    },
    Nothing
}

#[derive(Debug)]
pub enum JsUiRequestData {
    GetRoot,
    CreateInstance {
        widget_type: String,
        properties: HashMap<String, JsUiPropertyValue>,
    },
    CreateTextInstance {
        text: String,
    },
    AppendChild {
        parent: JsUiWidget,
        child: JsUiWidget,
    },
    RemoveChild {
        parent: JsUiWidget,
        child: JsUiWidget,
    },
    InsertBefore {
        parent: JsUiWidget,
        child: JsUiWidget,
        before_child: JsUiWidget,
    },
    SetProperties {
        widget: JsUiWidget,
        properties: HashMap<String, JsUiPropertyValue>,
    },
    SetText {
        widget: JsUiWidget,
        text: String,
    },
    CloneInstance {
        widget: JsUiWidget,
        update_payload: Vec<String>,
        widget_type: String,
        old_props: HashMap<String, JsUiPropertyValue>,
        new_props: HashMap<String, JsUiPropertyValue>,
        keep_children: bool,
    },
    ReplaceContainerChildren {
        container: JsUiWidget,
        new_children: Vec<JsUiWidget>,
    },
}

pub type UiWidgetId = u32;
pub type UiEventName = String;

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
        widget: JsUiWidget,
        #[serde(rename = "eventName")]
        event_name: UiEventName,
    },
    PluginCommand {
        #[serde(rename = "commandType")]
        command_type: String,
    }
}

#[derive(Debug)]
pub enum JsUiPropertyValue {
    Function,
    String(String),
    Number(f64),
    Bool(bool),
}

pub fn to_dbus(value: HashMap<String, JsUiPropertyValue>) -> DBusUiPropertyContainer {
    let properties: HashMap<_, _> = value.iter()
        .filter_map(|(key, value)| {
            match value {
                JsUiPropertyValue::Function => Some((key.to_owned(), (DBusUiPropertyValueType::Function, Value::U8(0).to_owned()))),
                JsUiPropertyValue::String(value) => Some((key.to_owned(), (DBusUiPropertyValueType::String, Value::Str(value.into()).to_owned()))),
                JsUiPropertyValue::Number(value) => Some((key.to_owned(), (DBusUiPropertyValueType::Number, Value::F64(value.to_owned()).to_owned()))),
                JsUiPropertyValue::Bool(value) => Some((key.to_owned(), (DBusUiPropertyValueType::Bool, Value::Bool(value.to_owned()).to_owned()))),
            }
        })
        .collect();

    DBusUiPropertyContainer { properties }
}

pub type JsUiWidgetId = u32;
pub type JsUiEventName = String;

#[derive(Debug, Deserialize, Serialize)]
pub struct JsUiWidget {
    #[serde(rename = "widgetId")]
    pub widget_id: UiWidgetId,
    #[serde(rename = "widgetType")]
    pub widget_type: String,
}

impl From<JsUiWidget> for DBusUiWidget {
    fn from(value: JsUiWidget) -> Self {
        Self {
            widget_id: value.widget_id,
            widget_type: value.widget_type
        }
    }
}

impl From<DBusUiWidget> for JsUiWidget {
    fn from(value: DBusUiWidget) -> Self {
        Self {
            widget_id: value.widget_id,
            widget_type: value.widget_type
        }
    }
}
