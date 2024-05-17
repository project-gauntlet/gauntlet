use std::collections::HashMap;

use common::model::{ActionShortcut, DownloadStatus, EntrypointId, PluginId, PluginPreferenceUserData, SearchResultItem, SettingsPlugin, UiPropertyValue, UiWidgetId};
use common::rpc::backend_server::{BackendServer, start_backend_server};

pub async fn start_mock_backend() {
    start_backend_server(Box::new(RpcBackendReadFromJson)).await;
}

struct RpcBackendReadFromJson;

#[tonic::async_trait]
impl BackendServer for RpcBackendReadFromJson {
    async fn search(&self, text: String) -> anyhow::Result<Vec<SearchResultItem>> {
        todo!()
    }

    async fn request_view_render(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, ActionShortcut>> {
        todo!()
    }

    async fn request_run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        todo!()
    }

    async fn request_run_generated_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        todo!()
    }

    async fn send_view_event(&self, plugin_id: PluginId, widget_id: UiWidgetId, event_name: String, event_arguments: Vec<UiPropertyValue>) -> anyhow::Result<()> {
        todo!()
    }

    async fn send_keyboard_event(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, key: String, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> anyhow::Result<()> {
        todo!()
    }

    async fn send_open_event(&self, plugin_id: PluginId, href: String) -> anyhow::Result<()> {
        todo!()
    }

    async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>> {
        unreachable!();
    }

    async fn set_plugin_state(&self, plugin_id: PluginId, enabled: bool) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn set_entrypoint_state(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn set_preference_value(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, preference_name: String, preference_value: PluginPreferenceUserData) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn download_status(&self) -> anyhow::Result<HashMap<PluginId, DownloadStatus>> {
        unreachable!();
    }

    async fn open_settings_window(&self) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn open_settings_window_preferences(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn remove_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn save_local_plugin(&self, path: String) -> anyhow::Result<()> {
        unreachable!();
    }
}
