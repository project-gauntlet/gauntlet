use iced::widget::scrollable::{Status, Style};
use iced::widget::{container, scrollable};
use iced::{border, Border, Color};

use crate::theme::{GauntletSettingsTheme, PRIMARY};

impl scrollable::Catalog for GauntletSettingsTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        let scrollbar = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::TRANSPARENT,
                border: border::rounded(4.0),
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
                        color: PRIMARY.to_iced(),
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
                        color: PRIMARY.to_iced(),
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