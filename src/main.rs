use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;

use deno_core::futures::task::AtomicWaker;
use gtk::glib;
use gtk::glib::MainContext;
use gtk::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::LocalSet;

use crate::react_side::{UiEventName, PropertyValue, ReactContext, run_react, UiEvent, UiRequest, UiRequestData, UiResponseData, UiWidgetId, UiWidget};

mod react_side;

fn main() -> glib::ExitCode {
    let (react_request_sender, react_request_receiver) = tokio::sync::mpsc::unbounded_channel::<UiRequest>();
    let react_request_receiver = Rc::new(RefCell::new(react_request_receiver));

    let (react_event_sender, react_event_receiver) = std::sync::mpsc::channel::<UiEvent>();
    let event_waker = Arc::new(AtomicWaker::new());

    let gtk_context = UiContext::new(react_request_receiver, react_event_sender, event_waker.clone());

    let app = gtk::Application::builder()
        .application_id("org.gtk_rs.HelloWorld2")
        .build();

    thread::spawn(move || {
        let react_context = ReactContext::new(react_event_receiver, event_waker, react_request_sender);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let local_set = LocalSet::new();

        local_set.block_on(&runtime, async {
            run_react(react_context).await
        })
    });

    app.connect_activate(move |app| {
        build_ui(app, gtk_context.clone());
    });

    app.run()
}


#[derive(Clone)]
pub struct UiContext {
    request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>,
    event_sender: Sender<UiEvent>,
    event_waker: Arc<AtomicWaker>,
}

impl UiContext {
    fn new(request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>, event_sender: Sender<UiEvent>, event_waker: Arc<AtomicWaker>) -> UiContext {
        Self {
            request_receiver,
            event_sender,
            event_waker,
        }
    }
}


#[derive(Debug)]
pub struct GtkContext {
    next_id: UiWidgetId,
    widget_map: HashMap<UiWidgetId, gtk::Widget>,
    event_signal_handlers: HashMap<(UiWidgetId, UiEventName), glib::SignalHandlerId>,
}


fn build_ui(app: &gtk::Application, ui_context: UiContext) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .deletable(false)
        .resizable(false)
        .decorated(false)
        .default_height(400)
        .default_width(650)
        .title("My GTK App")
        .build();

    let spacing = 12;
    let entry = gtk::Entry::builder()
        .margin_top(spacing)
        .margin_bottom(spacing)
        .margin_start(spacing)
        .margin_end(spacing)
        .build();

    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);

    let string_list = gtk::StringList::new(&[
        "test1", "test2", "test3", "test4", "test5", "test5", "test5", "test5", "test5", "test5",
        "test5", "test5", "test5", "test5", "test5", "test5", "test5", "test5", "test5", "test5",
        "test5", "test5", "test5", "test5", "test5", "test5", "test5",
    ]);

    let selection = gtk::SingleSelection::new(Some(string_list));

    let factory = gtk::SignalListItemFactory::new();
    {
        factory.connect_setup(move |_, list_item| {
            let image = gtk::Image::from_file(Path::new("extension_icon.png"));
            let label = gtk::Label::builder().margin_start(6).build();

            let gtk_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(6)
                .margin_end(6)
                .build();

            gtk_box.append(&image);
            gtk_box.append(&label);

            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");
            list_item.set_child(Some(&gtk_box));

            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });
    }

    let list_view = gtk::ListView::builder()
        .model(&selection)
        .factory(&factory)
        .show_separators(true)
        .build();

    let ui_context = ui_context.clone();
    let window_in_list_view_callback = window.clone();
    list_view.connect_activate(move |list_view, position| {
        let ui_context = ui_context.clone();
        // let model = list_view.model().expect("The model has to exist.");
        // let string_object = model
        //     .item(position)
        //     .and_downcast::<gtk::StringObject>()
        //     .expect("The item has to be an `StringObject`.");

        let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window_in_list_view_callback.set_child(Some(&gtk_box));

        let ui_context = ui_context.clone();
        MainContext::default().spawn_local(async move {
            let ui_context = ui_context.clone();
            let context = Rc::new(RefCell::new(GtkContext { widget_map: HashMap::new(), event_signal_handlers: HashMap::new(), next_id: 0 }));
            while let Some(request) = ui_context.request_receiver.borrow_mut().recv().await {
                println!("got value");
                let gtk_box = gtk_box.clone();

                let UiRequest { response_sender: oneshot, data } = request;

                let get_gui_widget = |widget: gtk::Widget| -> UiWidget {
                    let mut context = context.borrow_mut();
                    let id = context.next_id;
                    context.widget_map.insert(id, widget);

                    context.next_id += 1;

                    UiWidget {
                        widget_id: id
                    }
                };

                let get_gtk_widget = |gui_widget: UiWidget| -> gtk::Widget {
                    let context = context.borrow();
                    context.widget_map.get(&gui_widget.widget_id).unwrap().clone()
                };

                let register_signal_handler_id = |widget_id: UiWidgetId, event: &UiEventName, signal_id: glib::SignalHandlerId| {
                    let mut context = context.borrow_mut();
                    context.event_signal_handlers.insert((widget_id, event.clone()), signal_id)
                };

                let unregister_signal_handler_id = |widget_id: UiWidgetId, event: &UiEventName| {
                    let mut context = context.borrow_mut();
                    if let Some(signal_handler_id) = context.event_signal_handlers.remove(&(widget_id, event.clone())) {
                        context.widget_map.get(&widget_id).unwrap().disconnect(signal_handler_id);
                    }
                };

                match data {
                    UiRequestData::GetContainer => {
                        let response_data = UiResponseData::GetContainer {
                            container: get_gui_widget(gtk_box.upcast::<gtk::Widget>())
                        };
                        oneshot.send(response_data).unwrap();
                    }
                    UiRequestData::CreateInstance { widget_type: type_ } => {
                        let widget: gtk::Widget = match type_.as_str() {
                            "box" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
                            "button1" => {
                                // TODO: not sure if lifetime of children is ok here
                                let button = gtk::Button::with_label(&type_);

                                button.into()
                            }
                            _ => panic!("jsx_type {} not supported", type_)
                        };

                        let response_data = UiResponseData::CreateInstance {
                            widget: get_gui_widget(widget)
                        };
                        oneshot.send(response_data).unwrap();
                    }
                    UiRequestData::CreateTextInstance { text } => {
                        let label = gtk::Label::new(Some(&text));

                        let response_data = UiResponseData::CreateTextInstance {
                            widget: get_gui_widget(label.upcast::<gtk::Widget>())
                        };
                        oneshot.send(response_data).unwrap();
                    }
                    UiRequestData::AppendChild { parent, child } => {
                        let parent = get_gtk_widget(parent);
                        let child = get_gtk_widget(child);

                        if let Some(gtk_box) = parent.downcast_ref::<gtk::Box>() {
                            gtk_box.append(&child);
                        } else if let Some(button) = parent.downcast_ref::<gtk::Button>() {
                            button.set_child(Some(&child));
                        }
                        oneshot.send(UiResponseData::Unit).unwrap();
                    }
                    UiRequestData::RemoveChild { parent, child } => {
                        let parent = get_gtk_widget(parent)
                            .downcast::<gtk::Box>()
                            .unwrap();
                        let child = get_gtk_widget(child);

                        parent.remove(&child);
                        oneshot.send(UiResponseData::Unit).unwrap();
                    }
                    UiRequestData::InsertBefore { parent, child, before_child } => {
                        let parent = get_gtk_widget(parent);
                        let child = get_gtk_widget(child);
                        let before_child = get_gtk_widget(before_child);

                        child.insert_before(&parent, Some(&before_child));
                        oneshot.send(UiResponseData::Unit).unwrap();
                    }
                    UiRequestData::SetProperties {
                        widget,
                        properties
                    } => {
                        let widget_id = widget.widget_id;
                        let widget = get_gtk_widget(widget);

                        for (name, value) in properties {
                            println!("setting property {:?} to value {:?}", name, value);
                            match value {
                                PropertyValue::Function => {
                                    let button = widget.downcast_ref::<gtk::Button>().unwrap();

                                    let react_event_sender = ui_context.event_sender.clone();
                                    let event_waker = ui_context.event_waker.clone();

                                    match name.as_str() {
                                        "onClick" => {
                                            println!("connect button listener");
                                            let event_name = name.clone();

                                            let signal_handler_id = button.connect_clicked(move |button| {
                                                println!("button clicked");
                                                let event_name = name.clone();
                                                react_event_sender.send(UiEvent {
                                                    event_name,
                                                    widget_id,
                                                }).unwrap();
                                                event_waker.wake();
                                            });

                                            unregister_signal_handler_id(widget_id, &event_name);
                                            register_signal_handler_id(widget_id, &event_name, signal_handler_id);
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
                        let widget = get_gtk_widget(widget);

                        let label = widget
                            .downcast_ref::<gtk::Label>()
                            .expect("unable to set text to non label widget");

                        label.set_label(&text);

                        oneshot.send(UiResponseData::Unit).unwrap();
                    }
                }
            }
        });

        println!("test timeout")

        // println!("test {}", string_object.string());

        // let label = gtk::Label::builder()
        //     .margin_start(6)
        //     .label(string_object.string())
        //     .build();
        //
        // window_in_list_view_callback.set_child(Some(&label));
    });

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list_view)
        .vexpand(true)
        .margin_top(spacing)
        .margin_bottom(spacing)
        .margin_start(spacing)
        .margin_end(spacing)
        .build();

    let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    gtk_box.append(&entry);
    gtk_box.append(&separator);
    gtk_box.append(&scrolled_window);

    window.set_child(Some(&gtk_box));

    // // Before the window is first realized, set it up to be a layer surface
    // gtk_layer_shell::init_for_window(&window);
    //
    // // Order below normal windows
    // gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
    //
    // // Push other windows out of the way
    // gtk_layer_shell::auto_exclusive_zone_enable(&window);
    //
    // // The margins are the gaps around the window's edges
    // // Margins and anchors can be set like this...
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 40);
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 40);
    // gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Top, 20);
    //
    // // ... or like this
    // // Anchors are if the window is pinned to each edge of the output
    // let anchors = [
    //     (gtk_layer_shell::Edge::Left, true),
    //     (gtk_layer_shell::Edge::Right, true),
    //     (gtk_layer_shell::Edge::Top, false),
    //     (gtk_layer_shell::Edge::Bottom, true),
    // ];
    //
    // for (anchor, state) in anchors {
    //     gtk_layer_shell::set_anchor(&window, anchor, state);
    // }

    window.present();
}
