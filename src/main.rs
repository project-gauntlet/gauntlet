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
use relm4::{ComponentParts, ComponentSender, gtk, RelmApp,  SimpleComponent};
use relm4::typed_list_view::{RelmListItem, TypedListView};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::LocalSet;

use crate::gtk_side::start_request_receiver_loop;
use crate::plugins::{Plugin, PluginManager};
use crate::react_side::{PluginReactContext, run_react, UiEvent, UiRequest};

mod react_side;
mod gtk_side;
mod plugins;


const SPACING: i32 = 12;


#[derive(Debug, Clone)]
struct SearchEntry {
    entrypoint_name: String,
    entrypoint_id: String,
    plugin_name: String,
    plugin_id: String,
    image_path: Option<PathBuf>,
}

impl SearchEntry {
    fn new(
        entrypoint_name: &str,
        entrypoint_id: &str,
        plugin_name: &str,
        plugin_id: &str,
        image_path: Option<PathBuf>,
    ) -> Self {
        Self {
            entrypoint_name: entrypoint_name.to_owned(),
            entrypoint_id: entrypoint_id.to_owned(),
            plugin_name: plugin_name.to_owned(),
            plugin_id: plugin_id.to_owned(),
            image_path,
        }
    }
}

struct Widgets {
    image: gtk::Image,
    label: gtk::Label,
    sub_label: gtk::Label,
}

impl RelmListItem for SearchEntry {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        relm4::view! {
            my_box = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_top: 6,
                set_margin_bottom: 6,
                set_margin_start: 6,
                set_margin_end: 6,

                #[name = "image"]
                gtk::Image,

                #[name = "label"]
                gtk::Label {
                    set_margin_start: 6,
                },

                #[name = "sub_label"]
                gtk::Label {
                    set_margin_start: 6,
                },
            }
        }

        let widgets = Widgets {
            image,
            label,
            sub_label,
        };

        (my_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets {
            image,
            label,
            sub_label,
        } = widgets;

        if let Some(path) = &self.image_path {
            image.set_file(Some(path.to_str().unwrap())) // FIXME this shouldn't be fallible
        }

        label.set_label(&self.entrypoint_name);
        sub_label.set_label(&self.plugin_name);
    }
}

#[derive(Debug)]
enum AppMsg {
    OpenView {
        plugin_id: String,
        entrypoint_id: String,
    },
}

struct AppModel {
    window: gtk::ApplicationWindow,
    plugin_manager: PluginManager,
    entrypoints: Vec<SearchEntry>,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type Init = PluginManager;

    view! {
        #[name = "window"]
        gtk::ApplicationWindow {
            set_title: Some("My GTK App"),
            set_deletable: false,
            set_resizable: false,
            set_decorated: false,
            set_default_height: 400,
            set_default_width: 650,

            gtk::Box::new(gtk::Orientation::Vertical, 0) {
                gtk::Entry {
                    set_margin_top: SPACING,
                    set_margin_bottom: SPACING,
                    set_margin_start: SPACING,
                    set_margin_end: SPACING,
                },

                gtk::Separator::new(gtk::Orientation::Horizontal),

                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vexpand: true,
                    set_margin_top: SPACING,
                    set_margin_bottom: SPACING,
                    set_margin_start: SPACING,
                    set_margin_end: SPACING,

                    #[local_ref]
                    list_view -> gtk::ListView {
                        connect_activate[sender] => move |list_view, pos| {
                            let model = list_view
                                .model()
                                .expect("The model has to exist.");

                            let object = model
                                .item(pos)
                                .and_downcast::<glib::BoxedAnyObject>()
                                .expect("The item has to be an `BoxedAnyObject`, unless relm internals changed");

                            let item = object.borrow::<SearchEntry>();

                            sender.input(AppMsg::OpenView {
                                plugin_id: item.plugin_id.to_owned(),
                                entrypoint_id: item.entrypoint_id.to_owned()
                            });
                        }
                    },
                },
            }
        }
    }

    fn init(
        plugin_manager: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

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

        let mut list = TypedListView::<SearchEntry, gtk::SingleSelection>::new();

        list.extend_from_iter(entrypoints.clone());

        let model = AppModel {
            window: root.clone(),
            plugin_manager,
            entrypoints,
        };

        let list_view = &list.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::OpenView { plugin_id, entrypoint_id} => {
                create_list_view(
                    self.plugin_manager.clone(),
                    self.window.clone(),
                    &plugin_id,
                    &entrypoint_id,
                )
            }
        }
    }
}

fn create_list_view(mut plugin_manager: PluginManager, window: gtk::ApplicationWindow, plugin_id: &str, entrypoint_id: &str) {
    let mut ui_context = plugin_manager.ui_context(&plugin_id).unwrap();

    let prev_child = window.child().unwrap().clone();

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    window.set_child(Some(&container.clone()));
    ui_context.set_current_container(container.clone().upcast::<gtk::Widget>());
    ui_context.send_event(UiEvent::ViewCreated { view_name: entrypoint_id.to_owned() });

    let window = window.clone();
    let controller = gtk::EventControllerKey::new();
    controller.connect_key_pressed(move |_controller, key, _keycode, _state| {
        if key == Key::q {
            ui_context.send_event(UiEvent::ViewDestroyed);
            window.set_child(Some(&prev_child));
        }

        gtk::Inhibit(false)
    });
    container.add_controller(controller);
}

fn main() {
    let mut plugin_manager = PluginManager::create();

    let (react_contexts, ui_contexts) = plugin_manager.create_all_contexts();

    // TODO what is proper letter case here?
    let app = RelmApp::new("org.placeholdername.placeholdername");

    spawn_react_thread(react_contexts);

    start_request_receiver_loop(ui_contexts.clone());

    app.run::<AppModel>(plugin_manager.clone());
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