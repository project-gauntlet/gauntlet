use std::path::Path;

use gtk::gdk::Key;
use gtk::glib;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::typed_list_view::TypedListView;

use crate::plugins::PluginManager;
use crate::react_side::UiEvent;
use search_entry::SearchEntry;

mod search_entry;

const SPACING: i32 = 12;

pub struct AppModel {
    window: gtk::ApplicationWindow,
    plugin_manager: PluginManager,
    entrypoints: Vec<SearchEntry>,
}

#[derive(Debug)]
pub enum AppMsg {
    OpenView {
        plugin_id: String,
        entrypoint_id: String,
    },
}

#[relm4::component(pub)]
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
                                plugin_id: item.plugin_id().to_owned(),
                                entrypoint_id: item.entrypoint_id().to_owned()
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
    // FIXME this is ugly, but relm's conditional widgets seem broken when used on enums
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
