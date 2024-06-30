use iced::widget::rule;
use rule::Appearance;

use crate::ui::theme::{GauntletTheme, get_theme};

impl rule::StyleSheet for GauntletTheme {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> Appearance {
        let theme = get_theme();

        Appearance {
            color: theme.separator.color.to_iced(),
            width: 1,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}
