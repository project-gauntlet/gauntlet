use iced::Border;
use iced::widget::container;

use crate::theme::{GauntletSettingsTheme, BACKGROUND_LIGHTEST, BACKGROUND_DARKER};

#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Transparent,
    Box,
    TextInputLike
}

impl container::StyleSheet for GauntletSettingsTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            ContainerStyle::Transparent => Default::default(),
            ContainerStyle::Box => {
                container::Appearance {
                    background: Some(BACKGROUND_LIGHTEST.to_iced().into()),
                    border: Border {
                        color: BACKGROUND_DARKER.to_iced(),
                        radius: 10.0.into(),
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }
            ContainerStyle::TextInputLike => {
                container::Appearance {
                    background: Some(BACKGROUND_LIGHTEST.to_iced().into()),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: BACKGROUND_LIGHTEST.to_iced().into(),
                    },
                    ..Default::default()
                }
            }
        }
    }
}