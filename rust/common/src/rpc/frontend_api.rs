use std::collections::HashMap;

use gauntlet_utils::channel::RequestResult;
use gauntlet_utils_macros::boundary_gen;

use crate::model::EntrypointId;
use crate::model::PluginId;
use crate::model::RootWidget;
use crate::model::UiRenderLocation;
use crate::model::UiTheme;
use crate::model::UiWidgetId;
use crate::model::WindowPositionMode;

#[allow(async_fn_in_trait)]
#[boundary_gen(in_process)]
pub trait FrontendApi {
    async fn request_search_results_update(&self) -> RequestResult<()>;

    async fn replace_view(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
        data: HashMap<UiWidgetId, Vec<u8>>,
    ) -> RequestResult<()>;

    async fn toggle_window(&self) -> RequestResult<()>;

    async fn hide_window(&self) -> RequestResult<()>;

    async fn show_settings(&self) -> RequestResult<()>;

    async fn show_preference_required_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    ) -> RequestResult<()>;

    async fn show_plugin_error_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    ) -> RequestResult<()>;

    async fn show_hud(&self, display: String) -> RequestResult<()>;

    async fn update_loading_bar(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        show: bool,
    ) -> RequestResult<()>;

    async fn set_theme(&self, theme: UiTheme) -> RequestResult<()>;

    async fn set_window_position_mode(&self, mode: WindowPositionMode) -> RequestResult<()>;

    async fn open_generated_plugin_view(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    ) -> RequestResult<()>;

    async fn open_plugin_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> RequestResult<()>;

    async fn window_tracking_macos_focus_window(&self, window_uuid: String) -> RequestResult<()>;
}
