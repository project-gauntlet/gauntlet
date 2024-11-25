use crate::theme::{GauntletSettingsTheme, BACKGROUND_DARKER};
use iced::widget::rule;
use iced::widget::rule::Style;


impl rule::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>) -> Style {
        Style {
            color: BACKGROUND_DARKER.to_iced(),
            width: 1,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}
