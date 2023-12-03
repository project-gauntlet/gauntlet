use std::collections::HashMap;

use anyhow::anyhow;
use zbus::zvariant::Value;

use common::dbus::{DBusUiPropertyContainer, DBusUiPropertyValueType, DBusUiWidget};
use common::model::{EntrypointId, PluginId};

#[derive(Debug, Clone)]
pub struct NativeUiSearchResult {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
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
        widget: NativeUiWidget,
        widget_type: String,
        new_props: HashMap<String, NativeUiPropertyValue>,
        keep_children: bool,
    },
    ReplaceContainerChildren {
        container: NativeUiWidget,
        new_children: Vec<NativeUiWidget>,
    },
}

pub type NativeUiWidgetId = u32;

pub fn from_dbus(value: DBusUiPropertyContainer) -> anyhow::Result<HashMap<String, NativeUiPropertyValue>> {
    let result = value.properties
        .into_iter()
        .map(|(key, (value_type, value))| {
            let value = match &(value_type, value.into()) {
                (DBusUiPropertyValueType::String, Value::Str(value)) => NativeUiPropertyValue::String(value.to_string()),
                (DBusUiPropertyValueType::Number, Value::F64(value)) => NativeUiPropertyValue::Number(*value),
                (DBusUiPropertyValueType::Bool, Value::Bool(value)) => NativeUiPropertyValue::Bool(*value),
                (DBusUiPropertyValueType::Function, _) => NativeUiPropertyValue::Function,
                _ => {
                    return Err(anyhow!("invalid type"))
                }
            };

            Ok((key, value))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .collect::<HashMap<_, _>>();

    Ok(result)
}

#[derive(Debug, Clone)]
pub enum NativeUiPropertyValue {
    Function,
    String(String),
    Number(f64),
    Bool(bool),
}

impl NativeUiPropertyValue {
    pub fn as_string(&self) -> Option<&str> {
        if let NativeUiPropertyValue::String(val) = self {
            Some(val)
        } else {
            None
        }
    }
    pub fn as_number(&self) -> Option<&f64> {
        if let NativeUiPropertyValue::Number(val) = self {
            Some(val)
        } else {
            None
        }
    }
    pub fn as_bool(&self) -> Option<&bool> {
        if let NativeUiPropertyValue::Bool(val) = self {
            Some(val)
        } else {
            None
        }
    }
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
