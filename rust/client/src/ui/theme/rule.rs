use iced::widget::{rule, Rule};
use rule::Appearance;

use crate::ui::theme::{Element, GauntletTheme, get_theme, ThemableWidget};

#[derive(Default)]
pub enum RuleStyle {
    #[default]
    Default,
    ActionPanel,
    DefaultActionSeparator,
}

impl rule::StyleSheet for GauntletTheme {
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
            RuleStyle::DefaultActionSeparator => {
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

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Rule<GauntletTheme> {
    type Kind = RuleStyle;

    fn themed(self, kind: RuleStyle) -> Element<'a, Message> {
        self.style(kind).into()
    }
}
