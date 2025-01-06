use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use gauntlet_common::{settings_env_data_to_string, SettingsEnvData};
use gauntlet_common::model::{DownloadStatus, EntrypointId, PluginId, PluginPreferenceUserData, SettingsPlugin, UiPropertyValue, SearchResult, UiWidgetId, PhysicalKey, PhysicalShortcut, LocalSaveData, SettingsTheme};
use gauntlet_common::rpc::backend_server::BackendServer;

use crate::plugins::ApplicationManager;
use crate::search::SearchIndex;
use crate::SETTINGS_ENV;

pub struct BackendServerImpl {
    pub application_manager: Arc<ApplicationManager>,
}

impl BackendServerImpl {
    pub fn new(application_manager: Arc<ApplicationManager>) -> Self {
        Self {
            application_manager
        }
    }
}

#[tonic::async_trait]
impl BackendServer for BackendServerImpl {

    async fn show_window(&self) -> anyhow::Result<()> {
        self.application_manager.show_window().await
    }

    async fn show_settings_window(&self) -> anyhow::Result<()> {
        self.application_manager.handle_open_settings_window();

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

    async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> anyhow::Result<()> {
        let result = self.application_manager.set_global_shortcut(shortcut)
            .await;

        if let Err(err) = &result {
            tracing::warn!(target = "rpc", "error occurred when handling 'set_global_shortcut' request {:?}", err)
        }

        result
    }

    async fn get_global_shortcut(&self) -> anyhow::Result<(Option<PhysicalShortcut>, Option<String>)> {
        let result = self.application_manager.get_global_shortcut()
            .await?
            .unwrap_or((None, None));

        Ok(result)
    }

    async fn set_theme(&self, theme: SettingsTheme) -> anyhow::Result<()> {
        self.application_manager.set_theme(theme).await
    }

    async fn get_theme(&self) -> anyhow::Result<SettingsTheme> {
        self.application_manager.get_theme().await
    }

    async fn set_preference_value(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, preference_id: String, preference_value: PluginPreferenceUserData) -> anyhow::Result<()> {
        let result = self.application_manager.set_preference_value(plugin_id, entrypoint_id, preference_id, preference_value)
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
