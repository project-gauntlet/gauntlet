use std::cell::Cell;
use std::collections::HashMap;

use gauntlet_common::model::ListItemAccessories;
use gauntlet_common::model::ListItemWidget;
use gauntlet_common::model::ListSectionWidget;
use gauntlet_common::model::ListSectionWidgetOrderedMembers;
use gauntlet_common::model::ListWidget;
use gauntlet_common::model::ListWidgetOrderedMembers;
use gauntlet_common::model::PhysicalShortcut;
use iced::advanced::text::Shaping;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::vertical_rule;
use iced::Alignment;
use iced::Length;

use crate::ui::state::PluginViewState;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::widget::accessories::render_icon_accessory;
use crate::ui::widget::accessories::render_text_accessory;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::grid::render_section;
use crate::ui::widget::images::render_image;
use crate::ui::widget::state::RootState;

impl<'b> ComponentWidgets<'b> {
    pub fn render_list_widget<'a>(
        &self,
        list_widget: &ListWidget,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = list_widget.__id__;
        let RootState {
            show_action_panel,
            focused_item,
        } = self.root_state(widget_id);

        let mut pending: Vec<&ListItemWidget> = vec![];
        let mut items: Vec<Element<_>> = vec![];
        let index_counter = &Cell::new(0);
        let mut first_section = true;

        for members in &list_widget.content.ordered_members {
            match &members {
                ListWidgetOrderedMembers::ListItem(widget) => {
                    first_section = false;
                    pending.push(widget)
                }
                ListWidgetOrderedMembers::ListSection(widget) => {
                    if !pending.is_empty() {
                        let content: Vec<_> = pending
                            .iter()
                            .map(|widget| self.render_list_item_widget(widget, focused_item.index, index_counter))
                            .collect();

                        let content: Element<_> = column(content).into();

                        items.push(content);

                        pending = vec![];
                    }

                    items.push(self.render_list_section_widget(
                        widget,
                        focused_item.index,
                        index_counter,
                        first_section,
                    ));

                    first_section = false;
                }
            }
        }

        if !pending.is_empty() {
            let content: Vec<_> = pending
                .iter()
                .map(|widget| self.render_list_item_widget(widget, focused_item.index, index_counter))
                .collect();

            let content: Element<_> = column(content).into();

            items.push(content);
        }

        let content = if items.is_empty() {
            match &list_widget.content.empty_view {
                Some(widget) => self.render_empty_view_widget(widget),
                None => horizontal_space().into(),
            }
        } else {
            let content: Element<_> = column(items).width(Length::Fill).into();

            let content: Element<_> = container(content).width(Length::Fill).themed(ContainerStyle::ListInner);

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

            let detail: Element<_> = container(detail).width(Length::FillPortion(5)).into();

            let separator: Element<_> = vertical_rule(1).into();

            elements.push(separator);

            elements.push(detail);
        }

        let content: Element<_> = row(elements).height(Length::Fill).into();

        let focused_item_id = ComponentWidgets::list_focused_item_id(focused_item, list_widget);

        self.render_plugin_root(
            *show_action_panel,
            widget_id,
            focused_item_id,
            &list_widget.content.search_bar,
            &list_widget.content.actions,
            content,
            list_widget.is_loading.unwrap_or(false),
            plugin_view_state,
            entrypoint_name,
            action_shortcuts,
        )
    }

    fn render_list_section_widget<'a>(
        &self,
        widget: &ListSectionWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>,
        first_section: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<_> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                match members {
                    ListSectionWidgetOrderedMembers::ListItem(widget) => {
                        self.render_list_item_widget(widget, item_focus_index, index_counter)
                    }
                }
            })
            .collect();

        let content = column(content).into();

        let section_title_style = if first_section {
            RowStyle::ListFirstSectionTitle
        } else {
            RowStyle::ListSectionTitle
        };

        render_section(
            content,
            Some(&widget.title),
            &widget.subtitle,
            section_title_style,
            TextStyle::ListSectionTitle,
            TextStyle::ListSectionSubtitle,
        )
    }

    fn render_list_item_widget<'a>(
        &self,
        widget: &ListItemWidget,
        item_focus_index: Option<usize>,
        index_counter: &Cell<usize>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let icon: Option<Element<_>> = widget
            .icon
            .as_ref()
            .map(|icon| render_image(self.data, widget.__id__, icon, None));

        let title: Element<_> = text(widget.title.to_string()).shaping(Shaping::Advanced).into();
        let title: Element<_> = container(title).themed(ContainerStyle::ListItemTitle);

        let mut content = vec![title];

        if let Some(icon) = icon {
            let icon: Element<_> = container(icon).themed(ContainerStyle::ListItemIcon);

            content.insert(0, icon)
        }

        if let Some(subtitle) = &widget.subtitle {
            let subtitle: Element<_> = text(subtitle.to_string())
                .shaping(Shaping::Advanced)
                .themed(TextStyle::ListItemSubtitle);
            let subtitle: Element<_> = container(subtitle).themed(ContainerStyle::ListItemSubtitle);

            content.push(subtitle)
        }

        if widget.content.accessories.len() > 0 {
            let accessories: Vec<Element<_>> = widget
                .content
                .accessories
                .iter()
                .map(|accessory| {
                    match accessory {
                        ListItemAccessories::_0(widget) => render_text_accessory(self.data, widget),
                        ListItemAccessories::_1(widget) => render_icon_accessory(self.data, widget),
                    }
                })
                .collect();

            let accessories: Element<_> = row(accessories).into();

            let space = horizontal_space().into();

            content.push(space);
            content.push(accessories);
        }

        let content: Element<_> = row(content).align_y(Alignment::Center).into();

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

        let action_ids = self.get_action_ids();
        let primary_action = action_ids.first();

        let on_press_msg = match primary_action {
            None => ComponentWidgetEvent::Noop,
            Some(widget_id) => {
                ComponentWidgetEvent::RunPrimaryAction {
                    widget_id: *widget_id,
                    id: Some(widget.id.clone()),
                }
            }
        };

        button(content).on_press(on_press_msg).width(Length::Fill).themed(style)
    }
}
