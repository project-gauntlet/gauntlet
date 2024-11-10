use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use iced::widget::component;
use iced::widget::Component;

use common::model::{EntrypointId, PhysicalShortcut, PluginId, UiRenderLocation};

use crate::ui::client_context::ClientContext;
use crate::ui::state::PluginViewState;
use crate::ui::theme::{Element, GauntletTheme};
use crate::ui::widget::ComponentWidgetEvent;
use crate::ui::AppMsg;

pub struct ViewContainer {
    client_context: Arc<RwLock<ClientContext>>,
    plugin_view_state: PluginViewState,
    plugin_id: PluginId,
    plugin_name: String,
    entrypoint_id: EntrypointId,
    entrypoint_name: String,
    action_shortcuts: HashMap<String, PhysicalShortcut>,
}

pub fn view_container(
    client_context: Arc<RwLock<ClientContext>>,
    plugin_view_state: PluginViewState,
    plugin_id: PluginId,
    plugin_name: String,
    entrypoint_id: EntrypointId,
    entrypoint_name: String,
    action_shortcuts: HashMap<String, PhysicalShortcut>
) -> ViewContainer {
    ViewContainer {
        client_context,
        plugin_view_state,
        plugin_id,
        plugin_name,
        entrypoint_id,
        entrypoint_name,
        action_shortcuts
    }
}

impl Component<AppMsg, GauntletTheme> for ViewContainer {
    type State = ();
    type Event = ComponentWidgetEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<AppMsg> {
        Some(AppMsg::WidgetEvent {
            plugin_id: self.plugin_id.clone(),
            render_location: UiRenderLocation::View,
            widget_event: event,
        })
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let view_container = client_context.get_view_container();
        view_container.render_root_widget(
            &self.plugin_view_state,
            &self.action_shortcuts,
        )
    }
}

impl<'a> From<ViewContainer> for Element<'a, AppMsg> {
    fn from(container: ViewContainer) -> Self {
        component(container)
    }
}