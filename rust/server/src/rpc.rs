use std::collections::HashMap;

use common::{settings_env_data_to_string, SettingsEnvData};
use common::model::{DownloadStatus, EntrypointId, PluginId, PluginPreferenceUserData, SettingsPlugin, UiPropertyValue, SearchResult, UiWidgetId, PhysicalKey, PhysicalShortcut, LocalSaveData};
use common::rpc::backend_server::BackendServer;

use crate::plugins::ApplicationManager;
use crate::search::SearchIndex;
use crate::SETTINGS_ENV;

pub struct BackendServerImpl {
    pub application_manager: ApplicationManager,
}

impl BackendServerImpl {
    pub fn new(application_manager: ApplicationManager) -> Self {
        Self {
            application_manager
        }
    }
}

#[tonic::async_trait]
impl BackendServer for BackendServerImpl {
    async fn search(&self, text: String, render_inline_view: bool) -> anyhow::Result<Vec<SearchResult>> {
        let result = self.application_manager.search(&text);

        if render_inline_view {
            self.application_manager.handle_inline_view(&text);
        }

        result
    }

    async fn request_view_render(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, PhysicalShortcut>> {
        self.application_manager.handle_render_view(plugin_id.clone(), entrypoint_id.clone())
            .await;

        self.application_manager.action_shortcuts(plugin_id, entrypoint_id).await
    }

    async fn request_view_close(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        self.application_manager.handle_view_close(plugin_id);

        Ok(())
    }

    async fn request_run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        self.application_manager.handle_run_command(plugin_id, entrypoint_id)
            .await;

        Ok(())
    }

    async fn request_run_generated_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> anyhow::Result<()> {
        self.application_manager.handle_run_generated_command(plugin_id, entrypoint_id)
            .await;

        Ok(())
    }

    async fn send_view_event(&self, plugin_id: PluginId, widget_id: UiWidgetId, event_name: String, event_arguments: Vec<UiPropertyValue>) -> anyhow::Result<()> {
        self.application_manager.handle_view_event(PluginId::from_string(plugin_id), widget_id, event_name, event_arguments);

        Ok(())
    }

    async fn send_keyboard_event(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> anyhow::Result<()> {
        self.application_manager.handle_keyboard_event(
            plugin_id,
            entrypoint_id,
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        );

        Ok(())
    }

    async fn send_open_event(&self, _plugin_id: PluginId, href: String) -> anyhow::Result<()> {
        match open::that(&href) {
            Ok(()) => tracing::info!("Opened '{}' successfully.", href),
            Err(err) => tracing::error!("An error occurred when opening '{}': {}", href, err),
        }

        Ok(())
    }

    async fn plugins(&self) -> anyhow::Result<Vec<SettingsPlugin>> {
        let result = self.application_manager.plugins()
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'plugins' request {:?}", err)
        }

        result
    }

    async fn set_plugin_state(&self, plugin_id: PluginId, enabled: bool) -> anyhow::Result<()> {
        let result = self.application_manager.set_plugin_state(plugin_id, enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_plugin_state' request {:?}", err)
        }

        Ok(())
    }

    async fn set_entrypoint_state(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        let result = self.application_manager.set_entrypoint_state(plugin_id, entrypoint_id, enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_entrypoint_state' request {:?}", err)
        }

        Ok(())
    }

    async fn set_global_shortcut(&self, shortcut: PhysicalShortcut) -> anyhow::Result<()> {
        let result = self.application_manager.set_global_shortcut(shortcut)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_global_shortcut' request {:?}", err)
        }

        Ok(())
    }

    async fn get_global_shortcut(&self) -> anyhow::Result<PhysicalShortcut> {
        let result = self.application_manager.get_global_shortcut()
            .await?;

        Ok(result)
    }

    async fn set_preference_value(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, preference_name: String, preference_value: PluginPreferenceUserData) -> anyhow::Result<()> {
        let result = self.application_manager.set_preference_value(plugin_id, entrypoint_id, preference_name, preference_value)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_preference_value' request {:?}", err)
        }

        Ok(())
    }

    async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        let result = self.application_manager.download_plugin(plugin_id)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'download_plugin' request {:?}", err)
        }

        Ok(())
    }

    async fn download_status(&self) -> anyhow::Result<HashMap<PluginId, DownloadStatus>> {
        Ok(self.application_manager.download_status())
    }

    async fn open_settings_window(&self) -> anyhow::Result<()> {
        std::process::Command::new(std::env::current_exe()?)
            .args(["settings"])
            .spawn()
            .expect("failed to execute settings process");

        Ok(())
    }

    async fn open_settings_window_preferences(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> anyhow::Result<()> {

        let data = if let Some(entrypoint_id) = entrypoint_id {
            SettingsEnvData::OpenEntrypointPreferences {
                plugin_id: plugin_id.to_string(),
                entrypoint_id: entrypoint_id.to_string()
            }
        } else {
            SettingsEnvData::OpenPluginPreferences {
                plugin_id: plugin_id.to_string()
            }
        };

        std::process::Command::new(std::env::current_exe()?)
            .args(["settings"])
            .env(SETTINGS_ENV, settings_env_data_to_string(data))
            .spawn()
            .expect("failed to execute settings process"); // this can fail in dev if binary was replaced by frontend compilation

        Ok(())
    }

    async fn remove_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        let result = self.application_manager.remove_plugin(plugin_id)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'remove_plugin' request {:?}", err)
        }

        Ok(())
    }

    async fn save_local_plugin(&self, path: String) -> anyhow::Result<LocalSaveData> {
        let result = self.application_manager.save_local_plugin(&path)
            .await?;

        Ok(result)
    }
}
