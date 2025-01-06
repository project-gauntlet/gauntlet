use std::borrow::Borrow;

use iced::{Border, overlay};
use iced::widget::{pick_list, PickList};
use iced::widget::pick_list::Status;
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

impl pick_list::Catalog for GauntletComplexTheme {
    type Class<'a> = PickListStyle;

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        PickListStyle::Default
    }

    fn style(&self, _class: &<Self as pick_list::Catalog>::Class<'_>, status: Status) -> pick_list::Style {
        let theme = get_theme();
        let theme = &theme.form_input_select;

        let background_color = match status {
            Status::Active | Status::Opened => theme.background_color,
            Status::Hovered => theme.background_color_hovered,
        };

        let text_color = match status {
            Status::Active | Status::Opened => theme.text_color,
            Status::Hovered => theme.text_color_hovered,
        };

        pick_list::Style {
            text_color,
            background: background_color.into(),
            placeholder_color: NOT_INTENDED_TO_BE_USED,
            handle_color: text_color,
            border: Border {
                radius: theme.border_radius.into(),
                width: theme.border_width,
                color: theme.border_color.into(),
            },
        }
    }
}

impl overlay::menu::Catalog for GauntletComplexTheme {
    type Class<'a> = MenuStyle;

    fn default<'a>() -> <Self as overlay::menu::Catalog>::Class<'a> {
        MenuStyle::Default
    }

    fn style(&self, _class: &<Self as overlay::menu::Catalog>::Class<'_>) -> overlay::menu::Style {
        let theme = &self.form_input_select_menu; // TODO consider using root style

        overlay::menu::Style {
            text_color: theme.text_color,
            background: theme.background_color.into(),
            border: Border {
                radius: theme.border_radius.into(),
                width: theme.border_width,
                color: theme.border_color.into(),
            },
            selected_text_color: theme.text_color_selected,
            selected_background: theme.background_color_selected.into(),
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
        self.class(kind)
            // .padding() // TODO
            .into()
    }
}