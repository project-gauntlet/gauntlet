use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;

use deno_core::futures::task::AtomicWaker;
use gtk::gdk::Key;
use gtk::glib;
use gtk::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::LocalSet;
use crate::gtk_side::start_request_receiver_loop;
use crate::plugins::{Plugin, PluginManager};

use crate::react_side::{PluginReactContext, run_react, UiEvent, UiRequest};

mod react_side;
mod gtk_side;
mod plugins;

fn main() -> glib::ExitCode {
    let mut plugin_manager = PluginManager::create();

    let (react_contexts, ui_contexts) = plugin_manager.create_all_contexts();

    let app = gtk::Application::builder()
        .application_id("org.placeholdername.placeholdername") // TODO what is proper letter case here?
        .build();

    spawn_react_thread(react_contexts);

    start_request_receiver_loop(ui_contexts.clone());

    app.connect_activate(move |app| {
        build_ui(app, plugin_manager.clone());
    });

    app.run()
}

fn spawn_react_thread(react_contexts: Vec<PluginReactContext>) {
    let handle = move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let local_set = LocalSet::new();

        local_set.block_on(&runtime, async {
            let mut join_set = tokio::task::JoinSet::new();
            for react_context in react_contexts {
                join_set.spawn_local(async {
                    run_react(react_context).await
                });
            }
            while let Some(_) = join_set.join_next().await {
            }
        })
    };

    thread::Builder::new()
        .name("react-thread".into())
        .spawn(handle)
        .expect("failed to spawn thread");
}


#[derive(Clone)]
pub struct PluginUiContext {
    plugin: Plugin,
    request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>,
    event_sender: Sender<UiEvent>,
    event_waker: Arc<AtomicWaker>,
    inner: Rc<RefCell<Option<gtk::Widget>>>, // FIXME bare option after cloning it seems to be set to none?
}

impl PluginUiContext {
    fn new(plugin: Plugin, request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>, event_sender: Sender<UiEvent>, event_waker: Arc<AtomicWaker>) -> PluginUiContext {
        Self {
            plugin,
            request_receiver,
            event_sender,
            event_waker,
            inner: Rc::new(RefCell::new(None))
        }
    }

    async fn request_recv(&self) -> Option<UiRequest> {
        self.request_receiver.borrow_mut().recv().await
    }

    fn send_event(&self, event: UiEvent) {
        self.event_sender.send(event).unwrap();
        self.event_waker.wake();
    }

    fn current_container(&self) -> Option<gtk::Widget> {
        self.inner.borrow().clone()
    }

    fn set_current_container(&mut self, container: gtk::Widget) {
        *self.inner.borrow_mut() = Some(container);
    }
}


glib::wrapper! {
    pub struct SearchEntry(ObjectSubclass<imp::SearchEntry>);
}

impl SearchEntry {
    pub fn new(
        entrypoint_name: &str,
        entrypoint_id: &str,
        plugin_name: &str,
        plugin_id: &str,
        image_path: Option<PathBuf>,
    ) -> Self {
        glib::Object::builder()
            .property("entrypoint-name", entrypoint_name.to_owned())
            .property("entrypoint-id", entrypoint_id.to_owned())
            .property("plugin-name", plugin_name.to_owned())
            .property("plugin-id", plugin_id.to_owned())
            .property("image-path", image_path)
            .build()
    }
}

mod imp {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::SearchEntry)]
    pub struct SearchEntry {
        #[property(get, set)]
        entrypoint_name: RefCell<String>,
        #[property(get, set)]
        entrypoint_id: RefCell<String>,
        #[property(get, set)]
        plugin_name: RefCell<String>,
        #[property(get, set)]
        plugin_id: RefCell<String>,

        #[property(get, set, nullable)]
        image_path: RefCell<Option<PathBuf>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchEntry {
        const NAME: &'static str = "SearchEntry";
        type Type = super::SearchEntry;
    }

    impl ObjectImpl for SearchEntry {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

fn build_ui(app: &gtk::Application, plugin_manager: PluginManager) {
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

    let entrypoints: Vec<_> = plugin_manager.plugins()
        .iter()
        .flat_map(|plugin| {
            plugin.entrypoints()
                .iter()
                .map(|entrypoint| {
                    SearchEntry::new(
                        entrypoint.name(),
                        entrypoint.id(),
                        plugin.name(),
                        plugin.id(),
                        Some(Path::new("extension_icon.png").to_owned())
                    )
                })
        })
        .collect();

    let model = gtk::gio::ListStore::new(SearchEntry::static_type());

    model.extend_from_slice(&entrypoints);

    let selection = gtk::SingleSelection::new(Some(model));

    let factory = gtk::SignalListItemFactory::new();
    {
        factory.connect_setup(move |_, list_item| {
            let image = gtk::Image::new();
            let label = gtk::Label::builder().margin_start(6).build();
            let sub_label = gtk::Label::builder().margin_start(6).build();

            let gtk_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(6)
                .margin_end(6)
                .build();

            gtk_box.append(&image);
            gtk_box.append(&label);
            gtk_box.append(&sub_label);

            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");
            list_item.set_child(Some(&gtk_box));

            list_item
                .property_expression("item")
                .chain_property::<SearchEntry>("entrypoint-name")
                .bind(&label, "label", gtk::Widget::NONE);

            list_item
                .property_expression("item")
                .chain_property::<SearchEntry>("plugin-name")
                .bind(&sub_label, "label", gtk::Widget::NONE);

            list_item
                .property_expression("item")
                .chain_property::<SearchEntry>("image-path")
                .bind(&image, "file", gtk::Widget::NONE);
        });
    }

    let list_view = gtk::ListView::builder()
        .model(&selection)
        .factory(&factory)
        .show_separators(true)
        .build();

    let plugin_manager = plugin_manager.clone();
    let window_clone = window.clone();
    list_view.connect_activate(move |list_view, position| {

        let model = list_view.model().expect("The model has to exist.");

        let search_entry = model
            .item(position)
            .and_downcast::<SearchEntry>()
            .expect("The item has to be an `SearchEntry`.");

        let plugin_id = search_entry.plugin_id();
        let entrypoint_id = search_entry.entrypoint_id();

        let mut plugin_manager = plugin_manager.clone();
        let mut ui_context = plugin_manager.ui_context(&plugin_id).unwrap();

        let prev_child = window_clone.child().unwrap().clone();

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window_clone.set_child(Some(&container.clone()));
        ui_context.set_current_container(container.clone().upcast::<gtk::Widget>());
        ui_context.send_event(UiEvent::ViewCreated { view_name: entrypoint_id });

        let window_clone = window_clone.clone();
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |controller, key, keycode, state| {
            if key == Key::q {
                ui_context.send_event(UiEvent::ViewDestroyed);
                window_clone.set_child(Some(&prev_child));
            }

            gtk::Inhibit(false)
        });
        container.add_controller(controller);
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
