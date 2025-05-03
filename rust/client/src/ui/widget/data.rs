use std::collections::HashMap;
use std::sync::Arc;

use gauntlet_common::model::ActionPanelSectionWidgetOrderedMembers;
use gauntlet_common::model::ActionPanelWidgetOrderedMembers;
use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidget;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::ListSectionWidgetOrderedMembers;
use gauntlet_common::model::ListWidget;
use gauntlet_common::model::ListWidgetOrderedMembers;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::UiRenderLocation;
use gauntlet_common::model::UiWidgetId;
use iced::widget::text_input;
use iced::Task;

use crate::ui::grid_navigation::GridSectionData;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::widget::action_panel::convert_action_panel;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::grid::grid_width;
use crate::ui::widget::state::CheckboxState;
use crate::ui::widget::state::ComponentWidgetState;
use crate::ui::widget::state::DatePickerState;
use crate::ui::widget::state::RootState;
use crate::ui::widget::state::SelectState;
use crate::ui::widget::state::TextFieldState;
use crate::ui::AppMsg;

#[derive(Debug)]
pub struct ComponentWidgets<'b> {
    pub root_widget: &'b Option<Arc<RootWidget>>,
    pub state: &'b HashMap<UiWidgetId, ComponentWidgetState>,
    pub plugin_id: PluginId,
    pub data: &'b HashMap<UiWidgetId, Vec<u8>>,
}

impl<'b> ComponentWidgets<'b> {
    pub fn new(
        root_widget: &'b Option<Arc<RootWidget>>,
        state: &'b HashMap<UiWidgetId, ComponentWidgetState>,
        plugin_id: PluginId,
        data: &'b HashMap<UiWidgetId, Vec<u8>>,
    ) -> ComponentWidgets<'b> {
        Self {
            root_widget,
            state,
            plugin_id,
            data,
        }
    }

    pub fn text_field_state(&self, widget_id: UiWidgetId) -> &TextFieldState {
        let state = self.state.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::TextField(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    pub fn checkbox_state(&self, widget_id: UiWidgetId) -> &CheckboxState {
        let state = self.state.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Checkbox(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    pub fn date_picker_state(&self, widget_id: UiWidgetId) -> &DatePickerState {
        let state = self.state.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::DatePicker(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    pub fn select_state(&self, widget_id: UiWidgetId) -> &SelectState {
        let state = self.state.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Select(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }

    pub fn root_state(&self, widget_id: UiWidgetId) -> &RootState {
        let state = self.state.get(&widget_id).expect(&format!(
            "requested state should always be present for id: {}",
            widget_id
        ));

        match state {
            ComponentWidgetState::Root(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state),
        }
    }
}

impl<'b> ComponentWidgets<'b> {
    pub fn get_action_ids(&self) -> Vec<UiWidgetId> {
        let Some(root_widget) = &self.root_widget else {
            return vec![];
        };

        let Some(content) = &root_widget.content else {
            return vec![];
        };

        let actions = match content {
            RootWidgetMembers::Detail(widget) => &widget.content.actions,
            RootWidgetMembers::Form(widget) => &widget.content.actions,
            RootWidgetMembers::Inline(widget) => &widget.content.actions,
            RootWidgetMembers::List(widget) => &widget.content.actions,
            RootWidgetMembers::Grid(widget) => &widget.content.actions,
        };

        let mut result = vec![];
        match actions {
            None => {}
            Some(widget) => {
                for members in &widget.content.ordered_members {
                    match members {
                        ActionPanelWidgetOrderedMembers::Action(widget) => result.push(widget.__id__),
                        ActionPanelWidgetOrderedMembers::ActionPanelSection(widget) => {
                            for members in &widget.content.ordered_members {
                                match members {
                                    ActionPanelSectionWidgetOrderedMembers::Action(widget) => {
                                        result.push(widget.__id__)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    pub fn get_focused_item_id(&self) -> Option<String> {
        let Some(root_widget) = &self.root_widget else {
            return None;
        };

        let Some(content) = &root_widget.content else {
            return None;
        };

        match content {
            RootWidgetMembers::Detail(_) => None,
            RootWidgetMembers::Form(_) => None,
            RootWidgetMembers::Inline(_) => None,
            RootWidgetMembers::List(widget) => {
                let RootState { focused_item, .. } = self.root_state(widget.__id__);

                ComponentWidgets::list_focused_item_id(focused_item, widget)
            }
            RootWidgetMembers::Grid(widget) => {
                let RootState { focused_item, .. } = self.root_state(widget.__id__);

                ComponentWidgets::grid_focused_item_id(focused_item, widget)
            }
        }
    }

    pub fn focus_search_bar(&self, widget_id: UiWidgetId) -> Task<AppMsg> {
        let TextFieldState { text_input_id, .. } = self.text_field_state(widget_id);

        text_input::focus(text_input_id.clone())
    }

    pub fn grid_section_sizes(grid_widget: &GridWidget) -> Vec<GridSectionData> {
        let mut amount_per_section: Vec<GridSectionData> = vec![];
        let mut pending_section_size = 0;

        let mut cumulative_item_index = 0;
        let mut cumulative_row_index = 0;

        let mut cumulative_item_index_at_start = cumulative_item_index;
        let mut cumulative_row_index_at_start = cumulative_row_index;

        for members in &grid_widget.content.ordered_members {
            match &members {
                GridWidgetOrderedMembers::GridItem(_) => {
                    pending_section_size = pending_section_size + 1;
                }
                GridWidgetOrderedMembers::GridSection(widget) => {
                    if pending_section_size > 0 {
                        let width = grid_width(&grid_widget.columns);
                        amount_per_section.push(GridSectionData {
                            start_index: cumulative_item_index_at_start,
                            start_row_index: cumulative_row_index_at_start,
                            amount_in_section: pending_section_size,
                            width,
                        });

                        cumulative_item_index = cumulative_item_index + pending_section_size;
                        cumulative_row_index =
                            cumulative_row_index_at_start + (usize::div_ceil(pending_section_size, width));

                        cumulative_item_index_at_start = cumulative_item_index;
                        cumulative_row_index_at_start = cumulative_row_index;

                        pending_section_size = 0;
                    }

                    let section_amount = widget
                        .content
                        .ordered_members
                        .iter()
                        .filter(|members| matches!(members, GridSectionWidgetOrderedMembers::GridItem(_)))
                        .count();

                    let width = grid_width(&widget.columns);
                    amount_per_section.push(GridSectionData {
                        start_index: cumulative_item_index_at_start,
                        start_row_index: cumulative_row_index_at_start,
                        amount_in_section: section_amount,
                        width,
                    });

                    cumulative_item_index = cumulative_item_index + section_amount;
                    cumulative_row_index = cumulative_row_index_at_start + (usize::div_ceil(section_amount, width));

                    cumulative_item_index_at_start = cumulative_item_index;
                    cumulative_row_index_at_start = cumulative_row_index;
                }
            }
        }

        if pending_section_size > 0 {
            amount_per_section.push(GridSectionData {
                start_index: cumulative_item_index_at_start,
                start_row_index: cumulative_row_index_at_start,
                amount_in_section: pending_section_size,
                width: grid_width(&grid_widget.columns),
            });
        }

        amount_per_section
    }
}

impl<'b> ComponentWidgets<'b> {
    pub fn first_open(&self) -> AppMsg {
        let Some(root_widget) = &self.root_widget else {
            return AppMsg::Noop;
        };

        let Some(content) = &root_widget.content else {
            return AppMsg::Noop;
        };

        let widget_id = match content {
            RootWidgetMembers::List(widget) => {
                match &widget.content.search_bar {
                    None => return AppMsg::Noop,
                    Some(widget) => widget.__id__,
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.content.search_bar {
                    None => return AppMsg::Noop,
                    Some(widget) => widget.__id__,
                }
            }
            _ => return AppMsg::Noop,
        };

        AppMsg::FocusPluginViewSearchBar { widget_id }
    }

    pub fn list_focused_item_id(focused_item: &ScrollHandle, widget: &ListWidget) -> Option<String> {
        let mut items = vec![];

        for members in &widget.content.ordered_members {
            match &members {
                ListWidgetOrderedMembers::ListItem(item) => {
                    items.push(&item.id);
                }
                ListWidgetOrderedMembers::ListSection(section) => {
                    for members in &section.content.ordered_members {
                        match &members {
                            ListSectionWidgetOrderedMembers::ListItem(item) => {
                                items.push(&item.id);
                            }
                        }
                    }
                }
            }
        }

        match focused_item.get(&items) {
            None => None,
            Some(item_id) => Some(item_id.to_string()),
        }
    }

    pub fn list_item_focus_event(
        plugin_id: PluginId,
        focused_item: &ScrollHandle,
        widget: &ListWidget,
    ) -> Task<AppMsg> {
        let widget_event = match ComponentWidgets::list_focused_item_id(focused_item, widget) {
            None => {
                ComponentWidgetEvent::FocusListItem {
                    list_widget_id: widget.__id__,
                    item_id: None,
                }
            }
            Some(item_id) => {
                ComponentWidgetEvent::FocusListItem {
                    list_widget_id: widget.__id__,
                    item_id: Some(item_id),
                }
            }
        };

        Task::done(AppMsg::WidgetEvent {
            plugin_id,
            render_location: UiRenderLocation::View,
            widget_event,
        })
    }

    pub fn grid_focused_item_id(focused_item: &ScrollHandle, widget: &GridWidget) -> Option<String> {
        let mut items = vec![];

        for members in &widget.content.ordered_members {
            match &members {
                GridWidgetOrderedMembers::GridItem(item) => {
                    items.push(&item.id);
                }
                GridWidgetOrderedMembers::GridSection(section) => {
                    for members in &section.content.ordered_members {
                        match &members {
                            GridSectionWidgetOrderedMembers::GridItem(item) => {
                                items.push(&item.id);
                            }
                        }
                    }
                }
            }
        }

        match focused_item.get(&items) {
            None => None,
            Some(item_id) => Some(item_id.to_string()),
        }
    }

    pub fn grid_item_focus_event(
        plugin_id: PluginId,
        focused_item: &ScrollHandle,
        widget: &GridWidget,
    ) -> Task<AppMsg> {
        let widget_event = match ComponentWidgets::grid_focused_item_id(focused_item, widget) {
            None => {
                ComponentWidgetEvent::FocusGridItem {
                    grid_widget_id: widget.__id__,
                    item_id: None,
                }
            }
            Some(item_id) => {
                ComponentWidgetEvent::FocusGridItem {
                    grid_widget_id: widget.__id__,
                    item_id: Some(item_id),
                }
            }
        };

        Task::done(AppMsg::WidgetEvent {
            plugin_id,
            render_location: UiRenderLocation::View,
            widget_event,
        })
    }

    pub fn get_action_panel(&self, action_shortcuts: &HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
        let Some(root_widget) = &self.root_widget else {
            return None;
        };

        let Some(content) = &root_widget.content else {
            return None;
        };

        match content {
            RootWidgetMembers::Detail(widget) => convert_action_panel(&widget.content.actions, action_shortcuts),
            RootWidgetMembers::Form(widget) => convert_action_panel(&widget.content.actions, action_shortcuts),
            RootWidgetMembers::Inline(widget) => convert_action_panel(&widget.content.actions, action_shortcuts),
            RootWidgetMembers::List(widget) => convert_action_panel(&widget.content.actions, action_shortcuts),
            RootWidgetMembers::Grid(widget) => convert_action_panel(&widget.content.actions, action_shortcuts),
        }
    }
}
