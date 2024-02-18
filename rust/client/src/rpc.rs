use tonic::{Request, Response, Status};

use common::model::{PluginId, RenderLocation};
use common::rpc::{RpcClearInlineViewRequest, RpcClearInlineViewResponse, RpcRenderLocation, RpcReplaceViewRequest, RpcReplaceViewResponse, RpcShowWindowRequest, RpcShowWindowResponse};
use common::rpc::rpc_frontend_server::RpcFrontend;
use utils::channel::RequestSender;

use crate::model::{NativeUiRequestData, NativeUiResponseData};

pub struct RpcFrontendServerImpl {
    pub(crate) context_tx: RequestSender<NativeUiRequestData, NativeUiResponseData>
}

#[tonic::async_trait]
impl RpcFrontend for RpcFrontendServerImpl {
    async fn replace_view(&self, request: Request<RpcReplaceViewRequest>) -> Result<Response<RpcReplaceViewResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;
        let container = request.container.ok_or(Status::invalid_argument("container"))?;
        let top_level_view = request.top_level_view;
        let render_location = request.render_location;

        let container = container.try_into()
            .expect("unable to convert widget into native");

        let render_location = RpcRenderLocation::try_from(render_location)
            .map_err(|_| Status::invalid_argument("render_location"))?;

        let render_location = match render_location {
            RpcRenderLocation::InlineViewLocation => RenderLocation::InlineView,
            RpcRenderLocation::ViewLocation => RenderLocation::View,
        };

        let data = NativeUiRequestData::ReplaceView {
            plugin_id: PluginId::from_string(plugin_id),
            render_location,
            top_level_view,
            container
        };

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };

        Ok(Response::new(RpcReplaceViewResponse::default()))
    }

    async fn clear_inline_view(&self, request: Request<RpcClearInlineViewRequest>) -> Result<Response<RpcClearInlineViewResponse>, Status> {
        let request = request.into_inner();
        let plugin_id = request.plugin_id;

        let data = NativeUiRequestData::ClearInlineView {
            plugin_id: PluginId::from_string(plugin_id)
        };

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };

        Ok(Response::new(RpcClearInlineViewResponse::default()))
    }

    async fn show_window(&self, _: Request<RpcShowWindowRequest>) -> Result<Response<RpcShowWindowResponse>, Status> {
        let data = NativeUiRequestData::ShowWindow;

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };

        Ok(Response::new(RpcShowWindowResponse::default()))
    }
}


