use tonic::Request;
use tonic::transport::Channel;

use crate::model::{EntrypointId, PluginId, RenderLocation, UiWidget};
use crate::rpc::grpc::{RpcClearInlineViewRequest, RpcRenderLocation, RpcReplaceViewRequest, RpcShowPluginErrorViewRequest, RpcShowPreferenceRequiredViewRequest, RpcShowWindowRequest};
use crate::rpc::convert::ui_widget_to_rpc;
use crate::rpc::grpc::rpc_frontend_client::RpcFrontendClient;

#[derive(Debug, Clone)]
pub struct FrontendApi {
    client: RpcFrontendClient<Channel>,
}

impl FrontendApi {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: RpcFrontendClient::connect("http://127.0.0.1:42321").await?
        })
    }

    pub async fn replace_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: RenderLocation,
        top_level_view: bool,
        container: UiWidget,
    ) -> anyhow::Result<()> {
        let render_location = match render_location {
            RenderLocation::InlineView => RpcRenderLocation::InlineViewLocation,
            RenderLocation::View => RpcRenderLocation::ViewLocation,
        };

        let request = Request::new(RpcReplaceViewRequest {
            top_level_view,
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.to_string(),
            render_location: render_location.into(),
            container: Some(ui_widget_to_rpc(container)),
        });

        self.client.replace_view(request)
            .await
            .map(|_| ())
            .map_err(|err| err.into())
    }

    pub async fn clear_inline_view(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let request = Request::new(RpcClearInlineViewRequest {
            plugin_id: plugin_id.to_string()
        });

        self.client.clear_inline_view(request)
            .await
            .map(|_| ())
            .map_err(|err| err.into())
    }

    pub async fn show_window(&mut self) -> anyhow::Result<()> {
        self.client.show_window(Request::new(RpcShowWindowRequest::default()))
            .await
            .map(|_| ())
            .map_err(|err| err.into())
    }

    pub async fn show_preference_required_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> anyhow::Result<()> {
        let request = Request::new(RpcShowPreferenceRequiredViewRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.to_string(),
            plugin_preferences_required,
            entrypoint_preferences_required,
        });

        self.client.show_preference_required_view(request)
            .await
            .map(|_| ())
            .map_err(|err| err.into())
    }

    pub async fn show_plugin_error_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: RenderLocation,
    ) -> anyhow::Result<()> {
        let render_location = match render_location {
            RenderLocation::InlineView => RpcRenderLocation::InlineViewLocation,
            RenderLocation::View => RpcRenderLocation::ViewLocation,
        };

        let request = Request::new(RpcShowPluginErrorViewRequest {
            plugin_id: plugin_id.to_string(),
            entrypoint_id: entrypoint_id.to_string(),
            render_location: render_location.into(),
        });

        self.client.show_plugin_error_view(request)
            .await
            .map(|_| ())
            .map_err(|err| err.into())
    }
}