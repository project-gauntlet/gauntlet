use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tonic::transport::Server;

use crate::rpc::backend_api::BackendForCliApi;
use crate::rpc::backend_api::BackendForSettingsApi;
use crate::rpc::backend_api::BackendForToolsApi;
use crate::rpc::backend_api::handle_grpc_request_backend_for_cli_api;
use crate::rpc::backend_api::handle_grpc_request_backend_for_settings_api;
use crate::rpc::grpc::RpcBincode;
use crate::rpc::grpc::RpcSaveLocalPluginRequest;
use crate::rpc::grpc::RpcSaveLocalPluginResponse;
use crate::rpc::grpc::rpc_backend_server::RpcBackend;
use crate::rpc::grpc::rpc_backend_server::RpcBackendServer;

pub async fn wait_for_backend_server() {
    loop {
        let addr: SocketAddr = "127.0.0.1:42320".parse().unwrap();

        if TcpStream::connect(addr).await.is_ok() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn start_backend_server(
    cli: Box<dyn BackendForCliApi + Sync + Send>,
    tools: Box<dyn BackendForToolsApi + Sync + Send>,
    settings: Box<dyn BackendForSettingsApi + Sync + Send>,
) {
    let addr = "127.0.0.1:42320".parse().unwrap();

    Server::builder()
        .add_service(RpcBackendServer::new(RpcBackendServerImpl::new(cli, tools, settings)))
        .serve(addr)
        .await
        .expect("unable to start backend server");
}

struct RpcBackendServerImpl {
    cli: Box<dyn BackendForCliApi + Sync + Send>,
    tools: Box<dyn BackendForToolsApi + Sync + Send>,
    settings: Box<dyn BackendForSettingsApi + Sync + Send>,
}

impl RpcBackendServerImpl {
    pub fn new(
        cli: Box<dyn BackendForCliApi + Sync + Send>,
        tools: Box<dyn BackendForToolsApi + Sync + Send>,
        settings: Box<dyn BackendForSettingsApi + Sync + Send>,
    ) -> Self {
        Self { cli, settings, tools }
    }
}

#[tonic::async_trait]
impl RpcBackend for RpcBackendServerImpl {
    async fn backend_for_cli_api(&self, request: Request<RpcBincode>) -> Result<Response<RpcBincode>, Status> {
        let data = request.into_inner().data;

        let encoded = handle_grpc_request_backend_for_cli_api(self.cli.as_ref(), data).await?;

        Ok(Response::new(RpcBincode { data: encoded }))
    }

    async fn backend_for_settings_api(&self, request: Request<RpcBincode>) -> Result<Response<RpcBincode>, Status> {
        let data = request.into_inner().data;

        let encoded = handle_grpc_request_backend_for_settings_api(self.settings.as_ref(), data).await?;

        Ok(Response::new(RpcBincode { data: encoded }))
    }

    async fn save_local_plugin(
        &self,
        request: Request<RpcSaveLocalPluginRequest>,
    ) -> Result<Response<RpcSaveLocalPluginResponse>, Status> {
        let request = request.into_inner();
        let path = request.path;

        let local_save_data = self
            .tools
            .save_local_plugin(path)
            .await
            .map_err(|err| Status::internal(format!("{:#}", err)))?;

        Ok(Response::new(RpcSaveLocalPluginResponse {
            stdout_file_path: local_save_data.stdout_file_path,
            stderr_file_path: local_save_data.stderr_file_path,
        }))
    }
}
