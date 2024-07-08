use std::collections::HashMap;

use anyhow::anyhow;

use crate::model::{EntrypointId, PluginId, PluginPreference, PluginPreferenceUserData, PreferenceEnumValue, UiPropertyValue, SearchResult, SearchResultEntrypointType, UiWidget, PhysicalKey};
use crate::rpc::grpc::{rpc_ui_property_value, RpcEntrypointTypeSearchResult, RpcEnumValue, RpcPluginPreference, RpcPluginPreferenceUserData, RpcPluginPreferenceValueType, RpcSearchResult, RpcUiPropertyValue, RpcUiPropertyValueObject, RpcUiWidget, RpcUiWidgetId};
use crate::rpc::grpc::rpc_ui_property_value::Value;

pub(crate) fn ui_widget_to_rpc(value: UiWidget) -> RpcUiWidget {
    let children = value.widget_children.into_iter()
        .map(|child| ui_widget_to_rpc(child))
        .collect::<Vec<RpcUiWidget>>();

    let widget_id = RpcUiWidgetId {
        value: value.widget_id
    };

    RpcUiWidget {
        widget_id: Some(widget_id),
        widget_type: value.widget_type,
        widget_properties: ui_property_values_to_rpc(value.widget_properties),
        widget_children: children
    }
}

pub(crate) fn ui_widget_from_rpc(value: RpcUiWidget) -> anyhow::Result<UiWidget> {
    let children = value.widget_children.into_iter()
        .map(|child| ui_widget_from_rpc(child))
        .collect::<anyhow::Result<Vec<UiWidget>>>()?;

    let widget_id = value.widget_id
        .ok_or(anyhow!("invalid value widget_id"))?
        .value;

    Ok(UiWidget {
        widget_id,
        widget_type: value.widget_type,
        widget_properties: ui_property_values_from_rpc(value.widget_properties)?,
        widget_children: children,
    })
}

fn ui_property_values_to_rpc(value: HashMap<String, UiPropertyValue>) -> HashMap<String, RpcUiPropertyValue> {
    value.into_iter()
        .map(|(key, value)| (key, ui_property_value_to_rpc(value)))
        .collect()
}

pub(crate) fn ui_property_value_to_rpc(value: UiPropertyValue) -> RpcUiPropertyValue {
    match value {
        UiPropertyValue::String(value) => RpcUiPropertyValue { value: Some(Value::String(value)) },
        UiPropertyValue::Number(value) => RpcUiPropertyValue { value: Some(Value::Number(value)) },
        UiPropertyValue::Bool(value) => RpcUiPropertyValue { value: Some(Value::Bool(value)) },
        UiPropertyValue::Bytes(value) => RpcUiPropertyValue { value: Some(Value::Bytes(value)) },
        UiPropertyValue::Object(value) => {
            let value: HashMap<String, _> = value.into_iter()
                .map(|(name, value)| (name, ui_property_value_to_rpc(value)))
                .collect();

            RpcUiPropertyValue { value: Some(Value::Object(RpcUiPropertyValueObject { value })) }
        }
        UiPropertyValue::Undefined => RpcUiPropertyValue { value: Some(Value::Undefined(0)) },
    }
}

fn ui_property_values_from_rpc(value: HashMap<String, RpcUiPropertyValue>) -> anyhow::Result<HashMap<String, UiPropertyValue>> {
    let result = value
        .into_iter()
        .map(|(key, value)| Ok((key, ui_property_value_from_rpc(value)?)))

        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .collect::<HashMap<_, _>>();

    Ok(result)
}

pub fn ui_property_value_from_rpc(value: RpcUiPropertyValue) -> anyhow::Result<UiPropertyValue> {
    let value = value.value.ok_or(anyhow!("invalid property value"))?;

    let value = match value {
        Value::Undefined(_) => UiPropertyValue::Undefined,
        Value::String(value) => UiPropertyValue::String(value),
        Value::Number(value) => UiPropertyValue::Number(value),
        Value::Bool(value) => UiPropertyValue::Bool(value),
        Value::Bytes(value) => UiPropertyValue::Bytes(value),
        Value::Object(value) => UiPropertyValue::Object(ui_property_values_from_rpc(value.value)?)
    };

    Ok(value)
}

pub fn plugin_preference_user_data_from_rpc(value: RpcPluginPreferenceUserData) -> PluginPreferenceUserData {
    let value_type: RpcPluginPreferenceValueType = value.r#type.try_into().unwrap();
    match value_type {
        RpcPluginPreferenceValueType::Number => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Number(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::Number {
                value
            }
        }
        RpcPluginPreferenceValueType::String => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::String {
                value
            }
        }
        RpcPluginPreferenceValueType::Enum => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::Enum {
                value
            }
        }
        RpcPluginPreferenceValueType::Bool => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Bool(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::Bool {
                value
            }
        }
        RpcPluginPreferenceValueType::ListOfStrings => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value.value_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(value_list)
                }
                false => None
            };

            PluginPreferenceUserData::ListOfStrings {
                value,
            }
        }
        RpcPluginPreferenceValueType::ListOfNumbers => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value.value_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::Number(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(value_list)
                }
                false => None
            };

            PluginPreferenceUserData::ListOfNumbers {
                value,
            }
        }
        RpcPluginPreferenceValueType::ListOfEnums => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value.value_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(value_list)
                }
                false => None
            };

            PluginPreferenceUserData::ListOfEnums {
                value,
            }
        }
    }
}

pub fn plugin_preference_user_data_to_rpc(value: PluginPreferenceUserData) -> RpcPluginPreferenceUserData {
    match value {
        PluginPreferenceUserData::Number { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Number.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::Number(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::String { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::String.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::Enum { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Enum.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::Bool { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Bool.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::Bool(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::ListOfStrings { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfStrings.into(),
                value_list: value.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }).collect()).unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::ListOfNumbers { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfNumbers.into(),
                value_list: value.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(Value::Number(value)) }).collect()).unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::ListOfEnums { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfEnums.into(),
                value_list: value.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }).collect()).unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
    }
}

pub fn plugin_preference_to_rpc(value: PluginPreference) -> RpcPluginPreference {
    match value {
        PluginPreference::Number { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Number.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::Number(value)) }),
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::String { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::String.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }),
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::Enum { default, description, enum_values } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Enum.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }),
                description,
                enum_values: enum_values.into_iter()
                    .map(|value| RpcEnumValue { label: value.label, value: value.value })
                    .collect(),
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::Bool { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Bool.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::Bool(value)) }),
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::ListOfStrings { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfStrings.into(),
                default_list: default.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }).collect()).unwrap_or(vec![]),
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::ListOfNumbers { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfNumbers.into(),
                default_list: default.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::Number(value)) }).collect()).unwrap_or(vec![]),
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::ListOfEnums { default, enum_values, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfEnums.into(),
                default_list: default.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }).collect()).unwrap_or(vec![]),
                description,
                enum_values: enum_values.into_iter()
                    .map(|value| RpcEnumValue { label: value.label, value: value.value })
                    .collect(),
                ..RpcPluginPreference::default()
            }
        }
    }
}

pub fn plugin_preference_from_rpc(value: RpcPluginPreference) -> PluginPreference {
    let value_type: RpcPluginPreferenceValueType = value.r#type.try_into().unwrap();
    match value_type {
        RpcPluginPreferenceValueType::Number => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Number(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::Number {
                default,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::String => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::String {
                default,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::Enum => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::Enum {
                default,
                description: value.description,
                enum_values: value.enum_values.into_iter()
                    .map(|value| PreferenceEnumValue { label: value.label, value: value.value })
                    .collect()
            }
        }
        RpcPluginPreferenceValueType::Bool => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Bool(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::Bool {
                default,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfStrings => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value.default_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(default_list)
                },
                false => None
            };

            PluginPreference::ListOfStrings {
                default: default_list,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfNumbers => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value.default_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::Number(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(default_list)
                },
                false => None
            };

            PluginPreference::ListOfNumbers {
                default: default_list,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfEnums => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value.default_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(default_list)
                },
                false => None
            };

            PluginPreference::ListOfEnums {
                default: default_list,
                enum_values: value.enum_values.into_iter()
                    .map(|value| PreferenceEnumValue { label: value.label, value: value.value })
                    .collect(),
                description: value.description,
            }
        }
    }
}

pub fn ui_search_result_from_rpc(search_result: RpcSearchResult) -> SearchResult {
    let entrypoint_type = search_result.entrypoint_type
        .try_into()
        .unwrap();

    let entrypoint_type = match entrypoint_type {
        RpcEntrypointTypeSearchResult::SrCommand => SearchResultEntrypointType::Command,
        RpcEntrypointTypeSearchResult::SrView => SearchResultEntrypointType::View,
        RpcEntrypointTypeSearchResult::SrGeneratedCommand => SearchResultEntrypointType::GeneratedCommand,
    };

    let icon_path = Some(search_result.entrypoint_icon_path)
        .filter(|path| path != "");

    SearchResult {
        plugin_id: PluginId::from_string(search_result.plugin_id),
        plugin_name: search_result.plugin_name,
        entrypoint_id: EntrypointId::from_string(search_result.entrypoint_id),
        entrypoint_name: search_result.entrypoint_name,
        entrypoint_icon: icon_path,
        entrypoint_type,
    }
}

pub fn ui_search_result_to_rpc(item: SearchResult) -> RpcSearchResult {
    let entrypoint_type = match item.entrypoint_type {
        SearchResultEntrypointType::Command => RpcEntrypointTypeSearchResult::SrCommand,
        SearchResultEntrypointType::View => RpcEntrypointTypeSearchResult::SrView,
        SearchResultEntrypointType::GeneratedCommand => RpcEntrypointTypeSearchResult::SrGeneratedCommand,
    };

    RpcSearchResult {
        entrypoint_type: entrypoint_type.into(),
        entrypoint_name: item.entrypoint_name,
        entrypoint_id: item.entrypoint_id.to_string(),
        entrypoint_icon_path: item.entrypoint_icon.unwrap_or_default(),
        plugin_name: item.plugin_name,
        plugin_id: item.plugin_id.to_string(),
    }
}

pub fn physical_key_from_rpc(key: String) -> PhysicalKey {
    PhysicalKey::from_value(key)
}

pub fn physical_key_to_rpc(key: PhysicalKey) -> String {
    key.to_value()
 }
