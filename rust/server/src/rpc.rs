use std::collections::HashMap;
use std::sync::Arc;

use gauntlet_common::model::DownloadStatus;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::LocalSaveData;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::PluginPreferenceUserData;
use gauntlet_common::model::SettingsPlugin;
use gauntlet_common::model::SettingsTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::rpc::backend_api::BackendForCliApi;
use gauntlet_common::rpc::backend_api::BackendForSettingsApi;
use gauntlet_common::rpc::backend_api::BackendForToolsApi;
use gauntlet_utils::channel::RequestResult;

use crate::plugins::ApplicationManager;

pub struct BackendServerImpl {
    pub application_manager: Arc<ApplicationManager>,
}

impl BackendServerImpl {
    pub fn new(application_manager: Arc<ApplicationManager>) -> Self {
        Self { application_manager }
    }
}

#[tonic::async_trait]
impl BackendForCliApi for BackendServerImpl {
    async fn ping(&self) -> RequestResult<()> {
        // noop
        Ok(())
    }

    async fn show_window(&self) -> RequestResult<()> {
        self.application_manager.show_window().await.map_err(Into::into)
    }

    async fn show_settings_window(&self) -> RequestResult<()> {
        self.application_manager.handle_open_settings_window();

        Ok(())
    }

    async fn run_action(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_id: String,
    ) -> RequestResult<()> {
        self.application_manager
            .run_action(plugin_id, entrypoint_id, action_id)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl BackendForToolsApi for BackendServerImpl {
    async fn save_local_plugin(&self, path: String) -> RequestResult<LocalSaveData> {
        let result = self.application_manager.save_local_plugin(&path).await?;

        Ok(result)
    }
}

#[tonic::async_trait]
impl BackendForSettingsApi for BackendServerImpl {
    async fn plugins(&self) -> RequestResult<HashMap<PluginId, SettingsPlugin>> {
        let result = self.application_manager.plugins().await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'plugins' request {:?}",
                err
            )
        }

        result.map_err(Into::into)
    }

    async fn set_plugin_state(&self, plugin_id: PluginId, enabled: bool) -> RequestResult<()> {
        let result = self.application_manager.set_plugin_state(plugin_id, enabled).await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_plugin_state' request {:?}",
                err
            )
        }

        Ok(())
    }

    async fn set_entrypoint_state(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        enabled: bool,
    ) -> RequestResult<()> {
        let result = self
            .application_manager
            .set_entrypoint_state(plugin_id, entrypoint_id, enabled)
            .await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_entrypoint_state' request {:?}",
                err
            )
        }

        Ok(())
    }

    async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> RequestResult<Option<String>> {
        let result = self.application_manager.set_global_shortcut(shortcut).await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_global_shortcut' request {:?}",
                err
            )
        }

        Ok(result.err().map(|err| format!("{:#}", err)))
    }

    async fn get_global_shortcut(&self) -> RequestResult<(Option<PhysicalShortcut>, Option<String>)> {
        let result = self
            .application_manager
            .get_global_shortcut()
            .await?
            .map(|(shortcut, error)| (Some(shortcut), error))
            .unwrap_or((None, None));

        Ok(result)
    }

    async fn set_global_entrypoint_shortcut(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut: Option<PhysicalShortcut>,
    ) -> RequestResult<()> {
        self.application_manager
            .set_global_entrypoint_shortcut(plugin_id, entrypoint_id, shortcut)
            .await
            .map_err(Into::into)
    }

    async fn get_global_entrypoint_shortcuts(
        &self,
    ) -> RequestResult<HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>> {
        self.application_manager
            .get_global_entrypoint_shortcut()
            .await
            .map_err(Into::into)
    }

    async fn set_entrypoint_search_alias(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        alias: Option<String>,
    ) -> RequestResult<()> {
        self.application_manager
            .set_entrypoint_search_alias(plugin_id, entrypoint_id, alias)
            .await
            .map_err(Into::into)
    }

    async fn get_entrypoint_search_aliases(&self) -> RequestResult<HashMap<(PluginId, EntrypointId), String>> {
        self.application_manager
            .get_entrypoint_search_aliases()
            .await
            .map_err(Into::into)
    }

    async fn set_theme(&self, theme: SettingsTheme) -> RequestResult<()> {
        self.application_manager.set_theme(theme).await.map_err(Into::into)
    }

    async fn get_theme(&self) -> RequestResult<SettingsTheme> {
        self.application_manager.get_theme().await.map_err(Into::into)
    }

    async fn set_window_position_mode(&self, mode: WindowPositionMode) -> RequestResult<()> {
        self.application_manager
            .set_window_position_mode(mode)
            .await
            .map_err(Into::into)
    }

    async fn get_window_position_mode(&self) -> RequestResult<WindowPositionMode> {
        self.application_manager
            .get_window_position_mode()
            .await
            .map_err(Into::into)
    }

    async fn set_preference_value(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        preference_id: String,
        preference_value: PluginPreferenceUserData,
    ) -> RequestResult<()> {
        let result = self
            .application_manager
            .set_preference_value(plugin_id, entrypoint_id, preference_id, preference_value)
            .await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_preference_value' request {:?}",
                err
            )
        }

        Ok(())
    }

    async fn download_plugin(&self, plugin_id: PluginId) -> RequestResult<()> {
        let result = self.application_manager.download_plugin(plugin_id).await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'download_plugin' request {:?}",
                err
            )
        }

        Ok(())
    }

    async fn download_status(&self) -> RequestResult<HashMap<PluginId, DownloadStatus>> {
        Ok(self.application_manager.download_status())
    }

    async fn remove_plugin(&self, plugin_id: PluginId) -> RequestResult<()> {
        let result = self.application_manager.remove_plugin(plugin_id).await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'remove_plugin' request {:?}",
                err
            )
        }

        Ok(())
    }
}
