use iced_aw::style::spinner;
use crate::theme::GauntletSettingsTheme;

#[derive(Default)]
pub enum SpinnerStyle {
    #[default]
    Default,
}

impl spinner::StyleSheet for GauntletSettingsTheme {
    type Style = SpinnerStyle;

    fn appearance(&self, _style: &Self::Style) -> spinner::Appearance {
        spinner::Appearance {}
    }
}
