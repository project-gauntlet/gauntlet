use iced::Renderer;
use iced::widget::{Text, text};
use text::Appearance;

use crate::ui::theme::{Element, GauntletTheme, get_theme, ThemableWidget};

#[derive(Clone, Default)]
pub enum TextStyle {
    #[default]
    Default, // TODO is this used?

    EmptyViewSubtitle,
    ListItemSubtitle,
    ListSectionTitle,
    ListSectionSubtitle,
    GridSectionTitle,
    GridSectionSubtitle,
    MainListItemSubtext,
    MetadataItemLabel,
    TextAccessory,
    IconAccessory,
    GridItemTitle,
    GridItemSubTitle,
    InlineName,
    InlineSeparator,
    RootBottomPanelDefaultActionText,
    RootBottomPanelActionToggleText,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Text<'a, GauntletTheme, Renderer> {
    type Kind = TextStyle;

    fn themed(self, kind: TextStyle) -> Element<'a, Message> {
        match kind {
            TextStyle::MetadataItemLabel => {
                let theme = get_theme();

                self.style(kind)
                    .size(theme.metadata_item_label.text_size)
                    .into()
            }
            TextStyle::InlineName => {
                self.size(15)
                    .style(kind)
                    .into()
            }
            _ => {
                self.style(kind)
                    .into()
            }
        }
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
            TextStyle::ListSectionSubtitle => Appearance {
                color: Some(self.list_section_subtitle.text_color.to_iced()),
            },
            TextStyle::GridSectionTitle => Appearance {
                color: Some(self.grid_section_title.text_color.to_iced()),
            },
            TextStyle::GridSectionSubtitle => Appearance{
                color: Some(self.grid_section_subtitle.text_color.to_iced()),
            },
            TextStyle::MainListItemSubtext => Appearance {
                color: Some(self.main_list_item_sub_text.text_color.to_iced()),
            },
            TextStyle::MetadataItemLabel => Appearance {
                color: Some(self.metadata_item_label.text_color.to_iced()),
            },
            TextStyle::TextAccessory => Appearance {
                color: Some(self.text_accessory.text_color.to_iced()),
            },
            TextStyle::IconAccessory => Appearance {
                color: Some(self.icon_accessory.icon_color.to_iced()),
            },
            TextStyle::GridItemTitle => Appearance {
                color: Some(self.grid_item_title.text_color.to_iced()),
            },
            TextStyle::GridItemSubTitle => Appearance {
                color: Some(self.grid_item_subtitle.text_color.to_iced()),
            },
            TextStyle::InlineName => Appearance {
                color: Some(self.inline_name.text_color.to_iced()),
            },
            TextStyle::InlineSeparator => Appearance {
                color: Some(self.inline_separator.text_color.to_iced()),
            },
            TextStyle::RootBottomPanelDefaultActionText => Appearance {
                color: Some(self.root_bottom_panel_default_action_text.text_color.to_iced()),
            },
            TextStyle::RootBottomPanelActionToggleText => Appearance {
                color: Some(self.root_bottom_panel_action_toggle_text.text_color.to_iced()),
            }
        }
    }
}