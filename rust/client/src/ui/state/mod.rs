pub mod main_view;
mod plugin_view;

use std::collections::HashMap;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::SearchResult;
use iced::Task;
use iced::widget::text_input;
use iced::widget::text_input::focus;

use crate::ui::AppMsg;
use crate::ui::client_context::ClientContext;
use crate::ui::scroll_handle::ScrollContent;
use crate::ui::scroll_handle::ScrollHandle;
pub use crate::ui::state::main_view::MainViewState;
pub use crate::ui::state::plugin_view::PluginViewState;
use crate::ui::windows::WindowActionMsg;

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
    PendingPluginView {
        pending_plugin_view_data: PluginViewData,
    },
    ErrorView {
        error_view: ErrorViewData,
    },
    PluginView {
        // state
        plugin_view_data: PluginViewData,
        sub_state: PluginViewState,
        close_window_on_esc: bool,
    },
}

#[derive(Clone)]
pub struct PluginViewData {
    pub top_level_view: bool,
    pub plugin_id: PluginId,
    pub entrypoint_id: EntrypointId,
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
        #[allow(unused)]
        plugin_id: PluginId,
        #[allow(unused)]
        entrypoint_id: EntrypointId,
    },
    BackendTimeout,
    UnknownError {
        display: String,
    },
}

#[derive(Debug, Clone)]
pub enum LoadingBarState {
    Off,
    Pending,
    On,
}

impl GlobalState {
    pub fn new(search_field_id: text_input::Id) -> GlobalState {
        GlobalState::MainView {
            search_field_id,
            focused_search_result: ScrollHandle::new(None),
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

    pub fn new_plugin(plugin_view_data: PluginViewData, close_window_on_esc: bool) -> GlobalState {
        GlobalState::PluginView {
            plugin_view_data,
            sub_state: PluginViewState::new(),
            close_window_on_esc,
        }
    }

    pub fn new_pending_plugin(pending_plugin_view_data: PluginViewData) -> GlobalState {
        GlobalState::PendingPluginView {
            pending_plugin_view_data,
        }
    }

    pub fn initial(prev_global_state: &mut GlobalState) -> Task<AppMsg> {
        let search_field_id = text_input::Id::unique();

        *prev_global_state = GlobalState::new(search_field_id.clone());

        Task::batch([focus(search_field_id), Task::done(AppMsg::UpdateSearchResults)])
    }

    pub fn error(prev_global_state: &mut GlobalState, error_view_data: ErrorViewData) -> Task<AppMsg> {
        *prev_global_state = GlobalState::new_error(error_view_data);

        Task::none()
    }

    pub fn plugin(
        prev_global_state: &mut GlobalState,
        plugin_view_data: PluginViewData,
        close_window_on_esc: bool,
    ) -> Task<AppMsg> {
        *prev_global_state = GlobalState::new_plugin(plugin_view_data, close_window_on_esc);

        Task::none()
    }

    pub fn pending_plugin(
        prev_global_state: &mut GlobalState,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    ) -> Task<AppMsg> {
        let view_data = PluginViewData {
            top_level_view: true,
            plugin_id: plugin_id.clone(),
            entrypoint_id: entrypoint_id.clone(),
            action_shortcuts: HashMap::new(),
        };

        *prev_global_state = GlobalState::new_pending_plugin(view_data);

        Task::none()
    }

    pub fn pending_plugin_main_view(
        prev_global_state: &mut GlobalState,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    ) -> Task<AppMsg> {
        if let GlobalState::MainView {
            pending_plugin_view_data,
            ..
        } = prev_global_state
        {
            *pending_plugin_view_data = Some(PluginViewData {
                top_level_view: true,
                plugin_id: plugin_id.clone(),
                entrypoint_id: entrypoint_id.clone(),
                action_shortcuts: HashMap::new(),
            });
        }

        Task::none()
    }
}

pub trait Focus<T> {
    fn primary(&mut self, client_context: &ClientContext, focus_list: &ScrollContent<T>) -> Task<AppMsg>;
    fn secondary(&mut self, client_context: &ClientContext, focus_list: &ScrollContent<T>) -> Task<AppMsg>;
    fn back(&mut self, client_context: &ClientContext) -> Task<AppMsg>;
    fn next(&mut self, client_context: &ClientContext) -> Task<AppMsg>;
    fn previous(&mut self, client_context: &ClientContext) -> Task<AppMsg>;
    fn up(&mut self, client_context: &mut ClientContext, focus_list: &ScrollContent<T>) -> Task<AppMsg>;
    fn down(&mut self, client_context: &mut ClientContext, focus_list: &ScrollContent<T>) -> Task<AppMsg>;
    fn left(&mut self, client_context: &mut ClientContext, focus_list: &ScrollContent<T>) -> Task<AppMsg>;
    fn right(&mut self, client_context: &mut ClientContext, focus_list: &ScrollContent<T>) -> Task<AppMsg>;
}

impl Focus<SearchResult> for GlobalState {
    fn primary(&mut self, client_context: &ClientContext, focus_list: &ScrollContent<SearchResult>) -> Task<AppMsg> {
        match self {
            GlobalState::MainView {
                focused_search_result,
                sub_state,
                ..
            } => {
                match sub_state {
                    MainViewState::None => {
                        let Some(search_result) = focused_search_result.get(focus_list) else {
                            return Task::done(AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus);
                        };

                        Task::done(AppMsg::OnPrimaryActionMainViewNoPanel {
                            search_result: search_result.clone(),
                        })
                    }
                    MainViewState::SearchResultActionPanel {
                        search_result,
                        entrypoint_actions,
                        scroll_handle,
                    } => {
                        let Some(index) = scroll_handle.get_index(entrypoint_actions) else {
                            return Task::none();
                        };

                        Task::done(AppMsg::OnAnyActionMainViewSearchResultPanelKeyboardWithFocus {
                            search_result: search_result.clone(),
                            index,
                        })
                    }
                    MainViewState::InlineViewActionPanel { scroll_handle, actions } => {
                        let Some(index) = scroll_handle.get_index(actions) else {
                            return Task::none();
                        };

                        Task::done(AppMsg::OnAnyActionMainViewInlineViewPanelKeyboardWithFocus { index })
                    }
                }
            }
            GlobalState::PluginView {
                plugin_view_data,
                sub_state,
                ..
            } => {
                let Some(view) = client_context.get_view_container(&plugin_view_data.plugin_id) else {
                    return Task::none();
                };

                let action_ids = view.get_action_widgets_with_ids();
                let focused_item_id = view.get_focused_item_id();

                match sub_state {
                    PluginViewState::None => {
                        let Some((_, widget_id)) = action_ids.first() else {
                            return Task::none();
                        };

                        Task::done(AppMsg::OnAnyActionPluginViewNoPanelKeyboardWithFocus {
                            plugin_id: plugin_view_data.plugin_id.clone(),
                            widget_id: *widget_id,
                            id: focused_item_id,
                        })
                    }
                    PluginViewState::ActionPanel { scroll_handle } => {
                        let Some(&widget_id) = scroll_handle.get(&ScrollContent::new_with_ids(action_ids)) else {
                            return Task::none();
                        };

                        Task::done(AppMsg::OnAnyActionPluginViewAnyPanelKeyboardWithFocus {
                            plugin_id: plugin_view_data.plugin_id.clone(),
                            widget_id,
                            id: focused_item_id,
                        })
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }

    fn secondary(&mut self, client_context: &ClientContext, focus_list: &ScrollContent<SearchResult>) -> Task<AppMsg> {
        match self {
            GlobalState::MainView {
                focused_search_result,
                sub_state,
                ..
            } => {
                match sub_state {
                    MainViewState::None => {
                        let Some(search_result) = focused_search_result.get(focus_list) else {
                            return Task::done(AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus);
                        };

                        Task::done(AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithFocus {
                            search_result: search_result.clone(),
                        })
                    }
                    MainViewState::SearchResultActionPanel { .. } | MainViewState::InlineViewActionPanel { .. } => {
                        // secondary does nothing when action panel is opened
                        Task::none()
                    }
                }
            }
            GlobalState::PluginView {
                plugin_view_data,
                sub_state,
                ..
            } => {
                let Some(view) = client_context.get_view_container(&plugin_view_data.plugin_id) else {
                    return Task::none();
                };

                let action_ids = view.get_action_widgets();
                let focused_item_id = view.get_focused_item_id();

                match sub_state {
                    PluginViewState::None => {
                        let Some(widget_id) = action_ids.get(1) else {
                            return Task::none();
                        };

                        Task::done(AppMsg::OnAnyActionPluginViewNoPanelKeyboardWithFocus {
                            plugin_id: plugin_view_data.plugin_id.clone(),
                            widget_id: *widget_id,
                            id: focused_item_id,
                        })
                    }
                    PluginViewState::ActionPanel { .. } => {
                        // secondary does nothing when action panel is opened
                        Task::none()
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }

    fn back(&mut self, _client_context: &ClientContext) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { sub_state, .. } => {
                match sub_state {
                    MainViewState::None => Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow)),
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
                plugin_view_data:
                    PluginViewData {
                        top_level_view,
                        plugin_id,
                        entrypoint_id,
                        ..
                    },
                sub_state,
                close_window_on_esc,
            } => {
                match sub_state {
                    PluginViewState::None => {
                        if *top_level_view {
                            if *close_window_on_esc {
                                Task::batch([
                                    Task::done(AppMsg::RequestReactViewClose(plugin_id.clone())),
                                    Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow)),
                                ])
                            } else {
                                Task::batch([
                                    Task::done(AppMsg::RequestReactViewClose(plugin_id.clone())),
                                    GlobalState::initial(self),
                                ])
                            }
                        } else {
                            Task::done(AppMsg::RequestPluginViewPop(plugin_id.clone(), entrypoint_id.clone()))
                        }
                    }
                    PluginViewState::ActionPanel { .. } => Task::done(AppMsg::ToggleActionPanel { keyboard: true }),
                }
            }
            GlobalState::ErrorView { .. } => Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow)),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
    fn next(&mut self, _client_context: &ClientContext) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::PluginView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
    fn previous(&mut self, _client_context: &ClientContext) -> Task<AppMsg> {
        match self {
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::PluginView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
    fn up(&mut self, client_context: &mut ClientContext, focus_list: &ScrollContent<SearchResult>) -> Task<AppMsg> {
        match self {
            GlobalState::MainView {
                focused_search_result,
                sub_state,
                ..
            } => {
                match sub_state {
                    MainViewState::None => {
                        let (_, task) = focused_search_result.list_focus_up(focus_list.ids());

                        task.unwrap_or(Task::none())
                    }
                    MainViewState::SearchResultActionPanel {
                        entrypoint_actions,
                        scroll_handle,
                        ..
                    } => {
                        let (_, task) = scroll_handle.list_focus_up(entrypoint_actions.ids());

                        task.unwrap_or(Task::none())
                    }
                    MainViewState::InlineViewActionPanel { scroll_handle, actions } => {
                        let (_, task) = scroll_handle.list_focus_up(actions.ids());

                        task.unwrap_or(Task::none())
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PluginView {
                plugin_view_data,
                sub_state,
                ..
            } => {
                let Some(view) = client_context.get_mut_view_container(&plugin_view_data.plugin_id) else {
                    return Task::none();
                };

                match sub_state {
                    PluginViewState::None => view.focus_up(),
                    PluginViewState::ActionPanel { scroll_handle } => {
                        let action_ids = view.get_action_widgets_ids();

                        let (_, task) = scroll_handle.list_focus_up(action_ids);

                        task.unwrap_or(Task::none())
                    }
                }
            }
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
    fn down(&mut self, client_context: &mut ClientContext, focus_list: &ScrollContent<SearchResult>) -> Task<AppMsg> {
        match self {
            GlobalState::MainView {
                focused_search_result,
                sub_state,
                ..
            } => {
                match sub_state {
                    MainViewState::None => {
                        let (_, task) = focused_search_result.list_focus_down(focus_list.ids());

                        task.unwrap_or(Task::none())
                    }
                    MainViewState::SearchResultActionPanel {
                        entrypoint_actions,
                        scroll_handle,
                        ..
                    } => {
                        let (_, task) = scroll_handle.list_focus_down(entrypoint_actions.ids());

                        task.unwrap_or(Task::none())
                    }
                    MainViewState::InlineViewActionPanel { scroll_handle, actions } => {
                        let (_, task) = scroll_handle.list_focus_down(actions.ids());

                        task.unwrap_or(Task::none())
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PluginView {
                plugin_view_data,
                sub_state,
                ..
            } => {
                let Some(view) = client_context.get_mut_view_container(&plugin_view_data.plugin_id) else {
                    return Task::none();
                };

                match sub_state {
                    PluginViewState::None => view.focus_down(),
                    PluginViewState::ActionPanel { scroll_handle } => {
                        let action_ids = view.get_action_widgets_ids();

                        let (_, task) = scroll_handle.list_focus_down(action_ids);

                        task.unwrap_or(Task::none())
                    }
                }
            }
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
    fn left(&mut self, client_context: &mut ClientContext, _focus_list: &ScrollContent<SearchResult>) -> Task<AppMsg> {
        match self {
            GlobalState::PluginView {
                plugin_view_data,
                sub_state,
                ..
            } => {
                let Some(view) = client_context.get_mut_view_container(&plugin_view_data.plugin_id) else {
                    return Task::none();
                };

                match sub_state {
                    PluginViewState::None => view.focus_left(),
                    PluginViewState::ActionPanel { .. } => Task::none(),
                }
            }
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
    fn right(&mut self, client_context: &mut ClientContext, _focus_list: &ScrollContent<SearchResult>) -> Task<AppMsg> {
        match self {
            GlobalState::PluginView {
                plugin_view_data,
                sub_state,
                ..
            } => {
                let Some(view) = client_context.get_mut_view_container(&plugin_view_data.plugin_id) else {
                    return Task::none();
                };

                match sub_state {
                    PluginViewState::None => view.focus_right(),
                    PluginViewState::ActionPanel { .. } => Task::none(),
                }
            }
            GlobalState::MainView { .. } => Task::none(),
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
}
