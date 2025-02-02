use iced_aw::number_input::number_input;
use iced_aw::number_input::Style;
use iced_aw::style::Status;

use crate::theme::GauntletSettingsTheme;
use crate::theme::PRIMARY;
use crate::theme::PRIMARY_HOVERED;
use crate::theme::TEXT_DARKER;
use crate::theme::TEXT_LIGHTEST;

impl number_input::ExtendedCatalog for GauntletSettingsTheme {
    fn style(&self, class: &(), status: Status) -> Style {
        number_input::Catalog::style(self, class, status)
    }
}

impl number_input::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => active(),
            Status::Hovered => active(), // TODO proper style
            Status::Pressed => pressed(),
            Status::Disabled => disabled(),
            Status::Focused => active(),   // TODO proper style
            Status::Selected => pressed(), // TODO proper style
        }
    }
}

fn active() -> Style {
    Style {
        button_background: Some(PRIMARY.to_iced().into()),
        icon_color: TEXT_DARKER.to_iced(),
    }
}

fn pressed() -> Style {
    Style {
        button_background: Some(PRIMARY_HOVERED.to_iced().into()),
        icon_color: TEXT_DARKER.to_iced(),
    }
}

fn disabled() -> Style {
    Style {
        button_background: None,
        icon_color: TEXT_LIGHTEST.to_iced(),
    }
}
