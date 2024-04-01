use std::sync::{Arc, RwLock};

use iced::widget::{Component, horizontal_space};
use iced::widget::component;

use common::model::RenderLocation;

use crate::ui::AppMsg;
use crate::ui::client_context::ClientContext;
use crate::ui::theme::{Element, GauntletTheme};
use crate::ui::widget::{ComponentRenderContext, ComponentWidgetEvent};

pub struct InlineViewContainer {
    client_context: Arc<RwLock<ClientContext>>,
}

pub fn inline_view_container(client_context: Arc<RwLock<ClientContext>>) -> InlineViewContainer {
    InlineViewContainer {
        client_context,
    }
}

#[derive(Default)]
pub struct PluginContainerState {
    current_plugin: usize
}

pub enum InlineViewContainerEvent {
    WidgetEvent(ComponentWidgetEvent),
}

impl Component<AppMsg, GauntletTheme> for InlineViewContainer {
    type State = PluginContainerState;
    type Event = InlineViewContainerEvent;

    fn update(
        &mut self,
        state: &mut Self::State,
        event: Self::Event,
    ) -> Option<AppMsg> {
        match event {
            InlineViewContainerEvent::WidgetEvent(event) => {
                let client_context = self.client_context.read().expect("lock is poisoned");
                let containers = client_context.get_all_inline_view_containers();
                let (plugin_id, _) = &containers[state.current_plugin];

                Some(AppMsg::WidgetEvent {
                    plugin_id: plugin_id.to_owned(),
                    render_location: RenderLocation::InlineView,
                    widget_event: event,
                })
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<Self::Event> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let containers = client_context.get_all_inline_view_containers();

        // TODO for some reason, this returns None sometimes
        if let Some((_, container)) = &containers.get(state.current_plugin) {
            container.render_widget(ComponentRenderContext::None)
                .map(InlineViewContainerEvent::WidgetEvent)
        } else {
            horizontal_space()
                .into()
        }
    }
}

impl<'a> From<InlineViewContainer> for Element<'a, AppMsg> {
    fn from(container: InlineViewContainer) -> Self {
        component(container)
    }
}