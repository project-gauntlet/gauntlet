use iced::{Length, Padding, Renderer};
use iced::widget::{Button, Container, Image, Row};
use iced_aw::Grid;

use crate::ui::theme::{ButtonStyle, ContainerStyle, Element, GauntletTheme};

static THEME: once_cell::sync::OnceCell<ExternalTheme> = once_cell::sync::OnceCell::new();

pub static DEFAULT_THEME: ExternalTheme = ExternalTheme {
    action_panel: ExternalThemePaddingOnly {
        padding: padding_all(10.0),
    },
    action: ExternalThemePaddingOnly {
        padding: padding_all(5.0),
    },
    action_shortcut: ExternalThemePaddingOnly {
        padding: padding_all(0.0)
    },
    action_shortcut_modifier: ExternalThemePaddingSpacing {
        padding: padding_axis(0.0, 5.0),
        spacing: 10.0,
    },
    form_input: ExternalThemePaddingOnly {
        padding: padding_all(10.0)
    },
    metadata_tag_item: ExternalThemePaddingOnly {
        padding: padding_all(5.0),
    },
    metadata_item_value: ExternalThemePaddingOnly {
        padding: padding_all(5.0),
    },
    root_bottom_panel: ExternalThemePaddingOnly {
        padding: padding_all(5.0),
    },
    root_top_panel: ExternalThemePaddingOnly {
        padding: padding_all(10.0),
    },
    list_item_subtitle: ExternalThemePaddingOnly {
        padding: padding_all(3.0),
    },
    list_item_title: ExternalThemePaddingOnly {
        padding: padding_all(3.0),
    },
    content_paragraph: ExternalThemePaddingOnly {
        padding: padding_all(5.0)
    },
    content_code_block: ExternalThemePaddingOnly {
        padding: padding_all(0.0),
    },
    content_image: ExternalThemePaddingOnly {
        padding: padding_all(0.0)
    },
    inline: ExternalThemePaddingOnly {
        padding: padding_all(5.0)
    },
    empty_view_image: ExternalThemePaddingSize {
        padding: padding_all(10.0),
        size: ExternalThemeSize {
            width: 100.0,
            height: 100.0,
        },
    },
    grid_item: ExternalThemePaddingOnly {
        padding: padding_all(5.0),
    },
    content_horizontal_break: ExternalThemePaddingOnly {
        padding: padding_axis(10.0, 0.0),
    },
    grid: ExternalThemeSpacing {
        spacing: 10.0,
    },
    content_code_block_text: ExternalThemePaddingOnly {
        padding: padding_axis(3.0, 5.0),
    },
    metadata_separator: ExternalThemePaddingOnly {
        padding: padding_axis(10.0, 0.0),
    },
    root_top_panel_button: ExternalThemePaddingOnly {
        padding: padding_axis(3.0, 5.0),
    },
    root_bottom_panel_action_button: ExternalThemePaddingOnly {
        padding: padding_axis(0.0, 5.0),
    },
    list_item: ExternalThemePaddingOnly {
        padding: padding_all(5.0),
    },
    detail_metadata: ExternalThemePaddingOnly {
        padding: padding(5.0, 5.0, 0.0, 5.0), // zero because it is inside scrollable
    },
    detail_content: ExternalThemePaddingOnly {
        padding: padding(5.0, 5.0, 0.0, 5.0),
    },
    root_content: ExternalThemePaddingOnly {
        padding: padding(5.0, 5.0, 0.0, 5.0),
    },
    form_input_label: ExternalThemePaddingOnly {
        padding: padding_axis(5.0, 10.0),
    },
    list_section_title: ExternalThemePaddingOnly {
        padding: padding_axis(5.0, 8.0), // 5 + 3 to line up a section with items
    },
    grid_section_title: ExternalThemePaddingOnly {
        padding: padding_axis(5.0, 8.0), // 5 + 3 to line up a section with items
    },
};

const fn padding(top: f32, right: f32, bottom: f32, left: f32) -> ExternalThemePadding {
    ExternalThemePadding {
        top,
        right,
        bottom,
        left,
    }
}

const fn padding_all(value: f32) -> ExternalThemePadding {
    ExternalThemePadding {
        top: value,
        right: value,
        bottom: value,
        left: value,
    }
}

const fn padding_axis(vertical: f32, horizontal: f32) -> ExternalThemePadding {
    ExternalThemePadding {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

pub fn init_theme(theme: ExternalTheme) {
    THEME.set(theme).expect("already set");
}

fn get_theme() -> &'static ExternalTheme {
    THEME.get().expect("theme global var was not set")
}


#[derive(Debug, Clone)]
pub struct ExternalTheme {
    action: ExternalThemePaddingOnly,
    action_panel: ExternalThemePaddingOnly,
    action_shortcut: ExternalThemePaddingOnly,
    action_shortcut_modifier: ExternalThemePaddingSpacing,
    content_code_block: ExternalThemePaddingOnly,
    content_code_block_text: ExternalThemePaddingOnly,
    content_horizontal_break: ExternalThemePaddingOnly,
    content_image: ExternalThemePaddingOnly,
    content_paragraph: ExternalThemePaddingOnly,
    detail_content: ExternalThemePaddingOnly,
    detail_metadata: ExternalThemePaddingOnly,
    empty_view_image: ExternalThemePaddingSize,
    form_input: ExternalThemePaddingOnly,
    form_input_label: ExternalThemePaddingOnly,
    grid: ExternalThemeSpacing,
    grid_item: ExternalThemePaddingOnly,
    grid_section_title: ExternalThemePaddingOnly,
    inline: ExternalThemePaddingOnly,
    list_item: ExternalThemePaddingOnly,
    list_item_subtitle: ExternalThemePaddingOnly,
    list_item_title: ExternalThemePaddingOnly,
    list_section_title: ExternalThemePaddingOnly,
    metadata_item_value: ExternalThemePaddingOnly,
    metadata_separator: ExternalThemePaddingOnly,
    metadata_tag_item: ExternalThemePaddingOnly,
    root_bottom_panel: ExternalThemePaddingOnly,
    root_bottom_panel_action_button: ExternalThemePaddingOnly,
    root_content: ExternalThemePaddingOnly,
    root_top_panel: ExternalThemePaddingOnly,
    root_top_panel_button: ExternalThemePaddingOnly,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingOnly {
    padding: ExternalThemePadding,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingSize {
    padding: ExternalThemePadding,
    size: ExternalThemeSize
}

#[derive(Debug, Clone)]
pub struct ExternalThemePaddingSpacing {
    padding: ExternalThemePadding,
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSpacing {
    spacing: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemeSize {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
pub struct ExternalThemePadding {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl ExternalThemePadding {
    fn to_iced(&self) -> Padding {
        Padding {
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            left: self.left,
        }
    }
}

pub enum ThemeKindRow {
    ActionShortcut,
    FormInput,
    ListSectionTitle,
    GridSectionTitle,
}

pub enum ThemeKindButton {
    Action,
    RootBottomPanelActionButton,
    GridItem,
    ListItem,
    RootTopPanelBackButton,
}

pub enum ThemeKindContainer {
    ActionPanel,
    ActionShortcutModifier,
    ActionShortcutModifiersInit, // "init" means every item on list except last one
    ContentCodeBlock,
    ContentCodeBlockText,
    ContentHorizontalBreak,
    ContentImage,
    ContentParagraph,
    DetailContent,
    DetailMetadata,
    EmptyViewImage,
    FormInputLabel,
    Inline,
    ListItemSubtitle,
    ListItemTitle,
    MetadataItemValue,
    MetadataSeparator,
    MetadataTagItem,
    RootContent, // TODO "content" has different meaning in this context. need better name
    RootBottomPanel,
    RootTopPanel,
}

pub enum ThemeKindImage {
    EmptyViewImage,
}

pub enum ThemeKindGrid {
    Grid,
}

pub trait ThemableWidget<'a, Message> {
    type Kind;

    fn themed(self, name: Self::Kind) -> Element<'a, Message>;
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Row<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindRow;

    fn themed(self, name: ThemeKindRow) -> Element<'a, Message> {
        let theme = get_theme();

        match name {
            ThemeKindRow::ActionShortcut => {
                self.padding(theme.action_shortcut.padding.to_iced())
            }
            ThemeKindRow::FormInput => {
                self.padding(theme.form_input.padding.to_iced())
            }
            ThemeKindRow::ListSectionTitle => {
                self.padding(theme.list_section_title.padding.to_iced())
            }
            ThemeKindRow::GridSectionTitle => {
                self.padding(theme.grid_section_title.padding.to_iced())
            }
        }.into()
    }
}

impl<'a, Message: 'a + Clone> ThemableWidget<'a, Message> for Button<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindButton;

    fn themed(mut self, kind: ThemeKindButton) -> Element<'a, Message> {
        let theme = get_theme();

        match kind {
            ThemeKindButton::Action => {
                self.style(ButtonStyle::GauntletButton).padding(theme.action.padding.to_iced())
            },
            ThemeKindButton::RootBottomPanelActionButton => {
                self.style(ButtonStyle::Secondary).padding(theme.root_bottom_panel_action_button.padding.to_iced())
            },
            ThemeKindButton::GridItem => {
                self.style(ButtonStyle::GauntletGridButton).padding(theme.grid_item.padding.to_iced())
            }
            ThemeKindButton::RootTopPanelBackButton => {
                self.style(ButtonStyle::Secondary).padding(theme.root_top_panel_button.padding.to_iced())
            }
            ThemeKindButton::ListItem => {
                self.style(ButtonStyle::GauntletButton).padding(theme.list_item.padding.to_iced())
            }
        }.into()
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Container<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindContainer;

    fn themed(mut self, name: ThemeKindContainer) -> Element<'a, Message> {
        let theme = get_theme();

        match name {
            ThemeKindContainer::RootContent => {
                self.padding(theme.root_content.padding.to_iced())
            }
            ThemeKindContainer::ActionShortcutModifier => {
                self.style(ContainerStyle::Code).padding(theme.action_shortcut_modifier.padding.to_iced())
            }
            ThemeKindContainer::ActionShortcutModifiersInit => {
                let horizontal_spacing = theme.action_shortcut_modifier.spacing;
                self.padding(Padding::from([0.0, horizontal_spacing, 0.0, 0.0]))
            }
            ThemeKindContainer::ActionPanel => {
                self
                    .style(ContainerStyle::Background)
                    .padding(theme.action_panel.padding.to_iced())
                    .height(Length::Fixed(200.0))
                    .width(Length::Fixed(300.0))
            }
            ThemeKindContainer::MetadataTagItem => {
                self.padding(theme.metadata_tag_item.padding.to_iced())
            }
            ThemeKindContainer::MetadataItemValue => {
                self.padding(theme.metadata_item_value.padding.to_iced())
            }
            ThemeKindContainer::RootBottomPanel => {
                self.padding(theme.root_bottom_panel.padding.to_iced())
            }
            ThemeKindContainer::RootTopPanel => {
                self.padding(theme.root_top_panel.padding.to_iced())
            }
            ThemeKindContainer::ListItemSubtitle => {
                self.padding(theme.list_item_subtitle.padding.to_iced())
            }
            ThemeKindContainer::ListItemTitle => {
                self.padding(theme.list_item_title.padding.to_iced())
            }
            ThemeKindContainer::ContentParagraph => {
                self.padding(theme.content_paragraph.padding.to_iced())
            }
            ThemeKindContainer::ContentHorizontalBreak => {
                self.padding(theme.content_horizontal_break.padding.to_iced())
            }
            ThemeKindContainer::ContentCodeBlock => {
                self.padding(theme.content_code_block.padding.to_iced())
            }
            ThemeKindContainer::ContentCodeBlockText => {
                self.style(ContainerStyle::Code)
                    .padding(theme.content_code_block_text.padding.to_iced())
            }
            ThemeKindContainer::ContentImage => {
                self.padding(theme.content_image.padding.to_iced())
            }
            ThemeKindContainer::MetadataSeparator => {
                self.padding(theme.metadata_separator.padding.to_iced())
            }
            ThemeKindContainer::DetailMetadata => {
                self.padding(theme.detail_metadata.padding.to_iced())
            }
            ThemeKindContainer::DetailContent => {
                self.padding(theme.detail_content.padding.to_iced())
            }
            ThemeKindContainer::FormInputLabel => {
                self.padding(theme.form_input_label.padding.to_iced())
            }
            ThemeKindContainer::Inline => {
                self
                    .padding(theme.inline.padding.to_iced())
                    .height(100)
                    .max_height(100)
            }
            ThemeKindContainer::EmptyViewImage => {
                self.padding(theme.empty_view_image.padding.to_iced())
            }
        }.into()
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Image<iced::advanced::image::Handle> {
    type Kind = ThemeKindImage;

    fn themed(mut self, kind: ThemeKindImage) -> Element<'a, Message> {
        let theme = get_theme();

        match kind {
            ThemeKindImage::EmptyViewImage => {
                self.width(theme.empty_view_image.size.width)
                    .height(theme.empty_view_image.size.height)
            },
        }.into()
    }
}

impl<'a, Message: 'a + 'static> ThemableWidget<'a, Message> for Grid<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindGrid;

    fn themed(mut self, kind: ThemeKindGrid) -> Element<'a, Message> {
        let theme = get_theme();

        match kind {
            ThemeKindGrid::Grid => {
                self.spacing(theme.grid.spacing)
            },
        }.into()
    }
}
