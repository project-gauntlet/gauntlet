use crate::model::{JsAdditionalSearchItem, JsClipboardData, JsPreferenceUserData};
use crate::{JsRequest, JsResponse, JsUiRenderLocation};
use gauntlet_common::model::{EntrypointId, RootWidget, UiRenderLocation};
use std::collections::HashMap;
use anyhow::anyhow;
use gauntlet_utils::channel::RequestSender;

#[allow(async_fn_in_trait)]
pub trait BackendForPluginRuntimeApi {
    async fn reload_search_index(&self, generated_commands: Vec<JsAdditionalSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> ;
    async fn get_asset_data(&self, path: &str) -> anyhow::Result<Vec<u8>>;
    async fn get_command_generator_entrypoint_ids(&self) -> anyhow::Result<Vec<String>>;
    async fn get_plugin_preferences(&self) -> anyhow::Result<HashMap<String, JsPreferenceUserData>>;
    async fn get_entrypoint_preferences(&self, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, JsPreferenceUserData>>;
    async fn plugin_preferences_required(&self) -> anyhow::Result<bool>;
    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> anyhow::Result<bool>;
    async fn clipboard_read(&self) -> anyhow::Result<JsClipboardData>;
    async fn clipboard_read_text(&self) -> anyhow::Result<Option<String>>;
    async fn clipboard_write(&self, data: JsClipboardData) -> anyhow::Result<()>;
    async fn clipboard_write_text(&self, data: String) -> anyhow::Result<()>;
    async fn clipboard_clear(&self) -> anyhow::Result<()>;
    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> anyhow::Result<()>;
    async fn ui_show_hud(&self, display: String) -> anyhow::Result<()>;
    async fn ui_get_action_id_for_shortcut(
        &self,
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<Option<String>>;
    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> anyhow::Result<()>;
    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation
    ) -> anyhow::Result<()>;
    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    ) -> anyhow::Result<()>;
    async fn ui_clear_inline_view(&self) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct BackendForPluginRuntimeApiProxy {
    request_sender: RequestSender<JsRequest, Result<JsResponse, String>>
}

impl BackendForPluginRuntimeApiProxy {
    pub fn new(request_sender: RequestSender<JsRequest, Result<JsResponse, String>>) -> Self {
        Self {
            request_sender
        }
    }
}

impl BackendForPluginRuntimeApi for BackendForPluginRuntimeApiProxy {
    async fn reload_search_index(&self, generated_commands: Vec<JsAdditionalSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> {
        let request = JsRequest::ReloadSearchIndex {
            generated_commands,
            refresh_search_list,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn get_asset_data(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let request = JsRequest::GetAssetData {
            path: path.to_string(),
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::AssetData { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn get_command_generator_entrypoint_ids(&self) -> anyhow::Result<Vec<String>> {
        let request = JsRequest::GetCommandGeneratorEntrypointIds;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::CommandGeneratorEntrypointIds { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn get_plugin_preferences(&self) -> anyhow::Result<HashMap<String, JsPreferenceUserData>> {
        let request = JsRequest::GetPluginPreferences;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::PluginPreferences { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn get_entrypoint_preferences(&self, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, JsPreferenceUserData>> {
        let request = JsRequest::GetEntrypointPreferences {
            entrypoint_id,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::EntrypointPreferences { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn plugin_preferences_required(&self) -> anyhow::Result<bool> {
        let request = JsRequest::PluginPreferencesRequired;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::PluginPreferencesRequired { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> anyhow::Result<bool> {
        let request = JsRequest::EntrypointPreferencesRequired {
            entrypoint_id,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::EntrypointPreferencesRequired { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn clipboard_read(&self) -> anyhow::Result<JsClipboardData> {
        let request = JsRequest::ClipboardRead;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::ClipboardRead { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn clipboard_read_text(&self) -> anyhow::Result<Option<String>> {
        let request = JsRequest::ClipboardReadText;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::ClipboardReadText { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn clipboard_write(&self, data: JsClipboardData) -> anyhow::Result<()> {
        let request = JsRequest::ClipboardWrite {
            data
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn clipboard_write_text(&self, data: String) -> anyhow::Result<()> {
        let request = JsRequest::ClipboardWriteText {
            data,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn clipboard_clear(&self) -> anyhow::Result<()> {
        let request = JsRequest::ClipboardClear;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> anyhow::Result<()> {
        let request = JsRequest::UpdateLoadingBar {
            entrypoint_id,
            show,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_show_hud(&self, display: String) -> anyhow::Result<()> {
        let request = JsRequest::ShowHud {
            display,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_get_action_id_for_shortcut(&self, entrypoint_id: EntrypointId, key: String, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> anyhow::Result<Option<String>> {
        let request = JsRequest::GetActionIdForShortcut {
            entrypoint_id,
            key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::ActionIdForShortcut { data } => Ok(data),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> anyhow::Result<()> {
        let request = JsRequest::Render {
            entrypoint_id,
            render_location: match render_location {
                UiRenderLocation::InlineView => JsUiRenderLocation::InlineView,
                UiRenderLocation::View => JsUiRenderLocation::View
            },
            top_level_view,
            container,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_show_plugin_error_view(&self, entrypoint_id: EntrypointId, render_location: UiRenderLocation) -> anyhow::Result<()> {
        let request = JsRequest::ShowPluginErrorView {
            entrypoint_id,
            render_location: match render_location {
                UiRenderLocation::InlineView => JsUiRenderLocation::InlineView,
                UiRenderLocation::View => JsUiRenderLocation::View
            },
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_show_preferences_required_view(&self, entrypoint_id: EntrypointId, plugin_preferences_required: bool, entrypoint_preferences_required: bool) -> anyhow::Result<()> {
        let request = JsRequest::ShowPreferenceRequiredView {
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required,
        };

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }

    async fn ui_clear_inline_view(&self) -> anyhow::Result<()> {
        let request = JsRequest::ClearInlineView;

        match self.request_sender.send_receive(request).await?.map_err(|e| anyhow!(e))? {
            JsResponse::Nothing => Ok(()),
            value @ _ => panic!("Unexpected JsResponse type: {:?}", value)
        }
    }
}