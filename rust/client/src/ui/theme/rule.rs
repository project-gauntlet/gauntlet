use iced::widget::Rule;
use iced::widget::rule;
use iced::widget::rule::Style;

use crate::ui::theme::Element;
use crate::ui::theme::GauntletComplexTheme;
use crate::ui::theme::ThemableWidget;

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
                    snap: false,
                }
            }
            RuleStyle::ActionPanel => {
                Style {
                    color: self.separator.color,
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Percent(96.0),
                    snap: false,
                }
            }
            RuleStyle::PrimaryActionSeparator => {
                Style {
                    color: self.separator.color,
                    width: 1,
                    radius: 0.0.into(),
                    fill_mode: rule::FillMode::Percent(70.0),
                    snap: false,
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
