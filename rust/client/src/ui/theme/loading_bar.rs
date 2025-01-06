use crate::ui::custom_widgets::loading_bar;
use crate::ui::custom_widgets::loading_bar::{LoadingBar, Style};
use crate::ui::theme::{Element, ThemableWidget};
use crate::ui::GauntletComplexTheme;

#[derive(Default)]
pub enum LoadingBarStyle {
    #[default]
    Default,
}

impl loading_bar::Catalog for GauntletComplexTheme {
    type Class<'a> = LoadingBarStyle;

    fn default<'a>() -> Self::Class<'a> {
        LoadingBarStyle::Default
    }

    fn style(&self, _class: &Self::Class<'_>) -> Style {
        Style {
            background_color: self.loading_bar.background_color,
            loading_bar_color: self.loading_bar.loading_bar_color,
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for LoadingBar<'a, GauntletComplexTheme> {
    type Kind = LoadingBarStyle;

    fn themed(self, _kind: LoadingBarStyle) -> Element<'a, Message> {
        self.into()
    }
}
