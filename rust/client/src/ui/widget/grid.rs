use std::cell::Cell;
use std::collections::HashMap;

use gauntlet_common::model::GridItemWidget;
use gauntlet_common::model::GridSectionWidget;
use gauntlet_common::model::GridSectionWidgetOrderedMembers;
use gauntlet_common::model::GridWidget;
use gauntlet_common::model::GridWidgetOrderedMembers;
use gauntlet_common::model::PhysicalShortcut;
use iced::Length;
use iced::advanced::text::Shaping;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::grid;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use itertools::Itertools;

use crate::ui::state::PluginViewState;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::grid::GridStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::widget::accessories::render_icon_accessory;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;

impl<'b> ComponentWidgets<'b> {
    pub fn render_grid_widget<'a>(
        &self,
        grid_widget: &GridWidget,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let state = self.state.scrollable_root_state(grid_widget.__id__);
        let focused_item_id = self.get_focused_item_id();

        let content = if grid_widget.content.ordered_members.is_empty() {
            match &grid_widget.content.empty_view {
                Some(widget) => self.render_empty_view_widget(widget),
                None => horizontal_space().into(),
            }
        } else {
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
                            let content = self.render_grid_section(
                                &pending,
                                &grid_widget.columns,
                                &focused_item_id,
                                index_counter,
                            );

                            items.push(content);

                            pending = vec![];
                        }

                        items.push(self.render_grid_section_widget(
                            widget,
                            &focused_item_id,
                            index_counter,
                            first_section,
                        ));

                        first_section = false;
                    }
                }
            }

            if !pending.is_empty() {
                let content = self.render_grid_section(&pending, &grid_widget.columns, &focused_item_id, index_counter);

                items.push(content);
            }

            let content: Element<_> = column(items).into();

            let content: Element<_> = container(content).width(Length::Fill).themed(ContainerStyle::GridInner);

            let content: Element<_> = scrollable(content)
                .id(state.scroll_handle.scrollable_id.clone())
                .width(Length::Fill)
                .into();

            let content: Element<_> = container(content).width(Length::Fill).themed(ContainerStyle::Grid);

            content
        };

        self.render_plugin_root(
            state.show_action_panel,
            grid_widget.__id__,
            focused_item_id,
            &grid_widget.content.search_bar,
            &grid_widget.content.actions,
            content,
            grid_widget.is_loading.unwrap_or(false),
            plugin_view_state,
            entrypoint_name,
            action_shortcuts,
        )
    }

    fn render_grid_section_widget<'a>(
        &self,
        widget: &GridSectionWidget,
        item_focused_id: &Option<String>,
        index_counter: &Cell<usize>,
        first_section: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let items: Vec<_> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                match members {
                    GridSectionWidgetOrderedMembers::GridItem(widget) => widget,
                }
            })
            .collect();

        let content = self.render_grid_section(&items, &widget.columns, item_focused_id, index_counter);

        let section_title_style = if first_section {
            RowStyle::GridFirstSectionTitle
        } else {
            RowStyle::GridSectionTitle
        };

        render_section(
            content,
            Some(&widget.title),
            &widget.subtitle,
            section_title_style,
            TextStyle::GridSectionTitle,
            TextStyle::GridSectionSubtitle,
        )
    }

    fn render_grid_item_widget<'a>(
        &self,
        widget: &GridItemWidget,
        item_focused_id: &Option<String>,
        index_counter: &Cell<usize>,
        grid_width: usize,
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

        let style = match &item_focused_id {
            None => ButtonStyle::GridItem,
            Some(focused_index) => {
                if focused_index == &widget.id {
                    ButtonStyle::GridItemFocused
                } else {
                    ButtonStyle::GridItem
                }
            }
        };

        index_counter.set(index_counter.get() + 1);

        let action_ids = self.get_action_widgets();
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

        let content: Element<_> = button(content).on_press(on_press_msg).width(Length::Fill).themed(style);

        let mut sub_content_left = vec![];

        if let Some(title) = &widget.title {
            // TODO text truncation when iced supports it
            let title = text(title.to_string())
                .size(15)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::GridItemTitle);

            sub_content_left.push(title);
        }

        if let Some(subtitle) = &widget.subtitle {
            let subtitle = text(subtitle.to_string())
                .size(15)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::GridItemSubTitle);

            sub_content_left.push(subtitle);
        }

        let mut sub_content_right = vec![];
        if let Some(widget) = &widget.content.accessory {
            sub_content_right.push(render_icon_accessory(self.data, widget));
        }

        let sub_content_left: Element<_> = column(sub_content_left).width(Length::Fill).into();

        let sub_content_right: Element<_> = column(sub_content_right).width(Length::Shrink).into();

        let sub_content: Element<_> = row(vec![sub_content_left, sub_content_right]).themed(RowStyle::GridItemTitle);

        let content: Element<_> = column(vec![content, sub_content]).width(Length::Fill).into();

        let state = self.state.scrollable_item_state(widget.__id__);
        let content: Element<_> = container(content).id(state.id.clone()).into();

        content
    }

    fn render_grid_section<'a>(
        &self,
        items: &[&GridItemWidget],
        /*aspect_ratio: Option<&str>,*/
        columns: &Option<f64>,
        item_focused_id: &Option<String>,
        index_counter: &Cell<usize>,
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

        let columns = grid_width(columns);

        let rows: Vec<Element<_>> = items
            .iter()
            .map(|widget| self.render_grid_item_widget(widget, item_focused_id, index_counter, columns))
            .chunks(columns)
            .into_iter()
            .flat_map(|row_items| row_items)
            .collect();

        let grid = grid(rows).columns(columns).themed(GridStyle::Default);

        let grid = container(grid).themed(ContainerStyle::GridSection);

        grid
    }
}

pub fn grid_width(columns: &Option<f64>) -> usize {
    columns.map(|value| value.trunc() as usize).unwrap_or(5)
}

pub fn render_section<'a>(
    content: Element<'a, ComponentWidgetEvent>,
    title: Option<&str>,
    subtitle: &Option<String>,
    theme_kind_title: RowStyle,
    theme_kind_title_text: TextStyle,
    theme_kind_subtitle_text: TextStyle,
) -> Element<'a, ComponentWidgetEvent> {
    let mut title_content = vec![];

    if let Some(title) = title {
        let title: Element<_> = text(title.to_string())
            .shaping(Shaping::Advanced)
            .size(14)
            .themed(theme_kind_title_text);

        title_content.push(title)
    }

    if let Some(subtitle) = subtitle {
        let subtitle: Element<_> = text(subtitle.to_string())
            .shaping(Shaping::Advanced)
            .size(14)
            .themed(theme_kind_subtitle_text);

        title_content.push(subtitle)
    }

    if title_content.is_empty() {
        let space: Element<_> = horizontal_space().height(40).into();

        title_content.push(space)
    }

    let title_content = row(title_content).themed(theme_kind_title);

    column([title_content, content]).into()
}
