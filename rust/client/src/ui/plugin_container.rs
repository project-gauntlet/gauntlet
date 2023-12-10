use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use iced::{Element, Length};
use iced::Renderer;
use iced::widget::{Component, vertical_space};
use iced::widget::component;

use common::model::PluginId;

use crate::model::{NativeUiPropertyValue, NativeUiWidget, NativeUiWidgetId};
use crate::ui::widget::{ComponentWidgetEvent, ComponentWidgetWrapper};

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
    widget_map: HashMap<NativeUiWidgetId, ComponentWidgetWrapper>,
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
    fn create_native_widget(&mut self, widget_type: &str, create_fn: impl FnOnce(NativeUiWidgetId) -> anyhow::Result<ComponentWidgetWrapper>) -> anyhow::Result<NativeUiWidget> {
        let id = self.next_id;
        self.widget_map.insert(id, create_fn(id)?);

        self.next_id += 1;

        Ok(NativeUiWidget {
            widget_id: id,
            widget_type: widget_type.to_owned(),
        })
    }

    fn get_builtin_widget(&mut self, ui_widget: NativeUiWidget) -> ComponentWidgetWrapper {
        self.widget_map.get(&ui_widget.widget_id).unwrap().clone()
    }

    fn get_root(&mut self) -> NativeUiWidget {
        tracing::trace!("get_root is called");
        if let Entry::Vacant(value) = self.widget_map.entry(self.root_id) {
            value.insert(ComponentWidgetWrapper::root(self.root_id));
        };

        NativeUiWidget {
            widget_id: self.root_id,
            widget_type: "___root___".to_owned()
        }
    }

    fn create_instance(&mut self, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> anyhow::Result<NativeUiWidget> {
        tracing::trace!("create_instance is called. widget_type: {:?}, new_props: {:?}", widget_type, properties);
        let widget = self.create_native_widget(widget_type, |id| ComponentWidgetWrapper::widget(id, widget_type, properties));
        tracing::trace!("create_instance is returned. widget: {:?}", widget);
        widget
    }

    fn create_text_instance(&mut self, text: &str) -> anyhow::Result<NativeUiWidget> {
        tracing::trace!("create_text_instance is called. text: {:?}", text);
        let widget = self.create_native_widget("text", |id| ComponentWidgetWrapper::text_part(id, text));
        tracing::trace!("create_text_instance is returned. widget: {:?}", widget);
        widget
    }

    fn clone_instance(&mut self, widget: NativeUiWidget, widget_type: &str, new_props: HashMap<String, NativeUiPropertyValue>, keep_children: bool) -> anyhow::Result<NativeUiWidget> {
        tracing::trace!("clone_instance is called. widget: {:?}, widget_type: {:?}, new_props: {:?}, keep_children: {:?}", widget, widget_type, new_props, keep_children);

        let widget = self.get_builtin_widget(widget);

        let new_widget = self.create_native_widget(widget_type, |id| ComponentWidgetWrapper::widget(id, widget_type, new_props))?;

        if keep_children {
            let new_widget_builtin = self.get_builtin_widget(new_widget.clone());
            if new_widget_builtin.can_have_children() {
                new_widget_builtin.set_children(widget.get_children());
            }
        }

        tracing::trace!("clone_instance is returned. widget: {:?}", widget);

        Ok(new_widget)
    }

    fn append_child(&mut self, parent: NativeUiWidget, child: NativeUiWidget) {
        tracing::trace!("append_child is called. parent: {:?}, child: {:?}", parent, child);
        let parent = self.get_builtin_widget(parent);
        let child = self.get_builtin_widget(child);

        parent.append_child(child);
    }

    fn replace_container_children(&mut self, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        tracing::trace!("replace_container_children is called. container: {:?}, new_children: {:?}", container, new_children);
        let container = self.get_builtin_widget(container);

        let children = new_children.into_iter()
            .map(|child| self.get_builtin_widget(child))
            .collect();

        container.set_children(children);
    }
}

impl Component<ComponentWidgetEvent, Renderer> for PluginContainer {
    type State = ();
    type Event = ComponentWidgetEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<ComponentWidgetEvent> {
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

    pub fn get_root(&mut self, plugin_id: &PluginId) -> NativeUiWidget {
        self.get_view_container_mut(plugin_id).get_root()
    }

    pub fn create_instance(&mut self, plugin_id: &PluginId, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> anyhow::Result<NativeUiWidget> {
        self.get_view_container_mut(plugin_id).create_instance(widget_type, properties)
    }

    pub fn create_text_instance(&mut self, plugin_id: &PluginId, text: &str) -> anyhow::Result<NativeUiWidget> {
        self.get_view_container_mut(plugin_id).create_text_instance(text)
    }

    pub fn append_child(&mut self, plugin_id: &PluginId, parent: NativeUiWidget, child: NativeUiWidget) {
        self.get_view_container_mut(plugin_id).append_child(parent, child)
    }

    pub fn clone_instance(&mut self, plugin_id: &PluginId, widget: NativeUiWidget, widget_type: &str, new_props: HashMap<String, NativeUiPropertyValue>, keep_children: bool) -> anyhow::Result<NativeUiWidget> {
        self.get_view_container_mut(plugin_id).clone_instance(widget, widget_type, new_props, keep_children)
    }

    pub fn replace_container_children(&mut self, plugin_id: &PluginId, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        self.get_view_container_mut(plugin_id).replace_container_children(container, new_children)
    }
}