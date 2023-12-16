use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use iced::{Length};
use iced::widget::{Component, vertical_space};
use iced::widget::component;

use common::model::PluginId;

use crate::model::{NativeUiPropertyValue, NativeUiWidget, NativeUiWidgetId};
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
    fn create_native_widget(&mut self, create_fn: impl FnOnce(NativeUiWidgetId) -> anyhow::Result<ComponentWidgetWrapper>) -> anyhow::Result<NativeUiWidget> {
        let id = self.next_id;
        let widget = create_fn(id)?;
        self.widget_map.insert(id, widget.clone());

        self.next_id += 1;

        Ok(widget.as_native_widget())
    }

    fn get_component_widget(&mut self, ui_widget: NativeUiWidget) -> anyhow::Result<ComponentWidgetWrapper> {
        self.get_component_widget_by_id(ui_widget.widget_id)
    }

    fn get_component_widget_by_id(&mut self, widget_id: NativeUiWidgetId) -> anyhow::Result<ComponentWidgetWrapper> {
        let wrapper = self.widget_map.get(&widget_id)
            .ok_or(anyhow::anyhow!("widget with id {:?} doesn't exist", widget_id))?;

        Ok(wrapper.clone())
    }

    fn get_root(&mut self) -> NativeUiWidget {
        tracing::trace!("get_root is called");

        if let Entry::Vacant(value) = self.widget_map.entry(self.root_id) {
            value.insert(ComponentWidgetWrapper::root(self.root_id));
        };

        let widget = self.get_component_widget_by_id(self.root_id)
            .expect("there should always be a root widget");

        widget.as_native_widget()
    }

    fn create_instance(&mut self, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> anyhow::Result<NativeUiWidget> {
        tracing::trace!("create_instance is called. widget_type: {:?}, new_props: {:?}", widget_type, properties);
        let widget = self.create_native_widget(|id| ComponentWidgetWrapper::widget(id, widget_type, properties));
        tracing::trace!("create_instance is returned. widget: {:?}", widget);
        widget
    }

    fn create_text_instance(&mut self, text: &str) -> anyhow::Result<NativeUiWidget> {
        tracing::trace!("create_text_instance is called. text: {:?}", text);
        let widget = self.create_native_widget(|id| ComponentWidgetWrapper::text_part(id, text));
        tracing::trace!("create_text_instance is returned. widget: {:?}", widget);
        widget
    }

    fn clone_instance(&mut self, widget: NativeUiWidget, widget_type: &str, new_props: HashMap<String, NativeUiPropertyValue>, keep_children: bool) -> anyhow::Result<NativeUiWidget> {
        tracing::trace!("clone_instance is called. widget: {:?}, widget_type: {:?}, new_props: {:?}, keep_children: {:?}", widget, widget_type, new_props, keep_children);

        let widget = self.get_component_widget(widget)?;

        let new_widget = self.create_native_widget(|id| ComponentWidgetWrapper::widget(id, widget_type, new_props))?;

        if keep_children {
            let new_widget_builtin = self.get_component_widget(new_widget.clone())?;
            if let Ok(children) = widget.get_children() {
                new_widget_builtin.set_children(children).expect("should always succeed")
            }
        }

        tracing::trace!("clone_instance is returned. widget: {:?}", widget);

        Ok(new_widget)
    }

    fn append_child(&mut self, parent: NativeUiWidget, child: NativeUiWidget) -> anyhow::Result<()> {
        tracing::trace!("append_child is called. parent: {:?}, child: {:?}", parent, child);
        let parent = self.get_component_widget(parent)?;
        let child = self.get_component_widget(child)?;

        parent.append_child(child)
    }

    fn replace_container_children(&mut self, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) -> anyhow::Result<()> {
        tracing::trace!("replace_container_children is called. container: {:?}, new_children: {:?}", container, new_children);
        let container = self.get_component_widget(container)?;

        let children = new_children.into_iter()
            .map(|child| self.get_component_widget(child))
            .collect::<anyhow::Result<Vec<_>>>()?;

        container.set_children(children)
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

        if let Some(widget) = container.widget_map.get(&container.root_id) {
            widget.render_widget(ComponentRenderContext::None)
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

    pub fn append_child(&mut self, plugin_id: &PluginId, parent: NativeUiWidget, child: NativeUiWidget) -> anyhow::Result<()> {
        self.get_view_container_mut(plugin_id).append_child(parent, child)
    }

    pub fn clone_instance(&mut self, plugin_id: &PluginId, widget: NativeUiWidget, widget_type: &str, new_props: HashMap<String, NativeUiPropertyValue>, keep_children: bool) -> anyhow::Result<NativeUiWidget> {
        self.get_view_container_mut(plugin_id).clone_instance(widget, widget_type, new_props, keep_children)
    }

    pub fn replace_container_children(&mut self, plugin_id: &PluginId, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) -> anyhow::Result<()> {
        self.get_view_container_mut(plugin_id).replace_container_children(container, new_children)
    }
}