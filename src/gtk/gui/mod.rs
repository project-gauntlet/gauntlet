use std::path::Path;
use gtk::gdk::Key;
use gtk::glib;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmRemoveAllExt, SimpleComponent};
use relm4::typed_list_view::TypedListView;

use search_entry::SearchListEntry;
use crate::gtk::{PluginContainerContainer, PluginEventSenderContainer};

use crate::plugins::PluginManager;
use crate::react_side::UiEvent;
use crate::search::SearchHandle;

mod search_entry;

const SPACING: i32 = 12;

pub struct AppModel {
    window: gtk::ApplicationWindow,
    search: SearchHandle,
    list: TypedListView<SearchListEntry, gtk::SingleSelection>,
    plugin_manager: PluginManager,
    container_container: PluginContainerContainer,
    event_senders_container: PluginEventSenderContainer,
    state: AppState,
}

enum AppState {
    SearchView,
    PluginView {
        plugin_id: String,
        entrypoint_id: String,
    }
}

pub struct AppInput {
    pub search: SearchHandle,
    pub plugin_manager: PluginManager,
    pub container_container: PluginContainerContainer,
    pub event_senders_container: PluginEventSenderContainer,
}

#[derive(Debug)]
pub enum AppMsg {
    OpenView {
        plugin_container: gtk::Box,
        plugin_id: String,
        entrypoint_id: String,
    },
    CloseCurrentView,
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
            add_controller = gtk::EventControllerKey {
                 connect_key_released[sender] => move |_controller, key, _keycode, _state| {
                    if key == Key::Escape {
                        sender.input(AppMsg::CloseCurrentView);
                    }
                }
            },
            connect_is_active_notify => move |window| {
                if !window.is_active() {
                    // window.application().unwrap().quit()
                }
            },
            match model.state {
                AppState::SearchView => {
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
                                set_single_click_activate: true,
                                connect_activate[sender, plugin_container] => move |list_view, pos| {
                                    let item = get_item_from_list_view(list_view, pos);
                                    let item = item.borrow::<SearchListEntry>();

                                    sender.input(AppMsg::OpenView {
                                        plugin_container: plugin_container.clone(),
                                        plugin_id: item.plugin_id().to_owned(),
                                        entrypoint_id: item.entrypoint_id().to_owned()
                                    });
                                }
                            },
                        },
                    }
                },
                AppState::PluginView { .. } => {
                    #[name = "plugin_container"]
                    gtk::Box::new(gtk::Orientation::Vertical, 0) {
                        // plugin content
                    }
                }
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
        let container_container = init_data.container_container;
        let event_senders_container = init_data.event_senders_container;

        let list = TypedListView::<SearchListEntry, gtk::SingleSelection>::new();

        let mut model = AppModel {
            window: root.clone(),
            search,
            list,
            plugin_manager,
            container_container,
            event_senders_container,
            state: AppState::SearchView
        };

        model.initial_search();

        let list_view = &model.list.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::OpenView { plugin_container, plugin_id,  entrypoint_id} => {
                plugin_container.remove_all();

                self.event_senders_container.send_event(&plugin_id, UiEvent::ViewCreated {
                    view_name: entrypoint_id.to_owned()
                });

                self.container_container.set_current_container(&plugin_id, plugin_container.clone().upcast::<gtk::Widget>());

                self.state = AppState::PluginView {
                    plugin_id: plugin_id.clone(),
                    entrypoint_id: entrypoint_id.clone()
                };
            }
            AppMsg::CloseCurrentView => {
                match &self.state {
                    AppState::SearchView => {
                        self.window.application().unwrap().quit();
                    }
                    AppState::PluginView { plugin_id, .. } => {
                        self.event_senders_container.send_event(&plugin_id, UiEvent::ViewDestroyed);

                        self.state = AppState::SearchView;
                    }
                }
            }
            AppMsg::PromptChanged { value } => {
                self.search(&value);
            }
        }
    }
}

impl AppModel {
    fn initial_search(&mut self) {
        self.search("");
    }

    fn search(&mut self, value: &str) {
        let result: Vec<_> = self.search.search(value).unwrap()
            .into_iter()
            .map(|item| SearchListEntry::new(
                item.entrypoint_name,
                item.entrypoint_id,
                item.plugin_name,
                item.plugin_id,
                Some(Path::new("extension_icon.png").to_owned())
            ))
            .collect();

        self.list.clear();
        self.list.extend_from_iter(result);
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
