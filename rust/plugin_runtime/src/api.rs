use std::collections::HashMap;

use anyhow::anyhow;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::RootWidget;
use gauntlet_utils::channel::RequestError;
use gauntlet_utils_macros::boundary_gen;

use crate::model::JsClipboardData;
use crate::model::JsGeneratedSearchItem;
use crate::model::JsPreferenceUserData;
use crate::JsUiRenderLocation;

#[allow(async_fn_in_trait)]
#[boundary_gen]
pub trait BackendForPluginRuntimeApi {
    async fn reload_search_index(
        &self,
        generated_entrypoints: Vec<JsGeneratedSearchItem>,
        refresh_search_list: bool,
    ) -> anyhow::Result<()>;
    async fn get_asset_data(&self, path: String) -> anyhow::Result<Vec<u8>>;
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
        render_location: JsUiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> anyhow::Result<()>;
    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: JsUiRenderLocation,
    ) -> anyhow::Result<()>;
    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> anyhow::Result<()>;
    async fn ui_clear_inline_view(&self) -> anyhow::Result<()>;
}
