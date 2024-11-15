use iced::widget::{rule, Rule};
use rule::Appearance;

use crate::ui::theme::{Element, GauntletComplexTheme, get_theme, ThemableWidget};

#[derive(Default)]
pub enum RuleStyle {
    #[default]
    Default,
    ActionPanel,
    PrimaryActionSeparator,
}

impl rule::StyleSheet for GauntletComplexTheme {
    type Style = RuleStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let theme = get_theme();

        match style {
            RuleStyle::Default => {
                Appearance {
                    color: theme.separator.color.to_iced(),
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Full,
                }
            }
            RuleStyle::ActionPanel => {
                Appearance {
                    color: theme.separator.color.to_iced(),
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Percent(96.0),
                }
            }
            RuleStyle::PrimaryActionSeparator => {
                Appearance {
                    color: theme.separator.color.to_iced(),
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Percent(70.0),
                }
            }
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Rule<GauntletComplexTheme> {
    type Kind = RuleStyle;

    fn themed(self, kind: RuleStyle) -> Element<'a, Message> {
        self.style(kind).into()
    }
}
