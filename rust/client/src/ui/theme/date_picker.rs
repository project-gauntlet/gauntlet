use iced::Color;
use iced_aw::{date_picker, DatePicker};
use iced_aw::date_picker::Appearance;

use crate::ui::theme::{Element, GauntletComplexTheme, get_theme, ThemableWidget};

#[derive(Clone, Default)]
pub enum DatePickerStyle {
    #[default]
    Default,
}

impl date_picker::StyleSheet for GauntletComplexTheme {
    type Style = DatePickerStyle;

    fn active(&self, _: &Self::Style) -> Appearance {
        let theme = get_theme();
        let root_theme = &theme.root;
        let theme = &theme.form_input_date_picker;

        Appearance {
            background: theme.background_color.to_iced().into(),
            border_radius: root_theme.border_radius,
            border_width: root_theme.border_width,
            border_color: root_theme.border_color.to_iced(),
            text_color: theme.text_color.to_iced(),
            text_attenuated_color: theme.text_attenuated_color.to_iced(),
            day_background: theme.day_background_color.to_iced().into(),
        }
    }

    fn selected(&self, style: &Self::Style) -> Appearance {
        let theme = get_theme();
        let theme = &theme.form_input_date_picker;

        Appearance {
            day_background: theme.day_background_color_selected.to_iced().into(),
            text_color: theme.text_color_selected.to_iced(),
            ..self.active(style)
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let theme = get_theme();
        let theme = &theme.form_input_date_picker;

        Appearance {
            day_background: theme.day_background_color_hovered.to_iced().into(),
            text_color: theme.text_color_hovered.to_iced(),
            ..self.active(style)
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        Appearance {
            border_color: Color::from_rgb(0.5, 0.5, 0.5), // TODO move to theme?
            ..self.active(style)
        }
    }
}

impl<'a, Message: 'a + Clone + 'static> ThemableWidget<'a, Message> for DatePicker<'a, Message, GauntletComplexTheme> {
    type Kind = DatePickerStyle;

    fn themed(self, kind: DatePickerStyle) -> Element<'a, Message> {
        self.style(kind).into()
    }
}