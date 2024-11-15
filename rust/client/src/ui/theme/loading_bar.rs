use crate::ui::custom_widgets::loading_bar;
use crate::ui::custom_widgets::loading_bar::{Appearance, LoadingBar};
use crate::ui::theme::{Element, ThemableWidget};
use crate::ui::GauntletComplexTheme;

#[derive(Default)]
pub enum LoadingBarStyle {
    #[default]
    Default,
}

impl loading_bar::StyleSheet for GauntletComplexTheme {
    type Style = LoadingBarStyle;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background_color: self.loading_bar.background_color.to_iced(),
            loading_bar_color: self.loading_bar.loading_bar_color.to_iced(),
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for LoadingBar<GauntletComplexTheme> {
    type Kind = LoadingBarStyle;

    fn themed(self, _kind: LoadingBarStyle) -> Element<'a, Message> {
        self.into()
    }
}
