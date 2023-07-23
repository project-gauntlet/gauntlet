use std::cell::RefCell;
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
use crate::gtk_side::run_request_receiver_loop;

use crate::react_side::{ReactContext, run_react, UiEvent, UiRequest};

mod react_side;
mod gtk_side;

fn main() -> glib::ExitCode {
    let (react_context, ui_context) = create_contexts();

    let app = gtk::Application::builder()
        .application_id("org.gtk_rs.HelloWorld2")
        .build();

    thread::spawn(move || {
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
        build_ui(app, ui_context.clone());
    });

    app.run()
}

fn create_contexts() -> (ReactContext, UiContext) {
    let (react_request_sender, react_request_receiver) = tokio::sync::mpsc::unbounded_channel::<UiRequest>();
    let react_request_receiver = Rc::new(RefCell::new(react_request_receiver));

    let (react_event_sender, react_event_receiver) = std::sync::mpsc::channel::<UiEvent>();
    let event_waker = Arc::new(AtomicWaker::new());

    let ui_context = UiContext::new(react_request_receiver, react_event_sender, event_waker.clone());
    let react_context = ReactContext::new(react_event_receiver, event_waker, react_request_sender);

    (react_context, ui_context)
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
    list_view.connect_activate(move |_list_view, _position| {
        let ui_context = ui_context.clone();
        // let model = list_view.model().expect("The model has to exist.");
        // let string_object = model
        //     .item(position)
        //     .and_downcast::<gtk::StringObject>()
        //     .expect("The item has to be an `StringObject`.");

        let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window_in_list_view_callback.set_child(Some(&gtk_box));

        MainContext::default().spawn_local(async move {
            run_request_receiver_loop(ui_context.clone(), gtk_box.upcast::<gtk::Widget>()).await
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
