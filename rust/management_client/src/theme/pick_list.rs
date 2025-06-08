use iced::Border;
use iced::overlay;
use iced::widget::pick_list;

use crate::theme::BACKGROUND_DARKER;
use crate::theme::BACKGROUND_DARKEST;
use crate::theme::BUTTON_BORDER_RADIUS;
use crate::theme::GauntletSettingsTheme;
use crate::theme::PRIMARY;
use crate::theme::PRIMARY_HOVERED;
use crate::theme::TEXT_DARKEST;
use crate::theme::TEXT_LIGHTEST;

impl pick_list::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        ()
    }

    fn style(&self, _class: &(), status: pick_list::Status) -> pick_list::Style {
        pick_list_appearance(status)
    }
}

fn pick_list_appearance(status: pick_list::Status) -> pick_list::Style {
    use iced::widget::pick_list::Status;

    let background_color = match status {
        Status::Active | Status::Opened { is_hovered: _ } => PRIMARY.to_iced(),
        Status::Hovered => PRIMARY_HOVERED.to_iced(),
    };

    let text_color = match status {
        Status::Active | Status::Opened { is_hovered: _ } => TEXT_DARKEST.to_iced(),
        Status::Hovered => TEXT_DARKEST.to_iced(),
    };

    pick_list::Style {
        text_color,
        background: background_color.into(),
        placeholder_color: BACKGROUND_DARKER.to_iced(),
        handle_color: text_color,
        border: Border {
            color: BACKGROUND_DARKER.to_iced(),
            width: 1.0,
            radius: BUTTON_BORDER_RADIUS.into(),
        },
    }
}

impl overlay::menu::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> <Self as overlay::menu::Catalog>::Class<'a> {
        ()
    }

    fn style(&self, _class: &()) -> overlay::menu::Style {
        overlay::menu::Style {
            text_color: TEXT_LIGHTEST.to_iced(),
            background: BACKGROUND_DARKEST.to_iced().into(),
            border: Border {
                radius: BUTTON_BORDER_RADIUS.into(),
                width: 1.0,
                color: BACKGROUND_DARKER.to_iced().into(),
            },
            selected_text_color: TEXT_LIGHTEST.to_iced(),
            selected_background: BACKGROUND_DARKER.to_iced().into(),
        }
    }
}
