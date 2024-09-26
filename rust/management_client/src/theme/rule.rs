use iced::widget::rule;
use crate::theme::{GauntletSettingsTheme, BACKGROUND_DARKER};

#[derive(Default)]
pub enum RuleStyle {
    #[default]
    Default,
}

impl rule::StyleSheet for GauntletSettingsTheme {
    type Style = RuleStyle;

    fn appearance(&self, _: &Self::Style) -> rule::Appearance {
        rule::Appearance {
            color: BACKGROUND_DARKER.to_iced(),
            width: 1,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}
