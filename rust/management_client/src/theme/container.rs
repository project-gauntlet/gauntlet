use iced::{Border, Color};
use iced::widget::container;

use crate::theme::{GauntletSettingsTheme, BACKGROUND_LIGHTEST, BACKGROUND_DARKER, BACKGROUND_LIGHTER, DANGER, TRANSPARENT};

#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Transparent,
    Box,
    TextInputLike,
    TextInputMissingValue
}

impl container::StyleSheet for GauntletSettingsTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            ContainerStyle::Transparent => Default::default(),
            ContainerStyle::Box => {
                container::Appearance {
                    background: Some(BACKGROUND_DARKER.to_iced().into()),
                    border: Border {
                        color: BACKGROUND_LIGHTER.to_iced(),
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
            ContainerStyle::TextInputMissingValue => {
                let color = DANGER.to_iced();

                container::Appearance {
                    background: Some(Color::new(color.r, color.g, color.b, 0.3).into()),
                    border: Border {
                        color: TRANSPARENT.to_iced(),
                        radius: 4.0.into(),
                        width: 0.0,
                    },
                    ..Default::default()
                }
            }
        }
    }
}