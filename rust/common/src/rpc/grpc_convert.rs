use crate::model::PluginPreference;
use crate::model::PluginPreferenceUserData;
use crate::model::PreferenceEnumValue;
use crate::rpc::grpc::rpc_ui_property_value::Value;
use crate::rpc::grpc::RpcEnumValue;
use crate::rpc::grpc::RpcPluginPreference;
use crate::rpc::grpc::RpcPluginPreferenceUserData;
use crate::rpc::grpc::RpcPluginPreferenceValueType;
use crate::rpc::grpc::RpcUiPropertyValue;

pub fn plugin_preference_user_data_from_rpc(value: RpcPluginPreferenceUserData) -> PluginPreferenceUserData {
    let value_type: RpcPluginPreferenceValueType = value.r#type.try_into().unwrap();
    match value_type {
        RpcPluginPreferenceValueType::Number => {
            let value = value.value.map(|value| {
                match value.value.unwrap() {
                    Value::Number(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreferenceUserData::Number { value }
        }
        RpcPluginPreferenceValueType::String => {
            let value = value.value.map(|value| {
                match value.value.unwrap() {
                    Value::String(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreferenceUserData::String { value }
        }
        RpcPluginPreferenceValueType::Enum => {
            let value = value.value.map(|value| {
                match value.value.unwrap() {
                    Value::String(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreferenceUserData::Enum { value }
        }
        RpcPluginPreferenceValueType::Bool => {
            let value = value.value.map(|value| {
                match value.value.unwrap() {
                    Value::Bool(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreferenceUserData::Bool { value }
        }
        RpcPluginPreferenceValueType::ListOfStrings => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value
                        .value_list
                        .into_iter()
                        .flat_map(|value| {
                            value.value.map(|value| {
                                match value {
                                    Value::String(value) => value,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect();

                    Some(value_list)
                }
                false => None,
            };

            PluginPreferenceUserData::ListOfStrings { value }
        }
        RpcPluginPreferenceValueType::ListOfNumbers => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value
                        .value_list
                        .into_iter()
                        .flat_map(|value| {
                            value.value.map(|value| {
                                match value {
                                    Value::Number(value) => value,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect();

                    Some(value_list)
                }
                false => None,
            };

            PluginPreferenceUserData::ListOfNumbers { value }
        }
        RpcPluginPreferenceValueType::ListOfEnums => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value
                        .value_list
                        .into_iter()
                        .flat_map(|value| {
                            value.value.map(|value| {
                                match value {
                                    Value::String(value) => value,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect();

                    Some(value_list)
                }
                false => None,
            };

            PluginPreferenceUserData::ListOfEnums { value }
        }
    }
}

pub fn plugin_preference_user_data_to_rpc(value: PluginPreferenceUserData) -> RpcPluginPreferenceUserData {
    match value {
        PluginPreferenceUserData::Number { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Number.into(),
                value: value.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::Number(value)),
                    }
                }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::String { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::String.into(),
                value: value.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::String(value)),
                    }
                }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::Enum { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Enum.into(),
                value: value.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::String(value)),
                    }
                }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::Bool { value } => {
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::Bool.into(),
                value: value.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::Bool(value)),
                    }
                }),
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::ListOfStrings { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfStrings.into(),
                value_list: value
                    .map(|value| {
                        value
                            .into_iter()
                            .map(|value| {
                                RpcUiPropertyValue {
                                    value: Some(Value::String(value)),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::ListOfNumbers { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfNumbers.into(),
                value_list: value
                    .map(|value| {
                        value
                            .into_iter()
                            .map(|value| {
                                RpcUiPropertyValue {
                                    value: Some(Value::Number(value)),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
        PluginPreferenceUserData::ListOfEnums { value } => {
            let exists = value.is_some();
            RpcPluginPreferenceUserData {
                r#type: RpcPluginPreferenceValueType::ListOfEnums.into(),
                value_list: value
                    .map(|value| {
                        value
                            .into_iter()
                            .map(|value| {
                                RpcUiPropertyValue {
                                    value: Some(Value::String(value)),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]),
                value_list_exists: exists,
                ..RpcPluginPreferenceUserData::default()
            }
        }
    }
}

pub fn plugin_preference_to_rpc(value: PluginPreference) -> RpcPluginPreference {
    match value {
        PluginPreference::Number {
            name,
            default,
            description,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Number.into(),
                default: default.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::Number(value)),
                    }
                }),
                name,
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::String {
            name,
            default,
            description,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::String.into(),
                default: default.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::String(value)),
                    }
                }),
                name,
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::Enum {
            name,
            default,
            description,
            enum_values,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Enum.into(),
                default: default.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::String(value)),
                    }
                }),
                name,
                description,
                enum_values: enum_values
                    .into_iter()
                    .map(|value| {
                        RpcEnumValue {
                            label: value.label,
                            value: value.value,
                        }
                    })
                    .collect(),
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::Bool {
            name,
            default,
            description,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Bool.into(),
                default: default.map(|value| {
                    RpcUiPropertyValue {
                        value: Some(Value::Bool(value)),
                    }
                }),
                name,
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::ListOfStrings {
            name,
            default,
            description,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfStrings.into(),
                default_list: default
                    .map(|value| {
                        value
                            .into_iter()
                            .map(|value| {
                                RpcUiPropertyValue {
                                    value: Some(Value::String(value)),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]),
                name,
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::ListOfNumbers {
            name,
            default,
            description,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfNumbers.into(),
                default_list: default
                    .map(|value| {
                        value
                            .into_iter()
                            .map(|value| {
                                RpcUiPropertyValue {
                                    value: Some(Value::Number(value)),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]),
                name,
                description,
                ..RpcPluginPreference::default()
            }
        }
        PluginPreference::ListOfEnums {
            name,
            default,
            enum_values,
            description,
        } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfEnums.into(),
                default_list: default
                    .map(|value| {
                        value
                            .into_iter()
                            .map(|value| {
                                RpcUiPropertyValue {
                                    value: Some(Value::String(value)),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or(vec![]),
                name,
                description,
                enum_values: enum_values
                    .into_iter()
                    .map(|value| {
                        RpcEnumValue {
                            label: value.label,
                            value: value.value,
                        }
                    })
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
            let default = value.default.map(|value| {
                match value.value.unwrap() {
                    Value::Number(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreference::Number {
                default,
                name: value.name,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::String => {
            let default = value.default.map(|value| {
                match value.value.unwrap() {
                    Value::String(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreference::String {
                default,
                name: value.name,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::Enum => {
            let default = value.default.map(|value| {
                match value.value.unwrap() {
                    Value::String(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreference::Enum {
                default,
                name: value.name,
                description: value.description,
                enum_values: value
                    .enum_values
                    .into_iter()
                    .map(|value| {
                        PreferenceEnumValue {
                            label: value.label,
                            value: value.value,
                        }
                    })
                    .collect(),
            }
        }
        RpcPluginPreferenceValueType::Bool => {
            let default = value.default.map(|value| {
                match value.value.unwrap() {
                    Value::Bool(value) => value,
                    _ => unreachable!(),
                }
            });

            PluginPreference::Bool {
                default,
                name: value.name,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfStrings => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value
                        .default_list
                        .into_iter()
                        .flat_map(|value| {
                            value.value.map(|value| {
                                match value {
                                    Value::String(value) => value,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect();

                    Some(default_list)
                }
                false => None,
            };

            PluginPreference::ListOfStrings {
                default: default_list,
                name: value.name,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfNumbers => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value
                        .default_list
                        .into_iter()
                        .flat_map(|value| {
                            value.value.map(|value| {
                                match value {
                                    Value::Number(value) => value,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect();

                    Some(default_list)
                }
                false => None,
            };

            PluginPreference::ListOfNumbers {
                default: default_list,
                name: value.name,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfEnums => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value
                        .default_list
                        .into_iter()
                        .flat_map(|value| {
                            value.value.map(|value| {
                                match value {
                                    Value::String(value) => value,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect();

                    Some(default_list)
                }
                false => None,
            };

            PluginPreference::ListOfEnums {
                default: default_list,
                name: value.name,
                enum_values: value
                    .enum_values
                    .into_iter()
                    .map(|value| {
                        PreferenceEnumValue {
                            label: value.label,
                            value: value.value,
                        }
                    })
                    .collect(),
                description: value.description,
            }
        }
    }
}
