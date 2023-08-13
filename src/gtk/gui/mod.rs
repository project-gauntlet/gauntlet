use std::path::Path;
use gtk::gdk::Key;
use gtk::glib;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmRemoveAllExt, SimpleComponent};
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
    pub search_items: Vec<SearchItem>,
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
                        add_controller = gtk::EventControllerKey {
                             connect_key_released[sender] => move |_controller, key, _keycode, _state| {
                                if key == Key::q {
                                    sender.input(AppMsg::CloseCurrentView);
                                }
                            }
                        }
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
            state: AppState::SearchView
        };

        let list_view = &model.list.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::OpenView { plugin_container, plugin_id,  entrypoint_id} => {
                plugin_container.remove_all();

                let mut ui_context = self.plugin_manager.ui_context(&plugin_id).unwrap();
                ui_context.set_current_container(plugin_container.clone().upcast::<gtk::Widget>());
                ui_context.send_event(UiEvent::ViewCreated { view_name: entrypoint_id.to_owned() });

                self.state = AppState::PluginView {
                    plugin_id: plugin_id.clone(),
                    entrypoint_id: entrypoint_id.clone()
                };
            }
            AppMsg::CloseCurrentView => {
                match &self.state {
                    AppState::SearchView => {
                        panic!("invalid state");
                    }
                    AppState::PluginView { plugin_id, .. } => {
                        let mut ui_context = self.plugin_manager.ui_context(&plugin_id).unwrap();
                        ui_context.send_event(UiEvent::ViewDestroyed);

                        self.state = AppState::SearchView;
                    }
                }
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
