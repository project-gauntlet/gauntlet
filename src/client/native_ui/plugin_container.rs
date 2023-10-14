use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::{Element, Length};
use iced::Renderer;
use iced::widget::{button, column, Component, text, vertical_space};
use iced::widget::component;

use crate::client::model::{NativeUiPropertyValue, NativeUiWidget, NativeUiWidgetId};

pub struct PluginContainer {
    client_context: Arc<RwLock<ClientContext>>,
    plugin_uuid: String
}

pub fn plugin_container(client_context: Arc<RwLock<ClientContext>>, plugin_uuid: String) -> PluginContainer {
    PluginContainer {
        client_context,
        plugin_uuid
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
    fn get_native_widget(&mut self, widget: BuiltInWidget) -> NativeUiWidget {
        let id = self.next_id;
        self.widget_map.insert(id, BuiltInWidgetWrapper::new(id, widget));

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
            value.insert(BuiltInWidgetWrapper::new(self.root_id, BuiltInWidget::Container { children: vec![] }));
        };

        NativeUiWidget {
            widget_id: self.root_id
        }
    }

    fn create_instance(&mut self, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        let widget = match widget_type.as_ref() {
            "box" => BuiltInWidget::Container { children: vec![] },
            "button1" => BuiltInWidget::Button(widget_type.to_owned()),
            _ => panic!("widget_type {} not supported", widget_type)
        };

        self.get_native_widget(widget)
    }

    fn create_text_instance(&mut self, text: &str) -> NativeUiWidget {
        self.get_native_widget(BuiltInWidget::Text(text.to_owned()))
    }

    fn clone_instance(&mut self, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        let widget = self.create_instance(widget_type, properties);
        // let widget = self.get_builtin_widget(widget);
        widget
    }

    fn append_child(&mut self, parent: NativeUiWidget, child: NativeUiWidget) {
        let parent = self.get_builtin_widget(parent);
        let mut parent = parent.get_mut();
        match *parent {
            BuiltInWidget::Container { ref mut children } => {
                let child = self.get_builtin_widget(child);

                children.push(child)
            }
            BuiltInWidget::Button(_) => {}
            _ => panic!("parent not supported")
        };
    }

    fn replace_container_children(&mut self, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        let container = self.get_builtin_widget(container);
        let mut container = container.get_mut();
        match *container {
            BuiltInWidget::Container { ref mut children } => {
                *children = new_children.into_iter()
                    .map(|child| self.get_builtin_widget(child))
                    .collect();
            }
            BuiltInWidget::Button(_) => {}
            _ => panic!("not supported parent")
        };
    }
}

#[derive(Clone)]
struct BuiltInWidgetWrapper {
    id: NativeUiWidgetId,
    inner: Arc<RwLock<BuiltInWidget>>,
}

impl BuiltInWidgetWrapper {
    fn new(id: NativeUiWidgetId, widget: BuiltInWidget) -> Self {
        Self {
            id,
            inner: Arc::new(RwLock::new(widget)),
        }
    }

    fn id(&self) -> NativeUiWidgetId {
        self.id
    }

    fn get(&self) -> RwLockReadGuard<'_, BuiltInWidget> {
        self.inner.read().unwrap()
    }

    fn get_mut(&self) -> RwLockWriteGuard<'_, BuiltInWidget> {
        self.inner.write().unwrap()
    }
}

enum BuiltInWidget {
    Container {
        children: Vec<BuiltInWidgetWrapper>
    },
    Button(String),
    Text(String),
}

#[derive(Clone, Debug)]
pub enum BuiltInWidgetEvent {
    ButtonClick {
        widget_id: NativeUiWidgetId
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
        let client_context = self.client_context.read().unwrap();
        let container = client_context.get_view_container(&self.plugin_uuid);

        if let Some(widget) = container.widget_map.get(&container.root_id) {
            create_view_subtree(widget.clone())
        } else {
            vertical_space(Length::Fill).into()
        }
    }
}

fn create_view_subtree<'a>(widget_wrapper: BuiltInWidgetWrapper) -> Element<'a, BuiltInWidgetEvent> {
    let widget = widget_wrapper.get();
    match &*widget {
        BuiltInWidget::Container { children } => {
            let children: Vec<Element<_>> = children
                .into_iter()
                .map(|child| create_view_subtree(child.clone()))
                .collect();

            column(children)
                .into()
        }
        BuiltInWidget::Button(text_content) => {
            let text: Element<_> = text(text_content)
                .into();

            button(text)
                .on_press(BuiltInWidgetEvent::ButtonClick { widget_id: widget_wrapper.id() })
                .into()
        }
        BuiltInWidget::Text(text_content) => {
            text(text_content)
                .into()
        }
    }
}

impl<'a> From<PluginContainer> for Element<'a, BuiltInWidgetEvent> {
    fn from(container: PluginContainer) -> Self {
        component(container)
    }
}

pub struct ClientContext {
    pub containers: HashMap<String, PluginViewContainer>,
}

impl ClientContext {
    pub fn create_view_container(&mut self, plugin_uuid: impl ToString) {
        self.containers.insert(plugin_uuid.to_string(), PluginViewContainer::default());
    }

    pub fn get_view_container(&self, plugin_uuid: &str) -> &PluginViewContainer {
        self.containers.get(plugin_uuid).unwrap()
    }
    pub fn get_view_container_mut(&mut self, plugin_uuid: &str) -> &mut PluginViewContainer {
        self.containers.get_mut(plugin_uuid).unwrap()
    }

    pub fn get_container(&mut self, plugin_uuid: &str) -> NativeUiWidget {
        self.get_view_container_mut(plugin_uuid).get_container()
    }

    pub fn create_instance(&mut self, plugin_uuid: &str, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        self.get_view_container_mut(plugin_uuid).create_instance(widget_type, properties)
    }

    pub fn create_text_instance(&mut self, plugin_uuid: &str, text: &str) -> NativeUiWidget {
        self.get_view_container_mut(plugin_uuid).create_text_instance(text)
    }

    pub fn append_child(&mut self, plugin_uuid: &str, parent: NativeUiWidget, child: NativeUiWidget) {
        self.get_view_container_mut(plugin_uuid).append_child(parent, child)
    }

    pub fn clone_instance(&mut self, plugin_uuid: &str, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> NativeUiWidget {
        self.get_view_container_mut(plugin_uuid).clone_instance(widget_type, properties)
    }

    pub fn replace_container_children(&mut self, plugin_uuid: &str, container: NativeUiWidget, new_children: Vec<NativeUiWidget>) {
        self.get_view_container_mut(plugin_uuid).replace_container_children(container, new_children)
    }
}