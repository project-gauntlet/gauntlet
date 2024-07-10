use iced::widget::text;

use crate::theme::GauntletSettingsTheme;

#[derive(Default, Clone)]
pub enum TextStyle {
    #[default]
    Default
}

impl text::StyleSheet for GauntletSettingsTheme {
    type Style = TextStyle;

    fn appearance(&self, _: Self::Style) -> text::Appearance {
        text::Appearance {
            color: None,
        }
    }
}
