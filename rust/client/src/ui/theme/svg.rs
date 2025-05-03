use iced::widget::svg;
use iced::widget::svg::Status;
use iced::widget::svg::Style;

use crate::ui::GauntletComplexTheme;

impl svg::Catalog for GauntletComplexTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, _status: Status) -> Style {
        Style { color: None }
    }
}
