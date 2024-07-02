use iced::{Border, Color};
use iced::widget::scrollable;
use scrollable::Appearance;

use crate::ui::theme::{GauntletTheme, get_theme};

impl scrollable::StyleSheet for GauntletTheme {
    type Style = ();

    fn active(&self, _: &Self::Style) -> Appearance {
        appearance(ScrollbarState::Active)
    }

    fn hovered(&self, _: &Self::Style, is_mouse_over_scrollbar: bool) -> Appearance {
        if is_mouse_over_scrollbar {
            appearance(ScrollbarState::Hovered)
        } else {
            self.active(&())
        }
    }
}

enum ScrollbarState {
    Active,
    Hovered
}

fn appearance(state: ScrollbarState) -> Appearance {
    let theme = get_theme();
    let theme = &theme.scrollbar;

    let scroller_color = match state {
        ScrollbarState::Active => Color::TRANSPARENT,
        ScrollbarState::Hovered => theme.color.to_iced(),
    };

    Appearance {
        container: Default::default(),
        scrollbar: scrollable::Scrollbar {
            background: None,
            border: Border {
                color: Color::TRANSPARENT,
                ..Border::default()
            },
            scroller: scrollable::Scroller {
                color: scroller_color,
                border: Border {
                    color: theme.border_color.to_iced(),
                    width: theme.border_width.into(),
                    radius: theme.border_radius.into(),
                },
            },
        },
        gap: None,
    }
}