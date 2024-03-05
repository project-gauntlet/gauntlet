use std::fmt::Debug;
use tonic::transport::Channel;

tonic::include_proto!("_");

pub type FrontendClient = rpc_frontend_client::RpcFrontendClient<Channel>;
pub type BackendClient = rpc_backend_client::RpcBackendClient<Channel>;

