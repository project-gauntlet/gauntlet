use std::collections::HashMap;

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
use gauntlet_common::rpc::backend_server::start_backend_server;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApi;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApiProxy;
use gauntlet_utils::channel::RequestResult;

pub struct BackendServerImpl {
    pub proxy: ServerGrpcApiProxy,
}

impl BackendServerImpl {
    pub fn new(application_manager: ServerGrpcApiProxy) -> Self {
        Self {
            proxy: application_manager,
        }
    }
}

pub async fn run_grpc_server(grpc_api: ServerGrpcApiProxy) {
    start_backend_server(
        Box::new(BackendServerImpl::new(grpc_api.clone())),
        Box::new(BackendServerImpl::new(grpc_api.clone())),
        Box::new(BackendServerImpl::new(grpc_api.clone())),
    )
    .await
}

#[tonic::async_trait]
impl BackendForCliApi for BackendServerImpl {
    async fn ping(&self) -> RequestResult<()> {
        // noop
        Ok(())
    }

    async fn show_window(&self) -> RequestResult<()> {
        self.proxy.show_window().await?;

        Ok(())
    }

    async fn show_settings_window(&self) -> RequestResult<()> {
        self.proxy.show_settings_window().await?;

        Ok(())
    }

    async fn run_action(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_id: String,
    ) -> RequestResult<()> {
        self.proxy.run_action(plugin_id, entrypoint_id, action_id).await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl BackendForToolsApi for BackendServerImpl {
    async fn save_local_plugin(&self, path: String) -> RequestResult<LocalSaveData> {
        let result = self.proxy.save_local_plugin(path).await?;

        Ok(result)
    }
}

#[tonic::async_trait]
impl BackendForSettingsApi for BackendServerImpl {
    async fn plugins(&self) -> RequestResult<HashMap<PluginId, SettingsPlugin>> {
        self.proxy.plugins().await
    }

    async fn set_plugin_state(&self, plugin_id: PluginId, enabled: bool) -> RequestResult<()> {
        let result = self.proxy.set_plugin_state(plugin_id, enabled).await;

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
        let result = self.proxy.set_entrypoint_state(plugin_id, entrypoint_id, enabled).await;

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
        let result = self.proxy.set_global_shortcut(shortcut).await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_global_shortcut' request {:?}",
                err
            )
        }

        result
    }

    async fn get_global_shortcut(&self) -> RequestResult<(Option<PhysicalShortcut>, Option<String>)> {
        let result = self
            .proxy
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
        let result = self
            .proxy
            .set_global_entrypoint_shortcut(plugin_id, entrypoint_id, shortcut)
            .await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_global_entrypoint_shortcut' request {:?}",
                err
            )
        }

        result
    }

    async fn get_global_entrypoint_shortcuts(
        &self,
    ) -> RequestResult<HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>> {
        self.proxy.get_global_entrypoint_shortcuts().await
    }

    async fn set_entrypoint_search_alias(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        alias: Option<String>,
    ) -> RequestResult<()> {
        let result = self
            .proxy
            .set_entrypoint_search_alias(plugin_id, entrypoint_id, alias)
            .await;

        if let Err(err) = &result {
            tracing::warn!(
                target = "rpc",
                "error occurred when handling 'set_entrypoint_search_alias' request {:?}",
                err
            )
        }

        result
    }

    async fn get_entrypoint_search_aliases(&self) -> RequestResult<HashMap<(PluginId, EntrypointId), String>> {
        self.proxy.get_entrypoint_search_aliases().await
    }

    async fn set_theme(&self, theme: SettingsTheme) -> RequestResult<()> {
        self.proxy.set_theme(theme).await
    }

    async fn get_theme(&self) -> RequestResult<SettingsTheme> {
        self.proxy.get_theme().await
    }

    async fn set_window_position_mode(&self, mode: WindowPositionMode) -> RequestResult<()> {
        self.proxy.set_window_position_mode(mode).await
    }

    async fn get_window_position_mode(&self) -> RequestResult<WindowPositionMode> {
        self.proxy.get_window_position_mode().await
    }

    async fn set_preference_value(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        preference_id: String,
        preference_value: PluginPreferenceUserData,
    ) -> RequestResult<()> {
        let result = self
            .proxy
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
        let result = self.proxy.download_plugin(plugin_id).await;

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
        self.proxy.download_status().await
    }

    async fn remove_plugin(&self, plugin_id: PluginId) -> RequestResult<()> {
        let result = self.proxy.remove_plugin(plugin_id).await;

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
