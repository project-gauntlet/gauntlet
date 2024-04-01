use std::fmt::Debug;
use serde::{Deserialize, Serialize};

use tonic::transport::Channel;

use crate::rpc::rpc_ui_property_value::Value;

tonic::include_proto!("_");

pub type FrontendClient = rpc_frontend_client::RpcFrontendClient<Channel>;
pub type BackendClient = rpc_backend_client::RpcBackendClient<Channel>;


#[derive(Debug)]
pub enum RpcNoProtoBufPluginPreferenceUserData {
    Number {
        value: Option<f64>,
    },
    String {
        value: Option<String>,
    },
    Enum {
        value: Option<String>,
    },
    Bool {
        value: Option<bool>,
    },
    ListOfStrings {
        value: Option<Vec<String>>,
    },
    ListOfNumbers {
        value: Option<Vec<f64>>,
    },
    ListOfEnums {
        value: Option<Vec<String>>,
    },
}

pub fn plugin_preference_user_data_to_npb(value: RpcPluginPreferenceUserData) -> RpcNoProtoBufPluginPreferenceUserData {
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

            RpcNoProtoBufPluginPreferenceUserData::Number {
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

            RpcNoProtoBufPluginPreferenceUserData::String {
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

            RpcNoProtoBufPluginPreferenceUserData::Enum {
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

            RpcNoProtoBufPluginPreferenceUserData::Bool {
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

            RpcNoProtoBufPluginPreferenceUserData::ListOfStrings {
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

            RpcNoProtoBufPluginPreferenceUserData::ListOfNumbers {
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

            RpcNoProtoBufPluginPreferenceUserData::ListOfEnums {
                value,
            }
        }
    }
}

pub fn plugin_preference_user_data_from_npb(value: RpcNoProtoBufPluginPreferenceUserData) -> RpcPluginPreferenceUserData {
    match value {
        RpcNoProtoBufPluginPreferenceUserData::Number { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Number.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::Number(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        RpcNoProtoBufPluginPreferenceUserData::String { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::String.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        RpcNoProtoBufPluginPreferenceUserData::Enum { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Enum.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        RpcNoProtoBufPluginPreferenceUserData::Bool { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Bool.into(),
                value: value.map(|value| RpcUiPropertyValue { value: Some(Value::Bool(value)) }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        RpcNoProtoBufPluginPreferenceUserData::ListOfStrings { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfStrings.into(),
                value_list: value.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(Value::String(value)) }).collect()).unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
        RpcNoProtoBufPluginPreferenceUserData::ListOfNumbers { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfNumbers.into(),
                value_list: value.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(Value::Number(value)) }).collect()).unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
        RpcNoProtoBufPluginPreferenceUserData::ListOfEnums { value } => {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SettingsEnvData {
    OpenPluginPreferences {
        plugin_id: String,
    },
    OpenEntrypointPreferences {
        plugin_id: String,
        entrypoint_id: String,
    }
}

pub fn settings_env_data_to_string(data: SettingsEnvData) -> String {
    serde_json::to_string(&data).expect("unable to serialize settings env data")
}

pub fn settings_env_data_from_string(data: String) -> SettingsEnvData {
    serde_json::from_str(&data).expect("unable to serialize settings env data")
}
