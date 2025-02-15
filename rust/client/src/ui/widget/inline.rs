use gauntlet_common::model::InlineSeparatorWidget;
use gauntlet_common::model::InlineWidget;
use gauntlet_common::model::InlineWidgetOrderedMembers;
use iced::advanced::text::Shaping;
use iced::alignment::Horizontal;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::value;
use iced::widget::vertical_rule;
use iced::Alignment;
use iced::Length;
use iced_fonts::BOOTSTRAP_FONT;

use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::images::icon_to_bootstrap;

impl<'b> ComponentWidgets<'b> {
    fn render_inline_separator_widget<'a>(&self, widget: &InlineSeparatorWidget) -> Element<'a, ComponentWidgetEvent> {
        match &widget.icon {
            None => vertical_rule(1).into(),
            Some(icon) => {
                let top_rule: Element<_> = vertical_rule(1).into();

                let top_rule = container(top_rule).align_x(Horizontal::Center).into();

                let icon = value(icon_to_bootstrap(icon))
                    .font(BOOTSTRAP_FONT)
                    .size(45)
                    .themed(TextStyle::InlineSeparator);

                let bot_rule: Element<_> = vertical_rule(1).into();

                let bot_rule = container(bot_rule).align_x(Horizontal::Center).into();

                column([top_rule, icon, bot_rule]).align_x(Alignment::Center).into()
            }
        }
    }

    pub fn render_inline_widget<'a>(
        &self,
        widget: &InlineWidget,
        plugin_name: &str,
        entrypoint_name: &str,
    ) -> Element<'a, ComponentWidgetEvent> {
        let name: Element<_> = text(format!("{} - {}", plugin_name, entrypoint_name))
            .shaping(Shaping::Advanced)
            .themed(TextStyle::InlineName);

        let name: Element<_> = container(name).themed(ContainerStyle::InlineName);

        let content: Vec<Element<_>> = widget
            .content
            .ordered_members
            .iter()
            .map(|members| {
                match members {
                    InlineWidgetOrderedMembers::Content(widget) => {
                        let element = self.render_content_widget(widget, true);

                        container(element).into()
                    }
                    InlineWidgetOrderedMembers::InlineSeparator(widget) => self.render_inline_separator_widget(widget),
                }
            })
            .collect();

        let content: Element<_> = row(content).into();

        let content: Element<_> = container(content).themed(ContainerStyle::InlineInner);

        let content: Element<_> = column(vec![name, content]).width(Length::Fill).into();

        let content: Element<_> = container(content).width(Length::Fill).themed(ContainerStyle::Inline);

        content
    }
}
