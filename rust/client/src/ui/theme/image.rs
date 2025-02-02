use iced::widget::Image;

use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;

pub enum ImageStyle {
    MainListItemIcon,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Image<iced::advanced::image::Handle> {
    type Kind = ImageStyle;

    fn themed(self, kind: ImageStyle) -> Element<'a, Message> {
        match kind {
            ImageStyle::MainListItemIcon => self.width(18).height(18),
        }
        .into()
    }
}
