use iced::{Border, overlay};
use iced::widget::pick_list;

use crate::theme::{BUTTON_BORDER_RADIUS, GauntletSettingsTheme, PRIMARY, PRIMARY_HOVERED, TEXT_DARKEST, BACKGROUND_DARKER, BACKGROUND_DARKEST, TEXT_LIGHTEST};

#[derive(Clone, Default)]
pub enum PickListStyle {
    #[default]
    Default,
}

#[derive(Clone, Default)]
pub enum MenuStyle {
    #[default]
    Default,
}

impl pick_list::StyleSheet for GauntletSettingsTheme {
    type Style = PickListStyle;

    fn active(&self, _: &Self::Style) -> pick_list::Appearance {
        pick_list_appearance(PickListState::Active)
    }

    fn hovered(&self, _: &Self::Style) -> pick_list::Appearance {
        pick_list_appearance(PickListState::Hovered)
    }
}

enum PickListState {
    Active,
    Hovered
}

fn pick_list_appearance(state: PickListState) -> pick_list::Appearance {
    let background_color = match state {
        PickListState::Active => PRIMARY.to_iced(),
        PickListState::Hovered => PRIMARY_HOVERED.to_iced(),
    };

    let text_color = match state {
        PickListState::Active => TEXT_DARKEST.to_iced(),
        PickListState::Hovered => TEXT_DARKEST.to_iced(),
    };

    pick_list::Appearance {
        text_color,
        background: background_color.into(),
        placeholder_color: BACKGROUND_DARKER.to_iced(),
        handle_color: text_color,
        border: Border {
            color: BACKGROUND_DARKER.to_iced(),
            width: 1.0,
            radius: BUTTON_BORDER_RADIUS.into(),
        },
    }
}

impl overlay::menu::StyleSheet for GauntletSettingsTheme {
    type Style = MenuStyle;

    fn appearance(&self, _: &Self::Style) -> overlay::menu::Appearance {
        overlay::menu::Appearance {
            text_color: TEXT_LIGHTEST.to_iced(),
            background: BACKGROUND_DARKEST.to_iced().into(),
            border: Border {
                radius: BUTTON_BORDER_RADIUS.into(),
                width: 1.0,
                color: BACKGROUND_DARKER.to_iced().into(),
            },
            selected_text_color: TEXT_LIGHTEST.to_iced(),
            selected_background: BACKGROUND_DARKER.to_iced().into(),
        }
    }
}

impl From<PickListStyle> for MenuStyle {
    fn from(pick_list: PickListStyle) -> Self {
        match pick_list {
            PickListStyle::Default => Self::Default,
        }
    }
}