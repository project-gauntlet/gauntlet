use iced::widget::text;

use crate::theme::{GauntletSettingsTheme, TEXT_DARKER};

#[derive(Default, Clone)]
pub enum TextStyle {
    #[default]
    Default,
    Subtitle
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
        }
    }
}
