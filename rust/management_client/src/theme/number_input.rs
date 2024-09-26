use iced_aw::number_input;

use crate::theme::{GauntletSettingsTheme, PRIMARY, PRIMARY_HOVERED, TEXT_DARKER, TEXT_LIGHTEST};

#[derive(Default)]
pub enum NumberInputStyle {
    #[default]
    Default
}

impl number_input::StyleSheet for GauntletSettingsTheme {
    type Style = NumberInputStyle;

    fn active(&self, _: &Self::Style) -> number_input::Appearance {
        number_input::Appearance {
            button_background: Some(PRIMARY.to_iced().into()),
            icon_color: TEXT_DARKER.to_iced(),
        }
    }

    fn pressed(&self, _: &Self::Style) -> number_input::Appearance {
        number_input::Appearance {
            button_background: Some(PRIMARY_HOVERED.to_iced().into()),
            icon_color: TEXT_DARKER.to_iced(),
        }
    }

    fn disabled(&self, _: &Self::Style) -> number_input::Appearance {
        number_input::Appearance {
            button_background: None,
            icon_color: TEXT_LIGHTEST.to_iced(),
        }
    }
}