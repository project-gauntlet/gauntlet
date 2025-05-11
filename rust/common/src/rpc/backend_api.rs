use std::collections::HashMap;
use std::sync::Arc;

use gauntlet_utils::channel::RequestResult;
use gauntlet_utils_macros::boundary_gen;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Request;

use crate::model::DownloadStatus;
use crate::model::EntrypointId;
use crate::model::KeyboardEventOrigin;
use crate::model::LocalSaveData;
use crate::model::PhysicalKey;
use crate::model::PhysicalShortcut;
use crate::model::PluginId;
use crate::model::PluginPreferenceUserData;
use crate::model::SearchResult;
use crate::model::SettingsPlugin;
use crate::model::SettingsTheme;
use crate::model::UiPropertyValue;
use crate::model::UiSetupData;
use crate::model::UiWidgetId;
use crate::model::WindowPositionMode;
use crate::rpc::grpc::rpc_backend_client::RpcBackendClient;
use crate::rpc::grpc::RpcBincode;
use crate::rpc::grpc::RpcSaveLocalPluginRequest;

#[allow(async_fn_in_trait)]
#[boundary_gen(in_process)]
pub trait BackendForFrontendApi {
    async fn setup_data(&self) -> RequestResult<UiSetupData>;

    async fn setup_response(
        &self,
        global_shortcut_error: Option<String>,
        global_entrypoint_shortcuts_errors: HashMap<(PluginId, EntrypointId), Option<String>>,
    ) -> RequestResult<()>;

    async fn search(&self, text: String, render_inline_view: bool) -> RequestResult<Vec<SearchResult>>;

    async fn request_view_render(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    ) -> RequestResult<HashMap<String, PhysicalShortcut>>;

    async fn request_view_close(&self, plugin_id: PluginId) -> RequestResult<()>;

    async fn request_run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> RequestResult<()>;

    async fn request_run_generated_entrypoint(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    ) -> RequestResult<()>;

    async fn send_view_event(
        &self,
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    ) -> RequestResult<()>;

    async fn send_keyboard_event(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        origin: KeyboardEventOrigin,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> RequestResult<()>;

    async fn send_open_event(&self, plugin_id: PluginId, href: String) -> RequestResult<()>;

    async fn open_settings_window(&self) -> RequestResult<()>;

    async fn open_settings_window_preferences(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
    ) -> RequestResult<()>;

    async fn inline_view_shortcuts(&self) -> RequestResult<HashMap<PluginId, HashMap<String, PhysicalShortcut>>>;

    async fn run_entrypoint(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> RequestResult<()>;
}

#[boundary_gen(bincode, grpc)]
#[tonic::async_trait]
pub trait BackendForCliApi {
    async fn ping(&self) -> RequestResult<()>;

    async fn show_window(&self) -> RequestResult<()>;

    async fn show_settings_window(&self) -> RequestResult<()>;

    async fn run_action(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_id: String,
    ) -> RequestResult<()>;
}

#[tonic::async_trait]
pub trait BackendForToolsApi {
    async fn save_local_plugin(&self, path: String) -> RequestResult<LocalSaveData>;
}

#[boundary_gen(bincode, grpc)]
#[tonic::async_trait]
pub trait BackendForSettingsApi {
    async fn plugins(&self) -> RequestResult<HashMap<PluginId, SettingsPlugin>>;

    async fn set_plugin_state(&self, plugin_id: PluginId, enabled: bool) -> RequestResult<()>;

    async fn set_entrypoint_state(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        enabled: bool,
    ) -> RequestResult<()>;

    async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> RequestResult<Option<String>>;

    async fn get_global_shortcut(&self) -> RequestResult<(Option<PhysicalShortcut>, Option<String>)>;

    async fn set_global_entrypoint_shortcut(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut: Option<PhysicalShortcut>,
    ) -> RequestResult<()>;

    async fn get_global_entrypoint_shortcuts(
        &self,
    ) -> RequestResult<HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>>;

    async fn set_entrypoint_search_alias(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        alias: Option<String>,
    ) -> RequestResult<()>;

    async fn get_entrypoint_search_aliases(&self) -> RequestResult<HashMap<(PluginId, EntrypointId), String>>;

    async fn set_theme(&self, theme: SettingsTheme) -> RequestResult<()>;

    async fn get_theme(&self) -> RequestResult<SettingsTheme>;

    async fn set_window_position_mode(&self, mode: WindowPositionMode) -> RequestResult<()>;

    async fn get_window_position_mode(&self) -> RequestResult<WindowPositionMode>;

    async fn set_preference_value(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        preference_id: String,
        preference_value: PluginPreferenceUserData,
    ) -> RequestResult<()>;

    async fn download_plugin(&self, plugin_id: PluginId) -> RequestResult<()>;

    async fn download_status(&self) -> RequestResult<HashMap<PluginId, DownloadStatus>>;

    async fn remove_plugin(&self, plugin_id: PluginId) -> RequestResult<()>;
}

#[derive(Debug, Clone)]
pub struct GrpcBackendApi {
    client: Arc<Mutex<RpcBackendClient<Channel>>>,
}

impl GrpcBackendApi {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Arc::new(Mutex::new(RpcBackendClient::connect("http://127.0.0.1:42320").await?)),
        })
    }

    pub async fn backend_for_settings_api(&self, bytes: Vec<u8>) -> RequestResult<Vec<u8>> {
        let request = RpcBincode { data: bytes };

        let mut client = self.client.lock().await;

        let response = client
            .backend_for_settings_api(Request::new(request))
            .await?
            .into_inner()
            .data;

        Ok(response)
    }

    pub async fn backend_for_cli_api(&self, bytes: Vec<u8>) -> RequestResult<Vec<u8>> {
        let request = RpcBincode { data: bytes };

        let mut client = self.client.lock().await;

        let response = client
            .backend_for_cli_api(Request::new(request))
            .await?
            .into_inner()
            .data;

        Ok(response)
    }

    pub async fn save_local_plugin(&self, path: String) -> RequestResult<LocalSaveData> {
        let request = RpcSaveLocalPluginRequest { path };

        let mut client = self.client.lock().await;

        let response = client.save_local_plugin(Request::new(request)).await?.into_inner();

        Ok(LocalSaveData {
            stdout_file_path: response.stdout_file_path,
            stderr_file_path: response.stderr_file_path,
        })
    }
}
