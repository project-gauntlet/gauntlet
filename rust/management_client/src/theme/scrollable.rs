use iced::{Border, Color};
use iced::widget::scrollable;
use iced::widget::scrollable::Appearance;

use crate::theme::{GauntletSettingsTheme, PRIMARY};

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Default
}

impl scrollable::StyleSheet for GauntletSettingsTheme {
    type Style = ScrollableStyle;

    fn active(&self, _: &Self::Style) -> Appearance {
        appearance(ScrollbarState::Active)
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> Appearance {
        if is_mouse_over_scrollbar {
            appearance(ScrollbarState::Hovered)
        } else {
            self.active(style)
        }
    }
}

enum ScrollbarState {
    Active,
    Hovered
}

fn appearance(state: ScrollbarState) -> Appearance {
    let scroller_color = match state {
        ScrollbarState::Active => Color::TRANSPARENT,
        ScrollbarState::Hovered => PRIMARY.to_iced(),
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
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: None,
    }
}