use gauntlet_common::model::EmptyViewWidget;
use iced::advanced::text::Shaping;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_space;
use iced::widget::text;
use iced::Alignment;
use iced::Length;

use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::images::render_image;

impl<'b> ComponentWidgets<'b> {
    pub fn render_empty_view_widget<'a>(&self, widget: &EmptyViewWidget) -> Element<'a, ComponentWidgetEvent> {
        let image: Option<Element<_>> = widget
            .image
            .as_ref()
            .map(|image| render_image(self.data, widget.__id__, image, Some(TextStyle::EmptyViewSubtitle)));

        let title: Element<_> = text(widget.title.to_string()).shaping(Shaping::Advanced).into();

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
            let image: Element<_> = container(image).themed(ContainerStyle::EmptyViewImage);

            content.insert(0, image)
        }

        let content: Element<_> = column(content).align_x(Alignment::Center).into();

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}
