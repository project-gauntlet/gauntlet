use iced::Renderer;
use iced_aw::Grid;

use crate::ui::theme::get_theme;
use crate::ui::theme::Element;
use crate::ui::theme::GauntletComplexTheme;
use crate::ui::theme::ThemableWidget;

pub enum GridStyle {
    Default,
}

impl<'a, Message: 'a + 'static> ThemableWidget<'a, Message> for Grid<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = GridStyle;

    fn themed(self, kind: GridStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match kind {
            GridStyle::Default => self.spacing(theme.grid.spacing),
        }
        .into()
    }
}
