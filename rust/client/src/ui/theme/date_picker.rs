use crate::ui::theme::{Element, GauntletComplexTheme, ThemableWidget};
use iced::Color;
use iced_aw::date_picker::Style;
use iced_aw::style::date_picker::Catalog;
use iced_aw::style::Status;
use iced_aw::DatePicker;

#[derive(Clone, Default)]
pub enum DatePickerStyle {
    #[default]
    Default,
}

impl Catalog for GauntletComplexTheme {
    type Class<'a> = DatePickerStyle;

    fn default<'a>() -> Self::Class<'a> {
        DatePickerStyle::Default
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => active(self),
            Status::Hovered => hovered(self),
            Status::Pressed => hovered(self), // TODO proper styling
            Status::Disabled => hovered(self), // TODO proper styling
            Status::Focused => focused(self),
            Status::Selected => selected(self)
        }
    }
}


fn active(theme: &GauntletComplexTheme) -> Style {
    let root_theme = &theme.popup;
    let theme = &theme.form_input_date_picker;

    Style {
        background: theme.background_color.into(),
        border_radius: root_theme.border_radius,
        border_width: root_theme.border_width,
        border_color: root_theme.border_color,
        text_color: theme.text_color,
        text_attenuated_color: theme.text_attenuated_color,
        day_background: theme.day_background_color.into(),
    }
}

fn selected(theme: &GauntletComplexTheme) -> Style {
    let form_theme = &theme.form_input_date_picker;

    Style {
        day_background: form_theme.day_background_color_selected.into(),
        text_color: form_theme.text_color_selected,
        ..active(theme)
    }
}

fn hovered(theme: &GauntletComplexTheme) -> Style {
    let form_theme = &theme.form_input_date_picker;

    Style {
        day_background: form_theme.day_background_color_hovered.into(),
        text_color: form_theme.text_color_hovered,
        ..active(theme)
    }
}

fn focused(theme: &GauntletComplexTheme) -> Style {
    Style {
        border_color: Color::from_rgb(0.5, 0.5, 0.5), // TODO move to theme?
        ..active(theme)
    }
}


impl<'a, Message: 'a + Clone + 'static> ThemableWidget<'a, Message> for DatePicker<'a, Message, GauntletComplexTheme> {
    type Kind = DatePickerStyle;

    fn themed(self, kind: DatePickerStyle) -> Element<'a, Message> {
        self.class(kind).into()
    }
}