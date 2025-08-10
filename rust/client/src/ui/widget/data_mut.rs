use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidget;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::JsOption;
use gauntlet_common::model::ListSectionWidgetOrderedMembers;
use gauntlet_common::model::ListWidget;
use gauntlet_common::model::ListWidgetOrderedMembers;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::UiWidgetId;
use iced::Task;
use iced::advanced::widget::operate;
use iced::advanced::widget::operation::focusable::unfocus;
use iced::widget::container;
use iced::widget::text_input;
use itertools::Itertools;

use crate::ui::AppMsg;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::grid::grid_width;
use crate::ui::widget::state::ComponentWidgetStateContainer;

#[derive(Debug)]
pub struct ComponentWidgetsMut<'b> {
    pub root_widget: &'b mut Option<Arc<RootWidget>>,
    pub state: &'b mut ComponentWidgetStateContainer,
    pub data: &'b HashMap<UiWidgetId, Vec<u8>>,
    pub plugin_id: PluginId,
}

impl<'b> ComponentWidgetsMut<'b> {
    pub fn new(
        root_widget: &'b mut Option<Arc<RootWidget>>,
        state: &'b mut ComponentWidgetStateContainer,
        data: &'b mut HashMap<UiWidgetId, Vec<u8>>,
        plugin_id: &PluginId,
    ) -> ComponentWidgetsMut<'b> {
        Self {
            root_widget,
            state,
            data,
            plugin_id: plugin_id.clone(),
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

        match content {
            RootWidgetMembers::Detail(widget) => {
                let state = self.state.root_state_mut(widget.__id__);

                state.show_action_panel = !state.show_action_panel;
            }
            RootWidgetMembers::Form(widget) => {
                let state = self.state.root_state_mut(widget.__id__);

                state.show_action_panel = !state.show_action_panel;
            }
            RootWidgetMembers::Inline(widget) => {
                let state = self.state.root_state_mut(widget.__id__);

                state.show_action_panel = !state.show_action_panel;
            }
            RootWidgetMembers::List(widget) => {
                let state = self.state.scrollable_root_state_mut(widget.__id__);

                state.show_action_panel = !state.show_action_panel;
            }
            RootWidgetMembers::Grid(widget) => {
                let state = self.state.scrollable_root_state_mut(widget.__id__);

                state.show_action_panel = !state.show_action_panel;
            }
        }
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

        let state = self.state.text_field_state_mut(widget_id);

        if let Some(value) = text.chars().next().filter(|c| !c.is_control()) {
            state.state_value = format!("{}{}", state.state_value, value);

            text_input::focus(state.text_input_id.clone())
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

        let state = self.state.text_field_state_mut(widget_id);

        let mut chars = state.state_value.chars();
        chars.next_back();
        state.state_value = chars.as_str().to_owned();

        text_input::focus(state.text_input_id.clone())
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
                let ids = self.list_collect_ids(&widget);
                let mut scroll_handle = self.list_scroll_handle(widget);
                let (next_item, Some(focus_task)) = scroll_handle.list_focus_up(ids) else {
                    return Task::none();
                };

                let state = self.state.scrollable_root_state(widget.__id__);
                let focus_event = ComponentWidgets::from_mut(&self).list_item_focus_event(
                    self.plugin_id.clone(),
                    &state.scroll_handle,
                    next_item,
                    widget,
                );

                Task::batch([focus_event, focus_task])
            }
            RootWidgetMembers::Grid(widget) => {
                let ids = self.grid_collect_ids(widget);
                let mut scroll_handle = self.grid_scroll_handle(widget);
                let (next_item, Some(focus_task)) = scroll_handle.grid_focus_up(ids) else {
                    return Task::none();
                };

                let state = self.state.scrollable_root_state(widget.__id__);
                let focus_event = ComponentWidgets::from_mut(&self).grid_item_focus_event(
                    self.plugin_id.clone(),
                    &state.scroll_handle,
                    next_item,
                    widget,
                );

                Task::batch([focus_event, focus_task])
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
            RootWidgetMembers::List(widget) => {
                let ids = self.list_collect_ids(widget);
                let mut scroll_handle = self.list_scroll_handle(widget);
                let (next_item, Some(focus_task)) = scroll_handle.list_focus_down(ids) else {
                    return Task::none();
                };

                let state = self.state.scrollable_root_state(widget.__id__);
                let focus_event = ComponentWidgets::from_mut(&self).list_item_focus_event(
                    self.plugin_id.clone(),
                    &state.scroll_handle,
                    next_item,
                    widget,
                );

                Task::batch([focus_event, focus_task])
            }
            RootWidgetMembers::Grid(widget) => {
                let unfocus_search_bar = match &widget.content.search_bar {
                    Some(_) => operate(unfocus()),
                    None => Task::none(),
                };

                let ids = self.grid_collect_ids(widget);
                let mut scroll_handle = self.grid_scroll_handle(widget);
                let (next_item, Some(focus_task)) = scroll_handle.grid_focus_down(ids) else {
                    return Task::none();
                };

                let state = self.state.scrollable_root_state(widget.__id__);
                let focus_event = ComponentWidgets::from_mut(&self).grid_item_focus_event(
                    self.plugin_id.clone(),
                    &state.scroll_handle,
                    next_item,
                    widget,
                );

                Task::batch([unfocus_search_bar, focus_task, focus_event])
            }
            RootWidgetMembers::Inline(_) => Task::none(),
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
                let ids = self.grid_collect_ids(widget);
                let mut scroll_handle = self.grid_scroll_handle(widget);
                let (next_item, Some(focus_task)) = scroll_handle.grid_focus_left(ids) else {
                    return Task::none();
                };

                let state = self.state.scrollable_root_state(widget.__id__);
                let focus_event = ComponentWidgets::from_mut(&self).grid_item_focus_event(
                    self.plugin_id.clone(),
                    &state.scroll_handle,
                    next_item,
                    widget,
                );

                Task::batch([focus_event, focus_task])
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
            RootWidgetMembers::Grid(widget) => {
                let ids = self.grid_collect_ids(widget);
                let mut scroll_handle = self.grid_scroll_handle(widget);
                let (next_item, Some(focus_task)) = scroll_handle.grid_focus_right(ids) else {
                    return Task::none();
                };

                let state = self.state.scrollable_root_state(widget.__id__);
                let focus_event = ComponentWidgets::from_mut(&self).grid_item_focus_event(
                    self.plugin_id.clone(),
                    &state.scroll_handle,
                    next_item,
                    widget,
                );

                Task::batch([focus_event, focus_task])
            }
        }
    }

    pub fn set_focused_item_id(&mut self, target_id: Option<container::Id>) -> Task<AppMsg> {
        let Some(root) = self.root_widget else {
            return Task::none();
        };

        let Some(content) = &root.content else {
            return Task::none();
        };

        match content {
            RootWidgetMembers::List(widget) => {
                let state = self.state.scrollable_root_state_mut(widget.__id__);

                state.scroll_handle.set_current_focused_item(target_id)
            }
            RootWidgetMembers::Grid(widget) => {
                let state = self.state.scrollable_root_state_mut(widget.__id__);

                state.scroll_handle.set_current_focused_item(target_id)
            }
            _ => Task::none(),
        }
    }

    fn list_collect_ids(&self, list_widget: &ListWidget) -> Vec<container::Id> {
        list_widget
            .content
            .ordered_members
            .iter()
            .flat_map(|members| {
                match members {
                    ListWidgetOrderedMembers::ListItem(widget) => {
                        let state = self.state.scrollable_item_state(widget.__id__);

                        vec![state.id.clone()]
                    }
                    ListWidgetOrderedMembers::ListSection(widget) => {
                        widget
                            .content
                            .ordered_members
                            .iter()
                            .map(|members| {
                                match members {
                                    ListSectionWidgetOrderedMembers::ListItem(widget) => {
                                        let state = self.state.scrollable_item_state(widget.__id__);
                                        state.id.clone()
                                    }
                                }
                            })
                            .collect()
                    }
                }
            })
            .collect()
    }

    fn grid_collect_ids(&self, grid_widget: &GridWidget) -> Vec<Vec<container::Id>> {
        let global_columns = grid_width(&grid_widget.columns);

        fn create_section(items: Vec<container::Id>, columns: usize) -> Vec<Vec<container::Id>> {
            items
                .into_iter()
                .chunks(columns)
                .into_iter()
                .map(|iter| iter.collect())
                .collect()
        }

        let mut result = vec![];
        let mut pending = vec![];

        for members in &grid_widget.content.ordered_members {
            match members {
                GridWidgetOrderedMembers::GridItem(widget) => {
                    let state = self.state.scrollable_item_state(widget.__id__);

                    pending.push(state.id.clone())
                }
                GridWidgetOrderedMembers::GridSection(widget) => {
                    if !pending.is_empty() {
                        let pending = mem::replace(&mut pending, vec![]);
                        result.extend(create_section(pending, global_columns))
                    }

                    let section_columns = grid_width(&widget.columns);

                    let section = widget
                        .content
                        .ordered_members
                        .iter()
                        .map(|members| {
                            match members {
                                GridSectionWidgetOrderedMembers::GridItem(widget) => {
                                    let state = self.state.scrollable_item_state(widget.__id__);

                                    state.id.clone()
                                }
                            }
                        })
                        .collect();

                    result.extend(create_section(section, section_columns))
                }
            }
        }

        if !pending.is_empty() {
            let pending = mem::replace(&mut pending, vec![]);
            result.extend(create_section(pending, global_columns))
        }

        result
    }

    pub fn list_id_for_id(&self, widget: &ListWidget, target_item_id: &String) -> Option<container::Id> {
        for members in &widget.content.ordered_members {
            match &members {
                ListWidgetOrderedMembers::ListItem(item) => {
                    let state = self.state.scrollable_item_state(item.__id__);
                    if &item.id == target_item_id {
                        return Some(state.id.clone());
                    }
                }
                ListWidgetOrderedMembers::ListSection(section) => {
                    for members in &section.content.ordered_members {
                        match &members {
                            ListSectionWidgetOrderedMembers::ListItem(item) => {
                                let state = self.state.scrollable_item_state(item.__id__);
                                if &item.id == target_item_id {
                                    return Some(state.id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn grid_id_for_id(&self, widget: &GridWidget, target_item_id: &String) -> Option<container::Id> {
        for members in &widget.content.ordered_members {
            match &members {
                GridWidgetOrderedMembers::GridItem(item) => {
                    let state = self.state.scrollable_item_state(item.__id__);
                    if &item.id == target_item_id {
                        return Some(state.id.clone());
                    }
                }
                GridWidgetOrderedMembers::GridSection(section) => {
                    for members in &section.content.ordered_members {
                        match &members {
                            GridSectionWidgetOrderedMembers::GridItem(item) => {
                                let state = self.state.scrollable_item_state(item.__id__);
                                if &item.id == target_item_id {
                                    return Some(state.id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn grid_scroll_handle(&self, widget: &GridWidget) -> ScrollHandle {
        let state = self.state.scrollable_root_state(widget.__id__);
        match &widget.focused_item_id {
            JsOption::Undefined => state.scroll_handle.clone(),
            JsOption::Null => ScrollHandle::from(&state.scroll_handle, None),
            JsOption::Value(focused_item_id) => {
                ScrollHandle::from(&state.scroll_handle, self.grid_id_for_id(widget, focused_item_id))
            }
        }
    }

    fn list_scroll_handle(&self, widget: &ListWidget) -> ScrollHandle {
        let state = self.state.scrollable_root_state(widget.__id__);
        match &widget.focused_item_id {
            JsOption::Undefined => state.scroll_handle.clone(),
            JsOption::Null => ScrollHandle::from(&state.scroll_handle, None),
            JsOption::Value(focused_item_id) => {
                ScrollHandle::from(&state.scroll_handle, self.list_id_for_id(widget, focused_item_id))
            }
        }
    }
}
