use iced::widget::text;

use crate::theme::{DANGER_BRIGHT, GauntletSettingsTheme, SUCCESS, TEXT_DARKER};

#[derive(Default, Clone)]
pub enum TextStyle {
    #[default]
    Default,
    Subtitle,
    Positive,
    Destructive,
}

impl text::StyleSheet for GauntletSettingsTheme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            TextStyle::Default => {
                text::Appearance {
                    color: None,
                }
            }
            TextStyle::Subtitle => {
                text::Appearance {
                    color: Some(TEXT_DARKER.to_iced()),
                }
            }
            TextStyle::Positive => {
                text::Appearance {
                    color: Some(SUCCESS.to_iced()),
                }
            }
            TextStyle::Destructive => {
                text::Appearance {
                    color: Some(DANGER_BRIGHT.to_iced()),
                }
            }
        }
    }
}
