use std::collections::HashMap;

use crate::dbus::{DBusUiPropertyContainer, DBusUiPropertyOneValue, DBusUiPropertyZeroValue, DBusUiWidget};

#[derive(Debug)]
pub struct NativeUiSearchRequest {
    pub prompt: String
}

#[derive(Debug)]
pub struct NativeUiSearchResult {
    pub plugin_uuid: String,
    pub plugin_name: String,
    pub entrypoint_id: String,
    pub entrypoint_name: String,
}

#[derive(Debug)]
pub enum NativeUiResponseData {
    GetContainer {
        container: NativeUiWidget
    },
    CreateInstance {
        widget: NativeUiWidget
    },
    CreateTextInstance {
        widget: NativeUiWidget
    },
    Unit,
}

#[derive(Debug)]
pub enum NativeUiRequestData {
    GetContainer,
    CreateInstance {
        widget_type: String,
    },
    CreateTextInstance {
        text: String,
    },
    AppendChild {
        parent: NativeUiWidget,
        child: NativeUiWidget,
    },
    RemoveChild {
        parent: NativeUiWidget,
        child: NativeUiWidget,
    },
    InsertBefore {
        parent: NativeUiWidget,
        child: NativeUiWidget,
        before_child: NativeUiWidget,
    },
    SetProperties {
        widget: NativeUiWidget,
        properties: HashMap<String, NativeUiPropertyValue>,
    },
    SetText {
        widget: NativeUiWidget,
        text: String,
    },
}

pub type NativeUiWidgetId = u32;
pub type NativeUiEventName = String;

impl From<DBusUiPropertyContainer> for HashMap<String, NativeUiPropertyValue> {
    fn from(value: DBusUiPropertyContainer) -> Self {
        let properties_one: HashMap<_, _> = value.one
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    DBusUiPropertyOneValue::String(value) => NativeUiPropertyValue::String(value),
                    DBusUiPropertyOneValue::Number(value) => NativeUiPropertyValue::Number(value),
                    DBusUiPropertyOneValue::Bool(value) => NativeUiPropertyValue::Bool(value),
                };

                (key, value)
            })
            .collect();

        let mut properties: HashMap<_, _> = value.zero
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    DBusUiPropertyZeroValue::Function => NativeUiPropertyValue::Function,
                };

                (key, value)
            })
            .collect();

        properties.extend(properties_one);

        properties
    }
}

#[derive(Debug)]
pub enum NativeUiPropertyValue {
    Function,
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug)]
pub struct NativeUiWidget {
    pub widget_id: NativeUiWidgetId,
}

impl From<NativeUiWidget> for DBusUiWidget {
    fn from(value: NativeUiWidget) -> Self {
        Self {
            widget_id: value.widget_id
        }
    }
}

impl From<DBusUiWidget> for NativeUiWidget {
    fn from(value: DBusUiWidget) -> Self {
        Self {
            widget_id: value.widget_id
        }
    }
}
