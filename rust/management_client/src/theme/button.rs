use iced::Border;
use iced::widget::button;
use iced::widget::button::Appearance;

use crate::theme::{BACKGROUND_LIGHT, BACKGROUND_LIGHTER, BUTTON_BORDER_RADIUS, DANGER, GauntletSettingsTheme, PRIMARY, PRIMARY_HOVERED, SUCCESS, TEXT, TEXT_DARK};

#[derive(Default)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Positive,
    Destructive,
    TableRow,
    ViewSwitcher,
    ViewSwitcherSelected,
}

//noinspection RsSortImplTraitMembers
impl button::StyleSheet for GauntletSettingsTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        let (background_color, text_color) = match style {
            ButtonStyle::Primary => (PRIMARY.to_iced(), TEXT_DARK.to_iced()),
            ButtonStyle::Positive => (SUCCESS.to_iced(), TEXT_DARK.to_iced()),
            ButtonStyle::Destructive => (DANGER.to_iced(), TEXT.to_iced()),
            ButtonStyle::TableRow => {
                return Appearance {
                    background: None,
                    text_color: TEXT.to_iced(),
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcher => {
                return Appearance {
                    background: None,
                    text_color: TEXT.to_iced(),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcherSelected => {
                return Appearance {
                    background: Some(BACKGROUND_LIGHT.to_iced().into()),
                    text_color: TEXT.to_iced(),
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
            ButtonStyle::Primary => (PRIMARY_HOVERED.to_iced(), TEXT_DARK.to_iced()),
            ButtonStyle::Positive => (SUCCESS.to_iced(), TEXT_DARK.to_iced()), // TODO
            ButtonStyle::Destructive => (DANGER.to_iced(), TEXT.to_iced()), // TODO
            ButtonStyle::TableRow => {
                return Appearance {
                    background: None,
                    text_color: TEXT.to_iced(), // TODO
                    ..Default::default()
                }
            }
            ButtonStyle::ViewSwitcher => {
                return Appearance {
                    background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                    text_color: TEXT.to_iced(),
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
                    text_color: TEXT.to_iced(),
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
                return Appearance {
                    background: Some(BACKGROUND_LIGHT.to_iced().into()),
                    text_color: TEXT.to_iced(),
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

    fn focused(&self, style: &Self::Style, _is_active: bool) -> Appearance {
        self.hovered(style)
    }
}