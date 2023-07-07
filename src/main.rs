use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::{Future, poll_fn};
use std::net::SocketAddr;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::task::Poll;
use std::thread;

use deno_core::{op, OpState, serde_v8, v8};
use deno_core::futures::task::AtomicWaker;
use serde::{Serialize, Deserialize};
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::inspector_server::InspectorServer;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use gtk::glib;
use gtk::glib::{MainContext};
use gtk::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::LocalSet;

fn main() -> glib::ExitCode {
    let (react_request_sender, react_request_receiver) = tokio::sync::mpsc::unbounded_channel::<UiRequest>();
    let react_request_receiver = Rc::new(RefCell::new(react_request_receiver));

    let (react_event_sender, react_event_receiver) = std::sync::mpsc::channel::<GuiEvent>();
    let event_waker = Arc::new(AtomicWaker::new());

    let gtk_context = GtkContext::new(react_request_receiver, react_event_sender, event_waker.clone());

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
            let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("react_renderer/dist/main.js");
            let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();

            let inspector_server = Arc::new(
                InspectorServer::new(
                    "127.0.0.1:9229".parse::<SocketAddr>().unwrap(),
                    "test",
                )
            );

            let mut worker = MainWorker::bootstrap_from_options(
                main_module.clone(),
                PermissionsContainer::allow_all(),
                WorkerOptions {
                    module_loader: Rc::new(FsModuleLoader),
                    extensions: vec![gtk_ext::init_ops(
                        EventHandlers::new(),
                        react_context
                    )],
                    // maybe_inspector_server: Some(inspector_server.clone()),
                    // should_wait_for_inspector_session: true,
                    // should_break_on_first_statement: true,
                    maybe_inspector_server: None,
                    should_wait_for_inspector_session: false,
                    should_break_on_first_statement: false,
                    ..Default::default()
                },
            );

            worker.execute_main_module(&main_module).await.unwrap();
            worker.run_event_loop(false).await.unwrap();
        })
    });

    app.connect_activate(move |app| {
        build_ui(app, gtk_context.clone());
    });

    app.run()
}

#[must_use]
async fn make_request(state: &Rc<RefCell<OpState>>, data: UiRequestData) -> UiResponseData {
    let react_context = {
        state.borrow()
            .borrow::<ReactContext>()
            .clone()
    };

    let (tx, rx) = tokio::sync::oneshot::channel();

    react_context.request_sender.send(UiRequest { response_sender: tx, data }).unwrap();

    rx.await.unwrap()
}

#[op]
pub async fn op_gtk_get_container(state: Rc<RefCell<OpState>>) -> GuiWidget {
    println!("op_gtk_get_container");

    let container = match make_request(&state, UiRequestData::GetContainer).await {
        UiResponseData::GetContainer { container: container_pointer } => container_pointer,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    println!("op_gtk_get_container end");

    container
}

#[op]
pub async fn op_gtk_append_child(
    state: Rc<RefCell<OpState>>,
    parent: GuiWidget,
    child: GuiWidget,
) {
    println!("op_gtk_append_child");

    let data = UiRequestData::AppendChild { parent, child };

    let _ = make_request(&state, data).await;
}

#[op]
pub async fn op_gtk_remove_child(
    state: Rc<RefCell<OpState>>,
    parent: GuiWidget,
    child: GuiWidget,
) {
    println!("op_gtk_remove_child");

    let data = UiRequestData::RemoveChild { parent, child };

    let _ = make_request(&state, data).await;
}

#[op]
pub async fn op_gtk_insert_before(
    state: Rc<RefCell<OpState>>,
    parent: GuiWidget,
    child: GuiWidget,
    before_child: GuiWidget,
) {
    println!("op_gtk_insert_before");

    let data = UiRequestData::InsertBefore {
        parent,
        child,
        before_child,
    };

    let _ = make_request(&state, data);
}

#[op]
pub async fn op_gtk_create_instance(
    state: Rc<RefCell<OpState>>,
    jsx_type: String,
) -> GuiWidget {
    println!("op_gtk_create_instance");

    let data = UiRequestData::CreateInstance {
        type_: jsx_type,
    };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateInstance { widget: widget_pointer } => widget_pointer,
        value @ _ => panic!("unsupported response type {:?}", value),
    };
    println!("op_gtk_create_instance end");

    widget
}

#[op]
pub async fn op_gtk_create_text_instance(
    state: Rc<RefCell<OpState>>,
    text: String,
) -> GuiWidget {
    println!("op_gtk_create_text_instance");

    let data = UiRequestData::CreateTextInstance { text };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateTextInstance { widget: widget_pointer } => widget_pointer,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    return widget;
}

#[op(v8)]
pub fn op_gtk_set_properties<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: GuiWidget,
    props: HashMap<String, serde_v8::Value<'a>>,
) -> Result<impl Future<Output=Result<(), deno_core::anyhow::Error>> + 'static, deno_core::anyhow::Error> {
    println!("op_gtk_set_properties");

    let mut state_ref = state.borrow_mut();
    let event_listeners = state_ref.borrow_mut::<EventHandlers>();

    let properties = props.iter()
        .filter(|(name, _)| name.as_str() != "children")
        .map(|(name, value)| {
            let val = value.v8_value;
            if val.is_function() {
                let fn_value: v8::Local<v8::Function> = val.try_into().unwrap();
                let global_fn = v8::Global::new(scope, fn_value);
                event_listeners.add_listener(widget.widget_id, name.clone(), global_fn);
                (name.clone(), PropertyValue::Function)
            } else if val.is_string() {
                (name.clone(), PropertyValue::String(val.to_rust_string_lossy(scope)))
            } else if val.is_number() {
                (name.clone(), PropertyValue::Number(val.number_value(scope).unwrap()))
            } else if val.is_boolean() {
                (name.clone(), PropertyValue::Bool(val.boolean_value(scope)))
            } else {
                panic!("{:?}: {:?}", name, val.type_of(scope).to_rust_string_lossy(scope))
            }
        })
        .collect::<HashMap<_, _>>();

    let data = UiRequestData::SetProperties {
        widget,
        properties,
    };

    drop(state_ref);

    Ok(async move {
        let _ = make_request(&state, data).await;

        Ok(())
    })
}

#[op]
pub async fn op_get_next_pending_gui_event<'a>(
    state: Rc<RefCell<OpState>>,
) -> GuiEvent2 {

    let react_context = {
        state.borrow()
            .borrow::<ReactContext>()
            .clone()
    };

    poll_fn(|cx| {
        let receiver = &react_context.event_receiver;
        receiver.waker.register(cx.waker());
        let receiver = receiver.inner.borrow();

        match receiver.try_recv() {
            Ok(value) => {
                println!("Poll::Ready {:?}", value);
                let event = GuiEvent2 {
                    widget_id: GuiWidget {
                        widget_id: value.widget_id
                    },
                    event_name: value.event_name
                };
                Poll::Ready(event)
            },
            Err(TryRecvError::Disconnected) => panic!("disconnected"),
            Err(TryRecvError::Empty) => Poll::Pending
        }
    }).await
}

#[op(v8)]
pub fn op_call_event_listener(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: GuiWidget,
    event_name: String,
) {
    println!("op_call_event_listener");

    let event_handlers = {
        state.borrow()
            .borrow::<EventHandlers>()
            .clone()
    };

    event_handlers.call_listener_handler(scope, &widget.widget_id, &event_name)
}

#[op]
pub async fn op_gtk_set_text(
    state: Rc<RefCell<OpState>>,
    widget: GuiWidget,
    text: String,
) {
    println!("op_gtk_set_text");

    let data = UiRequestData::SetText { widget, text };

    let _ = make_request(&state, data).await;
}

deno_core::extension!(
    gtk_ext,
    ops = [
        op_gtk_get_container,
        op_gtk_create_instance,
        op_gtk_create_text_instance,
        op_gtk_append_child,
        op_gtk_insert_before,
        op_gtk_set_properties,
        op_gtk_set_text,
        op_get_next_pending_gui_event,
        op_call_event_listener,
    ],
    options = {
        event_listeners: EventHandlers,
        react_context: ReactContext,
    },
    state = |state, options| {
        state.put(options.event_listeners);
        state.put(options.react_context);
    },
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
        ext.force_op_registration();
    },
);

#[derive(Clone)]
pub struct ReactContext {
    event_receiver: EventReceiver,
    request_sender: UnboundedSender<UiRequest>
}

impl ReactContext {
    fn new(receiver: Receiver<GuiEvent>, event_waker: Arc<AtomicWaker>, request_sender: UnboundedSender<UiRequest>) -> ReactContext {
        Self {
            event_receiver: EventReceiver::new(receiver, event_waker),
            request_sender
        }
    }
}

#[derive(Clone)]
pub struct GtkContext {
    request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>,
    event_sender: Sender<GuiEvent>,
    event_waker: Arc<AtomicWaker>,
}

impl GtkContext {
    fn new(request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>, event_sender: Sender<GuiEvent>, event_waker: Arc<AtomicWaker>) -> GtkContext {
        Self {
            request_receiver,
            event_sender,
            event_waker
        }
    }
}

#[derive(Clone)]
pub struct EventHandlers {
    inner: Rc<RefCell<EventHandlersInner>>,
}

pub struct EventHandlersInner {
    listeners: HashMap<WidgetId, HashMap<GtkEventName, v8::Global<v8::Function>>>,
}

impl EventHandlers {
    fn new() -> EventHandlers {
        Self {
            inner: Rc::new(RefCell::new(
                EventHandlersInner {
                    listeners: HashMap::new()
                }
            ))
        }
    }

    fn add_listener(&mut self, widget: WidgetId, event_name: GtkEventName, function: v8::Global<v8::Function>) {
        let mut inner = self.inner.borrow_mut();
        inner.listeners.entry(widget).or_default().insert(event_name, function);
    }

    fn call_listener_handler(&self, scope: &mut v8::HandleScope, widget: &WidgetId, event_name: &GtkEventName) {
        let inner = self.inner.borrow();
        let option_func = inner.listeners.get(widget)
            .map(|handlers| handlers.get(event_name))
            .flatten();

        if let Some(func) = option_func {
            let local_fn = v8::Local::new(scope, func);
            scope.enqueue_microtask(local_fn);
        };
    }
}


#[derive(Clone)]
pub struct EventReceiver {
    inner: Rc<RefCell<Receiver<GuiEvent>>>,
    waker: Arc<AtomicWaker>,
}

impl EventReceiver {
    fn new(receiver: Receiver<GuiEvent>, waker: Arc<AtomicWaker>) -> EventReceiver {
        Self {
            inner: Rc::new(RefCell::new(receiver)),
            waker
        }
    }
}

#[derive(Debug)]
pub struct UiContext {
    next_id: WidgetId,
    widget_map: HashMap<WidgetId, gtk::Widget>,
    event_signal_handlers: HashMap<(WidgetId, GtkEventName), glib::SignalHandlerId>
}


#[derive(Debug)]
pub struct UiRequest {
    response_sender: tokio::sync::oneshot::Sender<UiResponseData>,
    data: UiRequestData,
}

#[derive(Debug)]
pub enum UiResponseData {
    GetContainer {
        container: GuiWidget
    },
    CreateInstance {
        widget: GuiWidget
    },
    CreateTextInstance {
        widget: GuiWidget
    },
    Unit,
}

#[derive(Debug)]
pub enum UiRequestData {
    GetContainer,
    CreateInstance {
        type_: String,
    },
    CreateTextInstance {
        text: String,
    },
    AppendChild {
        parent: GuiWidget,
        child: GuiWidget,
    },
    RemoveChild {
        parent: GuiWidget,
        child: GuiWidget,
    },
    InsertBefore {
        parent: GuiWidget,
        child: GuiWidget,
        before_child: GuiWidget,
    },
    SetProperties {
        widget: GuiWidget,
        properties: HashMap<String, PropertyValue>,
    },
    SetText {
        widget: GuiWidget,
        text: String,
    },
}

#[derive(Debug)]
pub enum PropertyValue {
    Function,
    String(String),
    Number(f64),
    Bool(bool),
}


#[derive(Debug)]
pub struct GuiEvent {
    pub widget_id: WidgetId,
    pub event_name: GtkEventName,
}

pub type WidgetId = u32;
pub type GtkEventName = String;

#[derive(Debug, Deserialize, Serialize)]
pub struct GuiWidget {
    widget_id: WidgetId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuiEvent2 {
    pub widget_id: GuiWidget,
    pub event_name: GtkEventName,
}

impl GuiWidget {
    fn new(widget_id: WidgetId) -> Self {
        Self { widget_id }
    }
}

fn build_ui(app: &gtk::Application, gtk_context: GtkContext) {
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

    let gtk_context = gtk_context.clone();
    let window_in_list_view_callback = window.clone();
    list_view.connect_activate(move |list_view, position| {
        let gtk_context = gtk_context.clone();
        // let model = list_view.model().expect("The model has to exist.");
        // let string_object = model
        //     .item(position)
        //     .and_downcast::<gtk::StringObject>()
        //     .expect("The item has to be an `StringObject`.");

        let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window_in_list_view_callback.set_child(Some(&gtk_box));

        let gtk_context = gtk_context.clone();
        MainContext::default().spawn_local(async move {
            let gtk_context = gtk_context.clone();
            let context = Rc::new(RefCell::new(UiContext { widget_map: HashMap::new(), event_signal_handlers: HashMap::new(), next_id: 0 }));
            while let Some(request) = gtk_context.request_receiver.borrow_mut().recv().await {
                println!("got value");
                let gtk_box = gtk_box.clone();

                let UiRequest { response_sender: oneshot, data } = request;

                let get_gui_widget = |widget: gtk::Widget| -> GuiWidget {
                    let mut context = context.borrow_mut();
                    let id = context.next_id;
                    context.widget_map.insert(id, widget);

                    context.next_id += 1;

                    GuiWidget::new(id)
                };

                let get_gtk_widget = |gui_widget: GuiWidget| -> gtk::Widget {
                    let context = context.borrow();
                    context.widget_map.get(&gui_widget.widget_id).unwrap().clone()
                };

                let register_signal_handler_id = |widget_id: WidgetId, event: &GtkEventName, signal_id: glib::SignalHandlerId| {
                    let mut context = context.borrow_mut();
                    context.event_signal_handlers.insert((widget_id, event.clone()), signal_id)
                };

                let unregister_signal_handler_id = |widget_id: WidgetId, event: &GtkEventName| {
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
                    UiRequestData::CreateInstance { type_ } => {
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

                                    let react_event_sender = gtk_context.event_sender.clone();
                                    let event_waker = gtk_context.event_waker.clone();

                                    match name.as_str() {
                                        "onClick" => {
                                            println!("connect button listener");
                                            let event_name = name.clone();

                                            let signal_handler_id = button.connect_clicked(move |button| {
                                                println!("button clicked");
                                                let event_name = name.clone();
                                                react_event_sender.send(GuiEvent {
                                                    event_name,
                                                    widget_id,
                                                }).unwrap();
                                                event_waker.wake();
                                            });

                                            unregister_signal_handler_id(widget_id, &event_name);
                                            register_signal_handler_id(widget_id, &event_name, signal_handler_id);
                                        },
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
