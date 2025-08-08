use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::SearchResult;
use gauntlet_common::model::SearchResultEntrypointAction;
use gauntlet_common::model::SearchResultEntrypointType;
use gauntlet_common::model::UiWidgetId;

use crate::ui::primary_shortcut;
use crate::ui::scroll_handle::ScrollContent;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::secondary_shortcut;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::action_panel::ActionPanelItem;
use crate::ui::widget::action_panel::action_item_container_id;

pub enum MainViewState {
    None,
    SearchResultActionPanel {
        search_result: SearchResult,
        entrypoint_actions: ScrollContent<SearchResultEntrypointAction>,
        scroll_handle: ScrollHandle,
    },
    InlineViewActionPanel {
        scroll_handle: ScrollHandle,
        actions: ScrollContent<UiWidgetId>,
    },
}

impl MainViewState {
    pub fn new() -> Self {
        MainViewState::None
    }

    pub fn initial(prev_state: &mut MainViewState) {
        *prev_state = Self::None
    }

    pub fn search_result_action_panel(prev_state: &mut MainViewState, focus_first: bool, search_result: SearchResult) {
        let first_action_item = if focus_first {
            Some(action_item_container_id(0))
        } else {
            None
        };

        let items = search_result
            .entrypoint_actions
            .iter()
            .enumerate()
            .map(|(index, action)| (action_item_container_id(index), action.clone()))
            .collect();

        *prev_state = Self::SearchResultActionPanel {
            search_result: search_result.clone(),
            entrypoint_actions: ScrollContent::new_with_ids(items),
            scroll_handle: ScrollHandle::new(first_action_item),
        }
    }

    pub fn inline_result_action_panel(
        prev_state: &mut MainViewState,
        focus_first: bool,
        actions: ScrollContent<UiWidgetId>,
    ) {
        let first_action_item = if focus_first {
            Some(action_item_container_id(0))
        } else {
            None
        };

        *prev_state = Self::InlineViewActionPanel {
            scroll_handle: ScrollHandle::new(first_action_item),
            actions,
        }
    }
}

pub fn search_result_action_panel(search_item: &SearchResult) -> Option<ActionPanel> {
    fn create_static(search_item: &SearchResult, label: &str) -> Option<ActionPanel> {
        let mut actions: Vec<_> = search_item
            .entrypoint_actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let physical_shortcut = if index == 0 {
                    Some(secondary_shortcut())
                } else {
                    action.shortcut.clone()
                };

                ActionPanelItem::Action {
                    label: action.label.clone(),
                    container_id: action_item_container_id(index + 1),
                    widget_id: index,
                    physical_shortcut,
                }
            })
            .collect();

        let primary_action_widget_id = 0;

        if actions.is_empty() {
            None
        } else {
            let label = label.to_string();

            let primary_action = ActionPanelItem::Action {
                label: label.clone(),
                container_id: action_item_container_id(0),
                widget_id: primary_action_widget_id,
                physical_shortcut: Some(primary_shortcut()),
            };

            actions.insert(0, primary_action);

            let action_panel = ActionPanel {
                title: Some(search_item.entrypoint_name.clone()),
                items: actions,
            };

            Some(action_panel)
        }
    }

    fn create_generated(search_item: &SearchResult) -> Option<ActionPanel> {
        let actions: Vec<_> = search_item
            .entrypoint_actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let physical_shortcut = match index {
                    0 => Some(primary_shortcut()),
                    1 => Some(secondary_shortcut()),
                    _ => action.shortcut.clone(),
                };

                ActionPanelItem::Action {
                    label: action.label.clone(),
                    container_id: action_item_container_id(index),
                    widget_id: index,
                    physical_shortcut,
                }
            })
            .collect();

        let action_panel = ActionPanel {
            title: Some(search_item.entrypoint_name.clone()),
            items: actions,
        };

        Some(action_panel)
    }

    match search_item.entrypoint_type {
        SearchResultEntrypointType::Command => create_static(search_item, "Run Command"),
        SearchResultEntrypointType::View => create_static(search_item, "Open View"),
        SearchResultEntrypointType::Generated => create_generated(search_item),
    }
}

pub fn search_result_bot_panel_right_info(search_item: &SearchResult) -> (String, usize, PhysicalShortcut) {
    fn create_static(search_item: &SearchResult, label: &str) -> (String, usize, PhysicalShortcut) {
        let primary_action_widget_id = 0;

        if search_item.entrypoint_actions.is_empty() {
            (label.to_string(), primary_action_widget_id, primary_shortcut())
        } else {
            let label = label.to_string();

            (label, primary_action_widget_id, primary_shortcut())
        }
    }

    fn create_generated(search_item: &SearchResult, label: &str) -> (String, usize, PhysicalShortcut) {
        let label = search_item
            .entrypoint_actions
            .first()
            .map(|action| action.label.clone())
            .unwrap_or_else(|| label.to_string()); // should never happen, because there is always at least one action

        let primary_action_widget_id = 0;

        (label, primary_action_widget_id, primary_shortcut())
    }

    match search_item.entrypoint_type {
        SearchResultEntrypointType::Command => create_static(search_item, "Run Command"),
        SearchResultEntrypointType::View => create_static(search_item, "Open View"),
        SearchResultEntrypointType::Generated => create_generated(search_item, "Run Command"),
    }
}
