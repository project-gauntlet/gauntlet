use std::collections::HashMap;

use crate::common::dbus::{DBusUiPropertyContainer, DBusUiPropertyOneValue, DBusUiPropertyZeroValue, DBusUiWidget};
use crate::common::model::{EntrypointUuid, PluginUuid};

#[derive(Debug, Clone)]
pub struct NativeUiSearchResult {
    pub plugin_uuid: PluginUuid,
    pub plugin_name: String,
    pub entrypoint_uuid: EntrypointUuid,
    pub entrypoint_name: String,
}

#[derive(Debug, Clone)]
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
    CloneInstance {
        widget: NativeUiWidget
    },
    Unit,
}

#[derive(Debug, Clone)]
pub enum NativeUiRequestData {
    GetContainer,
    CreateInstance {
        widget_type: String,
        properties: HashMap<String, NativeUiPropertyValue>,
    },
    CreateTextInstance {
        text: String,
    },
    AppendChild {
        parent: NativeUiWidget,
        child: NativeUiWidget,
    },
    CloneInstance {
        widget_type: String,
        properties: HashMap<String, NativeUiPropertyValue>,
    },
    ReplaceContainerChildren {
        container: NativeUiWidget,
        new_children: Vec<NativeUiWidget>,
    },
}

pub type NativeUiWidgetId = u32;

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

#[derive(Debug, Clone)]
pub enum NativeUiPropertyValue {
    Function,
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone)]
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
