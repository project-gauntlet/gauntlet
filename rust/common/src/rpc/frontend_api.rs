use utils::channel::RequestSender;

use crate::model::{EntrypointId, PluginId, UiRenderLocation, UiRequestData, UiResponseData, UiWidget};

#[derive(Debug, Clone)]
pub struct FrontendApi {
    frontend_sender: RequestSender<UiRequestData, UiResponseData>,
}

impl FrontendApi {
    pub fn new(frontend_sender: RequestSender<UiRequestData, UiResponseData>) -> Self {
        Self {
            frontend_sender
        }
    }

    pub async fn request_search_results_update(&mut self) -> anyhow::Result<()> {
        let _ = self.frontend_sender.send_receive(UiRequestData::RequestSearchResultUpdate).await;

        Ok(())
    }

    pub async fn replace_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: UiWidget,
    ) -> anyhow::Result<()> {
        let data = UiRequestData::ReplaceView {
            plugin_id,
            entrypoint_id,
            render_location,
            top_level_view,
            container,
        };

        let _ = self.frontend_sender.send_receive(data).await;

        Ok(())
    }

    pub async fn clear_inline_view(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let data = UiRequestData::ClearInlineView {
            plugin_id,
        };

        let _ = self.frontend_sender.send_receive(data).await;

        Ok(())
    }

    pub async fn show_window(&self) -> anyhow::Result<()> {
        let _ = self.frontend_sender.send_receive(UiRequestData::ShowWindow).await;

        Ok(())
    }

    pub async fn show_preference_required_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> anyhow::Result<()> {
        let data = UiRequestData::ShowPreferenceRequiredView {
            plugin_id,
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required,
        };

        let _ = self.frontend_sender.send_receive(data).await;

        Ok(())
    }

    pub async fn show_plugin_error_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    ) -> anyhow::Result<()> {
        let data = UiRequestData::ShowPluginErrorView {
            plugin_id,
            entrypoint_id,
            render_location,
        };

        let _ = self.frontend_sender.send_receive(data).await;

        Ok(())
    }
}