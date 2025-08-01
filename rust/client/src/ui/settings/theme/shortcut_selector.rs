use iced::Border;
use iced::widget::container::Style;

use crate::ui::settings::components::shortcut_selector;
use crate::ui::settings::components::shortcut_selector::Status;
use crate::ui::settings::theme::BACKGROUND_DARKER;
use crate::ui::settings::theme::BACKGROUND_LIGHTER;
use crate::ui::settings::theme::BUTTON_BORDER_RADIUS;
use crate::ui::settings::theme::GauntletSettingsTheme;
use crate::ui::settings::theme::PRIMARY;
use crate::ui::settings::theme::TRANSPARENT;

impl shortcut_selector::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status, transparent_background: bool) -> Style {
        let background = if transparent_background {
            TRANSPARENT.to_iced().into()
        } else {
            BACKGROUND_DARKER.to_iced().into()
        };

        match status {
            Status::Active => {
                Style {
                    background: Some(background),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            Status::Capturing => {
                Style {
                    background: Some(background),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        width: 2.0,
                        color: PRIMARY.to_iced(),
                    },
                    ..Default::default()
                }
            }
            Status::Hovered => {
                Style {
                    background: Some(background),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        width: 2.0,
                        color: BACKGROUND_LIGHTER.to_iced(),
                    },
                    ..Default::default()
                }
            }
        }
    }
}
