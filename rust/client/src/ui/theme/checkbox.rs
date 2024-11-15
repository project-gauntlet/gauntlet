use checkbox::Appearance;
use iced::{Border, Renderer};
use iced::widget::{checkbox, Checkbox};

use crate::ui::theme::{Element, GauntletComplexTheme, get_theme, NOT_INTENDED_TO_BE_USED, ThemableWidget};

#[derive(Default)]
pub enum CheckboxStyle {
    #[default]
    Default,
}

impl checkbox::StyleSheet for GauntletComplexTheme {
    type Style = CheckboxStyle;

    fn active(&self, _: &Self::Style, is_checked: bool) -> Appearance {
        let theme = &self.form_input_checkbox;

        let background = if is_checked {
            theme.background_color_checked.to_iced().into()
        } else {
            theme.background_color_unchecked.to_iced().into()
        };

        Appearance {
            background,
            icon_color: theme.icon_color.to_iced(),
            border: Border {
                radius: theme.border_radius.into(),
                width: theme.border_width,
                color: theme.border_color.to_iced().into(),
            },
            text_color: None,
        }
    }

    fn hovered(&self, _: &Self::Style, is_checked: bool) -> Appearance {
        let theme = &self.form_input_checkbox;

        let background = if is_checked {
            theme.background_color_checked_hovered.to_iced().into()
        } else {
            theme.background_color_unchecked_hovered.to_iced().into()
        };

        Appearance {
            background,
            icon_color: theme.icon_color.to_iced(),
            border: Border {
                radius: theme.border_radius.into(),
                width: theme.border_width,
                color: theme.border_color.to_iced().into(),
            },
            text_color: None,
        }
    }

    fn disabled(&self, _: &Self::Style, _is_checked: bool) -> Appearance {
        Appearance {
            background: NOT_INTENDED_TO_BE_USED.to_iced().into(),
            icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: NOT_INTENDED_TO_BE_USED.to_iced(),
            },
            text_color: None,
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Checkbox<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = CheckboxStyle;

    fn themed(self, style: CheckboxStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match style {
            CheckboxStyle::Default => {
                self.style(style)
                    // .spacing() // TODO
                    // .size()
            }
        }.into()
    }
}
