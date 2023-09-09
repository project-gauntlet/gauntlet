use std::collections::HashMap;

use gtk::glib;
use gtk::prelude::*;

use crate::gtk::{PluginContainerContainer, PluginEventSenderContainer};
use crate::react_side::{DBusUiPropertyContainer, DBusUiWidget, UiEvent, UiEventName, UiEventViewCreated, UiEventViewEvent, UiPropertyValue, UiRequestData, UiResponseData, UiWidget, UiWidgetId};
use crate::channel::RequestSender;

pub struct DbusClient {
    pub(crate) channel: RequestSender<(String, UiRequestData), UiResponseData>,
}

#[zbus::dbus_interface(name = "org.placeholdername.PlaceHolderName.Client")]
impl DbusClient {
    #[dbus_interface(signal)]
    pub async fn view_created_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_uuid: &str, event: UiEventViewCreated) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    pub async fn view_event_signal(signal_ctxt: &zbus::SignalContext<'_>, plugin_uuid: &str, event: UiEventViewEvent) -> zbus::Result<()>;

    async fn get_container(&mut self, plugin_uuid: &str) -> DBusUiWidget {
        let input = (plugin_uuid.to_owned(), UiRequestData::GetContainer);

        match self.channel.send_receive(input).await.unwrap() {
            UiResponseData::GetContainer { container } => container.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_instance(&mut self, plugin_uuid: &str, widget_type: &str) -> DBusUiWidget {
        let data = UiRequestData::CreateInstance { widget_type: widget_type.to_owned() };
        let input = (plugin_uuid.to_owned(), data);

        match self.channel.send_receive(input).await.unwrap() {
            UiResponseData::CreateInstance { widget } => widget.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn create_text_instance(&mut self, plugin_uuid: &str, text: &str) -> DBusUiWidget {
        let data = UiRequestData::CreateTextInstance { text: text.to_owned() };
        let input = (plugin_uuid.to_owned(), data);

        match self.channel.send_receive(input).await.unwrap() {
            UiResponseData::CreateTextInstance { widget } => widget.into(),
            value @ _ => panic!("unsupported response type {:?}", value),
        }
    }

    async fn append_child(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) {
        let data = UiRequestData::AppendChild { parent: parent.into(), child: child.into() };
        let input = (plugin_uuid.to_owned(), data);
        self.channel.send(input).unwrap();
    }

    async fn insert_before(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget, before_child: DBusUiWidget) {
        let data = UiRequestData::InsertBefore { parent: parent.into(), child: child.into(), before_child: before_child.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    async fn remove_child(&mut self, plugin_uuid: &str, parent: DBusUiWidget, child: DBusUiWidget) {
        let data = UiRequestData::RemoveChild { parent: parent.into(), child: child.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    async fn set_properties(
        &mut self,
        plugin_uuid: &str,
        widget: DBusUiWidget,
        properties: DBusUiPropertyContainer,
    ) {
        let data = UiRequestData::SetProperties { widget: widget.into(), properties: properties.into() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }

    fn set_text(&mut self, plugin_uuid: &str, widget: DBusUiWidget, text: &str) {
        let data = UiRequestData::SetText { widget: widget.into(), text: text.to_owned() };
        self.channel.send((plugin_uuid.to_owned(), data)).unwrap();
    }
}

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
