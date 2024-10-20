use container::Appearance;
use iced::{Border, Color, Length, Padding, Renderer};
use iced::border::Radius;
use iced::widget::{Container, container};

use crate::ui::theme::{Element, GauntletTheme, get_theme, ThemableWidget};

pub enum ContainerStyle {
    ActionPanel,
    ActionPanelTitle,
    ActionShortcutModifier,
    ActionShortcutModifiersInit, // "init" means every item on list except last one
    ContentCodeBlock,
    ContentCodeBlockText,
    ContentHorizontalBreak,
    ContentImage,
    ContentParagraph,
    DetailContent,
    DetailContentInner,
    DetailMetadata,
    EmptyViewImage,
    FormInputLabel,
    Inline,
    ListItemSubtitle,
    ListItemTitle,
    ListItemIcon,
    Main,
    MainList,
    MainListInner,
    MainListItemIcon,
    MainListItemSubText,
    MainListItemText,
    MainSearchBar,
    MetadataInner,
    MetadataItemValue,
    MetadataItemLabel,
    MetadataSeparator,
    MetadataTagItem,
    MetadataLinkIcon,
    Form,
    FormInner,
    PluginErrorViewDescription,
    PluginErrorViewTitle,
    PreferenceRequiredViewDescription,
    Root,
    RootBottomPanel,
    RootBottomPanelPrimaryActionText,
    RootBottomPanelActionToggleText,
    RootInner,
    RootTopPanel,
    Grid,
    GridInner,
    List,
    ListInner,
    TextAccessory,
    TextAccessoryIcon,
    IconAccessory,
    InlineInner,
    InlineName,
    HudInner,
    Hud,
    RootBottomPanelPrimaryActionButton,
}

#[derive(Default)]
pub enum ContainerStyleInner {
    #[default]
    Transparent,

    Tooltip,

    ActionPanel,
    ActionShortcutModifier,
    ContentCodeBlockText,
    Main,
    Root,
    ContentImage,
    RootBottomPanel,
    InlineInner,
    Hud,
}


impl container::StyleSheet for GauntletTheme {
    type Style = ContainerStyleInner;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ContainerStyleInner::Transparent => Default::default(),
            ContainerStyleInner::ActionPanel => {
                let root_theme = &self.root;
                let panel_theme = &self.action_panel;
                let background_color = &panel_theme.background_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: root_theme.border_radius.into(),
                        width: root_theme.border_width,
                        color: root_theme.border_color.to_iced(),
                    },
                    shadow: Default::default(),
                }
            }
            ContainerStyleInner::ActionShortcutModifier => {

                let theme = &self.action_shortcut_modifier;
                let background_color = &theme.background_color;
                let border_color = &theme.border_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: border_color.to_iced().into(),
                    },
                    shadow: Default::default(),
                }
            }
            ContainerStyleInner::ContentCodeBlockText => {
                let theme = &self.content_code_block_text;
                let background_color = &theme.background_color;
                let border_color = &theme.border_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: border_color.to_iced().into(),
                    },
                    shadow: Default::default(),
                }
            }
            ContainerStyleInner::Main => {
                let theme = &self.root;
                let background_color = &theme.background_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color.to_iced(),
                    },
                    shadow: Default::default(),
                }
            }
            ContainerStyleInner::Root => {
                let theme = &self.root;
                let background_color = &theme.background_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color.to_iced(),
                    },
                    shadow: Default::default(),
                }
            }
            ContainerStyleInner::Tooltip => {
                let theme = &self.root;
                let tooltip_theme = &self.tooltip;
                let background_color = &tooltip_theme.background_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color.to_iced(),
                    },
                    shadow: Default::default(),
                }
            }
            ContainerStyleInner::ContentImage => {
                let theme = &self.content_image;

                // TODO this border radius doesn't work on image, for some reason

                Appearance {
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Appearance::default()
                }
            }
            ContainerStyleInner::RootBottomPanel => {
                let root_theme = &self.root;
                let panel_theme = &self.root_bottom_panel;

                Appearance {
                    background: Some(panel_theme.background_color.to_iced().into()),
                    border: Border {
                        radius: Radius::from([0.0, 0.0, root_theme.border_radius, root_theme.border_radius]),
                        width: root_theme.border_width,
                        color: root_theme.border_color.to_iced(),
                    },
                    ..Appearance::default()
                }
            }
            ContainerStyleInner::InlineInner => {
                let theme = &self.inline_inner;

                Appearance {
                    background: Some(theme.background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color.to_iced(),
                    },
                    ..Appearance::default()
                }
            }
            ContainerStyleInner::Hud => {
                let theme = &self.hud;
                let background_color = &theme.background_color;

                Appearance {
                    text_color: None,
                    background: Some(background_color.to_iced().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color.to_iced(),
                    },
                    shadow: Default::default(),
                }
            }
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Container<'a, Message, GauntletTheme, Renderer> {
    type Kind = ContainerStyle;

    fn themed(self, name: ContainerStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match name {
            ContainerStyle::RootInner => {
                self.padding(theme.root_content.padding.to_iced())
            }
            ContainerStyle::ActionPanelTitle => {
                self.padding(theme.action_panel_title.padding.to_iced())
            }
            ContainerStyle::ActionShortcutModifier => {
                self.style(ContainerStyleInner::ActionShortcutModifier)
                    .padding(theme.action_shortcut_modifier.padding.to_iced())
            }
            ContainerStyle::ActionShortcutModifiersInit => {
                let horizontal_spacing = theme.action_shortcut_modifier.spacing;
                self.padding(Padding::from([0.0, horizontal_spacing, 0.0, 0.0]))
            }
            ContainerStyle::ActionPanel => {
                self.style(ContainerStyleInner::ActionPanel)
                    .padding(theme.action_panel.padding.to_iced())
                    .height(Length::Fixed(250.0))
                    .width(Length::Fixed(350.0))
            }
            ContainerStyle::MetadataTagItem => {
                self.padding(theme.metadata_tag_item.padding.to_iced())
            }
            ContainerStyle::MetadataItemLabel => {
                self.padding(theme.metadata_item_label.padding.to_iced())
            }
            ContainerStyle::MetadataLinkIcon => {
                self.padding(theme.metadata_link_icon.padding.to_iced())
            }
            ContainerStyle::MetadataItemValue => {
                self.padding(theme.metadata_item_value.padding.to_iced())
            }
            ContainerStyle::RootBottomPanel => {
                self.style(ContainerStyleInner::RootBottomPanel)
                    .padding(theme.root_bottom_panel.padding.to_iced())
            }
            ContainerStyle::RootTopPanel => {
                self.padding(theme.root_top_panel.padding.to_iced())
            }
            ContainerStyle::ListItemSubtitle => {
                self.padding(theme.list_item_subtitle.padding.to_iced())
            }
            ContainerStyle::ListItemTitle => {
                self.padding(theme.list_item_title.padding.to_iced())
            }
            ContainerStyle::ListItemIcon => {
                self.padding(theme.list_item_icon.padding.to_iced())
            }
            ContainerStyle::ContentParagraph => {
                self.padding(theme.content_paragraph.padding.to_iced())
            }
            ContainerStyle::ContentHorizontalBreak => {
                self.padding(theme.content_horizontal_break.padding.to_iced())
            }
            ContainerStyle::ContentCodeBlock => {
                self.padding(theme.content_code_block.padding.to_iced())
            }
            ContainerStyle::ContentCodeBlockText => {
                self.style(ContainerStyleInner::ContentCodeBlockText)
                    .padding(theme.content_code_block_text.padding.to_iced())
            }
            ContainerStyle::ContentImage => {
                self.style(ContainerStyleInner::ContentImage)
                    .padding(theme.content_image.padding.to_iced())
            }
            ContainerStyle::DetailContentInner => {
                self.padding(theme.metadata_content_inner.padding.to_iced())
            }
            ContainerStyle::MetadataInner => {
                self.padding(theme.metadata_inner.padding.to_iced())
            }
            ContainerStyle::MetadataSeparator => {
                self.padding(theme.metadata_separator.padding.to_iced())
            }
            ContainerStyle::DetailMetadata => {
                self.padding(theme.detail_metadata.padding.to_iced())
            }
            ContainerStyle::DetailContent => {
                self.padding(theme.detail_content.padding.to_iced())
            }
            ContainerStyle::FormInputLabel => {
                self.padding(theme.form_input_label.padding.to_iced())
            }
            ContainerStyle::Inline => {
                self.padding(theme.inline.padding.to_iced())
            }
            ContainerStyle::InlineInner => {
                self
                    .height(120)
                    .max_height(120)
                    .padding(theme.inline_inner.padding.to_iced())
                    .style(ContainerStyleInner::InlineInner)
            }
            ContainerStyle::InlineName => {
                self.padding(theme.inline_name.padding.to_iced())
            }
            ContainerStyle::EmptyViewImage => {
                self.padding(theme.empty_view_image.padding.to_iced())
            }
            ContainerStyle::Main => {
                self.style(ContainerStyleInner::Main)
            }
            ContainerStyle::MainListItemText => {
                self.padding(theme.main_list_item_text.padding.to_iced())
            }
            ContainerStyle::MainListItemSubText => {
                self.padding(theme.main_list_item_sub_text.padding.to_iced())
            }
            ContainerStyle::MainListItemIcon => {
                self.padding(theme.main_list_item_icon.padding.to_iced())
            }
            ContainerStyle::MainList => {
                self.padding(theme.main_list.padding.to_iced())
            }
            ContainerStyle::MainListInner => {
                self.padding(theme.main_list_inner.padding.to_iced())
            }
            ContainerStyle::MainSearchBar => {
                self.padding(theme.main_search_bar.padding.to_iced())
            }
            ContainerStyle::Root => {
                self.style(ContainerStyleInner::Root)
            }
            ContainerStyle::PluginErrorViewTitle => {
                self.padding(theme.plugin_error_view_title.padding.to_iced())
            }
            ContainerStyle::PluginErrorViewDescription => {
                self.padding(theme.plugin_error_view_description.padding.to_iced())
            }
            ContainerStyle::PreferenceRequiredViewDescription => {
                self.padding(theme.preference_required_view_description.padding.to_iced())
            }
            ContainerStyle::Form => {
                self.padding(theme.form.padding.to_iced())
            }
            ContainerStyle::FormInner => {
                self.padding(theme.form_inner.padding.to_iced())
            }
            ContainerStyle::GridInner => {
                self.padding(theme.grid_inner.padding.to_iced())
            }
            ContainerStyle::Grid => {
                self.padding(theme.grid.padding.to_iced())
            }
            ContainerStyle::List => {
                self.padding(theme.list.padding.to_iced())
            }
            ContainerStyle::ListInner => {
                self.padding(theme.list_inner.padding.to_iced())
            }
            ContainerStyle::RootBottomPanelActionToggleText => {
                self.padding(theme.root_bottom_panel_action_toggle_text.padding.to_iced())
            }
            ContainerStyle::RootBottomPanelPrimaryActionText => {
                self.padding(theme.root_bottom_panel_primary_action_text.padding.to_iced())
            }
            ContainerStyle::RootBottomPanelPrimaryActionButton => {
                self.padding(Padding::from([0.0, theme.root_bottom_panel.spacing, 0.0, 0.0]))
            }
            ContainerStyle::TextAccessory => {
                self.padding(theme.text_accessory.padding.to_iced())
            }
            ContainerStyle::TextAccessoryIcon => {
                let horizontal_spacing = theme.text_accessory.spacing;
                self.padding(Padding::from([0.0, horizontal_spacing, 0.0, 0.0]))
            }
            ContainerStyle::IconAccessory => {
                self.padding(theme.icon_accessory.padding.to_iced())
            }
            ContainerStyle::HudInner => {
                self.padding(theme.hud_content.padding.to_iced())
            }
            ContainerStyle::Hud => {
                self.style(ContainerStyleInner::Hud)
            }
        }.into()
    }
}

