mod main_view;
mod plugin_view;

use crate::ui::client_context::ClientContext;
use crate::ui::scroll_handle::{ScrollHandle, ESTIMATED_MAIN_LIST_ITEM_HEIGHT};
pub use crate::ui::state::main_view::MainViewState;
pub use crate::ui::state::plugin_view::PluginViewState;
use crate::ui::AppMsg;
use gauntlet_common::model::{EntrypointId, PhysicalShortcut, PluginId, SearchResult};
use iced::widget::text_input;
use iced::widget::text_input::focus;
use iced::Task;
use std::collections::HashMap;

pub enum GlobalState {
    MainView {
        // logic
        search_field_id: text_input::Id,

        // ephemeral state
        focused_search_result: ScrollHandle,

        // state
        sub_state: MainViewState,
        pending_plugin_view_data: Option<PluginViewData>,
        pending_plugin_view_loading_bar: LoadingBarState,
    },
    ErrorView {
        error_view: ErrorViewData,
    },
    PluginView {
        // state
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

#[derive(Debug, Clone)]
pub enum LoadingBarState {
    Off,
    Pending,
    On
}


impl GlobalState {
    pub fn new(search_field_id: text_input::Id) -> GlobalState {
        GlobalState::MainView {
            search_field_id,
            focused_search_result: ScrollHandle::new(true, ESTIMATED_MAIN_LIST_ITEM_HEIGHT, 7),
            sub_state: MainViewState::new(),
            pending_plugin_view_data: None,
            pending_plugin_view_loading_bar: LoadingBarState::Off,
        }
    }

    pub fn new_error(error_view_data: ErrorViewData) -> GlobalState {
        GlobalState::ErrorView {
            error_view: error_view_data,
        }
    }

    pub fn new_plugin(plugin_view_data: PluginViewData) -> GlobalState {
        GlobalState::PluginView {
            plugin_view_data,
            sub_state: PluginViewState::new(),
        }
    }

    pub fn initial(prev_global_state: &mut GlobalState) -> Task<AppMsg> {
        let search_field_id = text_input::Id::unique();

        *prev_global_state = GlobalState::new(search_field_id.clone());

        Task::batch([
            focus(search_field_id),
            Task::done(AppMsg::UpdateSearchResults),
        ])
    }

    pub fn error(prev_global_state: &mut GlobalState, error_view_data: ErrorViewData) -> Task<AppMsg> {
        *prev_global_state = GlobalState::new_error(error_view_data);

        Task::none()
    }

    pub fn plugin(prev_global_state: &mut GlobalState, plugin_view_data: PluginViewData) -> Task<AppMsg> {
        *prev_global_state = GlobalState::new_plugin(plugin_view_data);

        Task::none()
    }
}

pub trait Focus<T> {
    fn primary(&mut self, client_context: &ClientContext, focus_list: &[T]) -> Task<AppMsg>;
    fn secondary(&mut self, client_context: &ClientContext, focus_list: &[T]) -> Task<AppMsg>;
    fn back(&mut self, client_context: &ClientContext) -> Task<AppMsg>;
    fn next(&mut self, client_context: &ClientContext) -> Task<AppMsg>;
    fn previous(&mut self, client_context: &ClientContext) -> Task<AppMsg>;
    fn up(&mut self, client_context: &ClientContext, focus_list: &[T]) -> Task<AppMsg>;
    fn down(&mut self, client_context: &ClientContext, focus_list: &[T]) -> Task<AppMsg>;
    fn left(&mut self, client_context: &ClientContext, focus_list: &[T]) -> Task<AppMsg>;
    fn right(&mut self, client_context: &ClientContext, focus_list: &[T]) -> Task<AppMsg>;
}

impl Focus<SearchResult> for GlobalState {
    fn primary(&mut self, client_context: &ClientContext, focus_list: &[SearchResult]) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if let Some(search_result) = focused_search_result.get(focus_list) {
                            let search_result = search_result.clone();
                            Task::done(AppMsg::OnPrimaryActionMainViewNoPanel { search_result })
                        } else {
                            Task::done(AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus)
                        }
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item, .. } => {
                        match focused_action_item.index {
                            None => Task::none(),
                            Some(widget_id) => {
                                if let Some(search_result) = focused_search_result.get(&focus_list) {
                                    let search_result = search_result.clone();
                                    Task::done(AppMsg::OnAnyActionMainViewSearchResultPanelKeyboardWithFocus { search_result, widget_id })
                                } else {
                                    Task::none()
                                }
                            }
                        }
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item } => {
                        match focused_action_item.index {
                            None => Task::none(),
                            Some(widget_id) => {
                                Task::done(AppMsg::OnAnyActionMainViewInlineViewPanelKeyboardWithFocus { widget_id })
                            }
                        }
                    }
                }
            }
            GlobalState::PluginView { sub_state, .. } => {
                let action_ids = client_context.get_action_ids();
                let focused_item_id = client_context.get_focused_item_id();

                match sub_state {
                    PluginViewState::None => {
                        if let Some(widget_id) = action_ids.get(0) {
                            let widget_id = *widget_id;
                            Task::done(AppMsg::OnAnyActionPluginViewNoPanelKeyboardWithFocus { widget_id, id: focused_item_id })
                        } else {
                            Task::none()
                        }
                    },
                    PluginViewState::ActionPanel { focused_action_item, .. } => {
                        if let Some(widget_id) = focused_action_item.get(&action_ids) {
                            let widget_id = *widget_id;
                            Task::done(AppMsg::OnAnyActionPluginViewAnyPanelKeyboardWithFocus { widget_id, id: focused_item_id })
                        } else {
                            Task::none()
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none()
        }
    }

    fn secondary(&mut self, client_context: &ClientContext, focus_list: &[SearchResult]) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if let Some(search_result) = focused_search_result.get(focus_list) {
                            let search_result = search_result.clone();
                            Task::done(AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithFocus { search_result })
                        } else {
                            Task::done(AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus)
                        }
                    }
                    MainViewState::SearchResultActionPanel { .. } | MainViewState::InlineViewActionPanel { .. } => {
                        // secondary does nothing when action panel is opened
                        Task::none()
                    }
                }
            }
            GlobalState::PluginView { sub_state, .. } => {
                let action_ids = client_context.get_action_ids();
                let focused_item_id = client_context.get_focused_item_id();

                match sub_state {
                    PluginViewState::None => {
                        if let Some(widget_id) = action_ids.get(1) {
                            let widget_id = *widget_id;
                            Task::done(AppMsg::OnAnyActionPluginViewNoPanelKeyboardWithFocus { widget_id, id: focused_item_id })
                        } else {
                            Task::none()
                        }
                    },
                    PluginViewState::ActionPanel { .. } => {
                        // secondary does nothing when action panel is opened
                        Task::none()
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none()
        }
    }

    fn back(&mut self, _client_context: &ClientContext) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        Task::done(AppMsg::HideWindow)
                    }
                    MainViewState::SearchResultActionPanel { .. } => {
                        MainViewState::initial(sub_state);
                        Task::none()
                    }
                    MainViewState::InlineViewActionPanel { .. } => {
                        MainViewState::initial(sub_state);
                        Task::none()
                    }
                }
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

                            Task::batch([
                                Task::done(AppMsg::ClosePluginView(plugin_id)),
                                GlobalState::initial(self)
                            ])
                        } else {
                            let plugin_id = plugin_id.clone();
                            let entrypoint_id = entrypoint_id.clone();
                            Task::done(AppMsg::OpenPluginView(plugin_id, entrypoint_id))
                        }
                    }
                    PluginViewState::ActionPanel { .. } => {
                        Task::done(AppMsg::ToggleActionPanel { keyboard: true })
                    }
                }
            }
            GlobalState::ErrorView { .. } => {
                Task::done(AppMsg::HideWindow)
            }
        }
    }
    fn next(&mut self, _client_context: &ClientContext) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::PluginView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
        }
    }
    fn previous(&mut self, _client_context: &ClientContext) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::PluginView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
        }
    }
    fn up(&mut self, client_context: &ClientContext, _focus_list: &[SearchResult]) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        focused_search_result.focus_previous()
                            .unwrap_or_else(|| Task::none())
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item } => {
                        focused_action_item.focus_previous()
                            .unwrap_or_else(|| Task::none())
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item } => {
                        focused_action_item.focus_previous()
                            .unwrap_or_else(|| Task::none())
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PluginView { sub_state, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        client_context.focus_up()
                    },
                    PluginViewState::ActionPanel { focused_action_item } => {
                        focused_action_item.focus_previous()
                            .unwrap_or_else(|| Task::none())
                    }
                }
            },
        }
    }
    fn down(&mut self, client_context: &ClientContext, focus_list: &[SearchResult]) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { focused_search_result, sub_state, .. } => {
                match sub_state {
                    MainViewState::None => {
                        if focus_list.len() != 0 {
                            focused_search_result.focus_next(focus_list.len())
                                .unwrap_or_else(|| Task::none())
                        } else {
                            Task::none()
                        }
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item } => {
                        if let Some(search_item) = focused_search_result.get(focus_list) {
                            if search_item.entrypoint_actions.len() != 0 {
                                focused_action_item.focus_next(search_item.entrypoint_actions.len())
                                    .unwrap_or_else(|| Task::none())
                            } else {
                                Task::none()
                            }
                        } else {
                            Task::none()
                        }
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item } => {
                        match client_context.get_first_inline_view_action_panel() {
                            Some(action_panel) => {
                                if action_panel.action_count() != 0 {
                                    focused_action_item.focus_next(action_panel.action_count())
                                        .unwrap_or_else(|| Task::none())
                                } else {
                                    Task::none()
                                }
                            }
                            None => Task::none()
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PluginView { sub_state, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        client_context.focus_down()
                    },
                    PluginViewState::ActionPanel { focused_action_item } => {
                        let action_ids = client_context.get_action_ids();

                        if action_ids.len() != 0 {
                            focused_action_item.focus_next(action_ids.len())
                                .unwrap_or_else(|| Task::none())
                        } else {
                            Task::none()
                        }
                    }
                }
            }
        }
    }
    fn left(&mut self, client_context: &ClientContext, _focus_list: &[SearchResult]) -> Task<AppMsg> {
        match self {
            GlobalState::PluginView { sub_state, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        client_context.focus_left()
                    }
                    PluginViewState::ActionPanel { .. } => Task::none()
                }
            },
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
        }
    }
    fn right(&mut self, client_context: &ClientContext, _focus_list: &[SearchResult]) -> Task<AppMsg> {
        match self {
            GlobalState::PluginView { sub_state, .. } => {
                match sub_state {
                    PluginViewState::None => {
                        client_context.focus_right()
                    }
                    PluginViewState::ActionPanel { .. } => Task::none()
                }
            },
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
        }
    }
}
