use std::collections::HashMap;

use anyhow::anyhow;
use zbus::zvariant::Value;

use common::dbus::{DBusUiPropertyContainer, DBusUiPropertyValue, DBusUiPropertyValueType, DBusUiWidget};
use common::model::{EntrypointId, PluginId};

#[derive(Debug, Clone)]
pub struct NativeUiSearchResult {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
}

#[derive(Debug)]
pub enum NativeUiResponseData {
    ReplaceContainerChildren,
}

#[derive(Debug)]
pub enum NativeUiRequestData {
    ReplaceContainerChildren {
        container: NativeUiWidget,
        new_children: Vec<NativeUiWidget>,
    },
}

pub type NativeUiWidgetId = u32;

pub fn from_dbus(value: DBusUiPropertyContainer) -> anyhow::Result<HashMap<String, NativeUiPropertyValue>> {
    let result = value.0
        .into_iter()
        .map(|(key, DBusUiPropertyValue(value_type, value))| {
            let value = match &(value_type, value.into()) {
                (DBusUiPropertyValueType::String, Value::Str(value)) => NativeUiPropertyValue::String(value.to_string()),
                (DBusUiPropertyValueType::Number, Value::F64(value)) => NativeUiPropertyValue::Number(value.to_owned()),
                (DBusUiPropertyValueType::Bool, Value::Bool(value)) => NativeUiPropertyValue::Bool(value.to_owned()),
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
    pub widget_type: String,
    pub widget_properties: HashMap<String, NativeUiPropertyValue>,
    pub widget_children: Vec<NativeUiWidget>,
}

impl TryFrom<DBusUiWidget> for NativeUiWidget {
    type Error = anyhow::Error;

    fn try_from(value: DBusUiWidget) -> anyhow::Result<Self> {
        let children = value.widget_children.into_iter()
            .map(|child| child.try_into())
            .collect::<anyhow::Result<Vec<NativeUiWidget>>>()?;

        Ok(Self {
            widget_id: value.widget_id,
            widget_type: value.widget_type,
            widget_properties: from_dbus(value.widget_properties)?,
            widget_children: children,
        })
    }
}
