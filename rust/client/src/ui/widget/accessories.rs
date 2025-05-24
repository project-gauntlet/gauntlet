use std::collections::HashMap;

use gauntlet_common::model::IconAccessoryWidget;
use gauntlet_common::model::TextAccessoryWidget;
use gauntlet_common::model::UiWidgetId;
use iced::Alignment;
use iced::advanced::text::Shaping;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::tooltip;
use iced::widget::tooltip::Position;

use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::tooltip::TooltipStyle;
use crate::ui::widget::images::render_image;

pub fn render_icon_accessory<'a, T: 'a + Clone>(
    data: &HashMap<UiWidgetId, Vec<u8>>,
    widget: &IconAccessoryWidget,
) -> Element<'a, T> {
    let icon = render_image(data, widget.__id__, &widget.icon, Some(TextStyle::IconAccessory));

    let content = container(icon)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .themed(ContainerStyle::IconAccessory);

    match widget.tooltip.as_ref() {
        None => content,
        Some(tooltip_text) => {
            let tooltip_text: Element<_> = text(tooltip_text.to_string()).shaping(Shaping::Advanced).into();

            tooltip(content, tooltip_text, Position::Top).themed(TooltipStyle::Tooltip)
        }
    }
}

pub fn render_text_accessory<'a, T: 'a + Clone>(
    data: &HashMap<UiWidgetId, Vec<u8>>,
    widget: &TextAccessoryWidget,
) -> Element<'a, T> {
    let icon: Option<Element<_>> = widget
        .icon
        .as_ref()
        .map(|icon| render_image(data, widget.__id__, icon, Some(TextStyle::TextAccessory)));

    let text_content: Element<_> = text(widget.text.to_string())
        .shaping(Shaping::Advanced)
        .themed(TextStyle::TextAccessory);

    let mut content: Vec<Element<_>> = vec![];

    if let Some(icon) = icon {
        let icon: Element<_> = container(icon).themed(ContainerStyle::TextAccessoryIcon);

        content.push(icon)
    }

    content.push(text_content);

    let content: Element<_> = row(content).align_y(Alignment::Center).into();

    let content = container(content)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .themed(ContainerStyle::TextAccessory);

    match widget.tooltip.as_ref() {
        None => content,
        Some(tooltip_text) => {
            let tooltip_text: Element<_> = text(tooltip_text.to_string()).shaping(Shaping::Advanced).into();

            tooltip(content, tooltip_text, Position::Top).themed(TooltipStyle::Tooltip)
        }
    }
}
