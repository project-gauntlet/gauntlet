use iced::widget::button;
use iced::widget::button::Appearance;
use iced::Border;

use crate::theme::{GauntletSettingsTheme, BACKGROUND_DARKER, BACKGROUND_LIGHTER, BUTTON_BORDER_RADIUS, DANGER, PRIMARY, PRIMARY_HOVERED, SUCCESS, TEXT_DARKEST, TEXT_LIGHTEST};

#[derive(Default)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Positive,
    Destructive,
    TableRow,
    ViewSwitcher,
    ViewSwitcherSelected,
    DownloadInfo,
}

//noinspection RsSortImplTraitMembers
impl button::StyleSheet for GauntletSettingsTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        let (background_color, text_color) = match style {
            ButtonStyle::Primary => (PRIMARY.to_iced(), TEXT_DARKEST.to_iced()),
            ButtonStyle::Positive => (SUCCESS.to_iced(), TEXT_DARKEST.to_iced()),
            ButtonStyle::Destructive => (DANGER.to_iced(), TEXT_LIGHTEST.to_iced()),
            ButtonStyle::TableRow => {
                return Appearance {
                    background: None,
                    text_color: TEXT_LIGHTEST.to_iced(),
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcher => {
                return Appearance {
                    background: None,
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcherSelected => {
                return Appearance {
                    background: Some(BACKGROUND_DARKER.to_iced().into()),
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ButtonStyle::DownloadInfo => {
                return Appearance {
                    background: None,
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        };

        Appearance {
            background: Some(background_color.into()),
            text_color,
            border: Border {
                radius: BUTTON_BORDER_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let (background_color, text_color) = match style {
            ButtonStyle::Primary => (PRIMARY_HOVERED.to_iced(), TEXT_DARKEST.to_iced()),
            ButtonStyle::Positive => (SUCCESS.to_iced(), TEXT_DARKEST.to_iced()), // TODO
            ButtonStyle::Destructive => (DANGER.to_iced(), TEXT_LIGHTEST.to_iced()), // TODO
            ButtonStyle::TableRow => {
                return Appearance {
                    background: None,
                    text_color: TEXT_LIGHTEST.to_iced(), // TODO
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcher => {
                return Appearance {
                    background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcherSelected => {
                return Appearance {
                    background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ButtonStyle::DownloadInfo => {
                return Appearance {
                    background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        };

        Appearance {
            background: Some(background_color.into()),
            text_color,
            border: Border {
                radius: BUTTON_BORDER_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::ViewSwitcher | ButtonStyle::ViewSwitcherSelected => {
                Appearance {
                    background: Some(BACKGROUND_DARKER.to_iced().into()),
                    text_color: TEXT_LIGHTEST.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            _ => {
                self.active(style)
            }
        }
    }
}