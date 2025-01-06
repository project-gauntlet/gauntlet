use iced::{Border, Color};
use iced::widget::{container, scrollable};
use iced::widget::scrollable::{Status, Style};

use crate::ui::theme::{GauntletComplexTheme, get_theme};

impl scrollable::Catalog for GauntletComplexTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        let theme = get_theme();
        let theme = &theme.scrollbar;

        let scrollbar = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::TRANSPARENT,
                border: Border {
                    color: theme.border_color,
                    width: theme.border_width.into(),
                    radius: theme.border_radius.into(),
                },
            },
        };

        match status {
            Status::Active => Style {
                container: container::Style::default(),
                vertical_rail: scrollbar,
                horizontal_rail: scrollbar,
                gap: None,
            },
            Status::Hovered {
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
            } => {
                let hovered_scrollbar = scrollable::Rail {
                    scroller: scrollable::Scroller {
                        color: theme.color,
                        ..scrollbar.scroller
                    },
                    ..scrollbar
                };

                Style {
                    container: container::Style::default(),
                    vertical_rail: if is_vertical_scrollbar_hovered {
                        hovered_scrollbar
                    } else {
                        scrollbar
                    },
                    horizontal_rail: if is_horizontal_scrollbar_hovered {
                        hovered_scrollbar
                    } else {
                        scrollbar
                    },
                    gap: None,
                }
            }
            Status::Dragged {
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
            } => {
                let dragged_scrollbar = scrollable::Rail {
                    scroller: scrollable::Scroller {
                        color: theme.color,
                        ..scrollbar.scroller
                    },
                    ..scrollbar
                };

                Style {
                    container: container::Style::default(),
                    vertical_rail: if is_vertical_scrollbar_dragged {
                        dragged_scrollbar
                    } else {
                        scrollbar
                    },
                    horizontal_rail: if is_horizontal_scrollbar_dragged {
                        dragged_scrollbar
                    } else {
                        scrollbar
                    },
                    gap: None,
                }
            }
        }
    }
}
