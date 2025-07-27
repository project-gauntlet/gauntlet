use iced::Background;
use iced::Border;
use iced::Color;
use iced::widget::container;
use iced::widget::container::Style;

use crate::ui::settings::theme::BACKGROUND_DARKER;
use crate::ui::settings::theme::BACKGROUND_DARKEST;
use crate::ui::settings::theme::BACKGROUND_LIGHTER;
use crate::ui::settings::theme::DANGER;
use crate::ui::settings::theme::GauntletSettingsTheme;
use crate::ui::settings::theme::TEXT_LIGHTEST;
use crate::ui::settings::theme::TRANSPARENT;

pub enum ContainerStyle {
    WindowRoot,
    Transparent,
    Box,
    TextInputMissingValue,
    TableEvenRow,
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
            ContainerStyle::TextInputMissingValue => {
                let color = DANGER.to_iced();

                Style {
                    background: Some(Color::from_rgba(color.r, color.g, color.b, 0.3).into()),
                    border: Border {
                        color: TRANSPARENT.to_iced(),
                        radius: 4.0.into(),
                        width: 0.0,
                    },
                    ..Default::default()
                }
            }
            ContainerStyle::TableEvenRow => {
                Style {
                    background: Some(BACKGROUND_DARKER.to_iced().into()),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ContainerStyle::WindowRoot => {
                Style {
                    background: Some(Background::Color(BACKGROUND_DARKEST.to_iced())),
                    text_color: Some(TEXT_LIGHTEST.to_iced()),
                    ..Default::default()
                }
            }
        }
    }
}
