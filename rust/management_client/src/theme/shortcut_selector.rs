use iced::widget::container::Style;
use iced::Border;

use crate::components::shortcut_selector;
use crate::components::shortcut_selector::Status;
use crate::theme::GauntletSettingsTheme;
use crate::theme::BACKGROUND_DARKER;
use crate::theme::BUTTON_BORDER_RADIUS;
use crate::theme::PRIMARY;

impl shortcut_selector::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => {
                Style {
                    background: Some(BACKGROUND_DARKER.to_iced().into()),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            Status::Capturing => {
                Style {
                    background: Some(BACKGROUND_DARKER.to_iced().into()),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        width: 2.0,
                        color: PRIMARY.to_iced(),
                    },
                    ..Default::default()
                }
            }
        }
    }
}
