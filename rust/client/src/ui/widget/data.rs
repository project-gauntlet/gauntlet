use std::collections::HashMap;
use std::sync::Arc;

use gauntlet_common::model::ActionPanelSectionWidgetOrderedMembers;
use gauntlet_common::model::ActionPanelWidgetOrderedMembers;
use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidget;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::JsOption;
use gauntlet_common::model::ListSectionWidgetOrderedMembers;
use gauntlet_common::model::ListWidget;
use gauntlet_common::model::ListWidgetOrderedMembers;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::UiWidgetId;
use iced::Task;
use iced::widget::container;
use iced::widget::text_input;
use indexmap::IndexMap;

use crate::ui::AppMsg;
use crate::ui::scroll_handle::ScrollContent;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::action_panel::convert_action_panel;
use crate::ui::widget::data_mut::ComponentWidgetsMut;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::state::ComponentWidgetStateContainer;
use crate::ui::widget::state::TextFieldState;

#[derive(Debug)]
pub struct ComponentWidgets<'b> {
    pub root_widget: &'b Option<Arc<RootWidget>>,
    pub state: &'b ComponentWidgetStateContainer,
    pub data: &'b HashMap<UiWidgetId, Vec<u8>>,
}

impl<'b> ComponentWidgets<'b> {
    pub fn new(
        root_widget: &'b Option<Arc<RootWidget>>,
        state: &'b ComponentWidgetStateContainer,
        data: &'b HashMap<UiWidgetId, Vec<u8>>,
    ) -> ComponentWidgets<'b> {
        Self {
            root_widget,
            state,
            data,
        }
    }

    pub fn from_mut<'a>(widgets: &'b ComponentWidgetsMut<'a>) -> Self {
        Self {
            root_widget: &widgets.root_widget,
            state: &widgets.state,
            data: &widgets.data,
        }
    }
}

impl<'b> ComponentWidgets<'b> {
    pub fn get_action_widgets(&self) -> Vec<UiWidgetId> {
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
                match &widget.focused_item_id {
                    JsOption::Undefined => {
                        let state = self.state.scrollable_root_state(widget.__id__);

                        self.list_focused_item_id(
                            &state.scroll_handle,
                            state.scroll_handle.current_item_id.clone(),
                            widget,
                        )
                    }
                    JsOption::Null => None,
                    JsOption::Value(value) => Some(value.clone()),
                }
            }
            RootWidgetMembers::Grid(widget) => {
                match &widget.focused_item_id {
                    JsOption::Undefined => {
                        let state = self.state.scrollable_root_state(widget.__id__);

                        self.grid_focused_item_id(
                            &state.scroll_handle,
                            state.scroll_handle.current_item_id.clone(),
                            widget,
                        )
                    }
                    JsOption::Null => None,
                    JsOption::Value(value) => Some(value.clone()),
                }
            }
        }
    }

    pub fn focus_search_bar(&self, widget_id: UiWidgetId) -> Task<AppMsg> {
        let TextFieldState { text_input_id, .. } = self.state.text_field_state(widget_id);

        text_input::focus(text_input_id.clone())
    }
}

impl<'b> ComponentWidgets<'b> {
    pub fn first_open(&self, plugin_id: PluginId) -> AppMsg {
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

        AppMsg::FocusPluginViewSearchBar { plugin_id, widget_id }
    }

    pub fn list_focused_item_id(
        &self,
        scroll_handle: &ScrollHandle,
        target_item_id: Option<container::Id>,
        widget: &ListWidget,
    ) -> Option<String> {
        let mut items = IndexMap::new();

        for members in &widget.content.ordered_members {
            match &members {
                ListWidgetOrderedMembers::ListItem(item) => {
                    let state = self.state.scrollable_item_state(item.__id__);
                    items.insert(state.id.clone(), &item.id);
                }
                ListWidgetOrderedMembers::ListSection(section) => {
                    for members in &section.content.ordered_members {
                        match &members {
                            ListSectionWidgetOrderedMembers::ListItem(item) => {
                                let state = self.state.scrollable_item_state(item.__id__);
                                items.insert(state.id.clone(), &item.id);
                            }
                        }
                    }
                }
            }
        }

        match scroll_handle.get_by_id(&ScrollContent::new_with_ids(items), target_item_id) {
            None => None,
            Some(item_id) => Some(item_id.to_string()),
        }
    }

    pub fn list_item_focus_event(
        &self,
        plugin_id: PluginId,
        scroll_handle: &ScrollHandle,
        target_item_id: Option<container::Id>,
        widget: &ListWidget,
    ) -> Task<AppMsg> {
        let widget_event = match self.list_focused_item_id(scroll_handle, target_item_id, widget) {
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
            widget_event,
        })
    }

    pub fn grid_focused_item_id(
        &self,
        scroll_handle: &ScrollHandle,
        target_item_id: Option<container::Id>,
        widget: &GridWidget,
    ) -> Option<String> {
        let mut items = IndexMap::new();

        for members in &widget.content.ordered_members {
            match &members {
                GridWidgetOrderedMembers::GridItem(item) => {
                    let state = self.state.scrollable_item_state(item.__id__);
                    items.insert(state.id.clone(), &item.id);
                }
                GridWidgetOrderedMembers::GridSection(section) => {
                    for members in &section.content.ordered_members {
                        match &members {
                            GridSectionWidgetOrderedMembers::GridItem(item) => {
                                let state = self.state.scrollable_item_state(item.__id__);
                                items.insert(state.id.clone(), &item.id);
                            }
                        }
                    }
                }
            }
        }

        match scroll_handle.get_by_id(&ScrollContent::new_with_ids(items), target_item_id) {
            None => None,
            Some(item_id) => Some(item_id.to_string()),
        }
    }

    pub fn grid_item_focus_event(
        &self,
        plugin_id: PluginId,
        scroll_handle: &ScrollHandle,
        target_item_id: Option<container::Id>,
        widget: &GridWidget,
    ) -> Task<AppMsg> {
        let widget_event = match self.grid_focused_item_id(scroll_handle, target_item_id, widget) {
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
