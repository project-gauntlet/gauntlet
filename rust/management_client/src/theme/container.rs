use iced::widget::container;
use iced::widget::container::Style;
use iced::Border;
use iced::Color;

use crate::theme::GauntletSettingsTheme;
use crate::theme::BACKGROUND_DARKER;
use crate::theme::BACKGROUND_LIGHTER;
use crate::theme::BACKGROUND_LIGHTEST;
use crate::theme::DANGER;
use crate::theme::TRANSPARENT;

pub enum ContainerStyle {
    Transparent,
    Box,
    TextInputLike,
    TextInputMissingValue,
}

impl container::Catalog for GauntletSettingsTheme {
    type Class<'a> = ContainerStyle;

    fn default<'a>() -> Self::Class<'a> {
        ContainerStyle::Transparent
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            ContainerStyle::Transparent => Default::default(),
            ContainerStyle::Box => {
                Style {
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
                Style {
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

                Style {
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
