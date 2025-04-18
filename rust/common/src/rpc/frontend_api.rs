use std::collections::HashMap;

use anyhow::anyhow;
use gauntlet_utils::channel::RequestError;
use gauntlet_utils::channel::RequestSender;
use thiserror::Error;

use crate::model::EntrypointId;
use crate::model::PhysicalShortcut;
use crate::model::PluginId;
use crate::model::RootWidget;
use crate::model::UiRenderLocation;
use crate::model::UiRequestData;
use crate::model::UiResponseData;
use crate::model::UiTheme;
use crate::model::UiWidgetId;
use crate::model::WindowPositionMode;

#[derive(Error, Debug)]
pub enum FrontendApiError {
    #[error("Frontend wasn't able to process request in a timely manner")]
    TimeoutError,
    #[error("Other error: {0:?}")]
    OtherError(#[from] anyhow::Error),
}

impl From<RequestError> for FrontendApiError {
    fn from(error: RequestError) -> FrontendApiError {
        match error {
            RequestError::TimeoutError => FrontendApiError::TimeoutError,
            RequestError::OtherSideWasDropped => FrontendApiError::OtherError(anyhow!("other side was dropped")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrontendApi {
    frontend_sender: RequestSender<UiRequestData, UiResponseData>,
}

impl FrontendApi {
    pub fn new(frontend_sender: RequestSender<UiRequestData, UiResponseData>) -> Self {
        Self { frontend_sender }
    }

    pub async fn request_search_results_update(&self) -> Result<(), FrontendApiError> {
        let _ = self
            .frontend_sender
            .send_receive(UiRequestData::RequestSearchResultUpdate)
            .await;

        Ok(())
    }

    pub async fn replace_view(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
        images: HashMap<UiWidgetId, Vec<u8>>,
    ) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ReplaceView {
            plugin_id,
            plugin_name,
            entrypoint_id,
            entrypoint_name,
            render_location,
            top_level_view,
            container,
            images,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn clear_inline_view(&self, plugin_id: PluginId) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ClearInlineView { plugin_id };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn show_window(&self) -> Result<(), FrontendApiError> {
        let UiResponseData::Nothing = self.frontend_sender.send_receive(UiRequestData::ShowWindow).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn hide_window(&self) -> Result<(), FrontendApiError> {
        let UiResponseData::Nothing = self.frontend_sender.send_receive(UiRequestData::HideWindow).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn show_preference_required_view(
        &self,
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

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn show_plugin_error_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    ) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ShowPluginErrorView {
            plugin_id,
            entrypoint_id,
            render_location,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn show_hud(&self, display: String) -> Result<(), FrontendApiError> {
        let request = UiRequestData::ShowHud { display };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn update_loading_bar(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        show: bool,
    ) -> Result<(), FrontendApiError> {
        let request = UiRequestData::UpdateLoadingBar {
            plugin_id,
            entrypoint_id,
            show,
        };

        let UiResponseData::Nothing = self.frontend_sender.send_receive(request).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn set_global_shortcut(&self, shortcut: Option<PhysicalShortcut>) -> anyhow::Result<()> {
        let request = UiRequestData::SetGlobalShortcut { shortcut };

        let data = self
            .frontend_sender
            .send_receive(request)
            .await
            .map_err(|err| anyhow!("error: {:?}", err))?;

        match data {
            UiResponseData::Nothing => Ok(()),
            UiResponseData::Err(err) => Err(err),
        }
    }

    pub async fn set_global_entrypoint_shortcut(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut: Option<PhysicalShortcut>,
    ) -> anyhow::Result<()> {
        let request = UiRequestData::SetGlobalEntrypointShortcut {
            plugin_id,
            entrypoint_id,
            shortcut,
        };

        let data = self
            .frontend_sender
            .send_receive(request)
            .await
            .map_err(|err| anyhow!("error: {:?}", err))?;

        match data {
            UiResponseData::Nothing => Ok(()),
            UiResponseData::Err(err) => Err(err),
        }
    }

    pub async fn set_theme(&self, theme: UiTheme) -> anyhow::Result<()> {
        let request = UiRequestData::SetTheme { theme };

        let data = self
            .frontend_sender
            .send_receive(request)
            .await
            .map_err(|err| anyhow!("error: {:?}", err))?;

        match data {
            UiResponseData::Nothing => Ok(()),
            UiResponseData::Err(err) => Err(err),
        }
    }

    pub async fn set_window_position_mode(&self, mode: WindowPositionMode) -> anyhow::Result<()> {
        let request = UiRequestData::SetWindowPositionMode { mode };

        let data = self
            .frontend_sender
            .send_receive(request)
            .await
            .map_err(|err| anyhow!("error: {:?}", err))?;

        match data {
            UiResponseData::Nothing => Ok(()),
            UiResponseData::Err(err) => Err(err),
        }
    }

    pub async fn open_generated_plugin_view(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        action_index: usize,
    ) -> Result<(), FrontendApiError> {
        let data = UiRequestData::ShowGeneratedPluginView {
            plugin_id,
            plugin_name,
            entrypoint_id,
            entrypoint_name,
            action_index,
        };
        let UiResponseData::Nothing = self.frontend_sender.send_receive(data).await? else {
            unreachable!()
        };

        Ok(())
    }

    pub async fn open_plugin_view(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
    ) -> Result<(), FrontendApiError> {
        let data = UiRequestData::ShowPluginView {
            plugin_id,
            plugin_name,
            entrypoint_id,
            entrypoint_name,
        };
        let UiResponseData::Nothing = self.frontend_sender.send_receive(data).await? else {
            unreachable!()
        };

        Ok(())
    }
}
