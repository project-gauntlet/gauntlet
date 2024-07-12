use iced::Border;
use iced::widget::checkbox;
use iced::widget::checkbox::Appearance;

use crate::theme::{BACKGROUND, BACKGROUND_LIGHT, BACKGROUND_LIGHTER, BACKGROUND_LIGHTEST, GauntletSettingsTheme, PRIMARY, PRIMARY_HOVERED};

#[derive(Default)]
pub enum CheckboxStyle {
    #[default]
    Default,
}

impl checkbox::StyleSheet for GauntletSettingsTheme {
    type Style = CheckboxStyle;

    fn active(&self, _: &Self::Style, is_checked: bool) -> Appearance {
        let background = if is_checked {
            PRIMARY.to_iced().into()
        } else {
            BACKGROUND.to_iced().into()
        };

        Appearance {
            background,
            icon_color: BACKGROUND.to_iced(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: PRIMARY.to_iced().into(),
            },
            text_color: None,
        }
    }

    fn hovered(&self, _: &Self::Style, is_checked: bool) -> Appearance {
        let background = if is_checked {
            PRIMARY_HOVERED.to_iced().into()
        } else {
            BACKGROUND_LIGHT.to_iced().into()
        };

        Appearance {
            background,
            icon_color: BACKGROUND.to_iced(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: PRIMARY.to_iced().into(),
            },
            text_color: None,
        }
    }

    fn disabled(&self, _: &Self::Style, is_checked: bool) -> Appearance {
        let background = if is_checked {
            BACKGROUND_LIGHTER.to_iced().into()
        } else {
            BACKGROUND_LIGHT.to_iced().into()
        };

        Appearance {
            background,
            icon_color: BACKGROUND.to_iced(),
            border: Default::default(),
            text_color: None,
        }
    }
}