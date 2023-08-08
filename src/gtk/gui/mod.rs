use std::path::Path;
use gtk::gdk::Key;
use gtk::glib;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::typed_list_view::TypedListView;

use search_entry::SearchListEntry;

use crate::plugins::PluginManager;
use crate::react_side::UiEvent;
use crate::search::{SearchHandle, SearchItem};

mod search_entry;

const SPACING: i32 = 12;

pub struct AppModel {
    window: gtk::ApplicationWindow,
    search: SearchHandle,
    list: TypedListView<SearchListEntry, gtk::SingleSelection>,
    plugin_manager: PluginManager,
}

pub struct AppInput {
    pub search: SearchHandle,
    pub plugin_manager: PluginManager,
    pub search_items: Vec<SearchItem>,
}

#[derive(Debug)]
pub enum AppMsg {
    OpenView {
        plugin_id: String,
        entrypoint_id: String,
    },
    PromptChanged {
        value: String
    }
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type Init = AppInput;

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
                #[name = "search"]
                gtk::Entry {
                    set_margin_top: SPACING,
                    set_margin_bottom: SPACING,
                    set_margin_start: SPACING,
                    set_margin_end: SPACING,
                    connect_changed[sender] => move |entry| {
                        sender.input(AppMsg::PromptChanged {
                            value: entry.buffer().text().to_string(),
                        });
                    }
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
                            let item = get_item_from_list_view(list_view, pos);
                            let item = item.borrow::<SearchListEntry>();

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
        init_data: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let plugin_manager = init_data.plugin_manager;
        let search = init_data.search;
        let search_items: Vec<_> = init_data.search_items
            .into_iter()
            .map(|item| SearchListEntry::new(
                item.entrypoint_name,
                item.entrypoint_id,
                item.plugin_name,
                item.plugin_id,
                Some(Path::new("extension_icon.png").to_owned())
            ))
            .collect();

        let mut list = TypedListView::<SearchListEntry, gtk::SingleSelection>::new();

        list.extend_from_iter(search_items);

        let model = AppModel {
            window: root.clone(),
            search,
            list,
            plugin_manager,
        };

        let list_view = &model.list.view;

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
            AppMsg::PromptChanged { value } => {
                let result: Vec<_> = self.search.search(&value).unwrap()
                    .into_iter()
                    .map(|item| SearchListEntry::new(
                        item.entrypoint_name,
                        item.entrypoint_id,
                        item.plugin_name,
                        item.plugin_id,
                        None
                    ))
                    .collect();

                self.list.clear();
                self.list.extend_from_iter(result);
            }
        }
    }
}

fn get_item_from_list_view(list_view: &gtk::ListView, position: u32) -> glib::BoxedAnyObject {
    let model = list_view
        .model()
        .expect("The model has to exist.");

    let object = model
        .item(position)
        .and_downcast::<glib::BoxedAnyObject>()
        .expect("The item has to be an `BoxedAnyObject`, unless relm internals changed");

    return object;
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
