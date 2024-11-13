use common::model::{ActionPanelSectionWidget, ActionPanelSectionWidgetOrderedMembers, ActionPanelWidget, ActionPanelWidgetOrderedMembers, ActionWidget, CheckboxWidget, CodeBlockWidget, ContentWidget, ContentWidgetOrderedMembers, DatePickerWidget, DetailWidget, EmptyViewWidget, FormWidget, FormWidgetOrderedMembers, GridItemWidget, GridSectionWidget, GridSectionWidgetOrderedMembers, GridWidget, GridWidgetOrderedMembers, H1Widget, H2Widget, H3Widget, H4Widget, H5Widget, H6Widget, HorizontalBreakWidget, IconAccessoryWidget, Icons, Image, ImageWidget, InlineSeparatorWidget, InlineWidget, InlineWidgetOrderedMembers, ListItemAccessories, ListItemWidget, ListSectionWidget, ListSectionWidgetOrderedMembers, ListWidget, ListWidgetOrderedMembers, MetadataIconWidget, MetadataLinkWidget, MetadataSeparatorWidget, MetadataTagItemWidget, MetadataTagListWidget, MetadataTagListWidgetOrderedMembers, MetadataValueWidget, MetadataWidget, MetadataWidgetOrderedMembers, ParagraphWidget, PasswordFieldWidget, PhysicalKey, PhysicalShortcut, PluginId, RootWidget, RootWidgetMembers, SearchBarWidget, SelectWidget, SelectWidgetOrderedMembers, SeparatorWidget, TextAccessoryWidget, TextFieldWidget, UiWidgetId};
use common_ui::shortcut_to_text;
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::futures::StreamExt;
use iced::widget::image::Handle;
use iced::widget::text::Shaping;
use iced::widget::tooltip::Position;
use iced::widget::{button, checkbox, column, container, horizontal_rule, horizontal_space, image, mouse_area, pick_list, row, scrollable, text, text_input, tooltip, vertical_rule, Space};
use iced::{Alignment, Command, Font, Length};
use iced_aw::core::icons;
use iced_aw::date_picker::Date;
use iced_aw::floating_element::Offset;
use iced_aw::helpers::{date_picker, grid, grid_row, wrap_horizontal};
use iced_aw::{floating_element, GridRow};
use itertools::Itertools;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

use crate::model::UiViewEvent;
use crate::ui::custom_widgets::loading_bar::LoadingBar;
use crate::ui::grid_navigation::{grid_down_offset, grid_up_offset, GridSectionData};
use crate::ui::scroll_handle::{ScrollHandle, ESTIMATED_GRID_ITEM_HEIGHT, ESTIMATED_MAIN_LIST_ITEM_HEIGHT};
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


#[derive(Debug)]
pub struct ComponentWidgets<'b> {
    root_widget: &'b mut Option<RootWidget>,
    state: &'b mut HashMap<UiWidgetId, ComponentWidgetState>,
    images: &'b HashMap<UiWidgetId, bytes::Bytes>
}

impl<'b> ComponentWidgets<'b> {
    pub fn new(
        root_widget: &'b mut Option<RootWidget>,
        state: &'b mut HashMap<UiWidgetId, ComponentWidgetState>,
        images: &'b HashMap<UiWidgetId, bytes::Bytes>
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
                    result.insert(widget.__id__, ComponentWidgetState::root(0.0));
                }
                RootWidgetMembers::Form(widget) => {
                    result.insert(widget.__id__, ComponentWidgetState::root(0.0));

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
                    result.insert(widget.__id__, ComponentWidgetState::root(ESTIMATED_MAIN_LIST_ITEM_HEIGHT));

                    if let Some(widget) = &widget.content.search_bar {
                        result.insert(widget.__id__, ComponentWidgetState::text_field(&widget.value));
                    }
                }
                RootWidgetMembers::Grid(widget) => {
                    result.insert(widget.__id__, ComponentWidgetState::root(ESTIMATED_GRID_ITEM_HEIGHT));

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
    fn root(item_height: f32) -> ComponentWidgetState {
        ComponentWidgetState::Root(RootState {
            show_action_panel: false,
            focused_item: ScrollHandle::new(false, item_height), // TODO first focused?
        })
    }

    fn text_field(value: &Option<String>) -> ComponentWidgetState {
        ComponentWidgetState::TextField(TextFieldState {
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

    pub fn focus_up(&mut self) -> Command<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Command::none();
        };

        let Some(content) = &root_widget.content else {
            return Command::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Command::none(),
            RootWidgetMembers::Form(_) => Command::none(),
            RootWidgetMembers::Inline(_) => Command::none(),
            RootWidgetMembers::List(widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, widget.__id__);

                focused_item.focus_previous()
                    .unwrap_or_else(|| Command::none())
            }
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, grid_widget.__id__);

                let Some(current_index) = &focused_item.index else {
                    return Command::none();
                };

                let amount_per_section_total = Self::grid_section_sizes(grid_widget);

                match grid_up_offset(*current_index, amount_per_section_total) {
                    None => Command::none(),
                    Some(data) => {
                        let _ = focused_item.focus_previous_in(data.offset);

                        focused_item.scroll_to_offset(data.row_index, false)
                    }
                }
            }
        }
    }

    pub fn focus_down(&mut self) -> Command<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Command::none();
        };

        let Some(content) = &root_widget.content else {
            return Command::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Command::none(),
            RootWidgetMembers::Form(_) => Command::none(),
            RootWidgetMembers::Inline(_) => Command::none(),
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
                    .unwrap_or_else(|| Command::none())
            }
            RootWidgetMembers::Grid(grid_widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, grid_widget.__id__);

                let amount_per_section_total = Self::grid_section_sizes(grid_widget);

                let total = amount_per_section_total
                    .iter()
                    .map(|data| data.amount_in_section)
                    .sum();

                let Some(current_index) = &focused_item.index else {
                    let _ = focused_item.focus_next(total);

                    return focused_item.scroll_to_offset(0, false)
                };

                match grid_down_offset(*current_index, amount_per_section_total) {
                    None => Command::none(),
                    Some(data) => {
                        let _ = focused_item.focus_next_in(total, data.offset);

                        focused_item.scroll_to_offset(data.row_index, false)
                    }
                }
            }
        }
    }

    pub fn focus_left(&mut self) -> Command<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Command::none();
        };

        let Some(content) = &root_widget.content else {
            return Command::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Command::none(),
            RootWidgetMembers::Form(_) => Command::none(),
            RootWidgetMembers::Inline(_) => Command::none(),
            RootWidgetMembers::List(_) => Command::none(),
            RootWidgetMembers::Grid(widget) => {
                let RootState { focused_item, .. } = ComponentWidgets::root_state_mut_on_field(self.state, widget.__id__);

                let _ = focused_item.focus_previous();

                // focused_item.scroll_to(0)
                // TODO
                Command::none()
            }
        }
    }

    pub fn focus_right(&mut self) -> Command<AppMsg> {
        let Some(root_widget) = &self.root_widget else {
            return Command::none();
        };

        let Some(content) = &root_widget.content else {
            return Command::none();
        };

        match content {
            RootWidgetMembers::Detail(_) => Command::none(),
            RootWidgetMembers::Form(_) => Command::none(),
            RootWidgetMembers::Inline(_) => Command::none(),
            RootWidgetMembers::List(_) => Command::none(),
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
                Command::none()
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
        let content = widget.content.ordered_members
            .iter()
            .map(|members| {
                match members {
                    MetadataTagListWidgetOrderedMembers::MetadataTagItem(content) => self.render_metadata_tag_item_widget(&content)
                }
            })
            .collect();

        let value = wrap_horizontal(content)
            .into();

        render_metadata_item(&widget.label, value)
            .into()
    }

    fn render_metadata_link_widget<'a>(&self, widget: &MetadataLinkWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let icon: Element<_> = text(icons::Bootstrap::BoxArrowUpRight)
            .font(icons::BOOTSTRAP_FONT)
            .size(16)
            .into();

        let icon = container(icon)
            .themed(ContainerStyle::MetadataLinkIcon);

        let content: Element<_> = row([content, icon])
            .align_items(Alignment::Center)
            .into();

        let link: Element<_> = button(content)
            .on_press(ComponentWidgetEvent::LinkClick { widget_id: widget.__id__, href: widget.href.to_owned() })
            .themed(ButtonStyle::MetadataLink);

        let content: Element<_> = if widget.href.is_empty() {
            link
        } else {
            let href: Element<_> = text(&widget.href)
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
        let value = text(icon_to_bootstrap(&widget.icon))
            .font(icons::BOOTSTRAP_FONT)
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
            content = content.center_x()
        }

        content.themed(ContainerStyle::ContentParagraph)
    }

    fn render_image_widget<'a>(&self, widget: &ImageWidget, centered: bool) -> Element<'a, ComponentWidgetEvent> {
        // TODO image size, height and width
        let content: Element<_> = self.render_image(widget.__id__, &widget.source, None);

        let mut content = container(content)
            .width(Length::Fill);

        if centered {
            content = content.center_x()
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
                .center_x()
                .center_y()
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
        let TextFieldState { state_value } = self.text_field_state(widget.__id__);

        text_input("", state_value)
            .on_input(move |value| ComponentWidgetEvent::OnChangeTextField { widget_id, value })
            .themed(TextInputStyle::FormInput)
    }

    fn render_password_field_widget<'a>(&self, widget: &PasswordFieldWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value } = self.text_field_state(widget_id);

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
                            let label: Element<_> = text(label)
                                .shaping(Shaping::Advanced)
                                .horizontal_alignment(Horizontal::Right)
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
                        .align_items(Alignment::Center)
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
                    .center_x()
                    .into();

                let icon = text(icon_to_bootstrap(icon))
                    .font(icons::BOOTSTRAP_FONT)
                    .size(45)
                    .themed(TextStyle::InlineSeparator);

                let bot_rule: Element<_> = vertical_rule(1)
                    .into();

                let bot_rule = container(bot_rule)
                    .center_x()
                    .into();

                column([top_rule, icon, bot_rule])
                    .align_items(Alignment::Center)
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
                    InlineWidgetOrderedMembers::Content(widget) => self.render_content_widget(widget, true),
                    InlineWidgetOrderedMembers::InlineSeparator(widget) => {
                        let element = self.render_inline_separator_widget(widget);

                        container(element)
                            .width(Length::Fill)
                            .into()
                    }
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
            .map(|image| self.render_image(widget.__id__, image, Some(TextStyle::EmptyViewSubtitle)));

        let title: Element<_> = text(&widget.title)
            .shaping(Shaping::Advanced)
            .into();

        let subtitle: Element<_> = match &widget.description {
            None => horizontal_space().into(),
            Some(subtitle) => {
                text(subtitle)
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
            .align_items(Alignment::Center)
            .into();

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }


    fn render_search_bar_widget<'a>(&self, widget: &SearchBarWidget) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = widget.__id__;
        let TextFieldState { state_value } = self.text_field_state(widget_id);

        text_input(widget.placeholder.as_deref().unwrap_or_default(), state_value)
            .on_input(move |value| ComponentWidgetEvent::OnChangeSearchBar { widget_id, value })
            .themed(TextInputStyle::PluginSearchBar)
    }

    fn render_icon_accessory<'a>(&self, widget: &IconAccessoryWidget) -> Element<'a, ComponentWidgetEvent> {
        let icon = self.render_image(widget.__id__, &widget.icon, Some(TextStyle::IconAccessory));

        let content = container(icon)
            .center_x()
            .center_y()
            .themed(ContainerStyle::IconAccessory);

        match widget.tooltip.as_ref() {
            None => content,
            Some(tooltip_text) => {
                let tooltip_text: Element<_> = text(tooltip_text)
                    .shaping(Shaping::Advanced)
                    .into();

                tooltip(content, tooltip_text, Position::Top)
                    .themed(TooltipStyle::Tooltip)
            }
        }
    }

    fn render_text_accessory<'a>(&self, widget: &TextAccessoryWidget) -> Element<'a, ComponentWidgetEvent> {
        let icon: Option<Element<_>> = widget.icon
            .as_ref()
            .map(|icon| self.render_image(widget.__id__, icon, Some(TextStyle::TextAccessory)));

        let text_content: Element<_> = text(&widget.text)
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
            .align_items(Alignment::Center)
            .into();

        let content = container(content)
            .center_x()
            .center_y()
            .themed(ContainerStyle::TextAccessory);

        match widget.tooltip.as_ref() {
            None => content,
            Some(tooltip_text) => {
                let tooltip_text: Element<_> = text(tooltip_text)
                    .shaping(Shaping::Advanced)
                    .into();

                tooltip(content, tooltip_text, Position::Top)
                    .themed(TooltipStyle::Tooltip)
            }
        }
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

        for members in &list_widget.content.ordered_members {
            match &members {
                ListWidgetOrderedMembers::ListItem(widget) => {
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

                    items.push(self.render_list_section_widget(widget, focused_item.index, index_counter))
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
        index_counter: &Cell<usize>
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

        render_section(content, Some(&widget.title), &widget.subtitle, RowStyle::ListSectionTitle, TextStyle::ListSectionTitle, TextStyle::ListSectionSubtitle)
    }

    fn render_list_item_widget<'a>(
        &self,
        widget: &ListItemWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>
    ) -> Element<'a, ComponentWidgetEvent> {
        let icon: Option<Element<_>> = widget.icon
            .as_ref()
            .map(|icon| self.render_image(widget.__id__, icon, None));

        let title: Element<_> = text(&widget.title)
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
            let subtitle: Element<_> = text(subtitle)
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
                        ListItemAccessories::_0(widget) => self.render_text_accessory(widget),
                        ListItemAccessories::_1(widget) => self.render_icon_accessory(widget)
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
            .align_items(Alignment::Center)
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

        for members in &grid_widget.content.ordered_members {
            match &members {
                GridWidgetOrderedMembers::GridItem(widget) => {
                    pending.push(widget)
                }
                GridWidgetOrderedMembers::GridSection(widget) => {
                    if !pending.is_empty() {
                        let content = self.render_grid(&pending, &grid_widget.columns, focused_item.index, index_counter);

                        items.push(content);

                        pending = vec![];
                    }

                    items.push(self.render_grid_section_widget(widget, focused_item.index, index_counter))
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
        index_counter: &Cell<usize>
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

        render_section(content, Some(&widget.title), &widget.subtitle, RowStyle::GridSectionTitle, TextStyle::GridSectionTitle, TextStyle::GridSectionSubtitle)
    }

    fn render_grid_item_widget<'a>(
        &self,
        widget: &GridItemWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>
    ) -> Element<'a, ComponentWidgetEvent> {
        // TODO not needed column element?
        let content: Element<_> = column(vec![self.render_content_widget(&widget.content.content, true)])
            .height(130) // TODO dynamic height
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
            let title = text(title)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::GridItemTitle);

            sub_content_left.push(title);
        }

        if let Some(subtitle) = &widget.subtitle {
            let subtitle = text(subtitle)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::GridItemSubTitle);

            sub_content_left.push(subtitle);
        }

        let mut sub_content_right = vec![];
        if let Some(widget) = &widget.content.accessory {
            sub_content_right.push(self.render_icon_accessory(widget));
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
            .map(|widget| self.render_grid_item_widget(widget, item_focus_index, index_counter))
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
        let icon = text(icons::Bootstrap::ArrowLeft)
            .font(icons::BOOTSTRAP_FONT);

        let back_button: Element<_> = button(icon)
            .on_press(ComponentWidgetEvent::PreviousView)
            .themed(ButtonStyle::RootTopPanelBackButton);

        let search_bar_element = search_bar
            .as_ref()
            .map(|widget| self.render_search_bar_widget(widget))
            .unwrap_or_else(|| Space::with_width(Length::FillPortion(3)).into());

        let top_panel: Element<_> = row(vec![back_button, search_bar_element])
            .align_items(Alignment::Center)
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

    fn render_image<'a>(&self, widget_id: UiWidgetId, image_data: &Image, icon_style: Option<TextStyle>) -> Element<'a, ComponentWidgetEvent> {
        match image_data {
            Image::ImageSource(_) => {
                match self.images.get(&widget_id) {
                    Some(bytes) => {
                        image(Handle::from_memory(bytes.clone()))
                            .into()
                    }
                    None => {
                        horizontal_space()
                            .into()
                    }
                }
            }
            Image::Icons(icon) => {
                match icon_style {
                    None => {
                        text(icon_to_bootstrap(icon))
                            .font(icons::BOOTSTRAP_FONT)
                            .into()
                    }
                    Some(icon_style) => {
                        text(icon_to_bootstrap(icon))
                            .font(icons::BOOTSTRAP_FONT)
                            .themed(icon_style)
                    }
                }
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
    let label: Element<_> = text(label)
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
        let title: Element<_> = text(title)
            .shaping(Shaping::Advanced)
            .size(15)
            .themed(theme_kind_title_text);

        title_content.push(title)
    }

    if let Some(subtitle) = subtitle {
        let subtitle: Element<_> = text(subtitle)
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
                        .align_items(Alignment::Center)
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
    let entrypoint_name: Element<_> = text(entrypoint_name)
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

            let space = horizontal_space()
                .into();

            bottom_panel_content.push(space);

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);

                let rule: Element<_> = vertical_rule(1)
                    .style(RuleStyle::PrimaryActionSeparator)
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
                .align_items(Alignment::Center)
                .themed(RowStyle::RootBottomPanel);

            (!show_action_panel, Some(action_panel), bottom_panel)
        }
        None => {
            let space: Element<_> = Space::new(Length::Fill, panel_height)
                .into();

            let mut bottom_panel_content = vec![entrypoint_name, space];

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);
            }

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_items(Alignment::Center)
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

    let action_panel_element = match (action_panel, action_panel_scroll_handle) {
        (Some(action_panel), Some(action_panel_scroll_handle)) => render_action_panel(action_panel, on_action_click, action_panel_scroll_handle),
        _ => Space::with_height(1).into(),
    };

    floating_element(content, action_panel_element)
        .offset(Offset::from([8.0, 48.0])) // TODO calculate based on theme
        .hide(hide_action_panel)
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
    pub fn handle(self, _plugin_id: PluginId, state: &mut ComponentWidgetState) -> Option<UiViewEvent> {
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
                let ComponentWidgetState::DatePicker(DatePickerState { state_value: _, show_picker }) = state else {
                    panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                };

                *show_picker = !*show_picker;
                None
            }
            ComponentWidgetEvent::CancelDatePicker { widget_id } => {
                let ComponentWidgetState::DatePicker(DatePickerState { state_value: _, show_picker }) = state else {
                    panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                };

                *show_picker = false;
                None
            }
            ComponentWidgetEvent::SubmitDatePicker { widget_id, value } => {
                {
                    let ComponentWidgetState::DatePicker(DatePickerState { state_value: _, show_picker }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *show_picker = false;
                }

                Some(create_date_picker_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::ToggleCheckbox { widget_id, value } => {
                {
                    let ComponentWidgetState::Checkbox(CheckboxState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = !*state_value;
                }

                Some(create_checkbox_on_change_event(widget_id, value))
            }
            ComponentWidgetEvent::SelectPickList { widget_id, value } => {
                {
                    let ComponentWidgetState::Select(SelectState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = Some(value.clone());
                }

                Some(create_select_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeTextField { widget_id, value } => {
                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_text_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangePasswordField { widget_id, value } => {
                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value }) = state else {
                        panic!("unexpected state kind, widget_id: {:?} state: {:?}", widget_id, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_password_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeSearchBar { widget_id, value } => {
                {
                    let ComponentWidgetState::TextField(TextFieldState { state_value }) = state else {
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

fn icon_to_bootstrap(icon: &Icons) -> icons::Bootstrap {
    match icon {
        Icons::Airplane => icons::Bootstrap::Airplane,
        Icons::Alarm => icons::Bootstrap::Alarm,
        Icons::AlignCentre => icons::Bootstrap::AlignCenter,
        Icons::AlignLeft => icons::Bootstrap::AlignStart,
        Icons::AlignRight => icons::Bootstrap::AlignEnd,
        // Icons::Anchor => icons::Bootstrap::,
        Icons::ArrowClockwise => icons::Bootstrap::ArrowClockwise,
        Icons::ArrowCounterClockwise => icons::Bootstrap::ArrowCounterclockwise,
        Icons::ArrowDown => icons::Bootstrap::ArrowDown,
        Icons::ArrowLeft => icons::Bootstrap::ArrowLeft,
        Icons::ArrowRight => icons::Bootstrap::ArrowRight,
        Icons::ArrowUp => icons::Bootstrap::ArrowUp,
        Icons::ArrowLeftRight => icons::Bootstrap::ArrowLeftRight,
        Icons::ArrowsContract => icons::Bootstrap::ArrowsAngleContract,
        Icons::ArrowsExpand => icons::Bootstrap::ArrowsAngleExpand,
        Icons::AtSymbol => icons::Bootstrap::At,
        // Icons::BandAid => icons::Bootstrap::Bandaid,
        Icons::Cash => icons::Bootstrap::Cash,
        // Icons::BarChart => icons::Bootstrap::BarChart,
        // Icons::BarCode => icons::Bootstrap::,
        Icons::Battery => icons::Bootstrap::Battery,
        Icons::BatteryCharging => icons::Bootstrap::BatteryCharging,
        // Icons::BatteryDisabled => icons::Bootstrap::,
        Icons::Bell => icons::Bootstrap::Bell,
        Icons::BellDisabled => icons::Bootstrap::BellSlash,
        // Icons::Bike => icons::Bootstrap::Bicycle,
        // Icons::Binoculars => icons::Bootstrap::Binoculars,
        // Icons::Bird => icons::Bootstrap::,
        Icons::Bluetooth => icons::Bootstrap::Bluetooth,
        // Icons::Boat => icons::Bootstrap::,
        Icons::Bold => icons::Bootstrap::TypeBold,
        // Icons::Bolt => icons::Bootstrap::,
        // Icons::BoltDisabled => icons::Bootstrap::,
        Icons::Book => icons::Bootstrap::Book,
        Icons::Bookmark => icons::Bootstrap::Bookmark,
        Icons::Box => icons::Bootstrap::Box,
        // Icons::Brush => icons::Bootstrap::Brush,
        Icons::Bug => icons::Bootstrap::Bug,
        Icons::Building => icons::Bootstrap::Building,
        Icons::BulletPoints => icons::Bootstrap::ListUl,
        Icons::Calculator => icons::Bootstrap::Calculator,
        Icons::Calendar => icons::Bootstrap::Calendar,
        Icons::Camera => icons::Bootstrap::Camera,
        Icons::Car => icons::Bootstrap::CarFront,
        Icons::Cart => icons::Bootstrap::Cart,
        // Icons::Cd => icons::Bootstrap::,
        // Icons::Center => icons::Bootstrap::,
        Icons::Checkmark => icons::Bootstrap::Checktwo,
        // Icons::ChessPiece => icons::Bootstrap::,
        Icons::ChevronDown => icons::Bootstrap::ChevronDown,
        Icons::ChevronLeft => icons::Bootstrap::ChevronLeft,
        Icons::ChevronRight => icons::Bootstrap::ChevronRight,
        Icons::ChevronUp => icons::Bootstrap::ChevronUp,
        Icons::ChevronExpand => icons::Bootstrap::ChevronExpand,
        Icons::Circle => icons::Bootstrap::Circle,
        // Icons::CircleProgress100 => icons::Bootstrap::,
        // Icons::CircleProgress25 => icons::Bootstrap::,
        // Icons::CircleProgress50 => icons::Bootstrap::,
        // Icons::CircleProgress75 => icons::Bootstrap::,
        // Icons::ClearFormatting => icons::Bootstrap::,
        Icons::Clipboard => icons::Bootstrap::Clipboard,
        Icons::Clock => icons::Bootstrap::Clock,
        Icons::Cloud => icons::Bootstrap::Cloud,
        Icons::CloudLightning => icons::Bootstrap::CloudLightning,
        Icons::CloudRain => icons::Bootstrap::CloudRain,
        Icons::CloudSnow => icons::Bootstrap::CloudSnow,
        Icons::CloudSun => icons::Bootstrap::CloudSun,
        Icons::Code => icons::Bootstrap::Code,
        Icons::Gear => icons::Bootstrap::Gear,
        Icons::Coin => icons::Bootstrap::Coin,
        Icons::Command => icons::Bootstrap::Command,
        Icons::Compass => icons::Bootstrap::Compass,
        // Icons::ComputerChip => icons::Bootstrap::,
        // Icons::Contrast => icons::Bootstrap::,
        Icons::CreditCard => icons::Bootstrap::CreditCard,
        Icons::Crop => icons::Bootstrap::Crop,
        // Icons::Crown => icons::Bootstrap::,
        Icons::Document => icons::Bootstrap::FileEarmark,
        Icons::DocumentAdd => icons::Bootstrap::FileEarmarkPlus,
        Icons::DocumentDelete => icons::Bootstrap::FileEarmarkX,
        Icons::Dot => icons::Bootstrap::Dot,
        Icons::Download => icons::Bootstrap::Download,
        // Icons::Duplicate => icons::Bootstrap::,
        Icons::Eject => icons::Bootstrap::Eject,
        Icons::ThreeDots => icons::Bootstrap::ThreeDots,
        Icons::Envelope => icons::Bootstrap::Envelope,
        Icons::Eraser => icons::Bootstrap::Eraser,
        Icons::ExclamationMark => icons::Bootstrap::ExclamationLg,
        Icons::Eye => icons::Bootstrap::Eye,
        Icons::EyeDisabled => icons::Bootstrap::EyeSlash,
        Icons::EyeDropper => icons::Bootstrap::Eyedropper,
        Icons::Female => icons::Bootstrap::GenderFemale,
        Icons::Film => icons::Bootstrap::Film,
        Icons::Filter => icons::Bootstrap::Filter,
        Icons::Fingerprint => icons::Bootstrap::Fingerprint,
        Icons::Flag => icons::Bootstrap::Flag,
        Icons::Folder => icons::Bootstrap::Folder,
        Icons::FolderAdd => icons::Bootstrap::FolderPlus,
        Icons::FolderDelete => icons::Bootstrap::FolderMinus,
        Icons::Forward => icons::Bootstrap::Forward,
        Icons::GameController => icons::Bootstrap::Controller,
        Icons::Virus => icons::Bootstrap::Virus,
        Icons::Gift => icons::Bootstrap::Gift,
        Icons::Glasses => icons::Bootstrap::Eyeglasses,
        Icons::Globe => icons::Bootstrap::Globe,
        Icons::Hammer => icons::Bootstrap::Hammer,
        Icons::HardDrive => icons::Bootstrap::DeviceHdd,
        Icons::Headphones => icons::Bootstrap::Headphones,
        Icons::Heart => icons::Bootstrap::Heart,
        // Icons::HeartDisabled => icons::Bootstrap::,
        Icons::Heartbeat => icons::Bootstrap::Activity,
        Icons::Hourglass => icons::Bootstrap::Hourglass,
        Icons::House => icons::Bootstrap::House,
        Icons::Image => icons::Bootstrap::Image,
        Icons::Info => icons::Bootstrap::InfoLg,
        Icons::Italics => icons::Bootstrap::TypeItalic,
        Icons::Key => icons::Bootstrap::Key,
        Icons::Keyboard => icons::Bootstrap::Keyboard,
        Icons::Layers => icons::Bootstrap::Layers,
        // Icons::Leaf => icons::Bootstrap::,
        Icons::LightBulb => icons::Bootstrap::Lightbulb,
        Icons::LightBulbDisabled => icons::Bootstrap::LightbulbOff,
        Icons::Link => icons::Bootstrap::LinkFourfivedeg,
        Icons::List => icons::Bootstrap::List,
        Icons::Lock => icons::Bootstrap::Lock,
        // Icons::LockDisabled => icons::Bootstrap::,
        Icons::LockUnlocked => icons::Bootstrap::Unlock,
        // Icons::Logout => icons::Bootstrap::,
        // Icons::Lowercase => icons::Bootstrap::,
        // Icons::MagnifyingGlass => icons::Bootstrap::,
        Icons::Male => icons::Bootstrap::GenderMale,
        Icons::Map => icons::Bootstrap::Map,
        Icons::Maximize => icons::Bootstrap::Fullscreen,
        Icons::Megaphone => icons::Bootstrap::Megaphone,
        Icons::MemoryModule => icons::Bootstrap::Memory,
        Icons::MemoryStick => icons::Bootstrap::UsbDrive,
        Icons::Message => icons::Bootstrap::Chat,
        Icons::Microphone => icons::Bootstrap::Mic,
        Icons::MicrophoneDisabled => icons::Bootstrap::MicMute,
        Icons::Minimize => icons::Bootstrap::FullscreenExit,
        Icons::Minus => icons::Bootstrap::Dash,
        Icons::Mobile => icons::Bootstrap::Phone,
        // Icons::Monitor => icons::Bootstrap::,
        Icons::Moon => icons::Bootstrap::Moon,
        // Icons::Mountain => icons::Bootstrap::,
        Icons::Mouse => icons::Bootstrap::Mouse,
        Icons::Multiply => icons::Bootstrap::X,
        Icons::Music => icons::Bootstrap::MusicNoteBeamed,
        Icons::Network => icons::Bootstrap::BroadcastPin,
        Icons::Paperclip => icons::Bootstrap::Paperclip,
        Icons::Paragraph => icons::Bootstrap::TextParagraph,
        Icons::Pause => icons::Bootstrap::Pause,
        Icons::Pencil => icons::Bootstrap::Pencil,
        Icons::Person => icons::Bootstrap::Person,
        Icons::PersonAdd => icons::Bootstrap::PersonAdd,
        Icons::PersonRemove => icons::Bootstrap::PersonDash,
        Icons::Phone => icons::Bootstrap::Telephone,
        // Icons::PhoneRinging => icons::Bootstrap::,
        Icons::PieChart => icons::Bootstrap::PieChart,
        Icons::Capsule => icons::Bootstrap::Capsule,
        // Icons::Pin => icons::Bootstrap::,
        // Icons::PinDisabled => icons::Bootstrap::,
        Icons::Play => icons::Bootstrap::Play,
        Icons::Plug => icons::Bootstrap::Plug,
        Icons::Plus => icons::Bootstrap::Plus,
        // Icons::PlusMinusDivideMultiply => icons::Bootstrap::,
        Icons::Power => icons::Bootstrap::Power,
        Icons::Printer => icons::Bootstrap::Printer,
        Icons::QuestionMark => icons::Bootstrap::QuestionLg,
        Icons::Quotes => icons::Bootstrap::Quote,
        Icons::Receipt => icons::Bootstrap::Receipt,
        Icons::Repeat => icons::Bootstrap::Repeat,
        Icons::Reply => icons::Bootstrap::Reply,
        Icons::Rewind => icons::Bootstrap::Rewind,
        Icons::Rocket => icons::Bootstrap::Rocket,
        // Icons::Ruler => icons::Bootstrap::,
        Icons::Shield => icons::Bootstrap::Shield,
        Icons::Shuffle => icons::Bootstrap::Shuffle,
        Icons::Snippets => icons::Bootstrap::BodyText,
        Icons::Snowflake => icons::Bootstrap::Snow,
        // Icons::VolumeHigh => icons::Bootstrap::VolumeUp,
        // Icons::VolumeLow => icons::Bootstrap::VolumeDown,
        // Icons::VolumeOff => icons::Bootstrap::VolumeOff,
        // Icons::VolumeOn => icons::Bootstrap::,
        Icons::Star => icons::Bootstrap::Star,
        // Icons::StarDisabled => icons::Bootstrap::,
        Icons::Stop => icons::Bootstrap::Stop,
        Icons::Stopwatch => icons::Bootstrap::Stopwatch,
        Icons::StrikeThrough => icons::Bootstrap::TypeStrikethrough,
        Icons::Sun => icons::Bootstrap::SunFill, // TODO why Sun isn't in iced_aw?
        Icons::Scissors => icons::Bootstrap::Scissors,
        // Icons::Syringe => icons::Bootstrap::,
        Icons::Tag => icons::Bootstrap::Tag,
        Icons::Thermometer => icons::Bootstrap::Thermometer,
        Icons::Terminal => icons::Bootstrap::Terminal,
        Icons::Text => icons::Bootstrap::Fonts,
        Icons::TextCursor => icons::Bootstrap::CursorText,
        // Icons::TextSelection => icons::Bootstrap::,
        // Icons::Torch => icons::Bootstrap::,
        // Icons::Train => icons::Bootstrap::,
        Icons::Trash => icons::Bootstrap::Trash,
        Icons::Tree => icons::Bootstrap::Tree,
        Icons::Trophy => icons::Bootstrap::Trophy,
        Icons::People => icons::Bootstrap::People,
        Icons::Umbrella => icons::Bootstrap::Umbrella,
        Icons::Underline => icons::Bootstrap::TypeUnderline,
        Icons::Upload => icons::Bootstrap::Upload,
        // Icons::Uppercase => icons::Bootstrap::,
        Icons::Wallet => icons::Bootstrap::Wallet,
        Icons::Wand => icons::Bootstrap::Magic,
        // Icons::Warning => icons::Bootstrap::,
        // Icons::Weights => icons::Bootstrap::,
        Icons::Wifi => icons::Bootstrap::Wifi,
        Icons::WifiDisabled => icons::Bootstrap::WifiOff,
        Icons::Window => icons::Bootstrap::Window,
        Icons::Tools => icons::Bootstrap::Tools,
        Icons::Watch => icons::Bootstrap::Watch,
        Icons::XMark => icons::Bootstrap::XLg,
        Icons::Indent => icons::Bootstrap::Indent,
        Icons::Unindent => icons::Bootstrap::Unindent,
    }
}
