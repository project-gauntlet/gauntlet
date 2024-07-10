use iced::{Border, Color};
use iced::widget::text_input;
use iced::widget::text_input::Appearance;
use crate::theme::{BACKGROUND_LIGHT, GauntletSettingsTheme, NOT_INTENDED_TO_BE_USED, TEXT, TEXT_DARKER, TRANSPARENT};

#[derive(Default)]
pub enum TextInputStyle {
    #[default]
    FormInput
}

//noinspection RsSortImplTraitMembers
impl text_input::StyleSheet for GauntletSettingsTheme {
    type Style = TextInputStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        match style {
            TextInputStyle::FormInput => {
                Appearance {
                    background: TRANSPARENT.to_iced().into(),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: BACKGROUND_LIGHT.to_iced().into(),
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            }
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        match style {
            TextInputStyle::FormInput => {
                Appearance {
                    background: BACKGROUND_LIGHT.to_iced().into(),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: BACKGROUND_LIGHT.to_iced().into(),
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            }
        }
    }

    fn disabled(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: NOT_INTENDED_TO_BE_USED.to_iced().into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        TEXT_DARKER.to_iced()
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        TEXT.to_iced()
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.placeholder_color(style)
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        BACKGROUND_LIGHT.to_iced()
    }
}