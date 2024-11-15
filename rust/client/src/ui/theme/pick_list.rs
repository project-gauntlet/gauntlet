use std::borrow::Borrow;

use iced::{Border, overlay};
use iced::widget::{pick_list, PickList};

use crate::ui::theme::{Element, GauntletComplexTheme, get_theme, NOT_INTENDED_TO_BE_USED, ThemableWidget};

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

impl pick_list::StyleSheet for GauntletComplexTheme {
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
    let theme = get_theme();
    let theme = &theme.form_input_select;

    let background_color = match state {
        PickListState::Active => theme.background_color.to_iced(),
        PickListState::Hovered => theme.background_color_hovered.to_iced(),
    };

    let text_color = match state {
        PickListState::Active => theme.text_color.to_iced(),
        PickListState::Hovered => theme.text_color_hovered.to_iced(),
    };

    pick_list::Appearance {
        text_color,
        background: background_color.into(),
        placeholder_color: NOT_INTENDED_TO_BE_USED.to_iced(),
        handle_color: text_color,
        border: Border {
            radius: theme.border_radius.into(),
            width: theme.border_width,
            color: theme.border_color.to_iced().into(),
        },
    }
}

impl overlay::menu::StyleSheet for GauntletComplexTheme {
    type Style = MenuStyle;

    fn appearance(&self, _: &Self::Style) -> overlay::menu::Appearance {
        let theme = get_theme();
        let theme = &theme.form_input_select_menu; // TODO consider using root style

        overlay::menu::Appearance {
            text_color: theme.text_color.to_iced(),
            background: theme.background_color.to_iced().into(),
            border: Border {
                radius: theme.border_radius.into(),
                width: theme.border_width,
                color: theme.border_color.to_iced().into(),
            },
            selected_text_color: theme.text_color_selected.to_iced(),
            selected_background: theme.background_color_selected.to_iced().into(),
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

impl<'a, Message: 'a + Clone + 'static, T, L, V> ThemableWidget<'a, Message> for PickList<'a, T, L, V, Message, GauntletComplexTheme>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
{
    type Kind = PickListStyle;

    fn themed(self, kind: PickListStyle) -> Element<'a, Message> {
        self.style(kind)
            // .padding() // TODO
            .into()
    }
}