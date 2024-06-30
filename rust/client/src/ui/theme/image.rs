use iced::widget::Image;
use crate::ui::theme::{Element, get_theme, ThemableWidget};

pub enum ImageStyle {
    EmptyViewImage,
    MainListItemIcon,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Image<iced::advanced::image::Handle> {
    type Kind = ImageStyle;

    fn themed(self, kind: ImageStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match kind {
            ImageStyle::EmptyViewImage => {
                self.width(theme.empty_view_image.size.width)
                    .height(theme.empty_view_image.size.height)
            }
            ImageStyle::MainListItemIcon => {
                self.width(16)
                    .height(16)
            }
        }.into()
    }
}
