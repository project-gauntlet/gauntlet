use iced::Background;
use iced::Border;
use iced::widget::text_input;
use iced::widget::text_input::Status;
use iced::widget::text_input::Style;

use crate::ui::settings::theme::BACKGROUND_DARKER;
use crate::ui::settings::theme::BACKGROUND_LIGHTER;
use crate::ui::settings::theme::BACKGROUND_LIGHTEST;
use crate::ui::settings::theme::BUTTON_BORDER_RADIUS;
use crate::ui::settings::theme::GauntletSettingsTheme;
use crate::ui::settings::theme::TEXT_DARKER;
use crate::ui::settings::theme::TEXT_LIGHTEST;
use crate::ui::settings::theme::TRANSPARENT;

pub enum TextInputStyle {
    FormInput,
    EntrypointAlias,
}

impl text_input::Catalog for GauntletSettingsTheme {
    type Class<'a> = TextInputStyle;

    fn default<'a>() -> Self::Class<'a> {
        TextInputStyle::FormInput
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match class {
            TextInputStyle::EntrypointAlias => {
                let border = if let Status::Focused { is_hovered: _ } | Status::Hovered = status {
                    Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        width: 2.0,
                        color: BACKGROUND_LIGHTER.to_iced(),
                    }
                } else {
                    Border::default()
                };

                return Style {
                    background: Background::Color(TRANSPARENT.to_iced().into()),
                    border,
                    icon: TEXT_LIGHTEST.to_iced(),
                    placeholder: TEXT_DARKER.to_iced(),
                    value: TEXT_LIGHTEST.to_iced(),
                    selection: BACKGROUND_LIGHTEST.to_iced(),
                };
            }
            TextInputStyle::FormInput => {}
        }

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
            selection: BACKGROUND_LIGHTEST.to_iced(),
        };

        match status {
            Status::Active => active,
            Status::Hovered => {
                Style {
                    background: Background::Color(BACKGROUND_DARKER.to_iced().into()),
                    ..active
                }
            }
            Status::Focused { is_hovered: _ } => {
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
