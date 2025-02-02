use iced::widget::text_input;
use iced::widget::text_input::Status;
use iced::widget::text_input::Style;
use iced::Background;
use iced::Border;

use crate::theme::GauntletSettingsTheme;
use crate::theme::BACKGROUND_DARKER;
use crate::theme::TEXT_DARKER;
use crate::theme::TEXT_LIGHTEST;
use crate::theme::TRANSPARENT;

pub enum TextInputStyle {
    FormInput,
}

impl text_input::Catalog for GauntletSettingsTheme {
    type Class<'a> = TextInputStyle;

    fn default<'a>() -> Self::Class<'a> {
        TextInputStyle::FormInput
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        let active = Style {
            background: Background::Color(TRANSPARENT.to_iced().into()),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: BACKGROUND_DARKER.to_iced().into(),
            },
            icon: TEXT_LIGHTEST.to_iced(),
            placeholder: TEXT_DARKER.to_iced(),
            value: TEXT_LIGHTEST.to_iced(),
            selection: BACKGROUND_DARKER.to_iced(),
        };

        match status {
            Status::Active => active,
            Status::Hovered => {
                Style {
                    background: Background::Color(BACKGROUND_DARKER.to_iced().into()),
                    ..active
                }
            }
            Status::Focused => {
                Style {
                    background: Background::Color(BACKGROUND_DARKER.to_iced().into()),
                    ..active
                }
            }
            Status::Disabled => {
                Style {
                    background: Background::Color(BACKGROUND_DARKER.to_iced().into()),
                    value: active.placeholder,
                    ..active
                }
            }
        }
    }
}
