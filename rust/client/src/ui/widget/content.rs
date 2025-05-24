use gauntlet_common::model::CodeBlockWidget;
use gauntlet_common::model::ContentWidget;
use gauntlet_common::model::ContentWidgetOrderedMembers;
use gauntlet_common::model::H1Widget;
use gauntlet_common::model::H2Widget;
use gauntlet_common::model::H3Widget;
use gauntlet_common::model::H4Widget;
use gauntlet_common::model::H5Widget;
use gauntlet_common::model::H6Widget;
use gauntlet_common::model::HorizontalBreakWidget;
use gauntlet_common::model::ImageWidget;
use gauntlet_common::model::ParagraphWidget;
use gauntlet_common::model::SvgWidget;
use iced::Length;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;

use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::images::render_image;
use crate::ui::widget::images::render_svg;
use crate::ui::widget::text::TextRenderType;

impl<'b> ComponentWidgets<'b> {
    fn render_paragraph_widget<'a>(
        &self,
        widget: &ParagraphWidget,
        centered: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let paragraph: Element<_> = self.render_text(&widget.content.text, TextRenderType::None);

        let mut content = container(paragraph).width(Length::Fill);

        if centered {
            content = content.align_x(Horizontal::Center)
        }

        content.themed(ContainerStyle::ContentParagraph)
    }

    fn render_image_widget<'a>(&self, widget: &ImageWidget, centered: bool) -> Element<'a, ComponentWidgetEvent> {
        // TODO image size, height and width
        let content: Element<_> = render_image(self.data, widget.__id__, &widget.source, None);

        let mut content = container(content).width(Length::Fill);

        if centered {
            content = content.align_x(Horizontal::Center)
        }

        content.themed(ContainerStyle::ContentImage)
    }

    fn render_svg_widget<'a>(&self, widget: &SvgWidget, centered: bool) -> Element<'a, ComponentWidgetEvent> {
        // TODO svg size, height and width
        let content: Element<_> = render_svg(self.data, widget.__id__);

        let mut content = container(content).width(Length::Fill);

        if centered {
            content = content.align_x(Horizontal::Center)
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

    pub fn render_content_widget<'a>(
        &self,
        widget: &ContentWidget,
        centered: bool,
    ) -> Element<'a, ComponentWidgetEvent> {
        let content: Vec<_> = widget
            .content
            .ordered_members
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
                    ContentWidgetOrderedMembers::Svg(widget) => self.render_svg_widget(widget, centered),
                }
            })
            .collect();

        let content: Element<_> = column(content).into();

        if centered {
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
        } else {
            content
        }
    }
}
