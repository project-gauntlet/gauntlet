use common::model::{EntrypointId, PluginId, UiRenderLocation, UiRequestData, UiWidget};
use common::rpc::frontend_server::FrontendServer;
use utils::channel::RequestSender;

use crate::model::NativeUiResponseData;

pub struct FrontendServerImpl {
    context_tx: RequestSender<UiRequestData, NativeUiResponseData>
}

impl FrontendServerImpl {
    pub fn new(context_tx: RequestSender<UiRequestData, NativeUiResponseData>) -> Self {
        Self {
            context_tx
        }
    }
}

#[tonic::async_trait]
impl FrontendServer for FrontendServerImpl {
    async fn replace_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, container: UiWidget, top_level_view: bool, render_location: UiRenderLocation) {
        let data = UiRequestData::ReplaceView {
            plugin_id: PluginId::from_string(plugin_id),
            entrypoint_id: EntrypointId::from_string(entrypoint_id),
            render_location,
            top_level_view,
            container
        };

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };
    }

    async fn clear_inline_view(&self, plugin_id: PluginId) {
        let data = UiRequestData::ClearInlineView {
            plugin_id
        };

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };
    }

    async fn show_window(&self) {
        let data = UiRequestData::ShowWindow;

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };
    }

    async fn show_preference_required_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, plugin_preferences_required: bool, entrypoint_preferences_required: bool) {
        let data = UiRequestData::ShowPreferenceRequiredView {
            plugin_id,
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required,
        };

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };
    }

    async fn show_plugin_error_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, render_location: UiRenderLocation) {
        let data = UiRequestData::ShowPluginErrorView {
            plugin_id,
            entrypoint_id,
            render_location,
        };

        match self.context_tx.send_receive(data).await {
            NativeUiResponseData::Nothing => {}
        };
    }
}
