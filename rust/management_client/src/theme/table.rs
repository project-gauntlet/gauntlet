use iced::widget::container;
use iced::Border;

use crate::theme::{GauntletSettingsTheme, BACKGROUND_DARKER, TEXT_LIGHTEST};


impl iced_table::Catalog for GauntletSettingsTheme {
    type Style = ();

    fn header(&self, _: &Self::Style) -> container::Style {
        container::Style {
            text_color: Some(TEXT_LIGHTEST.to_iced()),
            background: Some(BACKGROUND_DARKER.to_iced().into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn footer(&self, _: &Self::Style) -> container::Style {
        container::Style {
            text_color: Some(TEXT_LIGHTEST.to_iced()),
            background: Some(BACKGROUND_DARKER.to_iced().into()),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // TODO selected and hovered upstream
    fn row(&self, _: &Self::Style, index: usize) -> container::Style {
        let background = if index % 2 == 0 {
            None
        } else {
            Some(BACKGROUND_DARKER.to_iced().into())
        };

        container::Style {
            background,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn divider(&self, _: &Self::Style, _hovered: bool) -> container::Style {
        container::Style {
            ..Default::default()
        }
    }
}
