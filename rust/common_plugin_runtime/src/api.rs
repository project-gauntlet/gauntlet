use std::collections::HashMap;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::RootWidget;
use gauntlet_utils::channel::RequestResult;
use gauntlet_utils_macros::boundary_gen;

use crate::model::JsClipboardData;
use crate::model::JsGeneratedSearchItem;
use crate::model::JsPreferenceUserData;
use crate::model::JsUiRenderLocation;

#[allow(async_fn_in_trait)]
#[boundary_gen(bincode, in_process)]
pub trait BackendForPluginRuntimeApi {
    async fn reload_search_index(
        &self,
        generated_entrypoints: Vec<JsGeneratedSearchItem>,
        refresh_search_list: bool,
    ) -> RequestResult<()>;
    async fn get_asset_data(&self, path: String) -> RequestResult<Vec<u8>>;
    async fn get_entrypoint_generator_entrypoint_ids(&self) -> RequestResult<Vec<String>>;
    async fn get_plugin_preferences(&self) -> RequestResult<HashMap<String, JsPreferenceUserData>>;
    async fn get_entrypoint_preferences(
        &self,
        entrypoint_id: EntrypointId,
    ) -> RequestResult<HashMap<String, JsPreferenceUserData>>;
    async fn plugin_preferences_required(&self) -> RequestResult<bool>;
    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> RequestResult<bool>;
    async fn clipboard_read(&self) -> RequestResult<JsClipboardData>;
    async fn clipboard_read_text(&self) -> RequestResult<Option<String>>;
    async fn clipboard_write(&self, data: JsClipboardData) -> RequestResult<()>;
    async fn clipboard_write_text(&self, data: String) -> RequestResult<()>;
    async fn clipboard_clear(&self) -> RequestResult<()>;
    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> RequestResult<()>;
    async fn ui_show_hud(&self, display: String) -> RequestResult<()>;
    async fn ui_hide_window(&self) -> RequestResult<()>;
    async fn ui_show_settings(&self) -> RequestResult<()>;
    async fn ui_get_action_id_for_shortcut(
        &self,
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> RequestResult<Option<String>>;
    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: JsUiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
    ) -> RequestResult<()>;
    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: JsUiRenderLocation,
    ) -> RequestResult<()>;
    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> RequestResult<()>;
}
