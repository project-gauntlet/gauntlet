use std::collections::HashMap;
use std::sync::Arc;

use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::ListSectionWidgetOrderedMembers;
use gauntlet_common::model::ListWidgetOrderedMembers;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::UiWidgetId;
use iced::Task;
use iced::widget::text_input;

use crate::ui::AppMsg;
use crate::ui::grid_navigation::grid_down_offset;
use crate::ui::grid_navigation::grid_up_offset;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::state::ComponentWidgetState;
use crate::ui::widget::state::RootState;
use crate::ui::widget::state::TextFieldState;

#[derive(Debug)]
pub struct ComponentWidgetsMut<'b> {
    pub root_widget: &'b mut Option<Arc<RootWidget>>,
    pub state: &'b mut HashMap<UiWidgetId, ComponentWidgetState>,
    pub plugin_id: PluginId,
}

impl<'b> ComponentWidgetsMut<'b> {
    pub fn new(
        root_widget: &'b mut Option<Arc<RootWidget>>,
        state: &'b mut HashMap<UiWidgetId, ComponentWidgetState>,
        plugin_id: &PluginId,
    ) -> ComponentWidgetsMut<'b> {
        Self {
            root_widget,
            state,
            plugin_id: plugin_id.clone(),
        }
    }

    #[allow(unused)]
    pub fn text_field_state_mut(&mut self, widget_id: UiWidgetId) -> &mut TextFieldState {
        Self::text_field_state_mut_on_state(&mut self.state, widget_id)
    }

    pub fn text_field_state_mut_on_state(
        state: &mut HashMap<UiWidgetId, ComponentWidgetState>,
        widget_id: UiWidgetId,
    ) -> &mut TextFieldState {
        let state = state.get_mut(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::TextField(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    pub fn root_state_mut(&mut self, widget_id: UiWidgetId) -> &mut RootState {
        Self::root_state_mut_on_field(&mut self.state, widget_id)
    }

    pub fn root_state_mut_on_field(
        state: &mut HashMap<UiWidgetId, ComponentWidgetState>,
        widget_id: UiWidgetId,
    ) -> &mut RootState {
        let state = state.get_mut(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Root(state) => state,
            _ => panic!("RootState expected, {:?} found", state),
        }
    }
}

impl<'b> ComponentWidgetsMut<'b> {
    pub fn toggle_action_panel(&mut self) {
        let Some(root_widget) = &self.root_widget else {
            return;
        };

        let Some(content) = &root_widget.content else {
            return;
        };

        let widget_id = match content {
            RootWidgetMembers::Detail(widget) => widget.__id__,
            RootWidgetMembers::Form(widget) => widget.__id__,
            RootWidgetMembers::Inline(widget) => widget.__id__,
            RootWidgetMembers::List(widget) => widget.__id__,
            RootWidgetMembers::Grid(widget) => widget.__id__,
        };

        let state = self.root_state_mut(widget_id);

        state.show_action_panel = !state.show_action_panel;
    }

    pub fn append_text(&mut self, text: &str) -> Task<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root_widget.content else {
            return Task::none();
        };

        let widget_id = match content {
            RootWidgetMembers::List(widget) => {
                match &widget.content.search_bar {
                    None => return Task::none(),
                    Some(widget) => widget.__id__,
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.content.search_bar {
                    None => return Task::none(),
                    Some(widget) => widget.__id__,
                }
            }
            _ => return Task::none(),
        };

        let TextFieldState {
            text_input_id,
            state_value,
        } = ComponentWidgetsMut::text_field_state_mut_on_state(&mut self.state, widget_id);

        if let Some(value) = text.chars().next().filter(|c| !c.is_control()) {
            *state_value = format!("{}{}", state_value, value);

            text_input::focus(text_input_id.clone())
        } else {
            Task::none()
        }
    }

    pub fn backspace_text(&mut self) -> Task<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root_widget.content else {
            return Task::none();
        };

        let widget_id = match content {
            RootWidgetMembers::List(widget) => {
                match &widget.content.search_bar {
                    None => return Task::none(),
                    Some(widget) => widget.__id__,
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.content.search_bar {
                    None => return Task::none(),
                    Some(widget) => widget.__id__,
                }
            }
            _ => return Task::none(),
        };

        let TextFieldState {
            text_input_id,
            state_value,
        } = ComponentWidgetsMut::text_field_state_mut_on_state(&mut self.state, widget_id);

        let mut chars = state_value.chars();
        chars.next_back();
        *state_value = chars.as_str().to_owned();

        text_input::focus(text_input_id.clone())
    }

    pub fn focus_up(&mut self) -> Task<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root_widget.content else {
            return Task::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Task::none(),
            RootWidgetMembers::Form(_) => Task::none(),
            RootWidgetMembers::Inline(_) => Task::none(),
            RootWidgetMembers::List(list_widget) => {
                let RootState { focused_item, .. } =
                    ComponentWidgetsMut::root_state_mut_on_field(&mut self.state, list_widget.__id__);

                let focus_task = focused_item.focus_previous().unwrap_or_else(|| Task::none());

                let item_focus_event =
                    ComponentWidgets::list_item_focus_event(self.plugin_id.clone(), focused_item, list_widget);

                Task::batch([item_focus_event, focus_task])
            }
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } =
                    ComponentWidgetsMut::root_state_mut_on_field(&mut self.state, grid_widget.__id__);

                let Some(current_index) = &focused_item.index else {
                    return Task::none();
                };

                let amount_per_section_total = ComponentWidgets::grid_section_sizes(grid_widget);

                let focus_task = match grid_up_offset(*current_index, amount_per_section_total) {
                    None => Task::none(),
                    Some(data) => {
                        match focused_item.focus_previous_in(data.offset) {
                            None => Task::none(),
                            Some(_) => focused_item.scroll_to(data.row_index),
                        }
                    }
                };

                let item_focus_event =
                    ComponentWidgets::grid_item_focus_event(self.plugin_id.clone(), focused_item, grid_widget);

                Task::batch([item_focus_event, focus_task])
            }
        }
    }

    pub fn focus_down(&mut self) -> Task<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root_widget.content else {
            return Task::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Task::none(),
            RootWidgetMembers::Form(_) => Task::none(),
            RootWidgetMembers::Inline(_) => Task::none(),
            RootWidgetMembers::List(widget) => {
                let RootState { focused_item, .. } =
                    ComponentWidgetsMut::root_state_mut_on_field(&mut self.state, widget.__id__);

                let total = widget
                    .content
                    .ordered_members
                    .iter()
                    .flat_map(|members| {
                        match members {
                            ListWidgetOrderedMembers::ListItem(widget) => vec![widget],
                            ListWidgetOrderedMembers::ListSection(widget) => {
                                widget
                                    .content
                                    .ordered_members
                                    .iter()
                                    .map(|members| {
                                        match members {
                                            ListSectionWidgetOrderedMembers::ListItem(widget) => widget,
                                        }
                                    })
                                    .collect()
                            }
                        }
                    })
                    .count();

                let focus_task = focused_item.focus_next(total).unwrap_or_else(|| Task::none());

                let item_focus_event =
                    ComponentWidgets::list_item_focus_event(self.plugin_id.clone(), focused_item, widget);

                Task::batch([item_focus_event, focus_task])
            }
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } =
                    ComponentWidgetsMut::root_state_mut_on_field(&mut self.state, grid_widget.__id__);

                let amount_per_section_total = ComponentWidgets::grid_section_sizes(grid_widget);

                let total = amount_per_section_total.iter().map(|data| data.amount_in_section).sum();

                let Some(current_index) = &focused_item.index else {
                    let unfocus = match &grid_widget.content.search_bar {
                        None => Task::none(),
                        Some(_) => {
                            // there doesn't seem to be an unfocus command but focusing non-existing input will unfocus all
                            text_input::focus(text_input::Id::unique())
                        }
                    };

                    let _ = focused_item.focus_next(total);

                    let item_focus_event =
                        ComponentWidgets::grid_item_focus_event(self.plugin_id.clone(), focused_item, grid_widget);

                    return Task::batch([unfocus, focused_item.scroll_to(0), item_focus_event]);
                };

                let focus_task = match grid_down_offset(*current_index, amount_per_section_total) {
                    None => Task::none(),
                    Some(data) => {
                        match focused_item.focus_next_in(total, data.offset) {
                            None => Task::none(),
                            Some(_) => focused_item.scroll_to(data.row_index),
                        }
                    }
                };

                let item_focus_event =
                    ComponentWidgets::grid_item_focus_event(self.plugin_id.clone(), focused_item, grid_widget);

                Task::batch([item_focus_event, focus_task])
            }
        }
    }

    pub fn focus_left(&mut self) -> Task<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root_widget.content else {
            return Task::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Task::none(),
            RootWidgetMembers::Form(_) => Task::none(),
            RootWidgetMembers::Inline(_) => Task::none(),
            RootWidgetMembers::List(_) => Task::none(),
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } =
                    ComponentWidgetsMut::root_state_mut_on_field(&mut self.state, grid_widget.__id__);

                let _ = focused_item.focus_previous();

                // focused_item.scroll_to(0)

                ComponentWidgets::grid_item_focus_event(self.plugin_id.clone(), focused_item, grid_widget)
            }
        }
    }

    pub fn focus_right(&mut self) -> Task<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root_widget.content else {
            return Task::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Task::none(),
            RootWidgetMembers::Form(_) => Task::none(),
            RootWidgetMembers::Inline(_) => Task::none(),
            RootWidgetMembers::List(_) => Task::none(),
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } =
                    ComponentWidgetsMut::root_state_mut_on_field(&mut self.state, grid_widget.__id__);

                let total = grid_widget
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
                    .count();

                let _ = focused_item.focus_next(total);

                // focused_item.scroll_to(0)

                ComponentWidgets::grid_item_focus_event(self.plugin_id.clone(), focused_item, grid_widget)
            }
        }
    }
}
