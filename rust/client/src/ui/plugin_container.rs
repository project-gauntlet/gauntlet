use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use iced::widget::Component;
use iced::widget::component;

use common::model::PluginId;

use crate::model::NativeUiWidget;
use crate::ui::theme::{Element, GauntletRenderer};
use crate::ui::widget::{ComponentRenderContext, ComponentWidgetEvent, ComponentWidgetWrapper};

pub struct PluginContainer {
    client_context: Arc<RwLock<ClientContext>>,
    plugin_id: PluginId
}

pub fn plugin_container(client_context: Arc<RwLock<ClientContext>>, plugin_id: PluginId) -> PluginContainer {
    PluginContainer {
        client_context,
        plugin_id
    }
}

pub struct PluginViewContainer {
    root_widget: ComponentWidgetWrapper,
}

impl Default for PluginViewContainer {
    fn default() -> Self {
        Self {
            root_widget: ComponentWidgetWrapper::root(0),
        }
    }
}

impl PluginViewContainer {
    fn create_component_widget(&mut self, ui_widget: NativeUiWidget) -> ComponentWidgetWrapper {
        let children = ui_widget.widget_children
            .into_iter()
            .map(|ui_widget| self.create_component_widget(ui_widget))
            .collect();

        ComponentWidgetWrapper::widget(ui_widget.widget_id, &ui_widget.widget_type, ui_widget.widget_properties, children)
            .expect("unable to create widget")
    }

    fn replace_container_children(&mut self, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        tracing::trace!("replace_container_children is called. container: {:?}, new_children: {:?}", container, new_children);

        let children = new_children.into_iter()
            .map(|child| self.create_component_widget(child))
            .collect::<Vec<_>>();

        self.root_widget.find_child_with_id(container.widget_id)
            .unwrap_or_else(|| panic!("widget with id {:?} doesn't exist", container.widget_id))
            .set_children(children)
            .expect("unable to set children");
    }
}

impl Component<ComponentWidgetEvent, GauntletRenderer> for PluginContainer {
    type State = ();
    type Event = ComponentWidgetEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<ComponentWidgetEvent> {
        Some(event)
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let container = client_context.get_view_container(&self.plugin_id);

        container.root_widget
            .render_widget(ComponentRenderContext::None)
    }
}

impl<'a> From<PluginContainer> for Element<'a, ComponentWidgetEvent> {
    fn from(container: PluginContainer) -> Self {
        component(container)
    }
}

pub struct ClientContext {
    pub containers: HashMap<PluginId, PluginViewContainer>,
}

impl ClientContext {
    pub fn create_view_container(&mut self, plugin_id: PluginId) {
        self.containers.insert(plugin_id, PluginViewContainer::default());
    }

    pub fn get_view_container(&self, plugin_id: &PluginId) -> &PluginViewContainer {
        self.containers.get(plugin_id).unwrap()
    }
    pub fn get_view_container_mut(&mut self, plugin_id: &PluginId) -> &mut PluginViewContainer {
        self.containers.get_mut(plugin_id).unwrap()
    }

    pub fn replace_container_children(&mut self, plugin_id: &PluginId, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        self.get_view_container_mut(plugin_id).replace_container_children(container, new_children)
    }
}