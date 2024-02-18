use std::fmt::Debug;
use tonic::transport::Channel;

tonic::include_proto!("_");

pub type FrontendClient = rpc_frontend_client::RpcFrontendClient<Channel>;
pub type BackendClient = rpc_backend_client::RpcBackendClient<Channel>;


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RpcSearchResult {
//     pub plugin_id: String,
//     pub plugin_name: String,
//     pub entrypoint_id: String,
//     pub entrypoint_name: String,
//     pub entrypoint_type: RpcEntrypointType,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RpcPlugin {
//     pub plugin_id: String,
//     pub plugin_name: String,
//     pub enabled: bool,
//     pub entrypoints: Vec<RpcEntrypoint>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RpcEntrypoint {
//     pub entrypoint_id: String,
//     pub entrypoint_name: String,
//     pub enabled: bool,
//     pub entrypoint_type: RpcEntrypointType,
// }
//
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct RpcUiWidget {
//     pub widget_id: RpcUiWidgetId,
//     pub widget_type: String,
//     pub widget_properties: HashMap<String, RpcUiPropertyValue>,
//     pub widget_children: Vec<RpcUiWidget>,
// }

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct RpcEventRenderView {
//     pub frontend: String,
//     pub entrypoint_id: String,
// }
//
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct RpcEventRunCommand {
//     pub entrypoint_id: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum RpcEntrypointType {
//     Command,
//     View,
//     InlineView,
// }
