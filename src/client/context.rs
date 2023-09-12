use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use relm4::gtk;

use crate::server::plugins::js::{UiEvent, UiEventName, UiPropertyValue, UiResponseData, UiWidget, UiWidgetId};
use crate::utils::channel::RequestSender;

pub struct ClientContext {
    pub contexts: HashMap<String, GtkContext>,
    pub containers: PluginContainerContainer,
}

impl ClientContext {
    pub fn get_container(&mut self, plugin_uuid: &str) -> UiResponseData {
        let container = self.containers.current_container(plugin_uuid).unwrap();
        let widget = self.contexts.get_mut(plugin_uuid)
            .unwrap()
            .get_ui_widget(container.clone());

        UiResponseData::GetContainer {
            container: widget
        }
    }

    pub fn create_instance(&mut self, plugin_uuid: &str, widget_type: &str) -> UiResponseData {
        let widget: gtk::Widget = match widget_type {
            "box" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
            "button1" => {
                // TODO: not sure if lifetime of children is ok here
                let button = gtk::Button::with_label(&widget_type);

                button.into()
            }
            _ => panic!("widget_type {} not supported", widget_type)
        };
        let widget = self.contexts.get_mut(plugin_uuid)
            .unwrap()
            .get_ui_widget(widget);

        UiResponseData::CreateInstance {
            widget
        }
    }

    pub fn create_text_instance(&mut self, plugin_uuid: &str, text: &str) -> UiResponseData {
        let label = gtk::Label::new(Some(&text));
        let widget = self.contexts.get_mut(plugin_uuid)
            .unwrap()
            .get_ui_widget(label.upcast::<gtk::Widget>());

        UiResponseData::CreateTextInstance {
            widget
        }
    }

    pub fn append_child(&mut self, plugin_uuid: &str, parent: UiWidget, child: UiWidget) {
        let parent = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(parent);
        let child = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(child);

        if let Some(gtk_box) = parent.downcast_ref::<gtk::Box>() {
            gtk_box.append(&child);
        } else if let Some(button) = parent.downcast_ref::<gtk::Button>() {
            button.set_child(Some(&child));
        }
    }

    pub fn remove_child(&mut self, plugin_uuid: &str, parent: UiWidget, child: UiWidget) {
        let parent = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(parent)
            .downcast::<gtk::Box>()
            .unwrap();

        let child = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(child);

        parent.remove(&child);
    }

    pub fn insert_before(&mut self, plugin_uuid: &str, parent: UiWidget, child: UiWidget, before_child: UiWidget) {
        let parent = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(parent);

        let child = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(child);

        let before_child = self.contexts.get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(before_child);

        child.insert_before(&parent, Some(&before_child));
    }

    pub async fn set_properties(
        &mut self,
        event_sender: PluginEventSenderContainer,
        plugin_uuid: &str,
        widget: UiWidget,
        properties: HashMap<String, UiPropertyValue>,
    ) {
        let widget_id = widget.widget_id;
        let widget = self.contexts
            .get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(widget);

        for (name, value) in properties {
            match value {
                UiPropertyValue::Function => {
                    let button = widget.downcast_ref::<gtk::Button>().unwrap();

                    match name.as_str() {
                        "onClick" => {
                            let event_name = name.clone();

                            let event_plugin_uuid = plugin_uuid.to_owned();
                            let event_sender = event_sender.clone();

                            let signal_handler_id = button.connect_clicked(move |_button| {
                                let event_name = name.clone();
                                let event = UiEvent::ViewEvent {
                                    event_name,
                                    widget_id,
                                };

                                event_sender.send_event(&event_plugin_uuid, event);
                            });

                            let context = self.contexts.get_mut(plugin_uuid).unwrap();
                            context.register_signal_handler_id(widget_id, &event_name, signal_handler_id);
                        }
                        _ => todo!()
                    };
                }
                UiPropertyValue::String(value) => {
                    widget.set_property(name.as_str(), value)
                }
                UiPropertyValue::Number(value) => {
                    widget.set_property(name.as_str(), value)
                }
                UiPropertyValue::Bool(value) => {
                    widget.set_property(name.as_str(), value)
                }
            }
        }
    }

    pub fn set_text(&mut self, plugin_uuid: &str, widget: UiWidget, text: &str) {
        let widget = self.contexts
            .get(plugin_uuid)
            .unwrap()
            .get_gtk_widget(widget);

        let label = widget
            .downcast_ref::<gtk::Label>()
            .expect("unable to set text to non label widget");

        label.set_label(&text);
    }
}


#[derive(Debug)]
pub struct GtkContext {
    next_id: UiWidgetId,
    widget_map: HashMap<UiWidgetId, gtk::Widget>,
    event_signal_handlers: HashMap<(UiWidgetId, UiEventName), glib::SignalHandlerId>,
}

impl GtkContext {
    pub fn new() -> Self {
        GtkContext { widget_map: HashMap::new(), event_signal_handlers: HashMap::new(), next_id: 0 }
    }

    fn get_ui_widget(&mut self, widget: gtk::Widget) -> UiWidget {
        let id = self.next_id;
        self.widget_map.insert(id, widget);

        self.next_id += 1;

        UiWidget {
            widget_id: id
        }
    }

    fn get_gtk_widget(&self, ui_widget: UiWidget) -> gtk::Widget {
        self.widget_map.get(&ui_widget.widget_id).unwrap().clone()
    }

    fn register_signal_handler_id(&mut self, widget_id: UiWidgetId, event: &UiEventName, signal_id: glib::SignalHandlerId) {
        if let Some(signal_handler_id) = self.event_signal_handlers.remove(&(widget_id, event.clone())) {
            self.widget_map.get(&widget_id).unwrap().disconnect(signal_handler_id);
        }
        self.event_signal_handlers.insert((widget_id, event.clone()), signal_id);
    }
}

#[derive(Clone)]
pub struct PluginContainerContainer { // creative name, isn't it?
    containers: Rc<RefCell<HashMap<String, gtk::Widget>>>,
}

impl PluginContainerContainer {
    pub(crate) fn new() -> Self {
        Self {
            containers: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    pub fn current_container(&self, plugin_id: &str) -> Option<gtk::Widget> {
        self.containers.borrow().get(plugin_id).cloned()
    }

    pub fn set_current_container(&mut self, plugin_id: &str, container: gtk::Widget) {
        self.containers.borrow_mut().insert(plugin_id.to_owned(), container);
    }
}

#[derive(Clone)]
pub struct PluginEventSenderContainer {
    sender: RequestSender<(String, UiEvent), ()>,
}

impl PluginEventSenderContainer {
    pub fn new(sender: RequestSender<(String, UiEvent), ()>) -> Self {
        Self {
            sender,
        }
    }

    pub fn send_event(&self, plugin_uuid: &str, event: UiEvent) {
        self.sender.send((plugin_uuid.to_owned(), event)).unwrap();
    }
}

