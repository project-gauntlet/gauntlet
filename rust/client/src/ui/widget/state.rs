use std::collections::HashMap;

use gauntlet_common::model::FormWidgetOrderedMembers;
use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::ListSectionWidgetOrderedMembers;
use gauntlet_common::model::ListWidgetOrderedMembers;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::UiWidgetId;
use iced::widget::container;
use iced::widget::text_input;

use crate::ui::scroll_handle::ScrollHandle;

pub fn create_state(root_widget: &RootWidget) -> ComponentWidgetStateContainer {
    let mut result = HashMap::new();

    match &root_widget.content {
        None => {}
        Some(members) => {
            match members {
                RootWidgetMembers::Detail(widget) => {
                    result.insert(widget.__id__, ComponentWidgetState::root());
                }
                RootWidgetMembers::Form(widget) => {
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
                            FormWidgetOrderedMembers::Select(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::select(&widget.value));
                            }
                            FormWidgetOrderedMembers::Separator(_) => {}
                        }
                    }

                    result.insert(widget.__id__, ComponentWidgetState::root());
                }
                RootWidgetMembers::List(widget) => {
                    if let Some(widget) = &widget.content.search_bar {
                        result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                    }

                    for members in &widget.content.ordered_members {
                        match members {
                            ListWidgetOrderedMembers::ListItem(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::scrollable_item());
                            }
                            ListWidgetOrderedMembers::ListSection(widget) => {
                                for members in &widget.content.ordered_members {
                                    match members {
                                        ListSectionWidgetOrderedMembers::ListItem(widget) => {
                                            result.insert(widget.__id__, ComponentWidgetState::scrollable_item());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    result.insert(widget.__id__, ComponentWidgetState::scrollable_root());
                }
                RootWidgetMembers::Grid(widget) => {
                    if let Some(widget) = &widget.content.search_bar {
                        result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                    }

                    for members in &widget.content.ordered_members {
                        match members {
                            GridWidgetOrderedMembers::GridItem(widget) => {
                                result.insert(widget.__id__, ComponentWidgetState::scrollable_item());
                            }
                            GridWidgetOrderedMembers::GridSection(widget) => {
                                for members in &widget.content.ordered_members {
                                    match members {
                                        GridSectionWidgetOrderedMembers::GridItem(widget) => {
                                            result.insert(widget.__id__, ComponentWidgetState::scrollable_item());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    result.insert(widget.__id__, ComponentWidgetState::scrollable_root());
                }
                RootWidgetMembers::Inline(_) => {}
            }
        }
    }

    ComponentWidgetStateContainer(result)
}

#[derive(Debug, Clone)]
pub enum ComponentWidgetState {
    ScrollableItem(ScrollableItemState),
    TextField(TextFieldState),
    Checkbox(CheckboxState),
    Select(SelectState),
    Root(RootState),
    ScrollableRoot(ScrollableRootState),
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
pub struct SelectState {
    pub state_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RootState {
    pub show_action_panel: bool,
}

#[derive(Debug, Clone)]
pub struct ScrollableRootState {
    pub show_action_panel: bool,
    pub scroll_handle: ScrollHandle,
}

#[derive(Debug, Clone)]
pub struct ScrollableItemState {
    pub id: container::Id,
}

impl ComponentWidgetState {
    fn root() -> ComponentWidgetState {
        ComponentWidgetState::Root(RootState {
            show_action_panel: false,
        })
    }

    fn scrollable_root() -> ComponentWidgetState {
        ComponentWidgetState::ScrollableRoot(ScrollableRootState {
            show_action_panel: false,
            scroll_handle: ScrollHandle::new(None),
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

    fn select(value: &Option<String>) -> ComponentWidgetState {
        ComponentWidgetState::Select(SelectState {
            state_value: value.to_owned(),
        })
    }

    fn scrollable_item() -> ComponentWidgetState {
        ComponentWidgetState::ScrollableItem(ScrollableItemState {
            id: container::Id::unique(),
        })
    }
}

#[derive(Debug)]
pub struct ComponentWidgetStateContainer(pub(crate) HashMap<UiWidgetId, ComponentWidgetState>);

impl ComponentWidgetStateContainer {
    pub fn text_field_state(&self, widget_id: UiWidgetId) -> &TextFieldState {
        let state = self.0.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::TextField(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    pub fn checkbox_state(&self, widget_id: UiWidgetId) -> &CheckboxState {
        let state = self.0.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Checkbox(state) => state,
            _ => panic!("CheckboxState expected, {:?} found", state),
        }
    }

    pub fn select_state(&self, widget_id: UiWidgetId) -> &SelectState {
        let state = self.0.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Select(state) => state,
            _ => panic!("SelectState expected, {:?} found", state),
        }
    }

    pub fn root_state(&self, widget_id: UiWidgetId) -> &RootState {
        let state = self.0.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Root(state) => state,
            _ => panic!("RootState expected, {:?} found", state),
        }
    }

    pub fn scrollable_item_state(&self, widget_id: UiWidgetId) -> &ScrollableItemState {
        let state = self.0.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::ScrollableItem(state) => state,
            _ => panic!("ScrollableItem expected, {:?} found", state),
        }
    }

    pub fn scrollable_root_state(&self, widget_id: UiWidgetId) -> &ScrollableRootState {
        let state = self.0.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::ScrollableRoot(state) => state,
            _ => panic!("ScrollableRoot expected, {:?} found", state),
        }
    }
}

impl ComponentWidgetStateContainer {
    pub fn text_field_state_mut(&mut self, widget_id: UiWidgetId) -> &mut TextFieldState {
        let state = self.0.get_mut(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::TextField(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    #[allow(unused)]
    pub fn scrollable_item_state_mut(&mut self, widget_id: UiWidgetId) -> &mut ScrollableItemState {
        let state = self.0.get_mut(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::ScrollableItem(state) => state,
            _ => panic!("ScrollableItem expected, {:?} found", state),
        }
    }

    pub fn scrollable_root_state_mut(&mut self, widget_id: UiWidgetId) -> &mut ScrollableRootState {
        let state = self.0.get_mut(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::ScrollableRoot(state) => state,
            _ => panic!("ScrollableRoot expected, {:?} found", state),
        }
    }

    pub fn root_state_mut(&mut self, widget_id: UiWidgetId) -> &mut RootState {
        let state = self.0.get_mut(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Root(state) => state,
            _ => panic!("RootState expected, {:?} found", state),
        }
    }
}
