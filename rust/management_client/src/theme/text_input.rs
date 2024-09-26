use crate::theme::{GauntletSettingsTheme, BACKGROUND_DARKER, NOT_INTENDED_TO_BE_USED, TEXT_DARKER, TEXT_LIGHTEST, TRANSPARENT};
use iced::widget::text_input;
use iced::widget::text_input::Appearance;
use iced::{Border, Color};

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
                        color: BACKGROUND_DARKER.to_iced().into(),
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
                    background: BACKGROUND_DARKER.to_iced().into(),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: BACKGROUND_DARKER.to_iced().into(),
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            }
        }
    }

    fn disabled(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: BACKGROUND_DARKER.to_iced().into(),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: BACKGROUND_DARKER.to_iced().into(),
            },
            icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        TEXT_DARKER.to_iced()
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        TEXT_LIGHTEST.to_iced()
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.placeholder_color(style)
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        BACKGROUND_DARKER.to_iced()
    }
}