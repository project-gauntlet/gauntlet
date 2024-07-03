use iced::Renderer;
use iced::widget::Tooltip;

use crate::ui::theme::{Element, GauntletTheme, get_theme, ThemableWidget};
use crate::ui::theme::container::ContainerStyleInner;

pub enum TooltipStyle {
    Tooltip,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Tooltip<'a, Message, GauntletTheme, Renderer> {
    type Kind = TooltipStyle;

    fn themed(self, kind: TooltipStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match kind {
            TooltipStyle::Tooltip => {
                self.style(ContainerStyleInner::Tooltip)
                    .padding(theme.tooltip.padding)
            }
        }.into()
    }
}
