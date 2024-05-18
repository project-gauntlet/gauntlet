use common::model::{EntrypointId, PluginId, UiPropertyValue, UiRenderLocation, UiWidget, UiWidgetId};

#[derive(Debug)]
pub enum NativeUiResponseData {
    Nothing,
}

#[derive(Debug)]
pub enum UiRequestData {
    ShowWindow,
    ClearInlineView {
        plugin_id: PluginId
    },
    ReplaceView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: UiWidget,
    },
    ShowPreferenceRequiredView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    ShowPluginErrorView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
    },
}

#[derive(Debug, Clone)]
pub enum UiViewEvent {
    View {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    Open {
        href: String
    },
}
