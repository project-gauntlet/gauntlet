use iced::Border;
use iced::widget::container;

use crate::theme::{BACKGROUND_LIGHT, BACKGROUND_OVERLAY, GauntletSettingsTheme};

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
                    background: Some(BACKGROUND_OVERLAY.to_iced().into()), // TODO
                    border: Border {
                        color: BACKGROUND_OVERLAY.to_iced(),
                        radius: 10.0.into(),
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }
            ContainerStyle::TextInputLike => {
                container::Appearance {
                    background: Some(BACKGROUND_LIGHT.to_iced().into()),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: BACKGROUND_LIGHT.to_iced().into(),
                    },
                    ..Default::default()
                }
            }
        }
    }
}