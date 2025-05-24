use iced::Border;
use iced::Renderer;
use iced::widget::Checkbox;
use iced::widget::checkbox;
use iced::widget::checkbox::Status;
use iced::widget::checkbox::Style;

use crate::ui::theme::Element;
use crate::ui::theme::GauntletComplexTheme;
use crate::ui::theme::NOT_INTENDED_TO_BE_USED;
use crate::ui::theme::ThemableWidget;

pub enum CheckboxStyle {
    Default,
}

impl checkbox::Catalog for GauntletComplexTheme {
    type Class<'a> = CheckboxStyle;

    fn default<'a>() -> Self::Class<'a> {
        CheckboxStyle::Default
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active { is_checked } => active(self, is_checked),
            Status::Hovered { is_checked } => hovered(self, is_checked),
            Status::Disabled { is_checked } => disabled(is_checked),
        }
    }
}

fn active(theme: &GauntletComplexTheme, is_checked: bool) -> Style {
    let theme = &theme.form_input_checkbox;

    let background = if is_checked {
        theme.background_color_checked.into()
    } else {
        theme.background_color_unchecked.into()
    };

    Style {
        background,
        icon_color: theme.icon_color,
        border: Border {
            radius: theme.border_radius.into(),
            width: theme.border_width,
            color: theme.border_color.into(),
        },
        text_color: None,
    }
}

fn hovered(theme: &GauntletComplexTheme, is_checked: bool) -> Style {
    let theme = &theme.form_input_checkbox;

    let background = if is_checked {
        theme.background_color_checked_hovered.into()
    } else {
        theme.background_color_unchecked_hovered.into()
    };

    Style {
        background,
        icon_color: theme.icon_color,
        border: Border {
            radius: theme.border_radius.into(),
            width: theme.border_width,
            color: theme.border_color.into(),
        },
        text_color: None,
    }
}

fn disabled(_is_checked: bool) -> Style {
    Style {
        background: NOT_INTENDED_TO_BE_USED.into(),
        icon_color: NOT_INTENDED_TO_BE_USED,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: NOT_INTENDED_TO_BE_USED,
        },
        text_color: None,
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Checkbox<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = CheckboxStyle;

    fn themed(self, style: CheckboxStyle) -> Element<'a, Message> {
        match style {
            CheckboxStyle::Default => {
                self.class(style)
                // .spacing() // TODO
                // .size()
            }
        }
        .into()
    }
}
