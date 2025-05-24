use gauntlet_common::model::MetadataIconWidget;
use gauntlet_common::model::MetadataLinkWidget;
use gauntlet_common::model::MetadataSeparatorWidget;
use gauntlet_common::model::MetadataTagItemWidget;
use gauntlet_common::model::MetadataTagListWidget;
use gauntlet_common::model::MetadataTagListWidgetOrderedMembers;
use gauntlet_common::model::MetadataValueWidget;
use gauntlet_common::model::MetadataWidget;
use gauntlet_common::model::MetadataWidgetOrderedMembers;
use iced::Alignment;
use iced::Length;
use iced::advanced::text::Shaping;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::tooltip;
use iced::widget::tooltip::Position;
use iced::widget::value;
use iced_fonts::BOOTSTRAP_FONT;
use iced_fonts::Bootstrap;

use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::tooltip::TooltipStyle;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::images::icon_to_bootstrap;
use crate::ui::widget::text::TextRenderType;

impl<'b> ComponentWidgets<'b> {
    fn render_metadata_tag_item_widget<'a>(&self, widget: &MetadataTagItemWidget) -> Element<'a, ComponentWidgetEvent> {
        let content: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let tag: Element<_> = button(content)
            .on_press(ComponentWidgetEvent::TagClick {
                widget_id: widget.__id__,
            })
            .themed(ButtonStyle::MetadataTagItem);

        container(tag).themed(ContainerStyle::MetadataTagItem)
    }

    fn render_metadata_tag_list_widget<'a>(
        &self,
        widget: &MetadataTagListWidget,
        is_in_list: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<Element<_>> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                match members {
                    MetadataTagListWidgetOrderedMembers::MetadataTagItem(content) => {
                        self.render_metadata_tag_item_widget(&content)
                    }
                }
            })
            .collect();

        let value = row(content).wrap().into();

        render_metadata_item(&widget.label, value, is_in_list).into()
    }

    fn render_metadata_link_widget<'a>(
        &self,
        widget: &MetadataLinkWidget,
        is_in_list: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let content: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let icon: Element<_> = value(Bootstrap::BoxArrowUpRight).font(BOOTSTRAP_FONT).size(16).into();

        let icon = container(icon).themed(ContainerStyle::MetadataLinkIcon);

        let content: Element<_> = row([content, icon]).align_y(Alignment::Center).into();

        let link: Element<_> = button(content)
            .on_press(ComponentWidgetEvent::LinkClick {
                widget_id: widget.__id__,
                href: widget.href.to_owned(),
            })
            .themed(ButtonStyle::MetadataLink);

        let content: Element<_> = if widget.href.is_empty() {
            link
        } else {
            let href: Element<_> = text(widget.href.to_string()).shaping(Shaping::Advanced).into();

            tooltip(link, href, Position::Top).themed(TooltipStyle::Tooltip)
        };

        render_metadata_item(&widget.label, content, is_in_list).into()
    }

    fn render_metadata_value_widget<'a>(
        &self,
        widget: &MetadataValueWidget,
        is_in_list: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let value: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        render_metadata_item(&widget.label, value, is_in_list).into()
    }

    fn render_metadata_icon_widget<'a>(
        &self,
        widget: &MetadataIconWidget,
        is_in_list: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let value = value(icon_to_bootstrap(&widget.icon))
            .font(BOOTSTRAP_FONT)
            .size(26)
            .into();

        render_metadata_item(&widget.label, value, is_in_list).into()
    }

    fn render_metadata_separator_widget<'a>(
        &self,
        _widget: &MetadataSeparatorWidget,
    ) -> Element<'a, ComponentWidgetEvent> {
        let separator: Element<_> = horizontal_rule(1).into();

        container(separator)
            .width(Length::Fill)
            .themed(ContainerStyle::MetadataSeparator)
    }

    pub fn render_metadata_widget<'a>(
        &self,
        widget: &MetadataWidget,
        is_in_list: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<Element<_>> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                match members {
                    MetadataWidgetOrderedMembers::MetadataTagList(content) => {
                        self.render_metadata_tag_list_widget(content, is_in_list)
                    }
                    MetadataWidgetOrderedMembers::MetadataLink(content) => {
                        self.render_metadata_link_widget(content, is_in_list)
                    }
                    MetadataWidgetOrderedMembers::MetadataValue(content) => {
                        self.render_metadata_value_widget(content, is_in_list)
                    }
                    MetadataWidgetOrderedMembers::MetadataIcon(content) => {
                        self.render_metadata_icon_widget(content, is_in_list)
                    }
                    MetadataWidgetOrderedMembers::MetadataSeparator(content) => {
                        self.render_metadata_separator_widget(content)
                    }
                }
            })
            .collect();

        let metadata: Element<_> = column(content).into();

        let metadata = container(metadata)
            .width(Length::Fill)
            .themed(ContainerStyle::MetadataInner);

        scrollable(metadata).width(Length::Fill).into()
    }
}

fn render_metadata_item<'a>(
    label: &str,
    value: Element<'a, ComponentWidgetEvent>,
    is_in_list: bool,
) -> Element<'a, ComponentWidgetEvent> {
    let label: Element<_> = text(label.to_string())
        .shaping(Shaping::Advanced)
        .themed(TextStyle::MetadataItemLabel);

    let label = container(label).themed(ContainerStyle::MetadataItemLabel);

    if is_in_list {
        let space = horizontal_space().into();

        let value = container(value).themed(ContainerStyle::MetadataItemValueInList);

        row(vec![label, space, value]).width(Length::Fill).into()
    } else {
        let value = container(value).themed(ContainerStyle::MetadataItemValue);

        column(vec![label, value]).into()
    }
}
