use std::sync::{Arc, RwLock};

use iced::widget::Component;
use iced::widget::component;

use common::dbus::RenderLocation;
use common::model::PluginId;

use crate::ui::AppMsg;
use crate::ui::client_context::ClientContext;
use crate::ui::theme::{Element, GauntletRenderer};
use crate::ui::widget::{ComponentRenderContext, ComponentWidgetEvent};

pub struct ViewContainer {
    client_context: Arc<RwLock<ClientContext>>,
    plugin_id: PluginId,
}

pub fn view_container(client_context: Arc<RwLock<ClientContext>>, plugin_id: PluginId) -> ViewContainer {
    ViewContainer {
        client_context,
        plugin_id
    }
}

impl Component<AppMsg, GauntletRenderer> for ViewContainer {
    type State = ();
    type Event = ComponentWidgetEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<AppMsg> {
        Some(AppMsg::WidgetEvent {
            plugin_id: self.plugin_id.clone(),
            render_location: RenderLocation::View,
            widget_event: event,
        })
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let view_container = client_context.get_view_container();
        view_container.render_widget(ComponentRenderContext::None)
    }
}

impl<'a> From<ViewContainer> for Element<'a, AppMsg> {
    fn from(container: ViewContainer) -> Self {
        component(container)
    }
}