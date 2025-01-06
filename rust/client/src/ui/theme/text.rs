use iced::Renderer;
use iced::widget::{Text, text};
use iced::widget::text::Style;
use crate::ui::theme::{Element, GauntletComplexTheme, get_theme, ThemableWidget};

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
    RootBottomPanelPrimaryActionText,
    RootBottomPanelActionToggleText,
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Text<'a, GauntletComplexTheme, Renderer> {
    type Kind = TextStyle;

    fn themed(self, kind: TextStyle) -> Element<'a, Message> {
        match kind {
            TextStyle::MetadataItemLabel => {
                let theme = get_theme();

                self.class(kind)
                    .size(theme.metadata_item_label.text_size)
                    .into()
            }
            TextStyle::InlineName => {
                self.size(15)
                    .class(kind)
                    .into()
            }
            _ => {
                self.class(kind)
                    .into()
            }
        }
    }
}

impl text::Catalog for GauntletComplexTheme {
    type Class<'a> = TextStyle;

    fn default<'a>() -> Self::Class<'a> {
        TextStyle::Default
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            TextStyle::Default => Default::default(),
            TextStyle::EmptyViewSubtitle => Style {
                color: Some(self.empty_view_subtitle.text_color),
            },
            TextStyle::ListItemSubtitle => Style {
                color: Some(self.list_item_subtitle.text_color),
            },
            TextStyle::ListSectionTitle => Style {
                color: Some(self.list_section_title.text_color),
            },
            TextStyle::ListSectionSubtitle => Style {
                color: Some(self.list_section_subtitle.text_color),
            },
            TextStyle::GridSectionTitle => Style {
                color: Some(self.grid_section_title.text_color),
            },
            TextStyle::GridSectionSubtitle => Style{
                color: Some(self.grid_section_subtitle.text_color),
            },
            TextStyle::MainListItemSubtext => Style {
                color: Some(self.main_list_item_sub_text.text_color),
            },
            TextStyle::MetadataItemLabel => Style {
                color: Some(self.metadata_item_label.text_color),
            },
            TextStyle::TextAccessory => Style {
                color: Some(self.text_accessory.text_color),
            },
            TextStyle::IconAccessory => Style {
                color: Some(self.icon_accessory.icon_color),
            },
            TextStyle::GridItemTitle => Style {
                color: Some(self.grid_item_title.text_color),
            },
            TextStyle::GridItemSubTitle => Style {
                color: Some(self.grid_item_subtitle.text_color),
            },
            TextStyle::InlineName => Style {
                color: Some(self.inline_name.text_color),
            },
            TextStyle::InlineSeparator => Style {
                color: Some(self.inline_separator.text_color),
            },
            TextStyle::RootBottomPanelPrimaryActionText => Style {
                color: Some(self.root_bottom_panel_primary_action_text.text_color),
            },
            TextStyle::RootBottomPanelActionToggleText => Style {
                color: Some(self.root_bottom_panel_action_toggle_text.text_color),
            }
        }
    }
}