use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::react_side::{PropertyValue, UiEvent, UiEventName, UiRequest, UiRequestData, UiResponseData, UiWidget, UiWidgetId};
use crate::PluginUiContext;

#[derive(Debug)]
pub struct GtkContext {
    next_id: UiWidgetId,
    widget_map: HashMap<UiWidgetId, gtk::Widget>,
    event_signal_handlers: HashMap<(UiWidgetId, UiEventName), glib::SignalHandlerId>,
}

impl GtkContext {
    fn new() -> Self {
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
        self.event_signal_handlers.insert((widget_id, event.clone()), signal_id);
    }

    fn unregister_signal_handler_id(&mut self, widget_id: UiWidgetId, event: &UiEventName) {
        if let Some(signal_handler_id) = self.event_signal_handlers.remove(&(widget_id, event.clone())) {
            self.widget_map.get(&widget_id).unwrap().disconnect(signal_handler_id);
        }
    }
}

pub(crate) fn start_request_receiver_loop(ui_contexts: Vec<PluginUiContext>) {
    for ui_context in ui_contexts {
        relm4::spawn_local(async {
            run_request_receiver_loop(ui_context).await
        });
    }
}

async fn run_request_receiver_loop(ui_context: PluginUiContext) {
    let context = Rc::new(RefCell::new(GtkContext::new()));

    while let Some(request) = ui_context.request_recv().await {
        let UiRequest { response_sender: oneshot, data } = request;

        let mut context = context.borrow_mut();

        match data {
            UiRequestData::GetContainer => {
                let container = ui_context.current_container().unwrap();
                let response_data = UiResponseData::GetContainer {
                    container: context.get_ui_widget(container)
                };
                oneshot.send(response_data).unwrap();
            }
            UiRequestData::CreateInstance { widget_type } => {
                let widget: gtk::Widget = match widget_type.as_str() {
                    "box" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
                    "button1" => {
                        // TODO: not sure if lifetime of children is ok here
                        let button = gtk::Button::with_label(&widget_type);

                        button.into()
                    }
                    _ => panic!("widget_type {} not supported", widget_type)
                };

                let response_data = UiResponseData::CreateInstance {
                    widget: context.get_ui_widget(widget)
                };
                oneshot.send(response_data).unwrap();
            }
            UiRequestData::CreateTextInstance { text } => {
                let label = gtk::Label::new(Some(&text));

                let response_data = UiResponseData::CreateTextInstance {
                    widget: context.get_ui_widget(label.upcast::<gtk::Widget>())
                };
                oneshot.send(response_data).unwrap();
            }
            UiRequestData::AppendChild { parent, child } => {
                let parent = context.get_gtk_widget(parent);
                let child = context.get_gtk_widget(child);

                if let Some(gtk_box) = parent.downcast_ref::<gtk::Box>() {
                    gtk_box.append(&child);
                } else if let Some(button) = parent.downcast_ref::<gtk::Button>() {
                    button.set_child(Some(&child));
                }
                oneshot.send(UiResponseData::Unit).unwrap();
            }
            UiRequestData::RemoveChild { parent, child } => {
                let parent = context.get_gtk_widget(parent)
                    .downcast::<gtk::Box>()
                    .unwrap();
                let child = context.get_gtk_widget(child);

                parent.remove(&child);
                oneshot.send(UiResponseData::Unit).unwrap();
            }
            UiRequestData::InsertBefore { parent, child, before_child } => {
                let parent = context.get_gtk_widget(parent);
                let child = context.get_gtk_widget(child);
                let before_child = context.get_gtk_widget(before_child);

                child.insert_before(&parent, Some(&before_child));
                oneshot.send(UiResponseData::Unit).unwrap();
            }
            UiRequestData::SetProperties {
                widget,
                properties
            } => {
                let widget_id = widget.widget_id;
                let widget = context.get_gtk_widget(widget);

                for (name, value) in properties {
                    println!("setting property {:?} to value {:?}", name, value);
                    match value {
                        PropertyValue::Function => {
                            let button = widget.downcast_ref::<gtk::Button>().unwrap();

                            let ui_context = ui_context.clone();

                            match name.as_str() {
                                "onClick" => {
                                    println!("connect button listener");
                                    let event_name = name.clone();

                                    let signal_handler_id = button.connect_clicked(move |_button| {
                                        println!("button clicked");
                                        let event_name = name.clone();
                                        ui_context.send_event(UiEvent::ViewEvent {
                                            event_name,
                                            widget_id,
                                        });
                                    });

                                    context.unregister_signal_handler_id(widget_id, &event_name);
                                    context.register_signal_handler_id(widget_id, &event_name, signal_handler_id);
                                }
                                _ => todo!()
                            };
                        }
                        PropertyValue::String(value) => {
                            widget.set_property(name.as_str(), value)
                        }
                        PropertyValue::Number(value) => {
                            widget.set_property(name.as_str(), value)
                        }
                        PropertyValue::Bool(value) => {
                            widget.set_property(name.as_str(), value)
                        }
                    }
                }

                oneshot.send(UiResponseData::Unit).unwrap();
            }
            UiRequestData::SetText { widget, text } => {
                let widget = context.get_gtk_widget(widget);

                let label = widget
                    .downcast_ref::<gtk::Label>()
                    .expect("unable to set text to non label widget");

                label.set_label(&text);

                oneshot.send(UiResponseData::Unit).unwrap();
            }
        }
    }
}