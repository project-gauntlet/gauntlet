use iced::widget::text;
use iced::widget::text::Style;

use crate::ui::settings::theme::DANGER_BRIGHT;
use crate::ui::settings::theme::GauntletSettingsTheme;
use crate::ui::settings::theme::SUCCESS;
use crate::ui::settings::theme::TEXT_DARKER;

pub enum TextStyle {
    Default,
    Subtitle,
    Positive,
    Destructive,
}

impl text::Catalog for GauntletSettingsTheme {
    type Class<'a> = TextStyle;

    fn default<'a>() -> Self::Class<'a> {
        TextStyle::Default
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            TextStyle::Default => Style { color: None },
            TextStyle::Subtitle => {
                Style {
                    color: Some(TEXT_DARKER.to_iced()),
                }
            }
            TextStyle::Positive => {
                Style {
                    color: Some(SUCCESS.to_iced()),
                }
            }
            TextStyle::Destructive => {
                Style {
                    color: Some(DANGER_BRIGHT.to_iced()),
                }
            }
        }
    }
}
