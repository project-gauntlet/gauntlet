use gauntlet_common::model::PluginId;
use gauntlet_common::model::UiWidgetId;

use crate::model::UiViewEvent;
use crate::ui::widget::state::CheckboxState;
use crate::ui::widget::state::ComponentWidgetState;
use crate::ui::widget::state::DatePickerState;
use crate::ui::widget::state::SelectState;
use crate::ui::widget::state::TextFieldState;
use crate::ui::AppMsg;

include!(concat!(env!("OUT_DIR"), "/components.rs"));

#[derive(Clone, Debug)]
pub enum ComponentWidgetEvent {
    LinkClick {
        widget_id: UiWidgetId,
        href: String,
    },
    TagClick {
        widget_id: UiWidgetId,
    },
    ActionClick {
        widget_id: UiWidgetId,
        id: Option<String>,
    },
    RunAction {
        widget_id: UiWidgetId,
        id: Option<String>,
    },
    ToggleDatePicker {
        widget_id: UiWidgetId,
    },
    OnChangeTextField {
        widget_id: UiWidgetId,
        value: String,
    },
    OnChangePasswordField {
        widget_id: UiWidgetId,
        value: String,
    },
    OnChangeSearchBar {
        widget_id: UiWidgetId,
        value: String,
    },
    SubmitDatePicker {
        widget_id: UiWidgetId,
        value: String,
    },
    CancelDatePicker {
        widget_id: UiWidgetId,
    },
    ToggleCheckbox {
        widget_id: UiWidgetId,
        value: bool,
    },
    SelectPickList {
        widget_id: UiWidgetId,
        value: String,
    },
    ToggleActionPanel {
        widget_id: UiWidgetId,
    },
    FocusListItem {
        list_widget_id: UiWidgetId,
        item_id: Option<String>,
    },
    FocusGridItem {
        grid_widget_id: UiWidgetId,
        item_id: Option<String>,
    },
    PreviousView,
    RunPrimaryAction {
        widget_id: UiWidgetId,
        id: Option<String>,
    },
    Noop,
}

impl ComponentWidgetEvent {
    pub fn handle(self, _plugin_id: PluginId, state: Option<&mut ComponentWidgetState>) -> Option<UiViewEvent> {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id: _, href } => Some(UiViewEvent::Open { href }),
            ComponentWidgetEvent::TagClick { widget_id } => Some(create_metadata_tag_item_on_click_event(widget_id)),
            ComponentWidgetEvent::RunAction { widget_id, id } | ComponentWidgetEvent::ActionClick { widget_id, id } => {
                Some(create_action_on_action_event(widget_id, id))
            }
            ComponentWidgetEvent::ToggleDatePicker { widget_id } => {
                let Some(state) = state else {
                    return None;
                };

                let ComponentWidgetState::DatePicker(DatePickerState {
                    state_value: _,
                    show_picker,
                }) = state
                else {
                    panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                };

                *show_picker = !*show_picker;
                None
            }
            ComponentWidgetEvent::CancelDatePicker { widget_id } => {
                let Some(state) = state else {
                    return None;
                };

                let ComponentWidgetState::DatePicker(DatePickerState {
                    state_value: _,
                    show_picker,
                }) = state
                else {
                    panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                };

                *show_picker = false;
                None
            }
            ComponentWidgetEvent::SubmitDatePicker { widget_id, value } => {
                let Some(state) = state else {
                    return None;
                };

                {
                    let ComponentWidgetState::DatePicker(DatePickerState {
                        state_value: _,
                        show_picker,
                    }) = state
                    else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *show_picker = false;
                }

                Some(create_date_picker_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::ToggleCheckbox { widget_id, value } => {
                let Some(state) = state else {
                    return None;
                };

                {
                    let ComponentWidgetState::Checkbox(CheckboxState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = !*state_value;
                }

                Some(create_checkbox_on_change_event(widget_id, value))
            }
            ComponentWidgetEvent::SelectPickList { widget_id, value } => {
                let Some(state) = state else {
                    return None;
                };

                {
                    let ComponentWidgetState::Select(SelectState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = Some(value.clone());
                }

                Some(create_select_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeTextField { widget_id, value } => {
                let Some(state) = state else {
                    return None;
                };

                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value, .. }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_text_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangePasswordField { widget_id, value } => {
                let Some(state) = state else {
                    return None;
                };

                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value, .. }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_password_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeSearchBar { widget_id, value } => {
                let Some(state) = state else {
                    return None;
                };

                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value, .. }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_search_bar_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::ToggleActionPanel { .. } => {
                Some(UiViewEvent::AppEvent {
                    event: AppMsg::ToggleActionPanel { keyboard: false },
                })
            }
            ComponentWidgetEvent::FocusListItem {
                list_widget_id,
                item_id,
            } => Some(create_list_on_item_focus_change_event(list_widget_id, item_id)),
            ComponentWidgetEvent::FocusGridItem {
                grid_widget_id,
                item_id,
            } => Some(create_grid_on_item_focus_change_event(grid_widget_id, item_id)),
            ComponentWidgetEvent::Noop | ComponentWidgetEvent::PreviousView => {
                panic!("widget_id on these events is not supposed to be called")
            }
            ComponentWidgetEvent::RunPrimaryAction { widget_id, id } => {
                Some(UiViewEvent::AppEvent {
                    event: AppMsg::OnAnyActionPluginViewAnyPanel { widget_id, id },
                })
            }
        }
    }

    pub fn widget_id(&self) -> UiWidgetId {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ActionClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::RunAction { widget_id, .. } => widget_id,
            ComponentWidgetEvent::TagClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SubmitDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::CancelDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleCheckbox { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SelectPickList { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangeTextField { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangePasswordField { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangeSearchBar { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleActionPanel { widget_id } => widget_id,
            ComponentWidgetEvent::FocusListItem { list_widget_id, .. } => list_widget_id,
            ComponentWidgetEvent::FocusGridItem { grid_widget_id, .. } => grid_widget_id,
            ComponentWidgetEvent::RunPrimaryAction { widget_id, .. } => widget_id,
            ComponentWidgetEvent::Noop | ComponentWidgetEvent::PreviousView => {
                panic!("widget_id on these events is not supposed to be called")
            }
        }
        .to_owned()
    }
}
