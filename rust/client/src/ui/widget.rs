use crate::model::UiViewEvent;
use crate::ui::custom_widgets::loading_bar::LoadingBar;
use crate::ui::grid_navigation::{grid_down_offset, grid_up_offset, GridSectionData};
use crate::ui::scroll_handle::{ScrollHandle, ESTIMATED_MAIN_LIST_ITEM_HEIGHT};
use crate::ui::state::PluginViewState;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::date_picker::DatePickerStyle;
use crate::ui::theme::grid::GridStyle;
use crate::ui::theme::pick_list::PickListStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::rule::RuleStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::text_input::TextInputStyle;
use crate::ui::theme::tooltip::TooltipStyle;
use crate::ui::theme::{Element, ThemableWidget};
use crate::ui::AppMsg;
use gauntlet_common::model::{ActionPanelSectionWidget, ActionPanelSectionWidgetOrderedMembers, ActionPanelWidget, ActionPanelWidgetOrderedMembers, ActionWidget, CheckboxWidget, CodeBlockWidget, ContentWidget, ContentWidgetOrderedMembers, DatePickerWidget, DetailWidget, EmptyViewWidget, FormWidget, FormWidgetOrderedMembers, GridItemWidget, GridSectionWidget, GridSectionWidgetOrderedMembers, GridWidget, GridWidgetOrderedMembers, H1Widget, H2Widget, H3Widget, H4Widget, H5Widget, H6Widget, HorizontalBreakWidget, IconAccessoryWidget, Icons, ImageLike, ImageWidget, InlineSeparatorWidget, InlineWidget, InlineWidgetOrderedMembers, ListItemAccessories, ListItemWidget, ListSectionWidget, ListSectionWidgetOrderedMembers, ListWidget, ListWidgetOrderedMembers, MetadataIconWidget, MetadataLinkWidget, MetadataSeparatorWidget, MetadataTagItemWidget, MetadataTagListWidget, MetadataTagListWidgetOrderedMembers, MetadataValueWidget, MetadataWidget, MetadataWidgetOrderedMembers, ParagraphWidget, PasswordFieldWidget, PhysicalKey, PhysicalShortcut, PluginId, RootWidget, RootWidgetMembers, SearchBarWidget, SelectWidget, SelectWidgetOrderedMembers, SeparatorWidget, TextAccessoryWidget, TextFieldWidget, UiWidgetId};
use gauntlet_common_ui::shortcut_to_text;
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::image::Handle;
use iced::widget::text::Shaping;
use iced::widget::tooltip::Position;
use iced::widget::{button, checkbox, column, container, horizontal_rule, horizontal_space, image, mouse_area, pick_list, row, scrollable, stack, text, text_input, tooltip, value, vertical_rule, Space};
use iced::{Alignment, Font, Length, Task};
use iced_aw::date_picker::Date;
use iced_aw::helpers::{date_picker, grid, grid_row};
use iced_aw::GridRow;
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use itertools::Itertools;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;

#[derive(Debug)]
pub struct ComponentWidgets<'b> {
    root_widget: &'b mut Option<Arc<RootWidget>>,
    state: &'b mut HashMap<UiWidgetId, ComponentWidgetState>,
    images: &'b HashMap<UiWidgetId, Vec<u8>>
}

impl<'b> ComponentWidgets<'b> {
    pub fn new(
        root_widget: &'b mut Option<Arc<RootWidget>>,
        state: &'b mut HashMap<UiWidgetId, ComponentWidgetState>,
        images: &'b HashMap<UiWidgetId, Vec<u8>>
    ) -> ComponentWidgets<'b> {
        Self {
            root_widget,
            state,
            images
        }
    }

    fn text_field_state(&self, widget_id: UiWidgetId) -> &TextFieldState {
        let state = self.state.get(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::TextField(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }

    fn text_field_state_mut(&mut self, widget_id: UiWidgetId) -> &mut TextFieldState {
        Self::text_field_state_mut_on_state(&mut self.state, widget_id)
    }

    fn text_field_state_mut_on_state(state: &mut HashMap<UiWidgetId, ComponentWidgetState>, widget_id: UiWidgetId) -> &mut TextFieldState {
        let state = state.get_mut(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::TextField(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }

    fn checkbox_state(&self, widget_id: UiWidgetId) -> &CheckboxState {
        let state = self.state.get(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::Checkbox(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }

    fn date_picker_state(&self, widget_id: UiWidgetId) -> &DatePickerState {
        let state = self.state.get(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::DatePicker(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }

    fn select_state(&self, widget_id: UiWidgetId) -> &SelectState {
        let state = self.state.get(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::Select(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }

    fn root_state(&self, widget_id: UiWidgetId) -> &RootState {
        let state = self.state.get(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::Root(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }

    fn root_state_mut(&mut self, widget_id: UiWidgetId) -> &mut RootState {
        Self::root_state_mut_on_field(&mut self.state, widget_id)
    }

    fn root_state_mut_on_field(state: &mut HashMap<UiWidgetId, ComponentWidgetState>, widget_id: UiWidgetId) -> &mut RootState {
        let state = state.get_mut(&widget_id).expect(&format!("requested state should always be present for id: {}", widget_id));

        match state {
            ComponentWidgetState::Root(state) => state,
            _ => panic!("TextFieldState expected, {:?} found", state)
        }
    }
}


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
                    result.insert(widget.__id__, ComponentWidgetState::root(ESTIMATED_MAIN_LIST_ITEM_HEIGHT, 7));

                    if let Some(widget) = &widget.content.search_bar {
                        result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                    }
                }
                RootWidgetMembers::Grid(widget) => {
                    // cursed heuristic
                    let has_title = widget.content
                        .ordered_members
                        .iter()
                        .flat_map(|members| match members {
                            GridWidgetOrderedMembers::GridItem(widget) => vec![widget],
                            GridWidgetOrderedMembers::GridSection(widget) => {
                                widget.content.ordered_members
                                    .iter()
                                    .map(|members| match members {
                                        GridSectionWidgetOrderedMembers::GridItem(widget) => widget
                                    })
                                    .collect()
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
struct TextFieldState {
    text_input_id: text_input::Id,
    state_value: String
}

#[derive(Debug, Clone)]
struct CheckboxState {
    state_value: bool
}

#[derive(Debug, Clone)]
struct DatePickerState {
    show_picker: bool,
    state_value: Date,
}

#[derive(Debug, Clone)]
struct SelectState {
    state_value: Option<String>
}

#[derive(Debug, Clone)]
struct RootState {
    show_action_panel: bool,
    focused_item: ScrollHandle<UiWidgetId>,
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
            state_value: value.to_owned().unwrap_or_default()
        })
    }

    fn checkbox(value: &Option<bool>) -> ComponentWidgetState {
        ComponentWidgetState::Checkbox(CheckboxState {
            state_value: value.to_owned().unwrap_or(false)
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
            state_value: value.to_owned()
        })
    }
}

#[derive(Debug, Clone)]
pub enum TextRenderType {
    None,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl<'b> ComponentWidgets<'b> {
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
                        ActionPanelWidgetOrderedMembers::Action(widget) => {
                            result.push(widget.__id__)
                        }
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

    fn grid_section_sizes(grid_widget: &GridWidget) -> Vec<GridSectionData> {
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
                        cumulative_row_index = cumulative_row_index_at_start + (usize::div_ceil(pending_section_size, width));

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
                    None => {
                        return Task::none()
                    }
                    Some(widget) => widget.__id__
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.content.search_bar {
                    None => {
                        return Task::none()
                    }
                    Some(widget) => widget.__id__
                }
            }
            _ => return Task::none()
        };

        let TextFieldState { text_input_id, state_value } = ComponentWidgets::text_field_state_mut_on_state(&mut self.state, widget_id);

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
                    None => {
                        return Task::none()
                    }
                    Some(widget) => widget.__id__
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.content.search_bar {
                    None => {
                        return Task::none()
                    }
                    Some(widget) => widget.__id__
                }
            }
            _ => return Task::none()
        };

        let TextFieldState { text_input_id, state_value } = ComponentWidgets::text_field_state_mut_on_state(&mut self.state, widget_id);

        let mut chars = state_value.chars();
        chars.next_back();
        *state_value = chars.as_str().to_owned();

        text_input::focus(text_input_id.clone())
    }

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
                    None => {
                        return AppMsg::Noop
                    }
                    Some(widget) => widget.__id__
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.content.search_bar {
                    None => {
                        return AppMsg::Noop
                    }
                    Some(widget) => widget.__id__
                }
            }
            _ => return AppMsg::Noop
        };

        AppMsg::FocusPluginViewSearchBar {
            widget_id
        }
    }

    pub fn focus_search_bar(&self, widget_id: UiWidgetId) -> Task<AppMsg> {
        let TextFieldState { text_input_id, .. } = self.text_field_state(widget_id);

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
            RootWidgetMembers::List(widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, widget.__id__);

                focused_item.focus_previous()
                    .unwrap_or_else(|| Task::none())
            }
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, grid_widget.__id__);

                let Some(current_index) = &focused_item.index else {
                    return Task::none();
                };

                let amount_per_section_total = Self::grid_section_sizes(grid_widget);

                match grid_up_offset(*current_index, amount_per_section_total) {
                    None => Task::none(),
                    Some(data) => {
                        match focused_item.focus_previous_in(data.offset) {
                            None => Task::none(),
                            Some(_) => focused_item.scroll_to(data.row_index)
                        }
                    }
                }
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
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, widget.__id__);

                let total = widget.content.ordered_members
                    .iter()
                    .flat_map(|members| {
                        match members {
                            ListWidgetOrderedMembers::ListItem(widget) => vec![widget],
                            ListWidgetOrderedMembers::ListSection(widget) => {
                                widget.content.ordered_members
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

                focused_item.focus_next(total)
                    .unwrap_or_else(|| Task::none())
            }
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, grid_widget.__id__);

                let amount_per_section_total = Self::grid_section_sizes(grid_widget);

                let total = amount_per_section_total
                    .iter()
                    .map(|data| data.amount_in_section)
                    .sum();

                let Some(current_index) = &focused_item.index else {
                    let unfocus = match &grid_widget.content.search_bar {
                        None => Task::none(),
                        Some(_) => {
                            // there doesn't seem to be an unfocus command but focusing non-existing input will unfocus all
                            text_input::focus(text_input::Id::unique())
                        }
                    };

                    let _ = focused_item.focus_next(total);

                    return Task::batch([
                        unfocus,
                        focused_item.scroll_to(0)
                    ])
                };

                match grid_down_offset(*current_index, amount_per_section_total) {
                    None => Task::none(),
                    Some(data) => {
                        match focused_item.focus_next_in(total, data.offset) {
                            None => Task::none(),
                            Some(_) => focused_item.scroll_to(data.row_index)
                        }
                    }
                }
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
            RootWidgetMembers::Grid(widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, widget.__id__);

                let _ = focused_item.focus_previous();

                // focused_item.scroll_to(0)
                // TODO
                Task::none()
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
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, grid_widget.__id__);

                let total = grid_widget.content.ordered_members
                    .iter()
                    .flat_map(|members| {
                        match members {
                            GridWidgetOrderedMembers::GridItem(widget) => vec![widget],
                            GridWidgetOrderedMembers::GridSection(widget) => {
                                widget.content.ordered_members
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
                Task::none()
            }
        }
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

    fn render_text<'a>(&self, value: &[String], context: TextRenderType) -> Element<'a, ComponentWidgetEvent> {
        let header = match context {
            TextRenderType::None => None,
            TextRenderType::H1 => Some(34),
            TextRenderType::H2 => Some(30),
            TextRenderType::H3 => Some(24),
            TextRenderType::H4 => Some(20),
            TextRenderType::H5 => Some(18),
            TextRenderType::H6 => Some(16),
        };

        let mut text = text(value.join(""))
            .shaping(Shaping::Advanced);

        if let Some(size) = header {
            text = text
                .size(size)
                .font(Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                })
        }

        text.into()
    }

    pub fn render_root_widget<'a>(
        &self,
        plugin_view_state: &PluginViewState,
        entrypoint_name: Option<&String>,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        match &self.root_widget {
            None => {
                horizontal_space()
                    .into()
            }
            Some(root) => {
                match &root.content {
                    None => {
                        horizontal_space()
                            .into()
                    }
                    Some(content) => {
                        let entrypoint_name = entrypoint_name.expect("entrypoint name should always exist after render");

                        match content {
                            RootWidgetMembers::Detail(widget) => {
                                let RootState { show_action_panel, .. } = self.root_state(widget.__id__);

                                let content = self.render_detail_widget(widget, false);

                                self.render_plugin_root(
                                    *show_action_panel,
                                    widget.__id__,
                                    &None,
                                    &widget.content.actions,
                                    content,
                                    widget.is_loading.unwrap_or(false),
                                    plugin_view_state,
                                    entrypoint_name,
                                    action_shortcuts,
                                )
                            },
                            RootWidgetMembers::Form(widget) => self.render_form_widget(widget, plugin_view_state, entrypoint_name, action_shortcuts),
                            RootWidgetMembers::List(widget) => self.render_list_widget(widget, plugin_view_state, entrypoint_name, action_shortcuts),
                            RootWidgetMembers::Grid(widget) => self.render_grid_widget(widget, plugin_view_state, entrypoint_name, action_shortcuts),
                            _ => {
                                panic!("used inline widget in non-inline place")
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn render_root_inline_widget<'a>(&self, plugin_name: Option<&String>, entrypoint_name: Option<&String>) -> Element<'a, ComponentWidgetEvent> {
        match &self.root_widget {
            None => {
                horizontal_space()
                    .into()
            }
            Some(root) => {
                match &root.content {
                    None => {
                        horizontal_space()
                            .into()
                    }
                    Some(content) => {
                        match content {
                            RootWidgetMembers::Inline(widget) => {
                                let entrypoint_name = entrypoint_name.expect("entrypoint name should always exist after render");
                                let plugin_name = plugin_name.expect("entrypoint name should always exist after render");

                                self.render_inline_widget(widget, plugin_name, entrypoint_name)
                            },
                            _ => {
                                panic!("used non-inline widget in inline place")
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_metadata_tag_item_widget<'a>(&self, widget: &MetadataTagItemWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let tag: Element<_> = button(content)
            .on_press(ComponentWidgetEvent::TagClick { widget_id: widget.__id__ })
            .themed(ButtonStyle::MetadataTagItem);

        container(tag)
            .themed(ContainerStyle::MetadataTagItem)
    }

    fn render_metadata_tag_list_widget<'a>(&self, widget: &MetadataTagListWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<Element<_>> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    MetadataTagListWidgetOrderedMembers::MetadataTagItem(content) => self.render_metadata_tag_item_widget(&content)
                }
            })
            .collect();

        let value = row(content)
            .wrap()
            .into();

        render_metadata_item(&widget.label, value)
            .into()
    }

    fn render_metadata_link_widget<'a>(&self, widget: &MetadataLinkWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let icon: Element<_> = value(Bootstrap::BoxArrowUpRight)
            .font(BOOTSTRAP_FONT)
            .size(16)
            .into();

        let icon = container(icon)
            .themed(ContainerStyle::MetadataLinkIcon);

        let content: Element<_> = row([content, icon])
            .align_y(Alignment::Center)
            .into();

        let link: Element<_> = button(content)
            .on_press(ComponentWidgetEvent::LinkClick { widget_id: widget.__id__, href: widget.href.to_owned() })
            .themed(ButtonStyle::MetadataLink);

        let content: Element<_> = if widget.href.is_empty() {
            link
        } else {
            let href: Element<_> = text(widget.href.to_string())
                .shaping(Shaping::Advanced)
                .into();

            tooltip(link, href, Position::Top)
                .themed(TooltipStyle::Tooltip)
        };

        render_metadata_item(&widget.label, content)
            .into()
    }

    fn render_metadata_value_widget<'a>(&self, widget: &MetadataValueWidget) -> Element<'a, ComponentWidgetEvent> {
        let value: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        render_metadata_item(&widget.label, value)
            .into()
    }

    fn render_metadata_icon_widget<'a>(&self, widget: &MetadataIconWidget) -> Element<'a, ComponentWidgetEvent> {
        let value = value(icon_to_bootstrap(&widget.icon))
            .font(BOOTSTRAP_FONT)
            .size(26)
            .into();

        render_metadata_item(&widget.label, value)
            .into()
    }

    fn render_metadata_separator_widget<'a>(&self, _widget: &MetadataSeparatorWidget) -> Element<'a, ComponentWidgetEvent> {
        let separator: Element<_> = horizontal_rule(1)
            .into();

        container(separator)
            .width(Length::Fill)
            .themed(ContainerStyle::MetadataSeparator)
    }

    fn render_metadata_widget<'a>(&self, widget: &MetadataWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<Element<_>> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    MetadataWidgetOrderedMembers::MetadataTagList(content) => self.render_metadata_tag_list_widget(content),
                    MetadataWidgetOrderedMembers::MetadataLink(content) => self.render_metadata_link_widget(content),
                    MetadataWidgetOrderedMembers::MetadataValue(content) => self.render_metadata_value_widget(content),
                    MetadataWidgetOrderedMembers::MetadataIcon(content) => self.render_metadata_icon_widget(content),
                    MetadataWidgetOrderedMembers::MetadataSeparator(content) => self.render_metadata_separator_widget(content),
                }
            })
            .collect();

        let metadata: Element<_> = column(content)
            .into();

        let metadata = container(metadata)
            .width(Length::Fill)
            .themed(ContainerStyle::MetadataInner);

        scrollable(metadata)
            .width(Length::Fill)
            .into()
    }

    fn render_paragraph_widget<'a>(&self, widget: &ParagraphWidget, centered: bool) -> Element<'a, ComponentWidgetEvent> {
        let paragraph: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let mut content = container(paragraph)
            .width(Length::Fill);

        if centered {
            content = content.align_x(Horizontal::Center)
        }

        content.themed(ContainerStyle::ContentParagraph)
    }

    fn render_image_widget<'a>(&self, widget: &ImageWidget, centered: bool) -> Element<'a, ComponentWidgetEvent> {
        // TODO image size, height and width
        let content: Element<_> = render_image(self.images, widget.__id__, &widget.source, None);

        let mut content = container(content)
            .width(Length::Fill);

        if centered {
            content = content.align_x(Horizontal::Center)
        }

        content.themed(ContainerStyle::ContentImage)
    }

    fn render_h1_widget<'a>(&self, widget: &H1Widget) -> Element<'a, ComponentWidgetEvent> {
        self.render_text(&widget.content.text, TextRenderType::H1)
    }

    fn render_h2_widget<'a>(&self, widget: &H2Widget) -> Element<'a, ComponentWidgetEvent> {
        self.render_text(&widget.content.text, TextRenderType::H2)
    }

    fn render_h3_widget<'a>(&self, widget: &H3Widget) -> Element<'a, ComponentWidgetEvent> {
        self.render_text(&widget.content.text, TextRenderType::H3)
    }

    fn render_h4_widget<'a>(&self, widget: &H4Widget) -> Element<'a, ComponentWidgetEvent> {
        self.render_text(&widget.content.text, TextRenderType::H4)
    }

    fn render_h5_widget<'a>(&self, widget: &H5Widget) -> Element<'a, ComponentWidgetEvent> {
        self.render_text(&widget.content.text, TextRenderType::H5)
    }

    fn render_h6_widget<'a>(&self, widget: &H6Widget) -> Element<'a, ComponentWidgetEvent> {
        self.render_text(&widget.content.text, TextRenderType::H6)
    }

    fn render_horizontal_break_widget<'a>(&self, _widget: &HorizontalBreakWidget) -> Element<'a, ComponentWidgetEvent> {
        let separator: Element<_> = horizontal_rule(1).into();

        container(separator)
            .width(Length::Fill)
            .themed(ContainerStyle::ContentHorizontalBreak)
    }

    fn render_code_block_widget<'a>(&self, widget: &CodeBlockWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let content = container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::ContentCodeBlockText);

        container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::ContentCodeBlock)
    }

    fn render_content_widget<'a>(&self, widget: &ContentWidget, centered: bool) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<_> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    ContentWidgetOrderedMembers::Paragraph(widget) => self.render_paragraph_widget(widget, centered),
                    ContentWidgetOrderedMembers::Image(widget) => self.render_image_widget(widget, centered),
                    ContentWidgetOrderedMembers::H1(widget) => self.render_h1_widget(widget),
                    ContentWidgetOrderedMembers::H2(widget) => self.render_h2_widget(widget),
                    ContentWidgetOrderedMembers::H3(widget) => self.render_h3_widget(widget),
                    ContentWidgetOrderedMembers::H4(widget) => self.render_h4_widget(widget),
                    ContentWidgetOrderedMembers::H5(widget) => self.render_h5_widget(widget),
                    ContentWidgetOrderedMembers::H6(widget) => self.render_h6_widget(widget),
                    ContentWidgetOrderedMembers::HorizontalBreak(widget) => self.render_horizontal_break_widget(widget),
                    ContentWidgetOrderedMembers::CodeBlock(widget) => self.render_code_block_widget(widget),
                }
            })
            .collect();

        let content: Element<_> = column(content)
            .into();

        if centered {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
        } else {
            content
        }
    }

    fn render_detail_widget<'a>(&self, widget: &DetailWidget, is_in_list: bool) -> Element<'a, ComponentWidgetEvent> {
        let metadata_element = widget.content.metadata
            .as_ref()
            .map(|widget| {
                let content = self.render_metadata_widget(widget);

                container(content)
                    .width(if is_in_list { Length::Fill } else { Length::FillPortion(2) })
                    .height(if is_in_list { Length::FillPortion(3) } else { Length::Fill })
                    .themed(ContainerStyle::DetailMetadata)
            });

        let content_element = widget.content.content
            .as_ref()
            .map(|widget| {
                let content_element: Element<_> = container(self.render_content_widget(widget, false))
                    .width(Length::Fill)
                    .themed(ContainerStyle::DetailContentInner);

                let content_element: Element<_> = scrollable(content_element)
                    .width(Length::Fill)
                    .into();

                let content_element: Element<_> = container(content_element)
                    .width(if is_in_list { Length::Fill } else { Length::FillPortion(3) })
                    .height(if is_in_list { Length::FillPortion(5) } else { Length::Fill })
                    .themed(ContainerStyle::DetailContent);

                content_element
            });

        let separator = if is_in_list {
            horizontal_rule(1)
                .into()
        } else {
            vertical_rule(1)
                .into()
        };

        let list_fn = |vec| {
            if is_in_list {
                column(vec)
                    .into()
            } else {
                row(vec)
                    .into()
            }
        };

        let content: Element<_> = match (content_element, metadata_element) {
            (Some(content_element), Some(metadata_element)) => {
                list_fn(vec![content_element, separator, metadata_element])
            }
            (Some(content_element), None) => {
                list_fn(vec![content_element])
            }
            (None, Some(metadata_element)) => {
                list_fn(vec![metadata_element])
            }
            (None, None) => {
                list_fn(vec![])
            }
        };

        content
    }

    fn render_text_field_widget<'a>(&self, widget: &TextFieldWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value, .. } = self.text_field_state(widget.__id__);

        text_input("", state_value)
            .on_input(move |value| ComponentWidgetEvent::OnChangeTextField { widget_id, value })
            .themed(TextInputStyle::FormInput)
    }

    fn render_password_field_widget<'a>(&self, widget: &PasswordFieldWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value, .. } = self.text_field_state(widget_id);

        text_input("", state_value)
            .secure(true)
            .on_input(move |value| ComponentWidgetEvent::OnChangePasswordField { widget_id, value })
            .themed(TextInputStyle::FormInput)
    }

    fn render_checkbox_widget<'a>(&self, widget: &CheckboxWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let CheckboxState { state_value } = self.checkbox_state(widget_id);

        checkbox(widget.title.as_deref().unwrap_or_default(), state_value.to_owned())
            .on_toggle(move |value| ComponentWidgetEvent::ToggleCheckbox { widget_id, value })
            .into()
    }

    fn render_date_picker_widget<'a>(&self, widget: &DatePickerWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let DatePickerState { state_value, show_picker } = self.date_picker_state(widget.__id__);

        let button_text = text(state_value.to_string())
            .shaping(Shaping::Advanced);

        let button = button(button_text)
            .on_press(ComponentWidgetEvent::ToggleDatePicker { widget_id: widget.__id__ });

        // TODO unable to customize buttons here, split to separate button styles
        //     DatePickerUnderlay,
        //     DatePickerOverlay,

        date_picker(
            show_picker.to_owned(),
            state_value.to_owned(),
            button,
            ComponentWidgetEvent::CancelDatePicker { widget_id },
            move |date| {
                ComponentWidgetEvent::SubmitDatePicker {
                    widget_id,
                    value: date.to_string(),
                }
            },
        ).themed(DatePickerStyle::Default)
    }

    fn render_select_widget<'a>(&self, widget: &SelectWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let SelectState { state_value } = self.select_state(widget_id);

        let items: Vec<_> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    SelectWidgetOrderedMembers::SelectItem(widget) => {
                        SelectItem {
                            value: widget.value.to_owned(),
                            label: widget.content.text.join(""),
                        }
                    }
                }
            })
            .collect();

        let state_value = state_value.clone()
            .map(|value| items.iter().find(|item| item.value == value))
            .flatten()
            .map(|value| value.clone());

        pick_list(
            items,
            state_value,
            move |item| ComponentWidgetEvent::SelectPickList { widget_id, value: item.value },
        ).themed(PickListStyle::Default)
    }

    fn render_separator_widget<'a>(&self, _widget: &SeparatorWidget) -> Element<'a, ComponentWidgetEvent> {
        horizontal_rule(1)
            .into()
    }

    fn render_form_widget<'a>(
        &self,
        widget: &FormWidget,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let RootState { show_action_panel, .. } = self.root_state(widget_id);

        let items: Vec<Element<_>> = widget.content.ordered_members
            .iter()
            .map(|members| {
                fn render_field<'c, 'd>(field: Element<'c, ComponentWidgetEvent>, label: &'d Option<String>) -> Element<'c, ComponentWidgetEvent> {
                    let before_or_label: Element<_> = match label {
                        None => {
                            Space::with_width(Length::FillPortion(2))
                                .into()
                        }
                        Some(label) => {
                            let label: Element<_> = text(label.to_string())
                                .shaping(Shaping::Advanced)
                                .align_x(Horizontal::Right)
                                .width(Length::Fill)
                                .into();

                            container(label)
                                .width(Length::FillPortion(2))
                                .themed(ContainerStyle::FormInputLabel)
                        }
                    };

                    let form_input = container(field)
                        .width(Length::FillPortion(3))
                        .into();

                    let after = Space::with_width(Length::FillPortion(2))
                        .into();

                    let content = vec![
                        before_or_label,
                        form_input,
                        after,
                    ];

                    let row: Element<_> = row(content)
                        .align_y(Alignment::Center)
                        .themed(RowStyle::FormInput);

                    row
                }

                match members {
                    FormWidgetOrderedMembers::Separator(widget) => self.render_separator_widget(widget),
                    FormWidgetOrderedMembers::TextField(widget) => render_field(self.render_text_field_widget(widget), &widget.label),
                    FormWidgetOrderedMembers::PasswordField(widget) => render_field(self.render_password_field_widget(widget), &widget.label),
                    FormWidgetOrderedMembers::Checkbox(widget) => render_field(self.render_checkbox_widget(widget), &widget.label),
                    FormWidgetOrderedMembers::DatePicker(widget) => render_field(self.render_date_picker_widget(widget), &widget.label),
                    FormWidgetOrderedMembers::Select(widget) => render_field(self.render_select_widget(widget), &widget.label)
                }
            })
            .collect();

        let content: Element<_> = column(items)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::FormInner);

        let content: Element<_> = scrollable(content)
            .width(Length::Fill)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::Form);

        self.render_plugin_root(
            *show_action_panel,
            widget_id,
            &None,
            &widget.content.actions,
            content,
            widget.is_loading.unwrap_or(false),
            plugin_view_state,
            entrypoint_name,
            action_shortcuts
        )
    }

    fn render_inline_separator_widget<'a>(&self, widget: &InlineSeparatorWidget) -> Element<'a, ComponentWidgetEvent> {
        match &widget.icon {
            None => vertical_rule(1).into(),
            Some(icon) => {
                let top_rule: Element<_> = vertical_rule(1)
                    .into();

                let top_rule = container(top_rule)
                    .align_x(Horizontal::Center)
                    .into();

                let icon = value(icon_to_bootstrap(icon))
                    .font(BOOTSTRAP_FONT)
                    .size(45)
                    .themed(TextStyle::InlineSeparator);

                let bot_rule: Element<_> = vertical_rule(1)
                    .into();

                let bot_rule = container(bot_rule)
                    .align_x(Horizontal::Center)
                    .into();

                column([top_rule, icon, bot_rule])
                    .align_x(Alignment::Center)
                    .into()
            }
        }
    }

    fn render_inline_widget<'a>(&self, widget: &InlineWidget, plugin_name: &str, entrypoint_name: &str) -> Element<'a, ComponentWidgetEvent> {
        let name: Element<_> = text(format!("{} - {}", plugin_name, entrypoint_name))
            .shaping(Shaping::Advanced)
            .themed(TextStyle::InlineName);

        let name: Element<_> = container(name)
            .themed(ContainerStyle::InlineName);

        let content: Vec<Element<_>> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    InlineWidgetOrderedMembers::Content(widget) => {
                        let element = self.render_content_widget(widget, true);

                        container(element)
                            .into()
                    },
                    InlineWidgetOrderedMembers::InlineSeparator(widget) => self.render_inline_separator_widget(widget)
                }
            })
            .collect();

        let content: Element<_> = row(content)
            .into();

        let content: Element<_> = container(content)
            .themed(ContainerStyle::InlineInner);

        let content: Element<_> = column(vec![name, content])
            .width(Length::Fill)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::Inline);

        content
    }

    fn render_empty_view_widget<'a>(&self, widget: &EmptyViewWidget) -> Element<'a, ComponentWidgetEvent> {
        let image: Option<Element<_>> = widget.image
            .as_ref()
            .map(|image| render_image(self.images, widget.__id__, image, Some(TextStyle::EmptyViewSubtitle)));

        let title: Element<_> = text(widget.title.to_string())
            .shaping(Shaping::Advanced)
            .into();

        let subtitle: Element<_> = match &widget.description {
            None => horizontal_space().into(),
            Some(subtitle) => {
                text(subtitle.to_string())
                    .shaping(Shaping::Advanced)
                    .themed(TextStyle::EmptyViewSubtitle)
            }
        };

        let mut content = vec![title, subtitle];
        if let Some(image) = image {
            let image: Element<_> = container(image)
                .themed(ContainerStyle::EmptyViewImage);

            content.insert(0, image)
        }

        let content: Element<_> = column(content)
            .align_x(Alignment::Center)
            .into();

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }


    fn render_search_bar_widget<'a>(&self, widget: &SearchBarWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value, text_input_id } = self.text_field_state(widget_id);

        text_input(widget.placeholder.as_deref().unwrap_or_default(), state_value)
            .id(text_input_id.clone())
            .ignore_with_modifiers(true)
            .on_input(move |value| ComponentWidgetEvent::OnChangeSearchBar { widget_id, value })
            .themed(TextInputStyle::PluginSearchBar)
    }

    fn render_list_widget<'a>(
        &self,
        list_widget: &ListWidget,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = list_widget.__id__;
        let RootState { show_action_panel, focused_item } = self.root_state(widget_id);

        let mut pending: Vec<&ListItemWidget> = vec![];
        let mut items: Vec<Element<_>> = vec![];
        let index_counter = &Cell::new(0);
        let mut first_section = true;

        for members in &list_widget.content.ordered_members {
            match &members {
                ListWidgetOrderedMembers::ListItem(widget) => {
                    first_section = false;
                    pending.push(widget)
                },
                ListWidgetOrderedMembers::ListSection(widget) => {
                    if !pending.is_empty() {
                        let content: Vec<_> = pending
                            .iter()
                            .map(|widget| self.render_list_item_widget(widget, focused_item.index, index_counter))
                            .collect();

                        let content: Element<_> = column(content)
                            .into();

                        items.push(content);

                        pending = vec![];
                    }

                    items.push(self.render_list_section_widget(widget, focused_item.index, index_counter, first_section));

                    first_section = false;
                },
            }
        }

        if !pending.is_empty() {
            let content: Vec<_> = pending
                .iter()
                .map(|widget| self.render_list_item_widget(widget, focused_item.index, index_counter))
                .collect();

            let content: Element<_> = column(content)
                .into();

            items.push(content);
        }

        let content = if items.is_empty() {
            match &list_widget.content.empty_view {
                Some(widget) => self.render_empty_view_widget(widget),
                None => horizontal_space().into()
            }
        } else {
            let content: Element<_> = column(items)
                .width(Length::Fill)
                .into();

            let content: Element<_> = container(content)
                .width(Length::Fill)
                .themed(ContainerStyle::ListInner);

            let content: Element<_> = scrollable(content)
                .id(focused_item.scrollable_id.clone())
                .width(Length::Fill)
                .into();

            let content: Element<_> = container(content)
                .width(Length::FillPortion(3))
                .themed(ContainerStyle::List);

            content
        };

        let mut elements = vec![content];

        if let Some(detail) = &list_widget.content.detail {
            let detail = self.render_detail_widget(detail, true);

            let detail: Element<_> = container(detail)
                .width(Length::FillPortion(5))
                .into();

            let separator: Element<_> = vertical_rule(1)
                .into();

            elements.push(separator);

            elements.push(detail);
        }

        let content: Element<_> = row(elements)
            .height(Length::Fill)
            .into();

        self.render_plugin_root(
            *show_action_panel,
            widget_id,
            &list_widget.content.search_bar,
            &list_widget.content.actions,
            content,
            list_widget.is_loading.unwrap_or(false),
            plugin_view_state,
            entrypoint_name,
            action_shortcuts
        )
    }

    fn render_list_section_widget<'a>(
        &self,
        widget: &ListSectionWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>,
        first_section: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<_> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    ListSectionWidgetOrderedMembers::ListItem(widget) => self.render_list_item_widget(widget, item_focus_index, index_counter)
                }
            })
            .collect();

        let content = column(content)
            .into();

        let section_title_style = if first_section { RowStyle::ListFirstSectionTitle } else { RowStyle::ListSectionTitle };

        render_section(content, Some(&widget.title), &widget.subtitle, section_title_style, TextStyle::ListSectionTitle, TextStyle::ListSectionSubtitle)
    }

    fn render_list_item_widget<'a>(
        &self,
        widget: &ListItemWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>
    ) -> Element<'a, ComponentWidgetEvent> {
        let icon: Option<Element<_>> = widget.icon
            .as_ref()
            .map(|icon| render_image(self.images, widget.__id__, icon, None));

        let title: Element<_> = text(widget.title.to_string())
            .shaping(Shaping::Advanced)
            .into();
        let title: Element<_> = container(title)
            .themed(ContainerStyle::ListItemTitle);

        let mut content = vec![title];

        if let Some(icon) = icon {
            let icon: Element<_> = container(icon)
                .themed(ContainerStyle::ListItemIcon);

            content.insert(0, icon)
        }

        if let Some(subtitle) = &widget.subtitle {
            let subtitle: Element<_> = text(subtitle.to_string())
                .shaping(Shaping::Advanced)
                .themed(TextStyle::ListItemSubtitle);
            let subtitle: Element<_> = container(subtitle)
                .themed(ContainerStyle::ListItemSubtitle);

            content.push(subtitle)
        }

        if widget.content.accessories.len() > 0 {
            let accessories: Vec<Element<_>> = widget.content.accessories
                .iter()
                .map(|accessory| {
                    match accessory {
                        ListItemAccessories::_0(widget) => render_text_accessory(self.images, widget),
                        ListItemAccessories::_1(widget) => render_icon_accessory(self.images, widget)
                    }
                })
                .collect();

            let accessories: Element<_> = row(accessories)
                .into();

            let space = horizontal_space()
                .into();

            content.push(space);
            content.push(accessories);
        }

        let content: Element<_> = row(content)
            .align_y(Alignment::Center)
            .into();

        let style = match item_focus_index {
            None => ButtonStyle::ListItem,
            Some(focused_index) => {
                if focused_index == index_counter.get() {
                    ButtonStyle::ListItemFocused
                } else {
                    ButtonStyle::ListItem
                }
            }
        };

        index_counter.set(index_counter.get() + 1);

        button(content)
            .on_press(ComponentWidgetEvent::ListItemClick { widget_id: widget.__id__ })
            .width(Length::Fill)
            .themed(style)
    }

    fn render_grid_widget<'a>(
        &self,
        grid_widget: &GridWidget,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let RootState { show_action_panel, focused_item } = self.root_state(grid_widget.__id__);

        let mut pending: Vec<&GridItemWidget> = vec![];
        let mut items: Vec<Element<_>> = vec![];
        let index_counter = &Cell::new(0);
        let mut first_section = true;

        for members in &grid_widget.content.ordered_members {
            match &members {
                GridWidgetOrderedMembers::GridItem(widget) => {
                    first_section = false;
                    pending.push(widget)
                }
                GridWidgetOrderedMembers::GridSection(widget) => {
                    if !pending.is_empty() {
                        let content = self.render_grid(&pending, &grid_widget.columns, focused_item.index, index_counter);

                        items.push(content);

                        pending = vec![];
                    }

                    items.push(self.render_grid_section_widget(widget, focused_item.index, index_counter, first_section));

                    first_section = false;
                }
            }
        }

        if !pending.is_empty() {
            let content = self.render_grid(&pending, &grid_widget.columns, focused_item.index, index_counter);

            items.push(content);
        }

        let content: Element<_> = column(items)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::GridInner);

        let content: Element<_> = scrollable(content)
            .id(focused_item.scrollable_id.clone())
            .width(Length::Fill)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .themed(ContainerStyle::Grid);

        self.render_plugin_root(
            *show_action_panel,
            grid_widget.__id__,
            &grid_widget.content.search_bar,
            &grid_widget.content.actions,
            content,
            grid_widget.is_loading.unwrap_or(false),
            plugin_view_state,
            entrypoint_name,
            action_shortcuts
        )
    }

    fn render_grid_section_widget<'a>(
        &self,
        widget: &GridSectionWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>,
        first_section: bool
    ) -> Element<'a, ComponentWidgetEvent> {
        let items: Vec<_> = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    GridSectionWidgetOrderedMembers::GridItem(widget) => widget
                }
            })
            .collect();

        let content = self.render_grid(&items, &widget.columns, item_focus_index, index_counter);

        let section_title_style = if first_section { RowStyle::GridFirstSectionTitle } else { RowStyle::GridSectionTitle };

        render_section(content, Some(&widget.title), &widget.subtitle, section_title_style, TextStyle::GridSectionTitle, TextStyle::GridSectionSubtitle)
    }

    fn render_grid_item_widget<'a>(
        &self,
        widget: &GridItemWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>,
        grid_width: usize
    ) -> Element<'a, ComponentWidgetEvent> {
        let height = match grid_width {
            ..4 => 130,
            4 => 150,
            5 => 130,
            6 => 110,
            7 => 90,
            8 => 70,
            8.. => 50,
        };

        let content: Element<_> = container(self.render_content_widget(&widget.content.content, true))
            .height(height)
            .into();

        let style = match item_focus_index {
            None => ButtonStyle::GridItem,
            Some(focused_index) => {
                if focused_index == index_counter.get() {
                    ButtonStyle::GridItemFocused
                } else {
                    ButtonStyle::GridItem
                }
            }
        };

        index_counter.set(index_counter.get() + 1);

        let content: Element<_> = button(content)
            .on_press(ComponentWidgetEvent::GridItemClick { widget_id: widget.__id__ })
            .width(Length::Fill)
            .themed(style);

        let mut sub_content_left = vec![];

        if let Some(title) = &widget.title {
            // TODO text truncation when iced supports it
            let title = text(title.to_string())
                .shaping(Shaping::Advanced)
                .themed(TextStyle::GridItemTitle);

            sub_content_left.push(title);
        }

        if let Some(subtitle) = &widget.subtitle {
            let subtitle = text(subtitle.to_string())
                .shaping(Shaping::Advanced)
                .themed(TextStyle::GridItemSubTitle);

            sub_content_left.push(subtitle);
        }

        let mut sub_content_right = vec![];
        if let Some(widget) = &widget.content.accessory {
            sub_content_right.push(render_icon_accessory(self.images, widget));
        }

        let sub_content_left: Element<_> = column(sub_content_left)
            .width(Length::Fill)
            .into();

        let sub_content_right: Element<_> = column(sub_content_right)
            .width(Length::Shrink)
            .into();

        let sub_content: Element<_> = row(vec![sub_content_left, sub_content_right])
            .themed(RowStyle::GridItemTitle);

        let content: Element<_> = column(vec![content, sub_content])
            .width(Length::Fill)
            .into();

        content
    }

    fn render_grid<'a>(
        &self,
        items: &[&GridItemWidget],
        /*aspect_ratio: Option<&str>,*/
        columns: &Option<f64>,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>
    ) -> Element<'a, ComponentWidgetEvent> {
        // TODO
        // let (width, height) = match aspect_ratio {
        //     None => (1, 1),
        //     Some("1") => (1, 1),
        //     Some("3/2") => (3, 2),
        //     Some("2/3") => (2, 3),
        //     Some("4/3") => (4, 3),
        //     Some("3/4") => (3, 4),
        //     Some("16/9") => (16, 9),
        //     Some("9/16") => (9, 16),
        //     Some(value) => panic!("unsupported aspect_ratio {:?}", value)
        // };

        let grid_width = grid_width(columns);

        let rows: Vec<GridRow<_, _, _>> = items
            .iter()
            .map(|widget| self.render_grid_item_widget(widget, item_focus_index, index_counter, grid_width))
            .chunks(grid_width)
            .into_iter()
            .map(|row_items| {
                let mut row_items: Vec<_> = row_items.collect();
                row_items.resize_with(grid_width, || horizontal_space().into());

                grid_row(row_items).into()
            })
            .collect();

        let grid: Element<_> = grid(rows)
            .width(Length::Fill)
            .vertical_alignment(Vertical::Top)
            .themed(GridStyle::Default);

        grid
    }

    fn render_top_panel<'a>(&self, search_bar: &Option<SearchBarWidget>) -> Element<'a, ComponentWidgetEvent> {
        let icon = value(Bootstrap::ArrowLeft)
            .font(BOOTSTRAP_FONT);

        let back_button: Element<_> = button(icon)
            .on_press(ComponentWidgetEvent::PreviousView)
            .themed(ButtonStyle::RootTopPanelBackButton);

        let search_bar_element = search_bar
            .as_ref()
            .map(|widget| self.render_search_bar_widget(widget))
            .unwrap_or_else(|| Space::with_width(Length::FillPortion(3)).into());

        let top_panel: Element<_> = row(vec![back_button, search_bar_element])
            .align_y(Alignment::Center)
            .themed(RowStyle::RootTopPanel);

        let top_panel: Element<_> = container(top_panel)
            .width(Length::Fill)
            .themed(ContainerStyle::RootTopPanel);

        top_panel
    }

    fn render_plugin_root<'a>(
        &self,
        show_action_panel: bool,
        widget_id: UiWidgetId,
        search_bar: &Option<SearchBarWidget>,
        action_panel: &Option<ActionPanelWidget>,
        content: Element<'a, ComponentWidgetEvent>,
        is_loading: bool,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent>  {

        let top_panel = self.render_top_panel(search_bar);

        let top_separator = if is_loading {
            LoadingBar::new()
                .into()
        } else {
            horizontal_rule(1)
                .into()
        };

        let mut action_panel = convert_action_panel(action_panel, &action_shortcuts);

        let primary_action = action_panel.as_mut()
            .map(|panel| panel.find_first())
            .flatten()
            .map(|(label, widget_id)| {
                let shortcut = PhysicalShortcut {
                    physical_key: PhysicalKey::Enter,
                    modifier_shift: false,
                    modifier_control: false,
                    modifier_alt: false,
                    modifier_meta: false
                };

                (label.to_string(), widget_id, shortcut)
            });

        match plugin_view_state {
            PluginViewState::None => {
                render_root(
                    show_action_panel,
                    top_panel,
                    top_separator,
                    None,
                    content,
                    primary_action,
                    action_panel,
                    None::<&ScrollHandle<UiWidgetId>>,
                    entrypoint_name,
                    || ComponentWidgetEvent::ToggleActionPanel { widget_id },
                    |widget_id| ComponentWidgetEvent::RunPrimaryAction { widget_id },
                    |widget_id| ComponentWidgetEvent::ActionClick { widget_id },
                    || ComponentWidgetEvent::Noop,
                )
            }
            PluginViewState::ActionPanel { focused_action_item } => {
                render_root(
                    show_action_panel,
                    top_panel,
                    top_separator,
                    None,
                    content,
                    primary_action,
                    action_panel,
                    Some(&focused_action_item),
                    entrypoint_name,
                    || ComponentWidgetEvent::ToggleActionPanel { widget_id },
                    |widget_id| ComponentWidgetEvent::RunPrimaryAction { widget_id },
                    |widget_id| ComponentWidgetEvent::ActionClick { widget_id },
                    || ComponentWidgetEvent::Noop,
                )
            }
        }
    }
}


#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectItem {
    value: String,
    label: String
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}


fn render_metadata_item<'a>(label: &str, value: Element<'a, ComponentWidgetEvent>) -> Element<'a, ComponentWidgetEvent> {
    let label: Element<_> = text(label.to_string())
        .shaping(Shaping::Advanced)
        .themed(TextStyle::MetadataItemLabel);

    let label = container(label)
        .themed(ContainerStyle::MetadataItemLabel);

    let value = container(value)
        .themed(ContainerStyle::MetadataItemValue);

    column(vec![label, value])
        .into()
}

fn grid_width(columns: &Option<f64>) -> usize {
    columns.map(|value| value.trunc() as usize).unwrap_or(5)
}


fn render_section<'a>(content: Element<'a, ComponentWidgetEvent>, title: Option<&str>, subtitle: &Option<String>, theme_kind_title: RowStyle, theme_kind_title_text: TextStyle, theme_kind_subtitle_text: TextStyle) -> Element<'a, ComponentWidgetEvent> {
    let mut title_content = vec![];

    if let Some(title) = title {
        let title: Element<_> = text(title.to_string())
            .shaping(Shaping::Advanced)
            .size(15)
            .themed(theme_kind_title_text);

        title_content.push(title)
    }

    if let Some(subtitle) = subtitle {
        let subtitle: Element<_> = text(subtitle.to_string())
            .shaping(Shaping::Advanced)
            .size(15)
            .themed(theme_kind_subtitle_text);

        title_content.push(subtitle)
    }

    if title_content.is_empty() {
        let space: Element<_> = horizontal_space()
            .height(40)
            .into();

        title_content.push(space)
    }

    let title_content = row(title_content)
        .themed(theme_kind_title);

    column([title_content, content])
        .into()
}


#[derive(Debug)]
pub struct ActionPanel {
    pub title: Option<String>,
    pub items: Vec<ActionPanelItem>
}

impl ActionPanel {
    pub fn action_count(&self) -> usize {
        self.items.iter().map(|item| item.action_count()).sum()
    }

    pub fn find_first(&self) -> Option<(String, UiWidgetId)> {
        ActionPanelItem::find_first(&self.items)
    }
}

#[derive(Debug)]
pub enum ActionPanelItem {
    Action {
        label: String,
        widget_id: UiWidgetId,
        physical_shortcut: Option<PhysicalShortcut>
    },
    ActionSection {
        title: Option<String>,
        items: Vec<ActionPanelItem>
    }
}

impl ActionPanelItem {
    fn action_count(&self) -> usize {
        match self {
            ActionPanelItem::Action { .. } => 1,
            ActionPanelItem::ActionSection { items, .. } => {
                items.iter().map(|item| item.action_count()).sum()
            }
        }
    }

    fn find_first(items: &[ActionPanelItem]) -> Option<(String, UiWidgetId)> {
        for item in items {
            match item {
                ActionPanelItem::Action { label, widget_id, .. } => {
                    return Some((label.to_string(), *widget_id))
                }
                ActionPanelItem::ActionSection { items, .. } => {
                    if let Some(item) = Self::find_first(items) {
                        return Some(item)
                    }
                }
            }
        }

        None
    }
}

fn convert_action_panel(action_panel: &Option<ActionPanelWidget>, action_shortcuts: &HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
    match action_panel {
        Some(ActionPanelWidget { content, title, .. }) => {
            fn action_widget_to_action(ActionWidget { __id__, id, label }: &ActionWidget, action_shortcuts: &HashMap<String, PhysicalShortcut>) -> ActionPanelItem {
                let physical_shortcut: Option<PhysicalShortcut> = id.as_ref()
                    .map(|id| action_shortcuts.get(id))
                    .flatten()
                    .cloned();

                ActionPanelItem::Action {
                    label: label.clone(),
                    widget_id: *__id__,
                    physical_shortcut,
                }
            }

            let items = content.ordered_members.iter()
                .map(|members| {
                    match members {
                        ActionPanelWidgetOrderedMembers::Action(widget) => {
                            action_widget_to_action(widget, action_shortcuts)
                        }
                        ActionPanelWidgetOrderedMembers::ActionPanelSection(ActionPanelSectionWidget { content, title, .. }) => {
                            let section_items = content.ordered_members
                                .iter()
                                .map(|members| {
                                    match members {
                                        ActionPanelSectionWidgetOrderedMembers::Action(widget) => action_widget_to_action(widget, action_shortcuts)
                                    }
                                })
                                .collect();

                            ActionPanelItem::ActionSection {
                                title: title.clone(),
                                items: section_items,
                            }
                        }
                    }
                })
                .collect();

            Some(ActionPanel {
                title: title.clone(),
                items,
            })
        }
        _ => None
    }
}

fn render_action_panel_items<'a, T: 'a + Clone>(
    title: Option<String>,
    items: Vec<ActionPanelItem>,
    action_panel_focus_index: Option<usize>,
    on_action_click: &dyn Fn(UiWidgetId) -> T,
    index_counter: &Cell<usize>
) -> Vec<Element<'a, T>> {
    let mut columns = vec![];

    if let Some(title) = title {
        let text: Element<_> = text(title)
            .shaping(Shaping::Advanced)
            .font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            })
            .into();

        let text = container(text)
            .themed(ContainerStyle::ActionPanelTitle);

        columns.push(text)
    }

    let mut place_separator = false;

    for item in items {
        match item {
            ActionPanelItem::Action { label, widget_id, physical_shortcut } => {
                if place_separator {
                    let separator: Element<_> = horizontal_rule(1)
                        .themed(RuleStyle::ActionPanel);

                    columns.push(separator);

                    place_separator = false;
                }

                let physical_shortcut = match index_counter.get() {
                    0 => Some(PhysicalShortcut { // primary
                        physical_key: PhysicalKey::Enter,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: false,
                    }),
                    1 => Some(PhysicalShortcut { // secondary
                        physical_key: PhysicalKey::Enter,
                        modifier_shift: true,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: false,
                    }),
                    _ => physical_shortcut
                };

                let shortcut_element: Option<Element<_>> = physical_shortcut.as_ref()
                    .map(|shortcut| render_shortcut(shortcut));

                let content: Element<_> = if let Some(shortcut_element) = shortcut_element {
                    let text: Element<_> = text(label)
                        .shaping(Shaping::Advanced)
                        .into();

                    let space: Element<_> = horizontal_space()
                        .into();

                    row([text, space, shortcut_element])
                        .align_y(Alignment::Center)
                        .into()
                } else {
                    text(label)
                        .shaping(Shaping::Advanced)
                        .into()
                };

                let style = match action_panel_focus_index {
                    None => ButtonStyle::Action,
                    Some(focused_index) => {
                        if focused_index == index_counter.get() {
                            ButtonStyle::ActionFocused
                        } else {
                            ButtonStyle::Action
                        }
                    }
                };

                index_counter.set(index_counter.get() + 1);

                let content = button(content)
                    .on_press(on_action_click(widget_id))
                    .width(Length::Fill)
                    .themed(style);

                columns.push(content);
            }
            ActionPanelItem::ActionSection { title, items } => {
                let separator: Element<_> = horizontal_rule(1)
                    .themed(RuleStyle::ActionPanel);

                columns.push(separator);

                let content = render_action_panel_items(title, items, action_panel_focus_index, on_action_click, index_counter);

                for content in content {
                    columns.push(content);
                }

                place_separator = true;
            }
        };
    }

    columns
}

fn render_action_panel<'a, T: 'a + Clone, F: Fn(UiWidgetId) -> T, ACTION>(
    action_panel: ActionPanel,
    on_action_click: F,
    action_panel_scroll_handle: &ScrollHandle<ACTION>,
) -> Element<'a, T> {
    let columns = render_action_panel_items(action_panel.title, action_panel.items, action_panel_scroll_handle.index, &on_action_click, &Cell::new(0));

    let actions: Element<_> = column(columns)
        .into();

    let actions: Element<_> = scrollable(actions)
        .id(action_panel_scroll_handle.scrollable_id.clone())
        .width(Length::Fill)
        .into();

    container(actions)
        .themed(ContainerStyle::ActionPanel)
}

pub fn render_root<'a, T: 'a + Clone, ACTION>(
    show_action_panel: bool,
    top_panel: Element<'a, T>,
    top_separator: Element<'a, T>,
    toast_text: Option<&str>,
    content: Element<'a, T>,
    primary_action: Option<(String, UiWidgetId, PhysicalShortcut)>,
    action_panel: Option<ActionPanel>,
    action_panel_scroll_handle: Option<&ScrollHandle<ACTION>>,
    entrypoint_name: &str,
    on_panel_toggle_click: impl Fn() -> T,
    on_panel_primary_click: impl Fn(UiWidgetId) -> T,
    on_action_click: impl Fn(UiWidgetId) -> T,
    noop_msg: impl Fn() -> T,
) -> Element<'a, T>  {
    let entrypoint_name: Element<_> = text(entrypoint_name.to_string())
        .shaping(Shaping::Advanced)
        .into();

    let panel_height = 16 + 8 + 2;  // TODO get value from theme

    let primary_action = match primary_action {
        Some((label, widget_id, shortcut)) => {
            let label: Element<_> = text(label)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::RootBottomPanelPrimaryActionText);

            let label: Element<_> = container(label)
                .themed(ContainerStyle::RootBottomPanelPrimaryActionText);

            let shortcut = render_shortcut(&shortcut);

            let content: Element<_> = row(vec![label, shortcut])
                .into();

            let content: Element<_> = button(content)
                .on_press(on_panel_primary_click(widget_id))
                .themed(ButtonStyle::RootBottomPanelPrimaryActionButton);

            let content: Element<_> = container(content)
                .themed(ContainerStyle::RootBottomPanelPrimaryActionButton);

            Some(content)
        }
        None => None
    };

    let (hide_action_panel, action_panel, bottom_panel) = match action_panel {
        Some(action_panel) => {
            let actions_text: Element<_> = text("Actions")
                .themed(TextStyle::RootBottomPanelActionToggleText);

            let actions_text: Element<_> = container(actions_text)
                .themed(ContainerStyle::RootBottomPanelActionToggleText);

            let shortcut = render_shortcut(&PhysicalShortcut {
                physical_key: PhysicalKey::KeyK,
                modifier_shift: false,
                modifier_control: false,
                modifier_alt: true,
                modifier_meta: false,
            });

            let mut bottom_panel_content = vec![entrypoint_name];

            if let Some(toast_text) = toast_text {
                let toast_text = text(toast_text.to_string())
                    .into();

                bottom_panel_content.push(toast_text);
            }

            let space = horizontal_space()
                .into();

            bottom_panel_content.push(space);

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);

                let rule: Element<_> = vertical_rule(1)
                    .class(RuleStyle::PrimaryActionSeparator)
                    .into();

                let rule: Element<_> = container(rule)
                    .width(Length::Shrink)
                    .height(panel_height)
                    .max_height(panel_height)
                    .into();

                bottom_panel_content.push(rule);
            }

            let action_panel_toggle_content: Element<_> = row(vec![actions_text, shortcut])
                .into();

            let action_panel_toggle: Element<_> = button(action_panel_toggle_content)
                .on_press(on_panel_toggle_click())
                .themed(ButtonStyle::RootBottomPanelActionToggleButton);

            bottom_panel_content.push(action_panel_toggle);

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_y(Alignment::Center)
                .themed(RowStyle::RootBottomPanel);

            (!show_action_panel, Some(action_panel), bottom_panel)
        }
        None => {
            let space: Element<_> = Space::new(Length::Fill, panel_height)
                .into();

            let mut bottom_panel_content = vec![];

            if let Some(toast_text) = toast_text {
                let toast_text = text(toast_text.to_string())
                    .into();

                bottom_panel_content.push(toast_text);
            } else {
                bottom_panel_content.push(entrypoint_name);
            }

            bottom_panel_content.push(space);

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);
            }

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_y(Alignment::Center)
                .themed(RowStyle::RootBottomPanel);

            (true, None, bottom_panel)
        }
    };

    let bottom_panel: Element<_> = container(bottom_panel)
        .width(Length::Fill)
        .themed(ContainerStyle::RootBottomPanel);

    let content: Element<_> = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .themed(ContainerStyle::RootInner);

    let content: Element<_> = column(vec![top_panel, top_separator, content, bottom_panel])
        .into();

    let content: Element<_> = mouse_area(content)
        .on_press(if hide_action_panel { noop_msg() } else { on_panel_toggle_click() })
        .into();

    let mut content = vec![content];

    if let (Some(action_panel), Some(action_panel_scroll_handle)) = (action_panel, action_panel_scroll_handle) {
        if !hide_action_panel {
            let action_panel = render_action_panel(action_panel, on_action_click, action_panel_scroll_handle);

            let action_panel: Element<_>= container(action_panel)
                .padding(gauntlet_common_ui::padding(0.0, 8.0, 48.0, 0.0))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill)
                .into();

            content.push(action_panel);
        }
    };

    stack(content)
        .into()
}


fn render_shortcut<'a, T: 'a>(shortcut: &PhysicalShortcut) -> Element<'a, T> {
    let mut result = vec![];

    let (
        key_name,
        alt_modifier_text,
        meta_modifier_text,
        control_modifier_text,
        shift_modifier_text
    ) = shortcut_to_text(shortcut);

    fn apply_modifier<'result, 'element, T: 'element>(
        result: &'result mut Vec<Element<'element, T>>,
        modifier: Option<Element<'element, T>>
    ) {
        if let Some(modifier) = modifier {
            let modifier: Element<_> = container(modifier)
                .themed(ContainerStyle::ActionShortcutModifier);

            let modifier: Element<_> = container(modifier)
                .themed(ContainerStyle::ActionShortcutModifiersInit);

            result.push(modifier);
        }
    }

    apply_modifier(&mut result, meta_modifier_text);
    apply_modifier(&mut result, control_modifier_text);
    apply_modifier(&mut result, shift_modifier_text);
    apply_modifier(&mut result, alt_modifier_text);

    let key_name: Element<_> = container(key_name)
        .themed(ContainerStyle::ActionShortcutModifier);

    result.push(key_name);

    row(result)
        .themed(RowStyle::ActionShortcut)
}

fn render_image<'a, T: 'a + Clone>(images: &HashMap<UiWidgetId, Vec<u8>>, widget_id: UiWidgetId, image_data: &ImageLike, icon_style: Option<TextStyle>) -> Element<'a, T> {
    match image_data {
        ImageLike::ImageSource(_) => {
            match images.get(&widget_id) {
                Some(bytes) => {
                    image(Handle::from_bytes(bytes.clone()))
                        .into()
                }
                None => {
                    horizontal_space()
                        .into()
                }
            }
        }
        ImageLike::Icons(icon) => {
            match icon_style {
                None => {
                    value(icon_to_bootstrap(icon))
                        .font(BOOTSTRAP_FONT)
                        .into()
                }
                Some(icon_style) => {
                    value(icon_to_bootstrap(icon))
                        .font(BOOTSTRAP_FONT)
                        .themed(icon_style)
                }
            }
        }
    }
}

pub fn render_icon_accessory<'a, T: 'a + Clone>(images: &HashMap<UiWidgetId, Vec<u8>>, widget: &IconAccessoryWidget) -> Element<'a, T> {
    let icon = render_image(images, widget.__id__, &widget.icon, Some(TextStyle::IconAccessory));

    let content = container(icon)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .themed(ContainerStyle::IconAccessory);

    match widget.tooltip.as_ref() {
        None => content,
        Some(tooltip_text) => {
            let tooltip_text: Element<_> = text(tooltip_text.to_string())
                .shaping(Shaping::Advanced)
                .into();

            tooltip(content, tooltip_text, Position::Top)
                .themed(TooltipStyle::Tooltip)
        }
    }
}

pub fn render_text_accessory<'a, T: 'a + Clone>(images: &HashMap<UiWidgetId, Vec<u8>>, widget: &TextAccessoryWidget) -> Element<'a, T> {
    let icon: Option<Element<_>> = widget.icon
        .as_ref()
        .map(|icon| render_image(images, widget.__id__, icon, Some(TextStyle::TextAccessory)));

    let text_content: Element<_> = text(widget.text.to_string())
        .shaping(Shaping::Advanced)
        .themed(TextStyle::TextAccessory);

    let mut content: Vec<Element<_>> = vec![];

    if let Some(icon) = icon {
        let icon: Element<_> = container(icon)
            .themed(ContainerStyle::TextAccessoryIcon);

        content.push(icon)
    }

    content.push(text_content);

    let content: Element<_> = row(content)
        .align_y(Alignment::Center)
        .into();

    let content = container(content)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .themed(ContainerStyle::TextAccessory);

    match widget.tooltip.as_ref() {
        None => content,
        Some(tooltip_text) => {
            let tooltip_text: Element<_> = text(tooltip_text.to_string())
                .shaping(Shaping::Advanced)
                .into();

            tooltip(content, tooltip_text, Position::Top)
                .themed(TooltipStyle::Tooltip)
        }
    }
}


#[derive(Clone, Debug)]
pub enum ComponentWidgetEvent {
    LinkClick {
        widget_id: UiWidgetId,
        href: String
    },
    TagClick {
        widget_id: UiWidgetId,
    },
    ActionClick {
        widget_id: UiWidgetId,
    },
    RunAction {
        widget_id: UiWidgetId
    },
    ToggleDatePicker {
        widget_id: UiWidgetId,
    },
    OnChangeTextField {
        widget_id: UiWidgetId,
        value: String
    },
    OnChangePasswordField {
        widget_id: UiWidgetId,
        value: String
    },
    OnChangeSearchBar {
        widget_id: UiWidgetId,
        value: String
    },
    SubmitDatePicker {
        widget_id: UiWidgetId,
        value: String
    },
    CancelDatePicker {
        widget_id: UiWidgetId,
    },
    ToggleCheckbox {
        widget_id: UiWidgetId,
        value: bool
    },
    SelectPickList {
        widget_id: UiWidgetId,
        value: String
    },
    ToggleActionPanel {
        widget_id: UiWidgetId,
    },
    ListItemClick {
        widget_id: UiWidgetId,
    },
    GridItemClick {
        widget_id: UiWidgetId,
    },
    PreviousView,
    RunPrimaryAction {
        widget_id: UiWidgetId,
    },
    Noop,
}

include!(concat!(env!("OUT_DIR"), "/components.rs"));

impl ComponentWidgetEvent {
    pub fn handle(self, _plugin_id: PluginId, state: Option<&mut ComponentWidgetState>) -> Option<UiViewEvent> {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id: _, href } => {
                Some(UiViewEvent::Open {
                    href
                })
            }
            ComponentWidgetEvent::TagClick { widget_id } => {
                Some(create_metadata_tag_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::RunAction { widget_id } | ComponentWidgetEvent::ActionClick { widget_id } => {
                Some(create_action_on_action_event(widget_id))
            }
            ComponentWidgetEvent::ToggleDatePicker { widget_id } => {
                let state = state.expect("state should always exist for ");

                let ComponentWidgetState::DatePicker(DatePickerState { state_value: _, show_picker }) = state else {
                    panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                };

                *show_picker = !*show_picker;
                None
            }
            ComponentWidgetEvent::CancelDatePicker { widget_id } => {
                let state = state.expect("state should always exist for ");

                let ComponentWidgetState::DatePicker(DatePickerState { state_value: _, show_picker }) = state else {
                    panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                };

                *show_picker = false;
                None
            }
            ComponentWidgetEvent::SubmitDatePicker { widget_id, value } => {
                let state = state.expect("state should always exist for ");

                {
                    let ComponentWidgetState::DatePicker(DatePickerState { state_value: _, show_picker }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *show_picker = false;
                }

                Some(create_date_picker_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::ToggleCheckbox { widget_id, value } => {
                let state = state.expect("state should always exist for ");

                {
                    let ComponentWidgetState::Checkbox(CheckboxState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = !*state_value;
                }

                Some(create_checkbox_on_change_event(widget_id, value))
            }
            ComponentWidgetEvent::SelectPickList { widget_id, value } => {
                let state = state.expect("state should always exist for ");

                {
                    let ComponentWidgetState::Select(SelectState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = Some(value.clone());
                }

                Some(create_select_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeTextField { widget_id, value } => {
                let state = state.expect("state should always exist for ");

                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value, .. }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_text_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangePasswordField { widget_id, value } => {
                let state = state.expect("state should always exist for ");

                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value, .. }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_password_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeSearchBar { widget_id, value } => {
                let state = state.expect("state should always exist for ");

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
                    event: AppMsg::ToggleActionPanel { keyboard: false }
                })
            }
            ComponentWidgetEvent::ListItemClick { widget_id } => {
                Some(create_list_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::GridItemClick { widget_id } => {
                Some(create_grid_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::Noop | ComponentWidgetEvent::PreviousView => {
                panic!("widget_id on these events is not supposed to be called")
            }
            ComponentWidgetEvent::RunPrimaryAction { widget_id } => {
                Some(UiViewEvent::AppEvent {
                    event: AppMsg::OnAnyActionPluginViewAnyPanel { widget_id }
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
            ComponentWidgetEvent::ListItemClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::GridItemClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::RunPrimaryAction { widget_id } => widget_id,
            ComponentWidgetEvent::Noop | ComponentWidgetEvent::PreviousView => panic!("widget_id on these events is not supposed to be called"),
        }.to_owned()
    }
}

pub fn parse_date(value: &str) -> Option<(i32, u32, u32)> {
    let ymd: Vec<_> = value.split("-")
        .collect();

    match ymd[..] {
        [year, month, day] => {
            let year = year.parse::<i32>();
            let month = month.parse::<u32>();
            let day = day.parse::<u32>();

            match (year, month, day) {
                (Ok(year), Ok(month), Ok(day)) => Some((year, month, day)),
                _ => None
            }
        }
        _ => None
    }
}

fn icon_to_bootstrap(icon: &Icons) -> Bootstrap {
    match icon {
        Icons::Airplane => Bootstrap::Airplane,
        Icons::Alarm => Bootstrap::Alarm,
        Icons::AlignCentre => Bootstrap::AlignCenter,
        Icons::AlignLeft => Bootstrap::AlignStart,
        Icons::AlignRight => Bootstrap::AlignEnd,
        // Icons::Anchor => Bootstrap::,
        Icons::ArrowClockwise => Bootstrap::ArrowClockwise,
        Icons::ArrowCounterClockwise => Bootstrap::ArrowCounterclockwise,
        Icons::ArrowDown => Bootstrap::ArrowDown,
        Icons::ArrowLeft => Bootstrap::ArrowLeft,
        Icons::ArrowRight => Bootstrap::ArrowRight,
        Icons::ArrowUp => Bootstrap::ArrowUp,
        Icons::ArrowLeftRight => Bootstrap::ArrowLeftRight,
        Icons::ArrowsContract => Bootstrap::ArrowsAngleContract,
        Icons::ArrowsExpand => Bootstrap::ArrowsAngleExpand,
        Icons::AtSymbol => Bootstrap::At,
        // Icons::BandAid => Bootstrap::Bandaid,
        Icons::Cash => Bootstrap::Cash,
        // Icons::BarChart => Bootstrap::BarChart,
        // Icons::BarCode => Bootstrap::,
        Icons::Battery => Bootstrap::Battery,
        Icons::BatteryCharging => Bootstrap::BatteryCharging,
        // Icons::BatteryDisabled => Bootstrap::,
        Icons::Bell => Bootstrap::Bell,
        Icons::BellDisabled => Bootstrap::BellSlash,
        // Icons::Bike => Bootstrap::Bicycle,
        // Icons::Binoculars => Bootstrap::Binoculars,
        // Icons::Bird => Bootstrap::,
        Icons::Bluetooth => Bootstrap::Bluetooth,
        // Icons::Boat => Bootstrap::,
        Icons::Bold => Bootstrap::TypeBold,
        // Icons::Bolt => Bootstrap::,
        // Icons::BoltDisabled => Bootstrap::,
        Icons::Book => Bootstrap::Book,
        Icons::Bookmark => Bootstrap::Bookmark,
        Icons::Box => Bootstrap::Box,
        // Icons::Brush => Bootstrap::Brush,
        Icons::Bug => Bootstrap::Bug,
        Icons::Building => Bootstrap::Building,
        Icons::BulletPoints => Bootstrap::ListUl,
        Icons::Calculator => Bootstrap::Calculator,
        Icons::Calendar => Bootstrap::Calendar,
        Icons::Camera => Bootstrap::Camera,
        Icons::Car => Bootstrap::CarFront,
        Icons::Cart => Bootstrap::Cart,
        // Icons::Cd => Bootstrap::,
        // Icons::Center => Bootstrap::,
        Icons::Checkmark => Bootstrap::Checktwo,
        // Icons::ChessPiece => Bootstrap::,
        Icons::ChevronDown => Bootstrap::ChevronDown,
        Icons::ChevronLeft => Bootstrap::ChevronLeft,
        Icons::ChevronRight => Bootstrap::ChevronRight,
        Icons::ChevronUp => Bootstrap::ChevronUp,
        Icons::ChevronExpand => Bootstrap::ChevronExpand,
        Icons::Circle => Bootstrap::Circle,
        // Icons::CircleProgress100 => Bootstrap::,
        // Icons::CircleProgress25 => Bootstrap::,
        // Icons::CircleProgress50 => Bootstrap::,
        // Icons::CircleProgress75 => Bootstrap::,
        // Icons::ClearFormatting => Bootstrap::,
        Icons::Clipboard => Bootstrap::Clipboard,
        Icons::Clock => Bootstrap::Clock,
        Icons::Cloud => Bootstrap::Cloud,
        Icons::CloudLightning => Bootstrap::CloudLightning,
        Icons::CloudRain => Bootstrap::CloudRain,
        Icons::CloudSnow => Bootstrap::CloudSnow,
        Icons::CloudSun => Bootstrap::CloudSun,
        Icons::Code => Bootstrap::Code,
        Icons::Gear => Bootstrap::Gear,
        Icons::Coin => Bootstrap::Coin,
        Icons::Command => Bootstrap::Command,
        Icons::Compass => Bootstrap::Compass,
        // Icons::ComputerChip => Bootstrap::,
        // Icons::Contrast => Bootstrap::,
        Icons::CreditCard => Bootstrap::CreditCard,
        Icons::Crop => Bootstrap::Crop,
        // Icons::Crown => Bootstrap::,
        Icons::Document => Bootstrap::FileEarmark,
        Icons::DocumentAdd => Bootstrap::FileEarmarkPlus,
        Icons::DocumentDelete => Bootstrap::FileEarmarkX,
        Icons::Dot => Bootstrap::Dot,
        Icons::Download => Bootstrap::Download,
        // Icons::Duplicate => Bootstrap::,
        Icons::Eject => Bootstrap::Eject,
        Icons::ThreeDots => Bootstrap::ThreeDots,
        Icons::Envelope => Bootstrap::Envelope,
        Icons::Eraser => Bootstrap::Eraser,
        Icons::ExclamationMark => Bootstrap::ExclamationLg,
        Icons::Eye => Bootstrap::Eye,
        Icons::EyeDisabled => Bootstrap::EyeSlash,
        Icons::EyeDropper => Bootstrap::Eyedropper,
        Icons::Female => Bootstrap::GenderFemale,
        Icons::Film => Bootstrap::Film,
        Icons::Filter => Bootstrap::Filter,
        Icons::Fingerprint => Bootstrap::Fingerprint,
        Icons::Flag => Bootstrap::Flag,
        Icons::Folder => Bootstrap::Folder,
        Icons::FolderAdd => Bootstrap::FolderPlus,
        Icons::FolderDelete => Bootstrap::FolderMinus,
        Icons::Forward => Bootstrap::Forward,
        Icons::GameController => Bootstrap::Controller,
        Icons::Virus => Bootstrap::Virus,
        Icons::Gift => Bootstrap::Gift,
        Icons::Glasses => Bootstrap::Eyeglasses,
        Icons::Globe => Bootstrap::Globe,
        Icons::Hammer => Bootstrap::Hammer,
        Icons::HardDrive => Bootstrap::DeviceHdd,
        Icons::Headphones => Bootstrap::Headphones,
        Icons::Heart => Bootstrap::Heart,
        // Icons::HeartDisabled => Bootstrap::,
        Icons::Heartbeat => Bootstrap::Activity,
        Icons::Hourglass => Bootstrap::Hourglass,
        Icons::House => Bootstrap::House,
        Icons::Image => Bootstrap::Image,
        Icons::Info => Bootstrap::InfoLg,
        Icons::Italics => Bootstrap::TypeItalic,
        Icons::Key => Bootstrap::Key,
        Icons::Keyboard => Bootstrap::Keyboard,
        Icons::Layers => Bootstrap::Layers,
        // Icons::Leaf => Bootstrap::,
        Icons::LightBulb => Bootstrap::Lightbulb,
        Icons::LightBulbDisabled => Bootstrap::LightbulbOff,
        Icons::Link => Bootstrap::LinkFourfivedeg,
        Icons::List => Bootstrap::List,
        Icons::Lock => Bootstrap::Lock,
        // Icons::LockDisabled => Bootstrap::,
        Icons::LockUnlocked => Bootstrap::Unlock,
        // Icons::Logout => Bootstrap::,
        // Icons::Lowercase => Bootstrap::,
        // Icons::MagnifyingGlass => Bootstrap::,
        Icons::Male => Bootstrap::GenderMale,
        Icons::Map => Bootstrap::Map,
        Icons::Maximize => Bootstrap::Fullscreen,
        Icons::Megaphone => Bootstrap::Megaphone,
        Icons::MemoryModule => Bootstrap::Memory,
        Icons::MemoryStick => Bootstrap::UsbDrive,
        Icons::Message => Bootstrap::Chat,
        Icons::Microphone => Bootstrap::Mic,
        Icons::MicrophoneDisabled => Bootstrap::MicMute,
        Icons::Minimize => Bootstrap::FullscreenExit,
        Icons::Minus => Bootstrap::Dash,
        Icons::Mobile => Bootstrap::Phone,
        // Icons::Monitor => Bootstrap::,
        Icons::Moon => Bootstrap::Moon,
        // Icons::Mountain => Bootstrap::,
        Icons::Mouse => Bootstrap::Mouse,
        Icons::Multiply => Bootstrap::X,
        Icons::Music => Bootstrap::MusicNoteBeamed,
        Icons::Network => Bootstrap::BroadcastPin,
        Icons::Paperclip => Bootstrap::Paperclip,
        Icons::Paragraph => Bootstrap::TextParagraph,
        Icons::Pause => Bootstrap::Pause,
        Icons::Pencil => Bootstrap::Pencil,
        Icons::Person => Bootstrap::Person,
        Icons::PersonAdd => Bootstrap::PersonAdd,
        Icons::PersonRemove => Bootstrap::PersonDash,
        Icons::Phone => Bootstrap::Telephone,
        // Icons::PhoneRinging => Bootstrap::,
        Icons::PieChart => Bootstrap::PieChart,
        Icons::Capsule => Bootstrap::Capsule,
        // Icons::Pin => Bootstrap::,
        // Icons::PinDisabled => Bootstrap::,
        Icons::Play => Bootstrap::Play,
        Icons::Plug => Bootstrap::Plug,
        Icons::Plus => Bootstrap::Plus,
        // Icons::PlusMinusDivideMultiply => Bootstrap::,
        Icons::Power => Bootstrap::Power,
        Icons::Printer => Bootstrap::Printer,
        Icons::QuestionMark => Bootstrap::QuestionLg,
        Icons::Quotes => Bootstrap::Quote,
        Icons::Receipt => Bootstrap::Receipt,
        Icons::Repeat => Bootstrap::Repeat,
        Icons::Reply => Bootstrap::Reply,
        Icons::Rewind => Bootstrap::Rewind,
        Icons::Rocket => Bootstrap::Rocket,
        // Icons::Ruler => Bootstrap::,
        Icons::Shield => Bootstrap::Shield,
        Icons::Shuffle => Bootstrap::Shuffle,
        Icons::Snippets => Bootstrap::BodyText,
        Icons::Snowflake => Bootstrap::Snow,
        // Icons::VolumeHigh => Bootstrap::VolumeUp,
        // Icons::VolumeLow => Bootstrap::VolumeDown,
        // Icons::VolumeOff => Bootstrap::VolumeOff,
        // Icons::VolumeOn => Bootstrap::,
        Icons::Star => Bootstrap::Star,
        // Icons::StarDisabled => Bootstrap::,
        Icons::Stop => Bootstrap::Stop,
        Icons::Stopwatch => Bootstrap::Stopwatch,
        Icons::StrikeThrough => Bootstrap::TypeStrikethrough,
        Icons::Sun => Bootstrap::SunFill, // TODO why Sun isn't in iced_aw?
        Icons::Scissors => Bootstrap::Scissors,
        // Icons::Syringe => Bootstrap::,
        Icons::Tag => Bootstrap::Tag,
        Icons::Thermometer => Bootstrap::Thermometer,
        Icons::Terminal => Bootstrap::Terminal,
        Icons::Text => Bootstrap::Fonts,
        Icons::TextCursor => Bootstrap::CursorText,
        // Icons::TextSelection => Bootstrap::,
        // Icons::Torch => Bootstrap::,
        // Icons::Train => Bootstrap::,
        Icons::Trash => Bootstrap::Trash,
        Icons::Tree => Bootstrap::Tree,
        Icons::Trophy => Bootstrap::Trophy,
        Icons::People => Bootstrap::People,
        Icons::Umbrella => Bootstrap::Umbrella,
        Icons::Underline => Bootstrap::TypeUnderline,
        Icons::Upload => Bootstrap::Upload,
        // Icons::Uppercase => Bootstrap::,
        Icons::Wallet => Bootstrap::Wallet,
        Icons::Wand => Bootstrap::Magic,
        // Icons::Warning => Bootstrap::,
        // Icons::Weights => Bootstrap::,
        Icons::Wifi => Bootstrap::Wifi,
        Icons::WifiDisabled => Bootstrap::WifiOff,
        Icons::Window => Bootstrap::Window,
        Icons::Tools => Bootstrap::Tools,
        Icons::Watch => Bootstrap::Watch,
        Icons::XMark => Bootstrap::XLg,
        Icons::Indent => Bootstrap::Indent,
        Icons::Unindent => Bootstrap::Unindent,
    }
}
