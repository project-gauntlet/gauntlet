use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use iced::{Element, Length};
use iced::Renderer;
use iced::widget::{Component, vertical_space};
use iced::widget::component;

use crate::model::{NativeUiPropertyValue, NativeUiWidget, NativeUiWidgetId};
use crate::ui::widget::{BuiltInWidgetEvent, BuiltInWidgetWrapper};
use common::model::PluginId;

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
    root_id: NativeUiWidgetId,
    next_id: NativeUiWidgetId,
    widget_map: HashMap<NativeUiWidgetId, BuiltInWidgetWrapper>,
}

impl Default for PluginViewContainer {
    fn default() -> Self {
        Self {
            root_id: 0,
            next_id: 1,
            widget_map: Default::default(),
        }
    }
}

impl PluginViewContainer {
    fn get_native_widget(&mut self, create_fn: impl FnOnce(NativeUiWidgetId) -> BuiltInWidgetWrapper) -> NativeUiWidget {
        let id = self.next_id;
        self.widget_map.insert(id, create_fn(id));

        self.next_id += 1;

        NativeUiWidget {
            widget_id: id
        }
    }

    fn get_builtin_widget(&mut self, ui_widget: NativeUiWidget) -> BuiltInWidgetWrapper {
        self.widget_map.get(&ui_widget.widget_id).unwrap().clone()
    }

    fn get_container(&mut self) -> NativeUiWidget {
        if let Entry::Vacant(value) = self.widget_map.entry(self.root_id) {
            value.insert(BuiltInWidgetWrapper::container(self.root_id));
        };

        NativeUiWidget {
            widget_id: self.root_id
        }
    }

    fn create_instance(&mut self, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        self.get_native_widget(|id| BuiltInWidgetWrapper::widget(id, widget_type, properties))
    }

    fn create_text_instance(&mut self, text: &str) -> NativeUiWidget {
        self.get_native_widget(|id| BuiltInWidgetWrapper::text(id, text))
    }

    fn append_child(&mut self, parent: NativeUiWidget, child: NativeUiWidget) {
        let parent = self.get_builtin_widget(parent);
        let child = self.get_builtin_widget(child);

        parent.append_child(child);
    }

    fn replace_container_children(&mut self, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        let container = self.get_builtin_widget(container);

        let children = new_children.into_iter()
            .map(|child| self.get_builtin_widget(child))
            .collect();

        container.set_children(children);
    }
}

impl Component<BuiltInWidgetEvent, Renderer> for PluginContainer {
    type State = ();
    type Event = BuiltInWidgetEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<BuiltInWidgetEvent> {
        Some(event)
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event, Renderer> {
        let client_context = self.client_context.read().expect("lock is poisoned");
        let container = client_context.get_view_container(&self.plugin_id);

        if let Some(widget) = container.widget_map.get(&container.root_id) {
            widget.render_widget()
        } else {
            vertical_space(Length::Fill).into()
        }
    }
}

impl<'a> From<PluginContainer> for Element<'a, BuiltInWidgetEvent> {
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

    pub fn get_container(&mut self, plugin_id: &PluginId) -> NativeUiWidget {
        self.get_view_container_mut(plugin_id).get_container()
    }

    pub fn create_instance(&mut self, plugin_id: &PluginId, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        self.get_view_container_mut(plugin_id).create_instance(widget_type, properties)
    }

    pub fn create_text_instance(&mut self, plugin_id: &PluginId, text: &str) -> NativeUiWidget {
        self.get_view_container_mut(plugin_id).create_text_instance(text)
    }

    pub fn append_child(&mut self, plugin_id: &PluginId, parent: NativeUiWidget, child: NativeUiWidget) {
        self.get_view_container_mut(plugin_id).append_child(parent, child)
    }

    pub fn clone_instance(&mut self, plugin_id: &PluginId, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        self.get_view_container_mut(plugin_id).create_instance(widget_type, properties)
    }

    pub fn replace_container_children(&mut self, plugin_id: &PluginId, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        self.get_view_container_mut(plugin_id).replace_container_children(container, new_children)
    }
}