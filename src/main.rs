use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use deno_core::{op, OpState};
use serde::{Serialize, Deserialize};
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::inspector_server::InspectorServer;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use gtk::glib;
use gtk::glib::MainContext;
use gtk::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::LocalSet;

fn main() -> glib::ExitCode {

    let (react_request_sender, react_request_receiver) = tokio::sync::mpsc::unbounded_channel();
    let react_request_receiver = Rc::new(RefCell::new(react_request_receiver));

    let app = gtk::Application::builder()
        .application_id("org.gtk_rs.HelloWorld2")
        .build();

    thread::spawn(|| {
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
                    "test"
                )
            );
            let mut worker = MainWorker::bootstrap_from_options(
                main_module.clone(),
                PermissionsContainer::allow_all(),
                WorkerOptions {
                    module_loader: Rc::new(FsModuleLoader),
                    extensions: vec![gtk_ext::init_ops(react_request_sender)],
                    // maybe_inspector_server: Some(inspector_server),
                    maybe_inspector_server: None,
                    should_wait_for_inspector_session: false,
                    should_break_on_first_statement: false,
                    ..Default::default()
                },
            );
            worker.execute_main_module(&main_module).await.unwrap();

            loop {
                worker.run_event_loop(false).await.unwrap();
            }
        })
    });

    app.connect_activate(move |app| {
        let react_request_receiver = Rc::clone(&react_request_receiver);
        build_ui(app, react_request_receiver);
    });

    app.run()
}

#[must_use]
async fn make_request(state: &Rc<RefCell<OpState>>, data: UiRequestData) -> UiResponseData {
    let sender = {
        state.borrow()
            .borrow::<UnboundedSender<UiRequest>>()
            .clone()
    };

    let (tx, rx) = tokio::sync::oneshot::channel();

    sender.send(UiRequest { response_sender: tx, data }).unwrap();

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
    // props: serde_v8::Value<'scope>,
) -> GuiWidget {

    println!("op_gtk_create_instance");

    let data = UiRequestData::CreateInstance {
        type_: jsx_type,
        // props: Default::default(),
    };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateInstance { widget: widget_pointer } => widget_pointer,
        value @ _=> panic!("unsupported response type {:?}", value),
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
        value @ _=> panic!("unsupported response type {:?}", value),
    };

    return widget;
}

deno_core::extension!(
    gtk_ext,
    ops = [
        op_gtk_get_container,
        op_gtk_create_instance,
        op_gtk_create_text_instance,
        op_gtk_append_child,
        op_gtk_insert_before,
    ],
    options = {
        react_request_sender: UnboundedSender<UiRequest>,
    },
    state = |state, options| {
        state.put(options.react_request_sender);
    },
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
        ext.force_op_registration();
    },
);

#[derive(Debug)]
pub struct UiContext {
    next_id: WidgetId,
    widget_map: HashMap<WidgetId, gtk::Widget>
}


#[derive(Debug)]
pub struct UiRequest {
    response_sender: tokio::sync::oneshot::Sender<UiResponseData>,
    data: UiRequestData
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
    Unit
}

#[derive(Debug)]
pub enum UiRequestData {
    GetContainer,
    CreateInstance {
        type_: String,
        // props: HashMap<String, serde_v8::Global>,
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
}

#[derive(Debug)]
pub enum GuiEvent {
    ButtonClick,
}

pub type WidgetId = u32;

#[derive(Debug, Deserialize, Serialize)]
pub struct GuiWidget {
    widget_id: WidgetId,
}

impl GuiWidget {
    fn new(widget_id: WidgetId) -> Self {
        Self { widget_id }
    }
}

fn build_ui(app: &gtk::Application, react_request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>) {
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

    let window_in_list_view_callback = window.clone();
    list_view.connect_activate(move |list_view, position| {
        let react_request_receiver = Rc::clone(&react_request_receiver);
        // let model = list_view.model().expect("The model has to exist.");
        // let string_object = model
        //     .item(position)
        //     .and_downcast::<gtk::StringObject>()
        //     .expect("The item has to be an `StringObject`.");

        let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window_in_list_view_callback.set_child(Some(&gtk_box));

        MainContext::default().spawn_local(async move {
            let react_request_receiver = Rc::clone(&react_request_receiver);
            let context = Rc::new(RefCell::new(UiContext { widget_map: HashMap::new(), next_id: 0 }));
            while let Some(request) = react_request_receiver.borrow_mut().recv().await {
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

                match data {
                    UiRequestData::GetContainer => {
                        let response_data = UiResponseData::GetContainer {
                            container: get_gui_widget(gtk_box.upcast::<gtk::Widget>())
                        };
                        oneshot.send(response_data).unwrap();
                    }
                    UiRequestData::CreateInstance { type_/*, props*/ } => {
                        let widget: gtk::Widget = match type_.as_str() {
                            "box" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
                            // "button1" => gtk::Box::new(gtk::Orientation::Horizontal, 6).into(),
                            "button1" => {
                                // let children_name = v8::String::new(scope, "children").unwrap();
                                // let on_click_name = v8::String::new(scope, "onClick").unwrap();

                                // let children = props.get(scope, children_name.into()).unwrap();
                                // let children: v8::Local<v8::String> = children.try_into().unwrap();
                                // let children: String = children.to_rust_string_lossy(scope);

                                // let on_click = props.get(scope, on_click_name.into()).unwrap();
                                // let on_click: v8::Local<v8::Function> = on_click.try_into().unwrap();
                                //
                                // let nested_scope = v8::HandleScope::new(scope);

                                // TODO: not sure if lifetime of children is ok here
                                let button = gtk::Button::with_label(&type_);
                                // let button = gtk::Button::with_label(&children);
                                button.connect_clicked(move |button| {
                                    // let nested_scope = &mut nested_scope;

                                    // let null: v8::Local<v8::Value> = v8::null(scope).into();

                                    // on_click.call(scope, null, &[]);

                                    // tx.send(GuiEvent::ButtonClick).unwrap();

                                    println!("button pressed");
                                });

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

                        let response_data = UiResponseData::CreateInstance {
                            widget: get_gui_widget(label.upcast::<gtk::Widget>())
                        };
                        oneshot.send(response_data).unwrap();
                    }
                    UiRequestData::AppendChild { parent, child } => {
                        let parent = get_gtk_widget(parent);
                        let child = get_gtk_widget(child);

                        if let Some(gtk_box) = parent.downcast_ref::<gtk::Box>() {
                            gtk_box.append(&child)
                        } else if let Some(button) = parent.downcast_ref::<gtk::Button>() {
                            button.set_child(Some(&child))
                        }
                    }
                    UiRequestData::RemoveChild { parent, child } => {
                        let parent = get_gtk_widget(parent)
                            .downcast::<gtk::Box>()
                            .unwrap();
                        let child = get_gtk_widget(child);

                        parent.remove(&child)
                    }
                    UiRequestData::InsertBefore { parent, child, before_child } => {
                        let parent = get_gtk_widget(parent);
                        let child = get_gtk_widget(child);
                        let before_child = get_gtk_widget(before_child);

                        child.insert_before(&parent, Some(&before_child))
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
