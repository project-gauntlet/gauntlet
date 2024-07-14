use iced::Border;
use iced::widget::container::Appearance;
use crate::components::shortcut_selector;
use crate::theme::{BACKGROUND_LIGHT, BUTTON_BORDER_RADIUS, GauntletSettingsTheme, PRIMARY};

#[derive(Default)]
pub enum ShortcutSelectorStyle {
    #[default]
    Default,
}

impl shortcut_selector::StyleSheet for GauntletSettingsTheme {
    type Style = ShortcutSelectorStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        match style {
            ShortcutSelectorStyle::Default => {
                Appearance {
                    background: Some(BACKGROUND_LIGHT.to_iced().into()),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        }
    }

    fn capturing(&self, style: &Self::Style) -> Appearance {
        match style {
            ShortcutSelectorStyle::Default => {
                Appearance {
                    background: Some(BACKGROUND_LIGHT.to_iced().into()),
                    border: Border {
                        radius: BUTTON_BORDER_RADIUS.into(),
                        width: 2.0,
                        color: PRIMARY.to_iced(),
                    },
                    ..Default::default()
                }
            }
        }
    }
}
