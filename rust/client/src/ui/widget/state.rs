use std::collections::HashMap;

use gauntlet_common::model::FormWidgetOrderedMembers;
use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::UiWidgetId;
use iced::widget::text_input;
use iced_aw::date_picker::Date;

use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::scroll_handle::ESTIMATED_MAIN_LIST_ITEM_HEIGHT;
use crate::ui::widget::grid::grid_width;

pub fn create_state(root_widget: &RootWidget) -> HashMap<UiWidgetId, ComponentWidgetState> {
    let mut result = HashMap::new();

    match &root_widget.content {
        None => {}
        Some(members) => {
            match members {
                RootWidgetMembers::Detail(widget) => {
                    result.insert(widget.__id__, ComponentWidgetState::root(0.0, 0));
                }
                RootWidgetMembers::Form(widget) => {
                    result.insert(widget.__id__, ComponentWidgetState::root(0.0, 0));

                    for members in &widget.content.ordered_members {
                        match members {
                            FormWidgetOrderedMembers::TextField(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                            }
                            FormWidgetOrderedMembers::PasswordField(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                            }
                            FormWidgetOrderedMembers::Checkbox(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::checkbox(&widget.value));
                            }
                            FormWidgetOrderedMembers::DatePicker(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::date_picker(&widget.value));
                            }
                            FormWidgetOrderedMembers::Select(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::select(&widget.value));
                            }
                            FormWidgetOrderedMembers::Separator(_) => {}
                        }
                    }
                }
                RootWidgetMembers::List(widget) => {
                    result.insert(
                        widget.__id__,
                        ComponentWidgetState::root(ESTIMATED_MAIN_LIST_ITEM_HEIGHT, 7),
                    );

                    if let Some(widget) = &widget.content.search_bar {
                        result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                    }
                }
                RootWidgetMembers::Grid(widget) => {
                    // cursed heuristic
                    let has_title = widget
                        .content
                        .ordered_members
                        .iter()
                        .flat_map(|members| {
                            match members {
                                GridWidgetOrderedMembers::GridItem(widget) => vec![widget],
                                GridWidgetOrderedMembers::GridSection(widget) => {
                                    widget
                                        .content
                                        .ordered_members
                                        .iter()
                                        .map(|members| {
                                            match members {
                                                GridSectionWidgetOrderedMembers::GridItem(widget) => widget,
                                            }
                                        })
                                        .collect()
                                }
                            }
                        })
                        .next()
                        .map(|widget| widget.title.is_some() || widget.subtitle.is_some())
                        .unwrap_or_default();

                    let (height, rows_per_view) = match grid_width(&widget.columns) {
                        ..4 => (150.0, 0),
                        4 => (150.0, 0),
                        5 => (130.0, 0),
                        6 => (110.0, 1),
                        7 => (90.0, 3),
                        8 => (if has_title { 50.0 } else { 50.0 }, if has_title { 3 } else { 4 }),
                        8.. => (50.0, 4),
                    };

                    result.insert(widget.__id__, ComponentWidgetState::root(height, rows_per_view));

                    if let Some(widget) = &widget.content.search_bar {
                        result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                    }
                }
                RootWidgetMembers::Inline(_) => {}
            }
        }
    }

    result
}

#[derive(Debug, Clone)]
pub enum ComponentWidgetState {
    TextField(TextFieldState),
    Checkbox(CheckboxState),
    DatePicker(DatePickerState),
    Select(SelectState),
    Root(RootState),
}

#[derive(Debug, Clone)]
pub struct TextFieldState {
    pub text_input_id: text_input::Id,
    pub state_value: String,
}

#[derive(Debug, Clone)]
pub struct CheckboxState {
    pub state_value: bool,
}

#[derive(Debug, Clone)]
pub struct DatePickerState {
    pub show_picker: bool,
    pub state_value: Date,
}

#[derive(Debug, Clone)]
pub struct SelectState {
    pub state_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RootState {
    pub show_action_panel: bool,
    pub focused_item: ScrollHandle,
}

impl ComponentWidgetState {
    fn root(item_height: f32, rows_per_view: usize) -> ComponentWidgetState {
        ComponentWidgetState::Root(RootState {
            show_action_panel: false,
            focused_item: ScrollHandle::new(false, item_height, rows_per_view),
        })
    }

    fn text_field(value: &Option<String>) -> ComponentWidgetState {
        ComponentWidgetState::TextField(TextFieldState {
            text_input_id: text_input::Id::unique(),
            state_value: value.to_owned().unwrap_or_default(),
        })
    }

    fn checkbox(value: &Option<bool>) -> ComponentWidgetState {
        ComponentWidgetState::Checkbox(CheckboxState {
            state_value: value.to_owned().unwrap_or(false),
        })
    }

    fn date_picker(value: &Option<String>) -> ComponentWidgetState {
        let value = value
            .to_owned()
            .map(|value| parse_date(&value))
            .flatten()
            .map(|(year, month, day)| Date::from_ymd(year, month, day))
            .unwrap_or(Date::today());

        ComponentWidgetState::DatePicker(DatePickerState {
            state_value: value,
            show_picker: false,
        })
    }

    fn select(value: &Option<String>) -> ComponentWidgetState {
        ComponentWidgetState::Select(SelectState {
            state_value: value.to_owned(),
        })
    }
}

fn parse_date(value: &str) -> Option<(i32, u32, u32)> {
    let ymd: Vec<_> = value.split("-").collect();

    match ymd[..] {
        [year, month, day] => {
            let year = year.parse::<i32>();
            let month = month.parse::<u32>();
            let day = day.parse::<u32>();

            match (year, month, day) {
                (Ok(year), Ok(month), Ok(day)) => Some((year, month, day)),
                _ => None,
            }
        }
        _ => None,
    }
}
