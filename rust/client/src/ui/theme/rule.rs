use iced::widget::rule::Style;
use iced::widget::{rule, Rule};

use crate::ui::theme::{Element, GauntletComplexTheme, ThemableWidget};

pub enum RuleStyle {
    Default,
    ActionPanel,
    PrimaryActionSeparator,
}

impl rule::Catalog for GauntletComplexTheme {
    type Class<'a> = RuleStyle;

    fn default<'a>() -> Self::Class<'a> {
        RuleStyle::Default
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            RuleStyle::Default => {
                Style {
                    color: self.separator.color,
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Full,
                }
            }
            RuleStyle::ActionPanel => {
                Style {
                    color: self.separator.color,
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Percent(96.0),
                }
            }
            RuleStyle::PrimaryActionSeparator => {
                Style {
                    color: self.separator.color,
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Percent(70.0),
                }
            }
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Rule<'a, GauntletComplexTheme> {
    type Kind = RuleStyle;

    fn themed(self, kind: RuleStyle) -> Element<'a, Message> {
        self.class(kind).into()
    }
}
