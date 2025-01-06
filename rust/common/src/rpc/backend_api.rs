use std::collections::HashMap;
use thiserror::Error;
use tonic::{Code, Request};
use tonic::transport::Channel;

use gauntlet_utils::channel::{RequestError, RequestSender};

use crate::model::{BackendRequestData, BackendResponseData, DownloadStatus, EntrypointId, KeyboardEventOrigin, LocalSaveData, PhysicalKey, PhysicalShortcut, PluginId, PluginPreferenceUserData, SearchResult, SettingsEntrypoint, SettingsEntrypointType, SettingsPlugin, SettingsTheme, UiPropertyValue, UiSetupData, UiWidgetId};
use crate::rpc::grpc::{RpcDownloadPluginRequest, RpcDownloadStatus, RpcDownloadStatusRequest, RpcEntrypointTypeSettings, RpcGetGlobalShortcutRequest, RpcGetThemeRequest, RpcPingRequest, RpcPluginsRequest, RpcRemovePluginRequest, RpcSaveLocalPluginRequest, RpcSetEntrypointStateRequest, RpcSetGlobalShortcutRequest, RpcSetPluginStateRequest, RpcSetPreferenceValueRequest, RpcSetThemeRequest, RpcShortcut, RpcShowSettingsWindowRequest, RpcShowWindowRequest};
use crate::rpc::grpc::rpc_backend_client::RpcBackendClient;
use crate::rpc::grpc_convert::{plugin_preference_from_rpc, plugin_preference_user_data_from_rpc, plugin_preference_user_data_to_rpc};

#[derive(Error, Debug, Clone)]
pub enum BackendForFrontendApiError {
    #[error("Frontend wasn't able to process request in a timely manner")]
    TimeoutError,
    #[error("Internal Error: {display:?}")]
    Internal {
        display: String
    },
}

impl From<RequestError> for BackendForFrontendApiError {
    fn from(error: RequestError) -> BackendForFrontendApiError {
        match error {
            RequestError::TimeoutError => BackendForFrontendApiError::TimeoutError,
            RequestError::OtherSideWasDropped => BackendForFrontendApiError::Internal { display: "other side was dropped".to_string() }
        }
    }
}

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

    pub async fn setup_data(&mut self) -> Result<UiSetupData, BackendForFrontendApiError> {
        let request = BackendRequestData::Setup;

        let BackendResponseData::SetupData { data } = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(data)
    }

    pub async fn setup_response(&mut self, global_shortcut_error: Option<String>) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::SetupResponse {
            global_shortcut_error
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn search(&mut self, text: String, render_inline_view: bool) -> Result<Vec<SearchResult>, BackendForFrontendApiError> {
        let request = BackendRequestData::Search {
            text,
            render_inline_view,
        };

        let BackendResponseData::Search { results } = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(results)
    }

    pub async fn request_view_render(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Result<HashMap<String, PhysicalShortcut>, BackendForFrontendApiError> {
        let request = BackendRequestData::RequestViewRender {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::RequestViewRender { shortcuts } = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(shortcuts)
    }

    pub async fn request_view_close(&mut self, plugin_id: PluginId) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::RequestViewClose {
            plugin_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn request_run_command(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::RequestRunCommand {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn request_run_generated_command(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, action_index: usize) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::RequestRunGeneratedCommand {
            plugin_id,
            entrypoint_id,
            action_index,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
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
    ) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::SendViewEvent {
            plugin_id,
            widget_id,
            event_name,
            event_arguments,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn send_keyboard_event(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        origin: KeyboardEventOrigin,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::SendKeyboardEvent {
            plugin_id,
            entrypoint_id,
            origin,
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn send_open_event(&mut self, plugin_id: PluginId, href: String) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::SendOpenEvent {
            plugin_id,
            href,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn open_settings_window(&mut self, ) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::OpenSettingsWindow;

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn open_settings_window_preferences(&mut self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> Result<(), BackendForFrontendApiError> {
        let request = BackendRequestData::OpenSettingsWindowPreferences {
            plugin_id,
            entrypoint_id,
        };

        let BackendResponseData::Nothing = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn inline_view_shortcuts(&self) -> Result<HashMap<PluginId, HashMap<String, PhysicalShortcut>>, BackendForFrontendApiError> {
        let request = BackendRequestData::InlineViewShortcuts;

        let BackendResponseData::InlineViewShortcuts { shortcuts } = self.backend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(shortcuts)
    }
}

#[derive(Error, Debug, Clone)]
pub enum BackendApiError {
    #[error("Timeout Error")]
    Timeout,
    #[error("Internal Backend Error: {display:?}")]
    Internal {
        display: String
    },
}

impl From<tonic::Status> for BackendApiError {
    fn from(error: tonic::Status) -> BackendApiError {
        match error.code() {
            Code::Ok => unreachable!(),
            Code::DeadlineExceeded => BackendApiError::Timeout,
            _ => BackendApiError::Internal {
                display: format!("{}", error)
            }
        }

    }
}

impl From<prost::UnknownEnumValue> for BackendApiError {
    fn from(error: prost::UnknownEnumValue) -> BackendApiError {
        BackendApiError::Internal {
            display: format!("{}", error)
        }
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

    pub async fn ping(&mut self) -> Result<(), BackendApiError> {
        let _ = self.client.ping(Request::new(RpcPingRequest::default()))
            .await?;

        Ok(())
    }

    pub async fn show_window(&mut self) -> Result<(), BackendApiError> {
        let _ = self.client.show_window(Request::new(RpcShowWindowRequest::default()))
            .await?;

        Ok(())
    }

    pub async fn show_settings_window(&mut self) -> Result<(), BackendApiError> {
        let _ = self.client.show_settings_window(Request::new(RpcShowSettingsWindowRequest::default()))
            .await?;

        Ok(())
    }

    pub async fn plugins(&mut self) -> Result<HashMap<PluginId, SettingsPlugin>, BackendApiError> {
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
                            RpcEntrypointTypeSettings::SEntrypointGenerator => SettingsEntrypointType::EntrypointGenerator
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

    pub async fn set_plugin_state(&mut self, plugin_id: PluginId, enabled: bool) -> Result<(), BackendApiError> {
        let request = RpcSetPluginStateRequest {
            plugin_id: plugin_id.to_string(),
            enabled,
        };

        self.client.set_plugin_state(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> Result<(), BackendApiError> {
        let request = RpcSetEntrypointStateRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.to_string(),
            enabled,
        };

        self.client.set_entrypoint_state(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn set_global_shortcut(&mut self, shortcut: Option<PhysicalShortcut>) -> Result<(), BackendApiError> {
        let request = RpcSetGlobalShortcutRequest {
            shortcut: shortcut.map(|shortcut| {
                RpcShortcut {
                    physical_key: shortcut.physical_key.to_value(),
                    modifier_shift: shortcut.modifier_shift,
                    modifier_control: shortcut.modifier_control,
                    modifier_alt: shortcut.modifier_alt,
                    modifier_meta: shortcut.modifier_meta,
                }
            })
        };

        self.client.set_global_shortcut(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn get_global_shortcut(&mut self) -> Result<(Option<PhysicalShortcut>, Option<String>), BackendApiError> {
        let response = self.client.get_global_shortcut(Request::new(RpcGetGlobalShortcutRequest::default()))
            .await?;

        let response = response.into_inner();

        Ok((
            response.shortcut
                .map(|shortcut| {
                    PhysicalShortcut {
                        physical_key: PhysicalKey::from_value(shortcut.physical_key),
                        modifier_shift: shortcut.modifier_shift,
                        modifier_control: shortcut.modifier_control,
                        modifier_alt: shortcut.modifier_alt,
                        modifier_meta: shortcut.modifier_meta,
                    }
                }),
            response.error
        ))
    }

    pub async fn set_theme(&mut self, theme: SettingsTheme) -> Result<(), BackendApiError> {
        let theme = match theme {
            SettingsTheme::AutoDetect => "AutoDetect",
            SettingsTheme::ThemeFile => "ThemeFile",
            SettingsTheme::Config => "Config",
            SettingsTheme::MacOSLight => "MacOSLight",
            SettingsTheme::MacOSDark => "MacOSDark",
            SettingsTheme::Legacy => "Legacy",
        };

        let request = RpcSetThemeRequest {
            theme: theme.to_string()
        };

        self.client.set_theme(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn get_theme(&mut self) -> Result<SettingsTheme, BackendApiError> {
        let response = self.client.get_theme(Request::new(RpcGetThemeRequest::default()))
            .await?;

        let theme = response.into_inner().theme;

        let theme = match theme.as_str() {
            "AutoDetect" => SettingsTheme::AutoDetect,
            "ThemeFile" => SettingsTheme::ThemeFile,
            "Config" => SettingsTheme::Config,
            "MacOSLight" => SettingsTheme::MacOSLight,
            "MacOSDark" => SettingsTheme::MacOSDark,
            "Legacy" => SettingsTheme::Legacy,
            _ => unreachable!()
        };

        Ok(theme)
    }

    pub async fn set_preference_value(&mut self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, id: String, user_data: PluginPreferenceUserData) -> Result<(), BackendApiError> {
        let request = RpcSetPreferenceValueRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.map(|id| id.to_string()).unwrap_or_default(),
            preference_id: id,
            preference_value: Some(plugin_preference_user_data_to_rpc(user_data)),
        };

        self.client.set_preference_value(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn download_plugin(&mut self, plugin_id: PluginId) -> Result<(), BackendApiError> {
        let request = RpcDownloadPluginRequest {
            plugin_id: plugin_id.to_string()
        };

        self.client.download_plugin(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn download_status(&mut self) -> Result<HashMap<PluginId, DownloadStatus>, BackendApiError> {
        let plugins = self.client.download_status(Request::new(RpcDownloadStatusRequest::default()))
            .await?
            .into_inner()
            .status_per_plugin
            .into_iter()
            .map(|(plugin_id, status)| {
                let plugin_id = PluginId::from_string(plugin_id);

                let status = match status.status.try_into()? {
                    RpcDownloadStatus::InProgress => DownloadStatus::InProgress,
                    RpcDownloadStatus::Done => DownloadStatus::Done,
                    RpcDownloadStatus::Failed => DownloadStatus::Failed { message: status.message },
                };

                Ok::<(PluginId, DownloadStatus), BackendApiError>((plugin_id, status))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(plugins)
    }

    pub async fn remove_plugin(&mut self, plugin_id: PluginId) -> Result<(), BackendApiError> {
        let request = RpcRemovePluginRequest { plugin_id: plugin_id.to_string() };

        self.client.remove_plugin(Request::new(request))
            .await?;

        Ok(())
    }

    pub async fn save_local_plugin(&mut self, path: String) -> Result<LocalSaveData, BackendApiError> {
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
