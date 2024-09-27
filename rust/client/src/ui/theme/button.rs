use button::Appearance;
use iced::{Border, Padding, Renderer};
use iced::widget::{button, Button};
use crate::ui::theme::{Element, GauntletTheme, get_theme, NOT_INTENDED_TO_BE_USED, padding_all, ThemableWidget, TRANSPARENT};

#[derive(Default, Debug, Clone, Copy)]
pub enum ButtonStyle {
    // #[default]
    ShouldNotBeUsed,

    #[default] // TODO Not supposed to be default but unable to customize datepicker buttons right now
    DatePicker,

    Action,
    GridItem,
    ListItem,
    MainListItem,
    MainListItemFocused,
    MetadataLink,
    RootBottomPanelActionToggleButton,
    RootTopPanelBackButton,
    MetadataTagItem,
}

enum ButtonState {
    Active,
    Hover
}

impl ButtonStyle {
    fn padding(&self) -> Padding {
        let theme = get_theme();

        match self {
            ButtonStyle::RootBottomPanelActionToggleButton => {
                let theme = &theme.root_bottom_panel_action_toggle_button;

                theme.padding.to_iced()
            },
            ButtonStyle::RootTopPanelBackButton => {
                let theme = &theme.root_top_panel_button;

                theme.padding.to_iced()
            },
            ButtonStyle::GridItem => {
                let theme = &theme.grid_item;

                theme.padding.to_iced()
            }
            ButtonStyle::Action => {
                let theme = &theme.action;

                theme.padding.to_iced()
            }
            ButtonStyle::ListItem => {
                let theme = &theme.list_item;

                theme.padding.to_iced()
            }
            ButtonStyle::MainListItem | ButtonStyle::MainListItemFocused => {
                let theme = &theme.main_list_item;

                theme.padding.to_iced()
            }
            ButtonStyle::MetadataLink => {
                padding_all(0.0).to_iced()
            }
            ButtonStyle::MetadataTagItem => {
                let theme = &theme.metadata_tag_item_button;
                theme.padding.to_iced()
            }
            ButtonStyle::ShouldNotBeUsed => {
                padding_all(5.0).to_iced()
            }
            ButtonStyle::DatePicker => {
                padding_all(5.0).to_iced()
            }
        }
    }

    fn appearance(&self, theme: &GauntletTheme, state: ButtonState) -> Appearance {
        let (background_color, background_color_hover, text_color, text_color_hover, border_radius, border_width, border_color) = match &self {
            ButtonStyle::RootBottomPanelActionToggleButton => {
                let theme = &theme.root_bottom_panel_action_toggle_button;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            },
            ButtonStyle::RootTopPanelBackButton => {
                let theme = &theme.root_top_panel_button;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            },
            ButtonStyle::GridItem => {
                let theme = &theme.grid_item;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::Action => {
                let theme = &theme.action;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::ListItem => {
                let theme = &theme.list_item;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::MainListItem => {
                let theme = &theme.main_list_item;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::MainListItemFocused => {
                let theme = &theme.main_list_item;
                (Some(&theme.background_color_hovered), Some(&theme.background_color_hovered), &theme.text_color_hovered, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::MetadataLink => {
                let theme = &theme.metadata_link;
                (None, None, &theme.text_color, &theme.text_color_hovered, &0.0, &1.0, &TRANSPARENT)
            }
            ButtonStyle::MetadataTagItem => {
                let theme = &theme.metadata_tag_item_button;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::ShouldNotBeUsed => {
                (Some(&NOT_INTENDED_TO_BE_USED), Some(&NOT_INTENDED_TO_BE_USED), &NOT_INTENDED_TO_BE_USED, &NOT_INTENDED_TO_BE_USED, &0.0, &1.0, &TRANSPARENT)
            }
            ButtonStyle::DatePicker => {
                let theme = &theme.form_input_date_picker_buttons;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
        };

        let active = Appearance {
            background: background_color.map(|color| color.to_iced().into()),
            text_color: text_color.to_iced(),
            border: Border {
                color: border_color.to_iced(),
                width: (*border_width).into(),
                radius: (*border_radius).into(),
            },
            ..Default::default()
        };

        match state {
            ButtonState::Active => active,
            ButtonState::Hover => {
                Appearance {
                    background: background_color_hover.map(|color| color.to_iced().into()),
                    text_color: text_color_hover.to_iced(),
                    ..active
                }
            }
        }
    }
}

impl button::StyleSheet for GauntletTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        style.appearance(self, ButtonState::Active)
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        style.appearance(self, ButtonState::Hover)
    }
}

impl<'a, Message: 'a + Clone> ThemableWidget<'a, Message> for Button<'a, Message, GauntletTheme, Renderer> {
    type Kind = ButtonStyle;

    fn themed(self, kind: ButtonStyle) -> Element<'a, Message> {
        self.style(kind).padding(kind.padding()).into()
    }
}