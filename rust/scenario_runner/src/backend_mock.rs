use std::collections::HashMap;

use common::model::{DownloadStatus, EntrypointId, PhysicalShortcut, PluginId, PluginPreferenceUserData, SearchResult, SettingsPlugin, UiPropertyValue, UiWidgetId};
use common::rpc::backend_server::{BackendServer, start_backend_server};

pub async fn start_screenshot_gen_backend() {
    start_backend_server(Box::new(RpcBackendScreenshotGen)).await;
}

struct RpcBackendScreenshotGen;

#[tonic::async_trait]
impl BackendServer for RpcBackendScreenshotGen {
    async fn search(&self, _text: String, _render_inline_view: bool) -> anyhow::Result<Vec<SearchResult>> {
        todo!();
    }

    async fn request_view_render(&self, _plugin_id: PluginId, _entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        unreachable!(); // screenshot gen is not interactive
    }

    async fn request_view_close(&self, _plugin_id: PluginId) -> anyhow::Result<()> {
        unreachable!(); // screenshot gen is not interactive
    }

    async fn request_run_command(&self, _plugin_id: PluginId, _entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        unreachable!(); // screenshot gen is not interactive
    }

    async fn request_run_generated_command(&self, _plugin_id: PluginId, _entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        unreachable!(); // screenshot gen is not interactive
    }

    async fn send_view_event(&self, _plugin_id: PluginId, _widget_id: UiWidgetId, _event_name: String, _event_arguments: Vec<UiPropertyValue>) -> anyhow::Result<()> {
        unreachable!(); // screenshot gen is not interactive
    }

    async fn send_keyboard_event(&self, _plugin_id: PluginId, _entrypoint_id: EntrypointId, _key: String, _modifier_shift: bool, _modifier_control: bool, _modifier_alt: bool, _modifier_meta: bool) -> anyhow::Result<()> {
        unreachable!(); // screenshot gen is not interactive
    }

    async fn send_open_event(&self, _plugin_id: PluginId, _href: String) -> anyhow::Result<()> {
        unreachable!(); // screenshot gen is not interactive
    }

    // settings
    async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>> {
        unreachable!();
    }

    async fn set_plugin_state(&self, _plugin_id: PluginId, _enabled: bool) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn set_entrypoint_state(&self, _plugin_id: PluginId, _entrypoint_id: EntrypointId, _enabled: bool) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn set_global_shortcut(&self, _shortcut: PhysicalShortcut) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn get_global_shortcut(&self) -> anyhow::Result<PhysicalShortcut> {
        unreachable!();
    }

    async fn set_preference_value(&self, _plugin_id: PluginId, _entrypoint_id: Option<EntrypointId>, _preference_name: String, _preference_value: PluginPreferenceUserData) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn download_plugin(&self, _plugin_id: PluginId) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn download_status(&self) -> anyhow::Result<HashMap<PluginId, DownloadStatus>> {
        unreachable!();
    }

    async fn open_settings_window(&self) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn open_settings_window_preferences(&self, _plugin_id: PluginId, _entrypoint_id: Option<EntrypointId>) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn remove_plugin(&self, _plugin_id: PluginId) -> anyhow::Result<()> {
        unreachable!();
    }

    async fn save_local_plugin(&self, _path: String) -> anyhow::Result<()> {
        unreachable!();
    }
}
