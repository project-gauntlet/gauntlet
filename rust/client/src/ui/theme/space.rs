use iced::widget::Space;

use crate::ui::theme::{Element, ThemableWidget};

pub enum ThemeKindSpace {
    MainListItemIcon,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Space {
    type Kind = ThemeKindSpace;

    fn themed(self, kind: ThemeKindSpace) -> Element<'a, Message> {
        match kind {
            ThemeKindSpace::MainListItemIcon => {
                self.width(18)
                    .height(18)
            }
        }.into()
    }
}

