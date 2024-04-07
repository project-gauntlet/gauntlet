use std::collections::HashMap;

use anyhow::anyhow;

use common::model::{EntrypointId, PluginId, PropertyValue, RenderLocation};
use common::rpc::{RpcUiPropertyValue, RpcUiWidget};
use common::rpc::rpc_ui_property_value::Value;

#[derive(Debug, Clone)]
pub struct NativeUiSearchResult {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub entrypoint_type: SearchResultEntrypointType,
}

#[derive(Debug, Clone)]
pub enum SearchResultEntrypointType {
    Command,
    View,
    GeneratedCommand,
}

#[derive(Debug)]
pub enum NativeUiResponseData {
    Nothing,
}

#[derive(Debug)]
pub enum NativeUiRequestData {
    ShowWindow,
    ClearInlineView {
        plugin_id: PluginId
    },
    ReplaceView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: RenderLocation,
        top_level_view: bool,
        container: NativeUiWidget,
    },
    ShowPreferenceRequiredView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    },
}

#[derive(Debug, Clone)]
pub enum NativeUiViewEvent {
    View {
        widget_id: NativeUiWidgetId,
        event_name: String,
        event_arguments: Vec<PropertyValue>,
    },
    Open {
        href: String
    }
}

pub type NativeUiWidgetId = u32;

pub fn from_rpc(value: HashMap<String, RpcUiPropertyValue>) -> anyhow::Result<HashMap<String, NativeUiPropertyValue>> {
    let result = value
        .into_iter()
        .map(|(key, value)| {
            let value = value.value.ok_or(anyhow!("invalid property value"))?;
            let value = match value {
                Value::String(value) => NativeUiPropertyValue::String(value),
                Value::Number(value) => NativeUiPropertyValue::Number(value),
                Value::Bool(value) => NativeUiPropertyValue::Bool(value),
                Value::Bytes(value) => NativeUiPropertyValue::Bytes(value),
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
    Bytes(Vec<u8>),
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
    pub fn as_bytes(&self) -> Option<&[u8]> {
        if let NativeUiPropertyValue::Bytes(val) = self {
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

impl TryFrom<RpcUiWidget> for NativeUiWidget {
    type Error = anyhow::Error;

    fn try_from(value: RpcUiWidget) -> anyhow::Result<Self> {
        let children = value.widget_children.into_iter()
            .map(|child| child.try_into())
            .collect::<anyhow::Result<Vec<NativeUiWidget>>>()?;

        let widget_id = value.widget_id
            .ok_or(anyhow!("invalid value widget_id"))?
            .value;

        Ok(Self {
            widget_id,
            widget_type: value.widget_type,
            widget_properties: from_rpc(value.widget_properties)?,
            widget_children: children,
        })
    }
}
