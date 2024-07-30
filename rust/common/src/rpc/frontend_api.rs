use thiserror::Error;
use utils::channel::{RequestError, RequestSender};

use crate::model::{EntrypointId, PluginId, UiRenderLocation, UiRequestData, UiResponseData, UiWidget};

#[derive(Error, Debug, Clone)]
pub enum FrontendApiError {
    #[error("Frontend wasn't able to process request in a timely manner")]
    TimeoutError,
}

impl From<RequestError> for FrontendApiError {
    fn from(error: RequestError) -> FrontendApiError {
        match error {
            RequestError::TimeoutError => FrontendApiError::TimeoutError,
        }
    }
}

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

    pub async fn request_search_results_update(&mut self) -> Result<(), FrontendApiError> {
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
    ) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ReplaceView {
            plugin_id,
            entrypoint_id,
            render_location,
            top_level_view,
            container,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await?;

        Ok(())
    }

    pub async fn clear_inline_view(&mut self, plugin_id: PluginId) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ClearInlineView {
            plugin_id,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await?;

        Ok(())
    }

    pub async fn show_window(&self) -> Result<(), FrontendApiError> {
        let UiResponseData::Nothing = self.frontend_sender.send_receive(UiRequestData::ShowWindow).await?;

        Ok(())
    }

    pub async fn show_preference_required_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ShowPreferenceRequiredView {
            plugin_id,
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await?;

        Ok(())
    }

    pub async fn show_plugin_error_view(
        &mut self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    ) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ShowPluginErrorView {
            plugin_id,
            entrypoint_id,
            render_location,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await?;

        Ok(())
    }
}