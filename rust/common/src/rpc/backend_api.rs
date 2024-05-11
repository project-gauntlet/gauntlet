use std::collections::HashMap;

use tonic::Request;
use tonic::transport::Channel;

use crate::model::{ActionShortcut, ActionShortcutKind, EntrypointId, PluginId, PluginPreference, PluginPreferenceUserData, PreferenceEnumValue, SettingsEntrypoint, SettingsEntrypointType, SettingsPlugin, UiPropertyValue, UiSearchResult, UiSearchResultEntrypointType, UiWidgetId};
use crate::rpc::convert::{plugin_preference_user_data_from_rpc, plugin_preference_user_data_to_rpc, ui_property_value_to_rpc};
use crate::rpc::grpc::rpc_backend_client::RpcBackendClient;
use crate::rpc::grpc::{RpcDownloadPluginRequest, RpcDownloadStatus, RpcDownloadStatusRequest, RpcEntrypointTypeSearchResult, RpcEntrypointTypeSettings, RpcEventKeyboardEvent, RpcEventRenderView, RpcEventRunCommand, RpcEventRunGeneratedCommand, RpcEventViewEvent, RpcOpenSettingsWindowPreferencesRequest, RpcOpenSettingsWindowRequest, RpcPluginPreference, RpcPluginPreferenceValueType, RpcPluginsRequest, RpcRemovePluginRequest, RpcRequestRunCommandRequest, RpcRequestRunGeneratedCommandRequest, RpcRequestViewRenderRequest, RpcRequestViewRenderResponseActionKind, RpcSaveLocalPluginRequest, RpcSearchRequest, RpcSendKeyboardEventRequest, RpcSendOpenEventRequest, RpcSendViewEventRequest, RpcSetEntrypointStateRequest, RpcSetPluginStateRequest, RpcSetPreferenceValueRequest, RpcUiWidgetId};
use crate::rpc::grpc::rpc_ui_property_value::Value;

#[derive(Debug, Clone)]
pub struct BackendApi {
    client: RpcBackendClient<Channel>
}

impl BackendApi {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: RpcBackendClient::connect("http://127.0.0.1:42320").await?
        })
    }

    pub async fn search(&mut self, text: String) -> anyhow::Result<Vec<UiSearchResult>> {
        let request = RpcSearchRequest { text };

        let search_result = self.client.search(Request::new(request))
            .await?
            .into_inner()
            .results
            .into_iter()
            .map(|search_result| {
                let entrypoint_type = search_result.entrypoint_type
                    .try_into()
                    .unwrap();

                let entrypoint_type = match entrypoint_type {
                    RpcEntrypointTypeSearchResult::SrCommand => UiSearchResultEntrypointType::Command,
                    RpcEntrypointTypeSearchResult::SrView => UiSearchResultEntrypointType::View,
                    RpcEntrypointTypeSearchResult::SrGeneratedCommand => UiSearchResultEntrypointType::GeneratedCommand,
                };

                let icon_path = Some(search_result.entrypoint_icon_path)
                    .filter(|path| path != "");

                UiSearchResult {
                    plugin_id: PluginId::from_string(search_result.plugin_id),
                    plugin_name: search_result.plugin_name,
                    entrypoint_id: EntrypointId::from_string(search_result.entrypoint_id),
                    entrypoint_name: search_result.entrypoint_name,
                    entrypoint_icon: icon_path,
                    entrypoint_type,
                }
            })
            .collect();

        Ok(search_result)
    }

    pub async fn request_view_render(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, ActionShortcut>> {
        let event = RpcEventRenderView {
            entrypoint_id: entrypoint_id.to_string(),
        };

        let request = RpcRequestViewRenderRequest {
            plugin_id: plugin_id.to_string(),
            event: Some(event),
        };

        let action_shortcuts = self.client.request_view_render(Request::new(request))
            .await?
            .into_inner()
            .action_shortcuts
            .into_iter()
            .map(|(id, value)| {
                let key = value.key;
                let kind = RpcRequestViewRenderResponseActionKind::try_from(value.kind)
                    .unwrap();

                let kind = match kind {
                    RpcRequestViewRenderResponseActionKind::Main => ActionShortcutKind::Main,
                    RpcRequestViewRenderResponseActionKind::Alternative => ActionShortcutKind::Alternative
                };

                (id, ActionShortcut { key, kind })
            })
            .collect::<HashMap<_, _>>();

        Ok(action_shortcuts)
    }

    pub async fn request_run_command(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        let event = RpcEventRunCommand {
            entrypoint_id: entrypoint_id.to_string(),
        };

        let request = RpcRequestRunCommandRequest {
            plugin_id: plugin_id.to_string(),
            event: Some(event),
        };

        self.client.request_run_command(Request::new(request)).await?;

        Ok(())
    }

    pub async fn request_run_generated_command(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        let event = RpcEventRunGeneratedCommand {
            entrypoint_id: entrypoint_id.to_string(),
        };

        let request = RpcRequestRunGeneratedCommandRequest {
            plugin_id: plugin_id.to_string(),
            event: Some(event),
        };

        self.client.request_run_generated_command(Request::new(request)).await?;

        Ok(())
    }

    pub async fn send_view_event(
        &mut self,
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>
    ) -> anyhow::Result<()> {
        let widget_id = RpcUiWidgetId { value: widget_id };
        let event_arguments = event_arguments
            .into_iter()
            .map(|value| ui_property_value_to_rpc(value))
            .collect();

        let event = RpcEventViewEvent {
            widget_id: Some(widget_id),
            event_name,
            event_arguments,
        };

        let request = RpcSendViewEventRequest {
            plugin_id: plugin_id.to_string(),
            event: Some(event),
        };

        self.client.send_view_event(Request::new(request)).await?;

        Ok(())
    }

    pub async fn send_keyboard_event(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        char: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<()> {
        let event = RpcEventKeyboardEvent {
            entrypoint_id: entrypoint_id.to_string(),
            key: char.to_string(),
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        };

        let request = RpcSendKeyboardEventRequest {
            plugin_id: plugin_id.to_string(),
            event: Some(event),
        };

        self.client.send_keyboard_event(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn send_open_event(&mut self, plugin_id: PluginId, href: String) -> anyhow::Result<()> {
        let request = RpcSendOpenEventRequest {
            plugin_id: plugin_id.to_string(),
            href,
        };

        self.client.send_open_event(Request::new(request)).await?;

        Ok(())
    }

    pub async fn plugins(&mut self) -> anyhow::Result<HashMap<PluginId, SettingsPlugin>> {
        let plugins = self.client.plugins(Request::new(RpcPluginsRequest::default()))
            .await?
            .into_inner()
            .plugins
            .into_iter()
            .map(|plugin| {
                let entrypoints: HashMap<_, _> = plugin.entrypoints
                    .into_iter()
                    .map(|entrypoint| {
                        let id = EntrypointId::from_string(entrypoint.entrypoint_id);
                        let entrypoint_type: RpcEntrypointTypeSettings = entrypoint.entrypoint_type.try_into()
                            .expect("download status failed"); // TODO proper error handling

                        let entrypoint_type = match entrypoint_type {
                            RpcEntrypointTypeSettings::SCommand => SettingsEntrypointType::Command,
                            RpcEntrypointTypeSettings::SView => SettingsEntrypointType::View,
                            RpcEntrypointTypeSettings::SInlineView => SettingsEntrypointType::InlineView,
                            RpcEntrypointTypeSettings::SCommandGenerator => SettingsEntrypointType::CommandGenerator
                        };

                        let entrypoint = SettingsEntrypoint {
                            enabled: entrypoint.enabled,
                            entrypoint_id: id.clone(),
                            entrypoint_name: entrypoint.entrypoint_name.clone(),
                            entrypoint_description: entrypoint.entrypoint_description,
                            entrypoint_type,
                            preferences: entrypoint.preferences.into_iter()
                                .map(|(key, value)| (key, plugin_preference_from_grpc(value)))
                                .collect(),
                            preferences_user_data: entrypoint.preferences_user_data.into_iter()
                                .map(|(key, value)| (key, plugin_preference_user_data_from_rpc(value)))
                                .collect(),
                        };
                        (id, entrypoint)
                    })
                    .collect();

                let id = PluginId::from_string(plugin.plugin_id);
                let plugin = SettingsPlugin {
                    plugin_id: id.clone(),
                    plugin_name: plugin.plugin_name,
                    plugin_description: plugin.plugin_description,
                    enabled: plugin.enabled,
                    entrypoints,
                    preferences: plugin.preferences.into_iter()
                        .map(|(key, value)| (key, plugin_preference_from_grpc(value)))
                        .collect(),
                    preferences_user_data: plugin.preferences_user_data.into_iter()
                        .map(|(key, value)| (key, plugin_preference_user_data_from_rpc(value)))
                        .collect(),
                };

                (id, plugin)
            })
            .collect();

        Ok(plugins)
    }

    pub async fn set_plugin_state(&mut self, plugin_id: PluginId, enabled: bool) -> anyhow::Result<()> {
        let request = RpcSetPluginStateRequest {
            plugin_id: plugin_id.to_string(),
            enabled,
        };

        self.client.set_plugin_state(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        let request = RpcSetEntrypointStateRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.to_string(),
            enabled,
        };

        self.client.set_entrypoint_state(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn set_preference_value(&mut self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, name: String, user_data: PluginPreferenceUserData) -> anyhow::Result<()> {
        let request = RpcSetPreferenceValueRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.map(|id| id.to_string()).unwrap_or_default(),
            preference_name: name,
            preference_value: Some(plugin_preference_user_data_to_rpc(user_data)),
        };

        self.client.set_preference_value(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn download_plugin(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let request = RpcDownloadPluginRequest {
            plugin_id: plugin_id.to_string()
        };

        self.client.download_plugin(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn download_status(&mut self) -> anyhow::Result<Vec<PluginId>> {
        let plugins = self.client.download_status(Request::new(RpcDownloadStatusRequest::default()))
            .await?
            .into_inner()
            .status_per_plugin
            .into_iter()
            .filter_map(|(plugin_id, status)| {
                let status: RpcDownloadStatus = status.status.try_into()
                    .expect("download status failed");

                match status {
                    RpcDownloadStatus::InProgress => None,
                    RpcDownloadStatus::Done => Some(PluginId::from_string(plugin_id)),
                    RpcDownloadStatus::Failed => Some(PluginId::from_string(plugin_id))
                }
            })
            .collect::<Vec<_>>();

        Ok(plugins)
    }

    pub async fn open_settings_window(&mut self, ) -> anyhow::Result<()> {
        self.client.open_settings_window(Request::new(RpcOpenSettingsWindowRequest::default()))
            .await?;

        Ok(())
    }

    pub async fn open_settings_window_preferences(&mut self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> anyhow::Result<()> {
        let request = RpcOpenSettingsWindowPreferencesRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.map(|val| val.to_string()).unwrap_or_default(),
        };

        self.client.open_settings_window_preferences(Request::new(request))
            .await?;

        Ok(())

    }

    pub async fn remove_plugin(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let request = RpcRemovePluginRequest { plugin_id: plugin_id.to_string() };

        self.client.remove_plugin(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn save_local_plugin(&mut self, path: String) -> anyhow::Result<()> {
        let request = RpcSaveLocalPluginRequest { path };

        self.client.save_local_plugin(Request::new(request))
            .await?;

        Ok(())
    }
}

fn plugin_preference_from_grpc(value: RpcPluginPreference) -> PluginPreference {
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