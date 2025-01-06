use button::Style;
use iced::{Border, Color, Padding, Renderer};
use iced::widget::{button, Button};
use iced::widget::button::Status;
use crate::ui::theme::{Element, GauntletComplexTheme, get_theme, NOT_INTENDED_TO_BE_USED, padding_all, ThemableWidget};

#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    ShouldNotBeUsed,

    DatePicker,

    Action,
    ActionFocused,
    GridItem,
    GridItemFocused,
    ListItem,
    ListItemFocused,
    MainListItem,
    MainListItemFocused,
    MetadataLink,
    RootBottomPanelActionToggleButton,
    RootBottomPanelPrimaryActionButton,
    RootTopPanelBackButton,
    MetadataTagItem,
}

impl ButtonStyle {
    fn padding(&self) -> Padding {
        let theme = get_theme();

        match self {
            ButtonStyle::RootBottomPanelActionToggleButton => {
                let theme = &theme.root_bottom_panel_action_toggle_button;

                theme.padding.to_iced()
            },
            ButtonStyle::RootBottomPanelPrimaryActionButton => {
                let theme = &theme.root_bottom_panel_action_toggle_button;

                theme.padding.to_iced()
            },
            ButtonStyle::RootTopPanelBackButton => {
                let theme = &theme.root_top_panel_button;

                theme.padding.to_iced()
            },
            ButtonStyle::GridItem | ButtonStyle::GridItemFocused  => {
                let theme = &theme.grid_item;

                theme.padding.to_iced()
            }
            ButtonStyle::Action | ButtonStyle::ActionFocused => {
                let theme = &theme.action;

                theme.padding.to_iced()
            }
            ButtonStyle::ListItem | ButtonStyle::ListItemFocused => {
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

    fn appearance(&self, theme: &GauntletComplexTheme, state: Status) -> Style {
        let (background_color, background_color_hover, background_color_pressed, text_color, text_color_hover, border_radius, border_width, border_color) = match &self {
            ButtonStyle::RootBottomPanelPrimaryActionButton | ButtonStyle::RootBottomPanelActionToggleButton => {
                let theme = &theme.root_bottom_panel_action_toggle_button;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            },
            ButtonStyle::RootTopPanelBackButton => {
                let theme = &theme.root_top_panel_button;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color_hovered), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            },
            ButtonStyle::GridItem => {
                let theme = &theme.grid_item;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::GridItemFocused => {
                let theme = &theme.grid_item;
                (Some(&theme.background_color_focused), Some(&theme.background_color_focused), Some(&theme.background_color), &theme.text_color_hovered, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::Action => {
                let theme = &theme.action;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::ActionFocused => {
                let theme = &theme.action;
                (Some(&theme.background_color_focused), Some(&theme.background_color_focused), Some(&theme.background_color), &theme.text_color_hovered, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::ListItem => {
                let theme = &theme.list_item;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::ListItemFocused => {
                let theme = &theme.list_item;
                (Some(&theme.background_color_focused), Some(&theme.background_color_focused), Some(&theme.background_color), &theme.text_color_hovered, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::MainListItem => {
                let theme = &theme.main_list_item;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::MainListItemFocused => {
                let theme = &theme.main_list_item;
                (Some(&theme.background_color_focused), Some(&theme.background_color_focused), Some(&theme.background_color), &theme.text_color_hovered, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::MetadataLink => {
                let theme = &theme.metadata_link;
                (None, None, None, &theme.text_color, &theme.text_color_hovered, &0.0, &1.0, &Color::TRANSPARENT)
            }
            ButtonStyle::MetadataTagItem => {
                let theme = &theme.metadata_tag_item_button;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
            ButtonStyle::ShouldNotBeUsed => {
                (Some(&NOT_INTENDED_TO_BE_USED), Some(&NOT_INTENDED_TO_BE_USED), Some(&NOT_INTENDED_TO_BE_USED), &NOT_INTENDED_TO_BE_USED, &NOT_INTENDED_TO_BE_USED, &0.0, &1.0, &Color::TRANSPARENT)
            }
            ButtonStyle::DatePicker => {
                let theme = &theme.form_input_date_picker_buttons;
                (Some(&theme.background_color), Some(&theme.background_color_hovered), Some(&theme.background_color), &theme.text_color, &theme.text_color_hovered, &theme.border_radius, &theme.border_width, &theme.border_color)
            }
        };

        let active = Style {
            background: background_color.map(|color| color.clone().into()),
            text_color: text_color.clone(),
            border: Border {
                color: border_color.clone(),
                width: (*border_width).into(),
                radius: (*border_radius).into(),
            },
            ..Default::default()
        };

        match state {
            Status::Active => active,
            Status::Pressed => {
                Style {
                    background: background_color_pressed.map(|color| color.clone().into()),
                    text_color: text_color_hover.clone(),
                    ..active
                }
            }
            Status::Hovered => {
                Style {
                    background: background_color_hover.map(|color| color.clone().into()),
                    text_color: text_color_hover.clone(),
                    ..active
                }
            }
            Status::Disabled => {
                Style {
                    background: Some(NOT_INTENDED_TO_BE_USED.into()),
                    ..active
                }
            }
        }
    }
}

impl button::Catalog for GauntletComplexTheme {
    type Class<'a> = ButtonStyle;

    fn default<'a>() -> Self::Class<'a> {
        // TODO Not supposed to be default but unable to customize datepicker buttons right now
        // ButtonStyle::ShouldNotBeUsed
        ButtonStyle::DatePicker
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class.appearance(self, status)
    }
}

impl<'a, Message: 'a + Clone> ThemableWidget<'a, Message> for Button<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = ButtonStyle;

    fn themed(self, kind: ButtonStyle) -> Element<'a, Message> {
        self.class(kind).padding(kind.padding()).into()
    }
}