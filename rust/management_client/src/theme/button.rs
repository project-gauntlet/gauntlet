use iced::widget::button;
use iced::widget::button::Status;
use iced::widget::button::Style;
use iced::Border;

use crate::theme::GauntletSettingsTheme;
use crate::theme::BACKGROUND_DARKER;
use crate::theme::BACKGROUND_LIGHTER;
use crate::theme::BUTTON_BORDER_RADIUS;
use crate::theme::DANGER;
use crate::theme::PRIMARY;
use crate::theme::PRIMARY_HOVERED;
use crate::theme::SUCCESS;
use crate::theme::TEXT_DARKEST;
use crate::theme::TEXT_LIGHTEST;

pub enum ButtonStyle {
    Primary,
    #[allow(unused)]
    Positive,
    Destructive,
    TableRow,
    ViewSwitcher,
    ViewSwitcherSelected,
    DownloadInfo,
}

impl button::Catalog for GauntletSettingsTheme {
    type Class<'a> = ButtonStyle;

    fn default<'a>() -> Self::Class<'a> {
        ButtonStyle::Primary
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => active(class),
            Status::Hovered => hovered(class),
            Status::Pressed => pressed(class),
            Status::Disabled => disabled(class),
        }
    }
}

fn active(class: &ButtonStyle) -> Style {
    let (background_color, text_color) = match class {
        ButtonStyle::Primary => (PRIMARY.to_iced(), TEXT_DARKEST.to_iced()),
        ButtonStyle::Positive => (SUCCESS.to_iced(), TEXT_DARKEST.to_iced()),
        ButtonStyle::Destructive => (DANGER.to_iced(), TEXT_LIGHTEST.to_iced()),
        ButtonStyle::TableRow => {
            return Style {
                background: None,
                text_color: TEXT_LIGHTEST.to_iced(),
                ..Default::default()
            }
        }
        ButtonStyle::ViewSwitcher => {
            return Style {
                background: None,
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        ButtonStyle::ViewSwitcherSelected => {
            return Style {
                background: Some(BACKGROUND_DARKER.to_iced().into()),
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        ButtonStyle::DownloadInfo => {
            return Style {
                background: None,
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    };

    Style {
        background: Some(background_color.into()),
        text_color,
        border: Border {
            radius: BUTTON_BORDER_RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn hovered(class: &ButtonStyle) -> Style {
    let (background_color, text_color) = match class {
        ButtonStyle::Primary => (PRIMARY_HOVERED.to_iced(), TEXT_DARKEST.to_iced()),
        ButtonStyle::Positive => (SUCCESS.to_iced(), TEXT_DARKEST.to_iced()), // TODO
        ButtonStyle::Destructive => (DANGER.to_iced(), TEXT_LIGHTEST.to_iced()), // TODO
        ButtonStyle::TableRow => {
            return Style {
                background: None,
                text_color: TEXT_LIGHTEST.to_iced(), // TODO
                ..Default::default()
            };
        }
        ButtonStyle::ViewSwitcher => {
            return Style {
                background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        ButtonStyle::ViewSwitcherSelected => {
            return Style {
                background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        ButtonStyle::DownloadInfo => {
            return Style {
                background: Some(BACKGROUND_LIGHTER.to_iced().into()),
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    };

    Style {
        background: Some(background_color.into()),
        text_color,
        border: Border {
            radius: BUTTON_BORDER_RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn pressed(class: &ButtonStyle) -> Style {
    match class {
        ButtonStyle::ViewSwitcher | ButtonStyle::ViewSwitcherSelected => {
            Style {
                background: Some(BACKGROUND_DARKER.to_iced().into()),
                text_color: TEXT_LIGHTEST.to_iced(),
                border: Border {
                    radius: BUTTON_BORDER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        _ => active(class),
    }
}

fn disabled(class: &ButtonStyle) -> Style {
    let style = active(class);

    Style {
        background: style.background.map(|background| background.scale_alpha(0.5)),
        text_color: style.text_color.scale_alpha(0.5),
        ..style
    }
}
