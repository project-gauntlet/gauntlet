use std::collections::HashMap;

use anyhow::anyhow;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::UiRenderLocation;
use gauntlet_utils::channel::RequestError;
use gauntlet_utils::channel::RequestSender;

use crate::model::JsClipboardData;
use crate::model::JsGeneratedSearchItem;
use crate::model::JsPreferenceUserData;
use crate::JsRequest;
use crate::JsResponse;
use crate::JsUiRenderLocation;

#[allow(async_fn_in_trait)]
pub trait BackendForPluginRuntimeApi {
    async fn reload_search_index(
        &self,
        generated_entrypoints: Vec<JsGeneratedSearchItem>,
        refresh_search_list: bool,
    ) -> anyhow::Result<()>;
    async fn get_asset_data(&self, path: &str) -> anyhow::Result<Vec<u8>>;
    async fn get_entrypoint_generator_entrypoint_ids(&self) -> anyhow::Result<Vec<String>>;
    async fn get_plugin_preferences(&self) -> anyhow::Result<HashMap<String, JsPreferenceUserData>>;
    async fn get_entrypoint_preferences(
        &self,
        entrypoint_id: EntrypointId,
    ) -> anyhow::Result<HashMap<String, JsPreferenceUserData>>;
    async fn plugin_preferences_required(&self) -> anyhow::Result<bool>;
    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> anyhow::Result<bool>;
    async fn clipboard_read(&self) -> anyhow::Result<JsClipboardData>;
    async fn clipboard_read_text(&self) -> anyhow::Result<Option<String>>;
    async fn clipboard_write(&self, data: JsClipboardData) -> anyhow::Result<()>;
    async fn clipboard_write_text(&self, data: String) -> anyhow::Result<()>;
    async fn clipboard_clear(&self) -> anyhow::Result<()>;
    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> anyhow::Result<()>;
    async fn ui_show_hud(&self, display: String) -> anyhow::Result<()>;
    async fn ui_hide_window(&self) -> anyhow::Result<()>;
    async fn ui_get_action_id_for_shortcut(
        &self,
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> anyhow::Result<Option<String>>;
    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> anyhow::Result<()>;
    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    ) -> anyhow::Result<()>;
    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> anyhow::Result<()>;
    async fn ui_clear_inline_view(&self) -> anyhow::Result<()>;
    async fn ui_synchronize_event(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct BackendForPluginRuntimeApiProxy {
    request_sender: RequestSender<JsRequest, Result<JsResponse, String>>,
}

impl BackendForPluginRuntimeApiProxy {
    pub fn new(request_sender: RequestSender<JsRequest, Result<JsResponse, String>>) -> Self {
        Self { request_sender }
    }

    async fn request(&self, request: JsRequest) -> anyhow::Result<JsResponse> {
        match self.request_sender.send_receive(request).await {
            Ok(ok) => Ok(ok.map_err(|e| anyhow!(e))?),
            Err(err) => {
                match err {
                    RequestError::TimeoutError => {
                        Err(anyhow!("Backend was unable to process message in a timely manner"))
                    }
                    RequestError::OtherSideWasDropped => Err(anyhow!("Plugin runtime is being stopped")),
                }
            }
        }
    }
}

impl BackendForPluginRuntimeApi for BackendForPluginRuntimeApiProxy {
    async fn reload_search_index(
        &self,
        generated_entrypoints: Vec<JsGeneratedSearchItem>,
        refresh_search_list: bool,
    ) -> anyhow::Result<()> {
        let request = JsRequest::ReloadSearchIndex {
            generated_entrypoints,
            refresh_search_list,
        };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn get_asset_data(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let request = JsRequest::GetAssetData { path: path.to_string() };

        match self.request(request).await? {
            JsResponse::AssetData { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn get_entrypoint_generator_entrypoint_ids(&self) -> anyhow::Result<Vec<String>> {
        let request = JsRequest::GetEntrypointGeneratorEntrypointIds;

        match self.request(request).await? {
            JsResponse::EntrypointGeneratorEntrypointIds { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn get_plugin_preferences(&self) -> anyhow::Result<HashMap<String, JsPreferenceUserData>> {
        let request = JsRequest::GetPluginPreferences;

        match self.request(request).await? {
            JsResponse::PluginPreferences { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn get_entrypoint_preferences(
        &self,
        entrypoint_id: EntrypointId,
    ) -> anyhow::Result<HashMap<String, JsPreferenceUserData>> {
        let request = JsRequest::GetEntrypointPreferences { entrypoint_id };

        match self.request(request).await? {
            JsResponse::EntrypointPreferences { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn plugin_preferences_required(&self) -> anyhow::Result<bool> {
        let request = JsRequest::PluginPreferencesRequired;

        match self.request(request).await? {
            JsResponse::PluginPreferencesRequired { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> anyhow::Result<bool> {
        let request = JsRequest::EntrypointPreferencesRequired { entrypoint_id };

        match self.request(request).await? {
            JsResponse::EntrypointPreferencesRequired { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn clipboard_read(&self) -> anyhow::Result<JsClipboardData> {
        let request = JsRequest::ClipboardRead;

        match self.request(request).await? {
            JsResponse::ClipboardRead { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn clipboard_read_text(&self) -> anyhow::Result<Option<String>> {
        let request = JsRequest::ClipboardReadText;

        match self.request(request).await? {
            JsResponse::ClipboardReadText { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn clipboard_write(&self, data: JsClipboardData) -> anyhow::Result<()> {
        let request = JsRequest::ClipboardWrite { data };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn clipboard_write_text(&self, data: String) -> anyhow::Result<()> {
        let request = JsRequest::ClipboardWriteText { data };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn clipboard_clear(&self) -> anyhow::Result<()> {
        let request = JsRequest::ClipboardClear;

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> anyhow::Result<()> {
        let request = JsRequest::UpdateLoadingBar { entrypoint_id, show };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_show_hud(&self, display: String) -> anyhow::Result<()> {
        let request = JsRequest::ShowHud { display };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_hide_window(&self) -> anyhow::Result<()> {
        let request = JsRequest::HideWindow;

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_get_action_id_for_shortcut(
        &self,
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> anyhow::Result<Option<String>> {
        let request = JsRequest::GetActionIdForShortcut {
            entrypoint_id,
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        };

        match self.request(request).await? {
            JsResponse::ActionIdForShortcut { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> anyhow::Result<()> {
        let request = JsRequest::Render {
            entrypoint_id,
            entrypoint_name,
            render_location: match render_location {
                UiRenderLocation::InlineView => JsUiRenderLocation::InlineView,
                UiRenderLocation::View => JsUiRenderLocation::View,
            },
            top_level_view,
            container,
        };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    ) -> anyhow::Result<()> {
        let request = JsRequest::ShowPluginErrorView {
            entrypoint_id,
            render_location: match render_location {
                UiRenderLocation::InlineView => JsUiRenderLocation::InlineView,
                UiRenderLocation::View => JsUiRenderLocation::View,
            },
        };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> anyhow::Result<()> {
        let request = JsRequest::ShowPreferenceRequiredView {
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required,
        };

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_clear_inline_view(&self) -> anyhow::Result<()> {
        let request = JsRequest::ClearInlineView;

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }

    async fn ui_synchronize_event(&self) -> anyhow::Result<()> {
        let request = JsRequest::SynchronizeEvent;

        match self.request(request).await? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value),
        }
    }
}
