use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;
use tonic::{Request, Response, Status};
use tonic::transport::Server;

use crate::model::{EntrypointId, PluginId, UiRenderLocation, UiWidget};
use crate::rpc::convert::ui_widget_from_rpc;
use crate::rpc::grpc::{RpcClearInlineViewRequest, RpcClearInlineViewResponse, RpcRenderLocation, RpcReplaceViewRequest, RpcReplaceViewResponse, RpcShowPluginErrorViewRequest, RpcShowPluginErrorViewResponse, RpcShowPreferenceRequiredViewRequest, RpcShowPreferenceRequiredViewResponse, RpcShowWindowRequest, RpcShowWindowResponse};
use crate::rpc::grpc::rpc_frontend_server::{RpcFrontend, RpcFrontendServer};

pub async fn wait_for_frontend_server() {
    loop {
        let addr: SocketAddr = "127.0.0.1:42321".parse().unwrap();

        if TcpStream::connect(addr).await.is_ok() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn start_frontend_server(server: Box<dyn FrontendServer + Sync + Send>) {
    let addr = "127.0.0.1:42321".parse().unwrap();

    Server::builder()
        .add_service(RpcFrontendServer::new(RpcFrontendServerImpl::new(server)))
        .serve(addr)
        .await
        .expect("frontend server didn't start");
}

struct RpcFrontendServerImpl {
    server: Box<dyn FrontendServer + Sync + Send>
}

impl RpcFrontendServerImpl {
    pub fn new(server: Box<dyn FrontendServer + Sync + Send>) -> Self {
        Self {
            server
        }
    }
}

#[tonic::async_trait]
pub trait FrontendServer {
    async fn replace_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        container: UiWidget,
        top_level_view: bool,
        render_location: UiRenderLocation
    );

    async fn clear_inline_view(&self, plugin_id: PluginId);

    async fn show_window(&self);

    async fn show_preference_required_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    );

    async fn show_plugin_error_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation
    );
}


#[tonic::async_trait]
impl RpcFrontend for RpcFrontendServerImpl {
    async fn replace_view(&self, request: Request<RpcReplaceViewRequest>) -> Result<Response<RpcReplaceViewResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let container = request.container.ok_or(Status::invalid_argument("container"))?;
        let top_level_view = request.top_level_view;
        let render_location = request.render_location;

        let container = ui_widget_from_rpc(container)
            .map_err(|_| Status::invalid_argument("container format"))?;

        let render_location = RpcRenderLocation::try_from(render_location)
            .map_err(|_| Status::invalid_argument("render_location"))?;

        let render_location = match render_location {
            RpcRenderLocation::InlineViewLocation => UiRenderLocation::InlineView,
            RpcRenderLocation::ViewLocation => UiRenderLocation::View,
        };

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.replace_view(plugin_id, entrypoint_id, container, top_level_view, render_location)
            .await;

        Ok(Response::new(RpcReplaceViewResponse::default()))
    }

    async fn clear_inline_view(&self, request: Request<RpcClearInlineViewRequest>) -> Result<Response<RpcClearInlineViewResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let plugin_id = PluginId::from_string(plugin_id);

        self.server.clear_inline_view(plugin_id)
            .await;

        Ok(Response::new(RpcClearInlineViewResponse::default()))
    }

    async fn show_window(&self, _: Request<RpcShowWindowRequest>) -> Result<Response<RpcShowWindowResponse>, Status> {

        self.server.show_window()
            .await;

        Ok(Response::new(RpcShowWindowResponse::default()))
    }

    async fn show_preference_required_view(&self, request: Request<RpcShowPreferenceRequiredViewRequest>) -> Result<Response<RpcShowPreferenceRequiredViewResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let plugin_preferences_required = request.plugin_preferences_required;
        let entrypoint_preferences_required = request.entrypoint_preferences_required;

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.show_preference_required_view(plugin_id, entrypoint_id, plugin_preferences_required, entrypoint_preferences_required)
            .await;

        Ok(Response::new(RpcShowPreferenceRequiredViewResponse::default()))
    }

    async fn show_plugin_error_view(&self, request: Request<RpcShowPluginErrorViewRequest>) -> Result<Response<RpcShowPluginErrorViewResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let entrypoint_id = request.entrypoint_id;
        let render_location = request.render_location;

        let render_location = RpcRenderLocation::try_from(render_location)
            .map_err(|_| Status::invalid_argument("render_location"))?;

        let render_location = match render_location {
            RpcRenderLocation::InlineViewLocation => UiRenderLocation::InlineView,
            RpcRenderLocation::ViewLocation => UiRenderLocation::View,
        };

        let plugin_id = PluginId::from_string(plugin_id);
        let entrypoint_id = EntrypointId::from_string(entrypoint_id);

        self.server.show_plugin_error_view(plugin_id, entrypoint_id, render_location)
            .await;

        Ok(Response::new(RpcShowPluginErrorViewResponse::default()))
    }
}



