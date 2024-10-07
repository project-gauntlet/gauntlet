use std::sync::{Arc, RwLock};

use iced::widget::component;
use iced::widget::{horizontal_space, Component};

use common::model::UiRenderLocation;

use crate::ui::client_context::ClientContext;
use crate::ui::theme::{Element, GauntletTheme};
use crate::ui::widget::{ActionPanel, ComponentRenderContext, ComponentWidgetEvent};
use crate::ui::AppMsg;

pub struct InlineViewContainer {
    client_context: Arc<RwLock<ClientContext>>,
}

pub fn inline_view_container(client_context: Arc<RwLock<ClientContext>>) -> InlineViewContainer {
    InlineViewContainer {
        client_context,
    }
}

impl Component<AppMsg, GauntletTheme> for InlineViewContainer {
    type State = ();
    type Event = ComponentWidgetEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<AppMsg> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let containers = client_context.get_all_inline_view_containers();

        match containers.first() {
            Some((plugin_id, _)) => Some(AppMsg::WidgetEvent {
                plugin_id: plugin_id.clone(),
                render_location: UiRenderLocation::InlineView,
                widget_event: event,
            }),
            None => None,
        }
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let containers = client_context.get_all_inline_view_containers();

        match containers.first() {
            Some((_, container)) => {
                container.render_widget(ComponentRenderContext::InlineRoot {
                    plugin_name: container.get_plugin_name(),
                    entrypoint_name: container.get_entrypoint_name(),
                })
            }
            None => {
                horizontal_space()
                    .into()
            }
        }
    }
}

impl<'a> From<InlineViewContainer> for Element<'a, AppMsg> {
    fn from(container: InlineViewContainer) -> Self {
        component(container)
    }
}

pub fn inline_view_action_panel(client_context: Arc<RwLock<ClientContext>>) -> Option<ActionPanel> {
    let client_context = client_context.read().expect("lock is poisoned");

    client_context.get_first_inline_view_action_panel()
}

