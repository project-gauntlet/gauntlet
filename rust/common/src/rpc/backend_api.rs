use std::collections::HashMap;

use tonic::Request;
use tonic::transport::Channel;

use utils::channel::RequestSender;

use crate::model::{BackendRequestData, BackendResponseData, EntrypointId, LocalSaveData, PhysicalKey, PhysicalShortcut, PluginId, PluginPreferenceUserData, SearchResult, SettingsEntrypoint, SettingsEntrypointType, SettingsPlugin, UiPropertyValue, UiWidgetId};
use crate::rpc::grpc::{RpcDownloadPluginRequest, RpcDownloadStatus, RpcDownloadStatusRequest, RpcEntrypointTypeSettings, RpcGetGlobalShortcutRequest, RpcPingRequest, RpcPluginsRequest, RpcRemovePluginRequest, RpcSaveLocalPluginRequest, RpcSetEntrypointStateRequest, RpcSetGlobalShortcutRequest, RpcSetPluginStateRequest, RpcSetPreferenceValueRequest, RpcShowWindowRequest};
use crate::rpc::grpc::rpc_backend_client::RpcBackendClient;
use crate::rpc::grpc_convert::{plugin_preference_from_rpc, plugin_preference_user_data_from_rpc, plugin_preference_user_data_to_rpc};

#[derive(Debug, Clone)]
pub struct BackendForFrontendApi {
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>
}

impl BackendForFrontendApi {
    pub fn new(backend_sender: RequestSender<BackendRequestData, BackendResponseData>) -> Self {
        Self {
            backend_sender
        }
    }

    pub async fn search(&mut self, text: String, render_inline_view: bool) -> anyhow::Result<Vec<SearchResult>> {
        let request = BackendRequestData::Search {
            text,
            render_inline_view,
        };

        let BackendResponseData::Search { results } = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(results)
    }

    pub async fn request_view_render(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        let request = BackendRequestData::RequestViewRender {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::RequestViewRender { shortcuts } = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(shortcuts)
    }

    pub async fn request_view_close(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let request = BackendRequestData::RequestViewClose {
            plugin_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn request_run_command(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        let request = BackendRequestData::RequestRunCommand {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn request_run_generated_command(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        let request = BackendRequestData::RequestRunGeneratedCommand {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn send_view_event(
        &mut self,
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>
    ) -> anyhow::Result<()> {
        let request = BackendRequestData::SendViewEvent {
            plugin_id,
            widget_id,
            event_name,
            event_arguments,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn send_keyboard_event(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<()> {
        let request = BackendRequestData::SendKeyboardEvent {
            plugin_id,
            entrypoint_id,
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn send_open_event(&mut self, plugin_id: PluginId, href: String) -> anyhow::Result<()> {
        let request = BackendRequestData::SendOpenEvent {
            plugin_id,
            href,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn open_settings_window(&mut self, ) -> anyhow::Result<()> {
        let request = BackendRequestData::OpenSettingsWindow;

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn open_settings_window_preferences(&mut self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> anyhow::Result<()> {
        let request = BackendRequestData::OpenSettingsWindowPreferences {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await else {
            unreachable!()
        };

        Ok(())
    }
}

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

    pub async fn ping(&mut self) -> anyhow::Result<()> {
        let _ = self.client.ping(Request::new(RpcPingRequest::default()))
            .await?;

        Ok(())
    }

    pub async fn show_window(&mut self) -> anyhow::Result<()> {
        let _ = self.client.show_window(Request::new(RpcShowWindowRequest::default()))
            .await?;

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
                                .map(|(key, value)| (key, plugin_preference_from_rpc(value)))
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
                        .map(|(key, value)| (key, plugin_preference_from_rpc(value)))
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

    pub async fn set_global_shortcut(&mut self, shortcut: PhysicalShortcut) -> anyhow::Result<()> {
        let request = RpcSetGlobalShortcutRequest {
            physical_key: shortcut.physical_key.to_value(),
            modifier_shift: shortcut.modifier_shift,
            modifier_control: shortcut.modifier_control,
            modifier_alt: shortcut.modifier_alt,
            modifier_meta: shortcut.modifier_meta,
        };

        self.client.set_global_shortcut(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn get_global_shortcut(&mut self) -> anyhow::Result<PhysicalShortcut> {
        let response = self.client.get_global_shortcut(Request::new(RpcGetGlobalShortcutRequest::default()))
            .await?;

        let response = response.into_inner();

        Ok(PhysicalShortcut {
            physical_key: PhysicalKey::from_value(response.physical_key),
            modifier_shift: response.modifier_shift,
            modifier_control: response.modifier_control,
            modifier_alt: response.modifier_alt,
            modifier_meta: response.modifier_meta,
        })
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

    pub async fn remove_plugin(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let request = RpcRemovePluginRequest { plugin_id: plugin_id.to_string() };

        self.client.remove_plugin(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn save_local_plugin(&mut self, path: String) -> anyhow::Result<LocalSaveData> {
        let request = RpcSaveLocalPluginRequest { path };

        let response = self.client.save_local_plugin(Request::new(request))
            .await?
            .into_inner();

        Ok(LocalSaveData {
            stdout_file_path: response.stdout_file_path,
            stderr_file_path: response.stderr_file_path,
        })
    }
}
