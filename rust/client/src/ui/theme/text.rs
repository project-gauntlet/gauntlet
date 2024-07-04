use iced::Renderer;
use iced::widget::{Text, text};
use text::Appearance;

use crate::ui::theme::{Element, GauntletTheme, ThemableWidget};

#[derive(Clone, Default)]
pub enum TextStyle {
    #[default]
    Default, // TODO is this used?

    EmptyViewSubtitle,
    ListItemSubtitle,
    ListSectionTitle,
    GridSectionTitle,
    MainListItemSubtext,
    MetadataItemLabel,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Text<'a, GauntletTheme, Renderer> {
    type Kind = TextStyle;

    fn themed(self, kind: TextStyle) -> Element<'a, Message> {
        self.style(kind).into()
    }
}

impl text::StyleSheet for GauntletTheme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> Appearance {
        match style {
            TextStyle::Default => Default::default(),
            TextStyle::EmptyViewSubtitle => Appearance {
                color: Some(self.empty_view_subtitle.text_color.to_iced()),
            },
            TextStyle::ListItemSubtitle => Appearance {
                color: Some(self.list_item_subtitle.text_color.to_iced()),
            },
            TextStyle::ListSectionTitle => Appearance {
                color: Some(self.list_section_title.text_color.to_iced()),
            },
            TextStyle::GridSectionTitle => Appearance {
                color: Some(self.grid_section_title.text_color.to_iced()),
            },
            TextStyle::MainListItemSubtext => Appearance {
                color: Some(self.main_list_item_sub_text.text_color.to_iced()),
            },
            TextStyle::MetadataItemLabel => Appearance {
                color: Some(self.metadata_item_label.text_color.to_iced()),
            }
        }
    }
}