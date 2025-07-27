use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::LocalSaveData;
use gauntlet_common::model::PluginId;
use gauntlet_common::rpc::backend_api::BackendForCliApi;
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
