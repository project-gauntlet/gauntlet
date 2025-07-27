use std::sync::Arc;

use gauntlet_utils::channel::RequestResult;
use gauntlet_utils_macros::boundary_gen;
use tokio::sync::Mutex;
use tonic::Request;
use tonic::transport::Channel;

use crate::model::EntrypointId;
use crate::model::LocalSaveData;
use crate::model::PluginId;
use crate::rpc::grpc::RpcBincode;
use crate::rpc::grpc::RpcSaveLocalPluginRequest;
use crate::rpc::grpc::rpc_backend_client::RpcBackendClient;

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
