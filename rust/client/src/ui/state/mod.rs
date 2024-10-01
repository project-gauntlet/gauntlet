mod main_view;
mod plugin_view;

use crate::ui::client_context::ClientContext;
use crate::ui::scroll_handle::ScrollHandle;
pub use crate::ui::state::main_view::MainViewState;
pub use crate::ui::state::plugin_view::PluginViewState;
use crate::ui::AppMsg;
use common::model::{EntrypointId, PhysicalShortcut, PluginId, SearchResult};
use iced::widget::text_input;
use iced::widget::text_input::focus;
use iced::Command;
use std::collections::HashMap;
use std::sync::{Arc, RwLock as StdRwLock};

pub enum GlobalState {
    MainView {
        // logic
        search_field_id: text_input::Id,

        // ephemeral state
        prompt: String,
        focused_search_result: ScrollHandle<SearchResult>,

        // state
        sub_state: MainViewState,
        pending_plugin_view_data: Option<PluginViewData>,
    },
    ErrorView {
        error_view: ErrorViewData,
    },
    PluginView {
        client_context: Arc<StdRwLock<ClientContext>>,
        plugin_view_data: PluginViewData,
        sub_state: PluginViewState,
    },
}

#[derive(Clone)]
pub struct PluginViewData {
    pub top_level_view: bool,
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub entrypoint_id: EntrypointId,
    pub entrypoint_name: String,
    pub action_shortcuts: HashMap<String, PhysicalShortcut>,
}

pub enum ErrorViewData {
    PreferenceRequired {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    PluginError {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    BackendTimeout,
    UnknownError {
        display: String
    },
}

impl GlobalState {
    pub fn new(search_field_id: text_input::Id) -> GlobalState {
        GlobalState::MainView {
            search_field_id,
            prompt: "".to_string(),
            focused_search_result: ScrollHandle::new(),
            sub_state: MainViewState::new(),
            pending_plugin_view_data: None,
        }
    }

    pub fn new_error(error_view_data: ErrorViewData) -> GlobalState {
        GlobalState::ErrorView {
            error_view: error_view_data,
        }
    }

    pub fn new_plugin(plugin_view_data: PluginViewData, client_context: Arc<StdRwLock<ClientContext>>) -> GlobalState {
        GlobalState::PluginView {
            client_context,
            plugin_view_data,
            sub_state: PluginViewState::new(),
        }
    }

    pub fn initial(prev_global_state: &mut GlobalState) -> Command<AppMsg> {
        let search_field_id = text_input::Id::unique();

        *prev_global_state = GlobalState::new(search_field_id.clone());

        Command::batch([
            focus(search_field_id),
            Command::perform(async {}, |_| AppMsg::UpdateSearchResults),
        ])
    }

    pub fn error(prev_global_state: &mut GlobalState, error_view_data: ErrorViewData) -> Command<AppMsg> {
        *prev_global_state = GlobalState::ErrorView {
            error_view: error_view_data,
        };

        Command::none()
    }

    pub fn plugin(prev_global_state: &mut GlobalState, plugin_view_data: PluginViewData, client_context: Arc<StdRwLock<ClientContext>>) -> Command<AppMsg> {
        *prev_global_state = GlobalState::PluginView {
            client_context,
            plugin_view_data,
            sub_state: PluginViewState::new(),
        };

        Command::none()
    }
}

pub trait Focus<T> {
    fn enter(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn escape(&mut self) -> Command<AppMsg>;
    fn tab(&mut self) -> Command<AppMsg>;
    fn shift_tab(&mut self) -> Command<AppMsg>;
    fn arrow_up(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn arrow_down(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn arrow_left(&mut self, focus_list: &[T]) -> Command<AppMsg>;
    fn arrow_right(&mut self, focus_list: &[T]) -> Command<AppMsg>;
}

impl Focus<SearchResult> for GlobalState {
    fn enter(&mut self, focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                if let Some(search_item) = focused_search_result.get(focus_list) {
                    match sub_state {
                        MainViewState::None => {
                            let search_item = search_item.clone();
                            Command::perform(async {}, |_| AppMsg::RunSearchItemAction(search_item, None))
                        }
                        MainViewState::ActionPanel { .. } => {
                            sub_state.enter(&search_item.entrypoint_actions)
                        }
                    }
                } else {
                    Command::none()
                }
            }
            GlobalState::PluginView { sub_state, client_context, .. } => {
                let client_context = client_context.read().expect("lock is poisoned");

                let action_ids = client_context.get_action_ids();

                sub_state.enter(&action_ids)
            }
            GlobalState::ErrorView { .. } => Command::none()
        }
    }

    fn escape(&mut self) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { sub_state, .. } => {
                sub_state.escape()
            }
            GlobalState::PluginView {
                plugin_view_data: PluginViewData {
                    top_level_view,
                    plugin_id,
                    entrypoint_id,
                    ..
                },
                sub_state,
                ..
            } => {
                match sub_state {
                    PluginViewState::None => {
                        if *top_level_view {
                            let plugin_id = plugin_id.clone();

                            Command::batch([
                                Command::perform(async {}, |_| AppMsg::ClosePluginView(plugin_id)),
                                GlobalState::initial(self)
                            ])
                        } else {
                            let plugin_id = plugin_id.clone();
                            let entrypoint_id = entrypoint_id.clone();
                            Command::perform(async {}, |_| AppMsg::OpenPluginView(plugin_id, entrypoint_id))
                        }
                    }
                    PluginViewState::ActionPanel { .. } => {
                        sub_state.escape()
                    }
                }
            }
            GlobalState::ErrorView { .. } => {
                Command::perform(async {}, |_| AppMsg::HideWindow)
            }
        }
    }
    fn tab(&mut self) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::PluginView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
    fn shift_tab(&mut self) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::PluginView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
    fn arrow_up(&mut self, focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        focused_search_result.focus_previous()
                    }
                    MainViewState::ActionPanel { .. } => {
                        if let Some(search_item) = focused_search_result.get(focus_list) {
                            sub_state.arrow_up(&search_item.entrypoint_actions)
                        } else {
                            Command::none()
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Command::none(),
            GlobalState::PluginView { sub_state, client_context, .. } => {
                match sub_state {
                    PluginViewState::None => Command::none(),
                    PluginViewState::ActionPanel { .. } => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        let action_ids = client_context.get_action_ids();

                        sub_state.arrow_up(&action_ids)
                    }
                }
            },
        }
    }
    fn arrow_down(&mut self, focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if focus_list.len() != 0 {
                            focused_search_result.focus_next(focus_list.len())
                        } else {
                            Command::none()
                        }
                    }
                    MainViewState::ActionPanel { .. } => {
                        if let Some(search_item) = focused_search_result.get(focus_list) {
                            sub_state.arrow_down(&search_item.entrypoint_actions)
                        } else {
                            Command::none()
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Command::none(),
            GlobalState::PluginView { sub_state, client_context, .. } => {
                match sub_state {
                    PluginViewState::None => Command::none(),
                    PluginViewState::ActionPanel { .. } => {
                        let client_context = client_context.read().expect("lock is poisoned");

                        let action_ids = client_context.get_action_ids();

                        sub_state.arrow_down(&action_ids)
                    }
                }
            }
        }
    }
    fn arrow_left(&mut self, _focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::PluginView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
    fn arrow_right(&mut self, _focus_list: &[SearchResult]) -> Command<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Command::none(),
            GlobalState::PluginView { .. } => Command::none(),
            GlobalState::ErrorView { .. } => Command::none(),
        }
    }
}
