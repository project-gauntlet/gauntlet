use gauntlet_common::model::DetailWidget;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::vertical_rule;
use iced::Length;

use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;

impl<'b> ComponentWidgets<'b> {
    pub fn render_detail_widget<'a>(
        &self,
        widget: &DetailWidget,
        is_in_list: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let metadata_element = widget.content.metadata.as_ref().map(|widget| {
            let content = self.render_metadata_widget(widget, is_in_list);

            container(content)
                .width(
                    if is_in_list {
                        Length::Fill
                    } else {
                        Length::FillPortion(2)
                    },
                )
                .height(
                    if is_in_list {
                        Length::FillPortion(3)
                    } else {
                        Length::Fill
                    },
                )
                .themed(ContainerStyle::DetailMetadata)
        });

        let content_element = widget.content.content.as_ref().map(|widget| {
            let content_element: Element<_> = container(self.render_content_widget(widget, false))
                .width(Length::Fill)
                .themed(ContainerStyle::DetailContentInner);

            let content_element: Element<_> = scrollable(content_element).width(Length::Fill).into();

            let content_element: Element<_> = container(content_element)
                .width(
                    if is_in_list {
                        Length::Fill
                    } else {
                        Length::FillPortion(3)
                    },
                )
                .height(
                    if is_in_list {
                        Length::FillPortion(3)
                    } else {
                        Length::Fill
                    },
                )
                .themed(ContainerStyle::DetailContent);

            content_element
        });

        let separator = if is_in_list {
            horizontal_rule(1).into()
        } else {
            vertical_rule(1).into()
        };

        let list_fn = |vec| {
            if is_in_list {
                column(vec).into()
            } else {
                row(vec).into()
            }
        };

        let content: Element<_> = match (content_element, metadata_element) {
            (Some(content_element), Some(metadata_element)) => {
                list_fn(vec![content_element, separator, metadata_element])
            }
            (Some(content_element), None) => list_fn(vec![content_element]),
            (None, Some(metadata_element)) => list_fn(vec![metadata_element]),
            (None, None) => list_fn(vec![]),
        };

        content
    }
}
