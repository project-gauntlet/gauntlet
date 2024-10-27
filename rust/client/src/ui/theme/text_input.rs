use iced::widget::{text_input, TextInput};
use iced::{Border, Color, Renderer};
use text_input::Appearance;

use crate::ui::theme::{Element, GauntletTheme, ThemableWidget, NOT_INTENDED_TO_BE_USED};

#[derive(Default)]
pub enum TextInputStyle {
    #[default]
    ShouldNotBeUsed,

    MainSearch,
    PluginSearchBar,
    FormInput,
}

// noinspection RsSortImplTraitMembers
impl text_input::StyleSheet for GauntletTheme {
    type Style = TextInputStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        match style {
            TextInputStyle::ShouldNotBeUsed => {
                Appearance {
                    background: NOT_INTENDED_TO_BE_USED.to_iced().into(),
                    border: Border {
                        color: NOT_INTENDED_TO_BE_USED.to_iced().into(),
                        ..Border::default()
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            },
            TextInputStyle::FormInput => {
                let theme = &self.form_input_text_field;

                Appearance {
                    background: theme.background_color.to_iced().into(),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color.to_iced().into(),
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            },
            TextInputStyle::MainSearch | TextInputStyle::PluginSearchBar => {
                Appearance {
                    background: Color::TRANSPARENT.into(),
                    border: Border {
                        color: Color::TRANSPARENT,
                        ..Border::default()
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        match style {
            TextInputStyle::ShouldNotBeUsed => {
                Appearance {
                    background: NOT_INTENDED_TO_BE_USED.to_iced().into(),
                    border: Border {
                        color: NOT_INTENDED_TO_BE_USED.to_iced().into(),
                        ..Border::default()
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            },
            TextInputStyle::FormInput => {
                let theme = &self.form_input_text_field;

                Appearance {
                    background: theme.background_color_hovered.to_iced().into(),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color_hovered.to_iced().into(),
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            },
            TextInputStyle::MainSearch | TextInputStyle::PluginSearchBar => {
                Appearance {
                    background: Color::TRANSPARENT.into(),
                    border: Border {
                        color: Color::TRANSPARENT,
                        ..Border::default()
                    },
                    icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
                }
            },
        }
    }

    fn disabled(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: NOT_INTENDED_TO_BE_USED.to_iced().into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: Color::TRANSPARENT,
            },
            icon_color: NOT_INTENDED_TO_BE_USED.to_iced(),
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        self.form_input_text_field.text_color_placeholder.to_iced()
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        self.form_input_text_field.text_color.to_iced()
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        self.placeholder_color(style)
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        self.form_input_text_field.selection_color.to_iced()
    }
}

impl<'a, Message: 'a + Clone> ThemableWidget<'a, Message> for TextInput<'a, Message, GauntletTheme, Renderer> {
    type Kind = TextInputStyle;

    fn themed(self, kind: TextInputStyle) -> Element<'a, Message> {
        match kind {
            TextInputStyle::PluginSearchBar => {
                self.style(kind)
                    .padding(0)
                    .into()
            }
            _ => {
                self.style(kind)
                    // .padding() // TODO
                    .into()
            }
        }
    }
}