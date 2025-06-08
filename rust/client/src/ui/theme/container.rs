use iced::Border;
use iced::Color;
use iced::Length;
use iced::Renderer;
use iced::Shadow;
use iced::Vector;
use iced::widget::Container;
use iced::widget::container;
use iced::widget::container::Style;

use crate::ui::theme::Element;
use crate::ui::theme::GauntletComplexTheme;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::get_theme;

pub enum ContainerStyle {
    ActionPanel,
    ActionPanelTitle,
    ActionSectionTitle,
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
    MainListItemAlias,
    MainListItemIcon,
    MainListItemSubText,
    MainListItemText,
    MainSearchBar,
    MetadataInner,
    MetadataItemValue,
    MetadataItemValueInList,
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

pub enum ContainerStyleInner {
    Transparent,

    Tooltip,

    ActionPanel,
    ActionShortcutModifier,
    ContentCodeBlockText,
    Main,
    MainListItemAlias,
    Root,
    ContentImage,
    RootBottomPanel,
    InlineInner,
    Hud,
}

impl container::Catalog for GauntletComplexTheme {
    type Class<'a> = ContainerStyleInner;

    fn default<'a>() -> Self::Class<'a> {
        ContainerStyleInner::Transparent
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        match class {
            ContainerStyleInner::Transparent => Default::default(),
            ContainerStyleInner::ActionPanel => {
                let root_theme = &self.popup;
                let panel_theme = &self.action_panel;
                let background_color = &panel_theme.background_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: root_theme.border_radius.into(),
                        width: root_theme.border_width,
                        color: root_theme.border_color,
                    },
                    shadow: Shadow {
                        color: Color::from_rgba8(0, 0, 0, 0.50),
                        offset: Vector::new(0.0, 5.0),
                        blur_radius: 25.0,
                    },
                    snap: false,
                }
            }
            ContainerStyleInner::ActionShortcutModifier => {
                let theme = &self.action_shortcut_modifier;
                let background_color = &theme.background_color;
                let border_color = &theme.border_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: border_color.clone().into(),
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
            ContainerStyleInner::ContentCodeBlockText => {
                let theme = &self.content_code_block_text;
                let background_color = &theme.background_color;
                let border_color = &theme.border_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: border_color.clone().into(),
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
            ContainerStyleInner::Main => {
                let theme = &self.root;
                let background_color = &theme.background_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: theme.border_radius.clone().into(),
                        width: theme.border_width,
                        color: theme.border_color,
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
            ContainerStyleInner::Root => {
                let theme = &self.root;
                let background_color = &theme.background_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: theme.border_radius.clone().into(),
                        width: theme.border_width,
                        color: theme.border_color,
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
            ContainerStyleInner::Tooltip => {
                let theme = &self.popup;
                let tooltip_theme = &self.tooltip;
                let background_color = &tooltip_theme.background_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: theme.border_radius.clone().into(),
                        width: theme.border_width,
                        color: theme.border_color,
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
            ContainerStyleInner::ContentImage => {
                let theme = &self.content_image;

                // TODO this border radius doesn't work on image, for some reason

                Style {
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Style::default()
                }
            }
            ContainerStyleInner::RootBottomPanel => {
                let root_theme = &self.root;
                let panel_theme = &self.root_bottom_panel;

                Style {
                    background: Some(panel_theme.background_color.into()),
                    border: Border {
                        radius: gauntlet_common_ui::radius(
                            0.0,
                            0.0,
                            root_theme.border_radius,
                            root_theme.border_radius,
                        ),
                        width: root_theme.border_width,
                        color: root_theme.border_color,
                    },
                    ..Style::default()
                }
            }
            ContainerStyleInner::InlineInner => {
                let theme = &self.inline_inner;

                Style {
                    background: Some(theme.background_color.into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color,
                    },
                    ..Style::default()
                }
            }
            ContainerStyleInner::Hud => {
                let theme = &self.hud;
                let background_color = &theme.background_color;

                Style {
                    text_color: None,
                    background: Some(background_color.clone().into()),
                    border: Border {
                        radius: theme.border_radius.into(),
                        width: theme.border_width,
                        color: theme.border_color,
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
            ContainerStyleInner::MainListItemAlias => {
                let theme = &self.main_list_item_alias;

                Style {
                    text_color: None,
                    background: None,
                    border: Border {
                        color: theme.background_color.clone().into(),
                        width: 2.0,
                        radius: theme.border_radius.into(),
                    },
                    shadow: Default::default(),
                    snap: false,
                }
            }
        }
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Container<'a, Message, GauntletComplexTheme, Renderer> {
    type Kind = ContainerStyle;

    fn themed(self, name: ContainerStyle) -> Element<'a, Message> {
        let theme = get_theme();

        match name {
            ContainerStyle::RootInner => self.padding(0.0),
            ContainerStyle::ActionPanelTitle => self.padding(theme.action_panel_title.padding.to_iced()),
            ContainerStyle::ActionSectionTitle => self.padding(theme.action_section_title.padding.to_iced()),
            ContainerStyle::ActionShortcutModifier => {
                self.class(ContainerStyleInner::ActionShortcutModifier)
                    .padding(theme.action_shortcut_modifier.padding.to_iced())
            }
            ContainerStyle::ActionShortcutModifiersInit => {
                let horizontal_spacing = theme.action_shortcut_modifier.spacing;
                self.padding(gauntlet_common_ui::padding(0.0, horizontal_spacing, 0.0, 0.0))
            }
            ContainerStyle::ActionPanel => {
                self.class(ContainerStyleInner::ActionPanel)
                    .padding(theme.action_panel.padding.to_iced())
                    .height(Length::Fixed(250.0))
                    .width(Length::Fixed(350.0))
            }
            ContainerStyle::MetadataTagItem => self.padding(theme.metadata_tag_item.padding.to_iced()),
            ContainerStyle::MetadataItemLabel => self.padding(theme.metadata_item_label.padding.to_iced()),
            ContainerStyle::MetadataLinkIcon => self.padding(theme.metadata_link_icon.padding.to_iced()),
            ContainerStyle::MetadataItemValue => self.padding(theme.metadata_item_value.padding.to_iced()),
            ContainerStyle::MetadataItemValueInList => {
                self.padding(theme.metadata_item_value_in_list.padding.to_iced())
            }
            ContainerStyle::RootBottomPanel => {
                self.class(ContainerStyleInner::RootBottomPanel)
                    .padding(theme.root_bottom_panel.padding.to_iced())
            }
            ContainerStyle::RootTopPanel => self.padding(theme.root_top_panel.padding.to_iced()),
            ContainerStyle::ListItemSubtitle => self.padding(theme.list_item_subtitle.padding.to_iced()),
            ContainerStyle::ListItemTitle => self.padding(theme.list_item_title.padding.to_iced()),
            ContainerStyle::ListItemIcon => self.padding(theme.list_item_icon.padding.to_iced()),
            ContainerStyle::ContentParagraph => self.padding(theme.content_paragraph.padding.to_iced()),
            ContainerStyle::ContentHorizontalBreak => self.padding(theme.content_horizontal_break.padding.to_iced()),
            ContainerStyle::ContentCodeBlock => self.padding(theme.content_code_block.padding.to_iced()),
            ContainerStyle::ContentCodeBlockText => {
                self.class(ContainerStyleInner::ContentCodeBlockText)
                    .padding(theme.content_code_block_text.padding.to_iced())
            }
            ContainerStyle::ContentImage => {
                self.class(ContainerStyleInner::ContentImage)
                    .padding(theme.content_image.padding.to_iced())
            }
            ContainerStyle::DetailContentInner => self.padding(theme.metadata_content_inner.padding.to_iced()),
            ContainerStyle::MetadataInner => self.padding(theme.metadata_inner.padding.to_iced()),
            ContainerStyle::MetadataSeparator => self.padding(theme.metadata_separator.padding.to_iced()),
            ContainerStyle::DetailMetadata => self.padding(theme.detail_metadata.padding.to_iced()),
            ContainerStyle::DetailContent => self.padding(theme.detail_content.padding.to_iced()),
            ContainerStyle::FormInputLabel => self.padding(theme.form_input_label.padding.to_iced()),
            ContainerStyle::Inline => self.padding(theme.inline.padding.to_iced()),
            ContainerStyle::InlineInner => {
                self.height(120)
                    .max_height(120)
                    .padding(theme.inline_inner.padding.to_iced())
                    .class(ContainerStyleInner::InlineInner)
            }
            ContainerStyle::InlineName => self.padding(theme.inline_name.padding.to_iced()),
            ContainerStyle::EmptyViewImage => {
                self.padding(theme.empty_view_image.padding.to_iced())
                    .max_width(theme.empty_view_image.size.width)
                    .max_height(theme.empty_view_image.size.height)
            }
            ContainerStyle::Main => self.class(ContainerStyleInner::Main),
            ContainerStyle::MainListItemText => self.padding(theme.main_list_item_text.padding.to_iced()),
            ContainerStyle::MainListItemSubText => self.padding(theme.main_list_item_sub_text.padding.to_iced()),
            ContainerStyle::MainListItemIcon => self.padding(theme.main_list_item_icon.padding.to_iced()),
            ContainerStyle::MainListItemAlias => {
                self.padding(theme.main_list_item_alias.padding.to_iced())
                    .class(ContainerStyleInner::MainListItemAlias)
            }
            ContainerStyle::MainList => self.padding(theme.main_list.padding.to_iced()),
            ContainerStyle::MainListInner => self.padding(theme.main_list_inner.padding.to_iced()),
            ContainerStyle::MainSearchBar => self.padding(theme.main_search_bar.padding.to_iced()),
            ContainerStyle::Root => self.class(ContainerStyleInner::Root),
            ContainerStyle::PluginErrorViewTitle => self.padding(theme.plugin_error_view_title.padding.to_iced()),
            ContainerStyle::PluginErrorViewDescription => {
                self.padding(theme.plugin_error_view_description.padding.to_iced())
            }
            ContainerStyle::PreferenceRequiredViewDescription => {
                self.padding(theme.preference_required_view_description.padding.to_iced())
            }
            ContainerStyle::Form => self.padding(theme.form.padding.to_iced()),
            ContainerStyle::FormInner => self.padding(theme.form_inner.padding.to_iced()),
            ContainerStyle::GridInner => self.padding(theme.grid_inner.padding.to_iced()),
            ContainerStyle::Grid => self.padding(theme.grid.padding.to_iced()),
            ContainerStyle::List => self.padding(theme.list.padding.to_iced()),
            ContainerStyle::ListInner => self.padding(theme.list_inner.padding.to_iced()),
            ContainerStyle::RootBottomPanelActionToggleText => {
                self.padding(theme.root_bottom_panel_action_toggle_text.padding.to_iced())
            }
            ContainerStyle::RootBottomPanelPrimaryActionText => {
                self.padding(theme.root_bottom_panel_primary_action_text.padding.to_iced())
            }
            ContainerStyle::RootBottomPanelPrimaryActionButton => {
                self.padding(gauntlet_common_ui::padding(
                    0.0,
                    theme.root_bottom_panel.spacing,
                    0.0,
                    0.0,
                ))
            }
            ContainerStyle::TextAccessory => self.padding(theme.text_accessory.padding.to_iced()),
            ContainerStyle::TextAccessoryIcon => {
                let horizontal_spacing = theme.text_accessory.spacing;
                self.padding(gauntlet_common_ui::padding(0.0, horizontal_spacing, 0.0, 0.0))
            }
            ContainerStyle::IconAccessory => self.padding(theme.icon_accessory.padding.to_iced()),
            ContainerStyle::HudInner => self.padding(theme.hud_content.padding.to_iced()),
            ContainerStyle::Hud => self.class(ContainerStyleInner::Hud),
        }
        .into()
    }
}
