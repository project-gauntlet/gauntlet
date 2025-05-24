use iced::Font;
use iced::advanced::text::Shaping;
use iced::font::Weight;
use iced::widget::text;

use crate::ui::theme::Element;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;

#[derive(Debug, Clone)]
pub enum TextRenderType {
    None,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl<'b> ComponentWidgets<'b> {
    pub fn render_text<'a>(&self, value: &[String], context: TextRenderType) -> Element<'a, ComponentWidgetEvent> {
        let header = match context {
            TextRenderType::None => None,
            TextRenderType::H1 => Some(34),
            TextRenderType::H2 => Some(30),
            TextRenderType::H3 => Some(24),
            TextRenderType::H4 => Some(20),
            TextRenderType::H5 => Some(18),
            TextRenderType::H6 => Some(16),
        };

        let mut text = text(value.join("")).shaping(Shaping::Advanced);

        if let Some(size) = header {
            text = text.size(size).font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            })
        }

        text.into()
    }
}
