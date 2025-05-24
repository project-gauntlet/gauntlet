use iced::Border;
use iced::widget::checkbox;
use iced::widget::checkbox::Status;
use iced::widget::checkbox::Style;

use crate::theme::BACKGROUND_DARKER;
use crate::theme::BACKGROUND_DARKEST;
use crate::theme::BACKGROUND_LIGHTER;
use crate::theme::GauntletSettingsTheme;
use crate::theme::PRIMARY;
use crate::theme::PRIMARY_HOVERED;

impl checkbox::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active { is_checked } => active(is_checked),
            Status::Hovered { is_checked } => hovered(is_checked),
            Status::Disabled { is_checked } => disabled(is_checked),
        }
    }
}

fn active(is_checked: bool) -> Style {
    let background = if is_checked {
        PRIMARY.to_iced().into()
    } else {
        BACKGROUND_DARKEST.to_iced().into()
    };

    Style {
        background,
        icon_color: BACKGROUND_DARKEST.to_iced(),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: PRIMARY.to_iced().into(),
        },
        text_color: None,
    }
}

fn hovered(is_checked: bool) -> Style {
    let background = if is_checked {
        PRIMARY_HOVERED.to_iced().into()
    } else {
        BACKGROUND_DARKER.to_iced().into()
    };

    Style {
        background,
        icon_color: BACKGROUND_DARKEST.to_iced(),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: PRIMARY.to_iced().into(),
        },
        text_color: None,
    }
}

fn disabled(is_checked: bool) -> Style {
    let background = if is_checked {
        BACKGROUND_LIGHTER.to_iced().into()
    } else {
        BACKGROUND_DARKER.to_iced().into()
    };

    Style {
        background,
        icon_color: BACKGROUND_DARKEST.to_iced(),
        border: Default::default(),
        text_color: None,
    }
}
