use iced::widget::text_input::{Status, Style};
use iced::widget::{text_input, TextInput};
use iced::{Border, Color, Renderer};

use crate::ui::theme::{Element, GauntletComplexTheme, ThemableWidget, NOT_INTENDED_TO_BE_USED};

pub enum TextInputStyle {
    ShouldNotBeUsed,

    MainSearch,
    PluginSearchBar,
    FormInput,
}

impl text_input::Catalog for GauntletComplexTheme {
    type Class<'a> = TextInputStyle;

    fn default<'a>() -> Self::Class<'a> {
        TextInputStyle::ShouldNotBeUsed
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => active(self, class),
            Status::Hovered => focused(self, class), // TODO proper style
            Status::Focused => focused(self, class),
            Status::Disabled => disabled(),
        }
    }
}

fn active(theme: &GauntletComplexTheme, style: &TextInputStyle) -> Style {
    match style {
        TextInputStyle::ShouldNotBeUsed => {
            Style {
                background: NOT_INTENDED_TO_BE_USED.into(),
                border: Border {
                    color: NOT_INTENDED_TO_BE_USED.into(),
                    ..Border::default()
                },
                icon: NOT_INTENDED_TO_BE_USED,
                placeholder: NOT_INTENDED_TO_BE_USED,
                value: NOT_INTENDED_TO_BE_USED,
                selection: NOT_INTENDED_TO_BE_USED,
            }
        },
        TextInputStyle::FormInput => {
            let theme = &theme.form_input_text_field;

            Style {
                background: theme.background_color.into(),
                border: Border {
                    radius: theme.border_radius.into(),
                    width: theme.border_width,
                    color: theme.border_color.into(),
                },
                icon: NOT_INTENDED_TO_BE_USED,
                placeholder: theme.text_color_placeholder,
                value: theme.text_color,
                selection: theme.selection_color,
            }
        },
        TextInputStyle::MainSearch | TextInputStyle::PluginSearchBar => {
            Style {
                background: Color::TRANSPARENT.into(),
                border: Border {
                    color: Color::TRANSPARENT,
                    ..Border::default()
                },
                icon: NOT_INTENDED_TO_BE_USED,
                placeholder: theme.form_input_text_field.text_color_placeholder, // TODO fix
                value: theme.form_input_text_field.text_color, // TODO fix
                selection: theme.form_input_text_field.selection_color, // TODO fix
            }
        },
    }
}

fn focused(theme: &GauntletComplexTheme, style: &TextInputStyle) -> Style {
    match style {
        TextInputStyle::ShouldNotBeUsed => {
            Style {
                background: NOT_INTENDED_TO_BE_USED.into(),
                border: Border {
                    color: NOT_INTENDED_TO_BE_USED.into(),
                    ..Border::default()
                },
                icon: NOT_INTENDED_TO_BE_USED,
                placeholder: NOT_INTENDED_TO_BE_USED,
                value: NOT_INTENDED_TO_BE_USED,
                selection: NOT_INTENDED_TO_BE_USED,
            }
        },
        TextInputStyle::FormInput => {
            let theme = &theme.form_input_text_field;

            Style {
                background: theme.background_color_hovered.into(),
                border: Border {
                    radius: theme.border_radius.into(),
                    width: theme.border_width,
                    color: theme.border_color_hovered.into(),
                },
                icon: NOT_INTENDED_TO_BE_USED,
                placeholder: theme.text_color_placeholder,
                value: theme.text_color,
                selection: theme.selection_color,
            }
        },
        TextInputStyle::MainSearch | TextInputStyle::PluginSearchBar => {
            Style {
                background: Color::TRANSPARENT.into(),
                border: Border {
                    color: Color::TRANSPARENT,
                    ..Border::default()
                },
                icon: NOT_INTENDED_TO_BE_USED,
                placeholder: theme.form_input_text_field.text_color_placeholder, // TODO fix
                value: theme.form_input_text_field.text_color, // TODO fix
                selection: theme.form_input_text_field.selection_color, // TODO fix
            }
        },
    }
}

fn disabled() -> Style {
    Style {
        background: NOT_INTENDED_TO_BE_USED.into(),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: Color::TRANSPARENT,
        },
        icon: NOT_INTENDED_TO_BE_USED,
        placeholder: NOT_INTENDED_TO_BE_USED,
        value: NOT_INTENDED_TO_BE_USED,
        selection: NOT_INTENDED_TO_BE_USED,
    }
}

impl<'a, Message: 'a + Clone> ThemableWidget<'a, Message> for TextInput<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = TextInputStyle;

    fn themed(self, kind: TextInputStyle) -> Element<'a, Message> {
        match kind {
            TextInputStyle::PluginSearchBar => {
                self.class(kind)
                    .padding(0)
                    .into()
            }
            _ => {
                self.class(kind)
                    // .padding() // TODO
                    .into()
            }
        }
    }
}