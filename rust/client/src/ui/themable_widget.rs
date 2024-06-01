use iced::{Length, Padding, Renderer};
use iced::widget::{Button, Container, Row};
use iced_aw::Grid;
use serde::Deserialize;

use crate::ui::theme::{ButtonStyle, ContainerStyle, Element, GauntletTheme};

#[derive(Deserialize, Debug, Clone)]
pub enum GLength {
    Default,
    Fixed(f32)
}

pub enum ThemeKindRow {
    DetailContent,
    ActionShortcut,
    FormInput,
    ListGridSectionTitle,
}

pub enum ThemeKindButton {
    Action,
    ActionPopup,
    GridItem,
    TopPanelBackButton,
    ListItem,
}

pub enum ThemeKindContainer {
    Root,
    ActionShortcutModifier,
    ActionShortcutModifiersInit, // "init" means every item on list except last one
    ActionPanel,
    MetadataTagItem,
    MetadataItemValue,
    RootBottomPanel,
    ContentParagraph,
    RootTopPanel,
    ListItemSubtitle,
    ListItemTitle,
    ContentHorizontalBreak,
    ContentCodeBlockText,
    MetadataSeparator,
    DetailMetadata,
    DetailContent,
    FormInputLabel,
    Inline,
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
        match name {
            ThemeKindRow::DetailContent => {
                self.padding(Padding::new(0.0))
            },
            ThemeKindRow::ActionShortcut => {
                self.padding(Padding::new(0.0))
            },
            ThemeKindRow::FormInput => {
                self.padding(Padding::new(10.0))
            }
            ThemeKindRow::ListGridSectionTitle => {
                self.padding(Padding::from([5.0, 8.0])) // 5 + 3 to line up a section with items
            }
        }.into()
    }
}

impl<'a, Message: 'a + Clone> ThemableWidget<'a, Message> for Button<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindButton;

    fn themed(mut self, kind: ThemeKindButton) -> Element<'a, Message> {
        match kind {
            ThemeKindButton::Action => {
                self.style(ButtonStyle::GauntletButton)
            },
            ThemeKindButton::ActionPopup => {
                self.style(ButtonStyle::Secondary).padding(Padding::from([0.0, 5.0]))
            },
            ThemeKindButton::GridItem => {
                self.style(ButtonStyle::GauntletGridButton).padding(Padding::new(5.0))
            }
            ThemeKindButton::TopPanelBackButton => {
                self.padding(Padding::from([3.0, 5.0])).style(ButtonStyle::Secondary)
            }
            ThemeKindButton::ListItem => {
                self.style(ButtonStyle::GauntletButton).padding(Padding::new(5.0))
            }
        }.into()
    }
}

impl<'a, Message: 'a> ThemableWidget<'a, Message> for Container<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindContainer;

    fn themed(mut self, name: ThemeKindContainer) -> Element<'a, Message> {
        match name {
            ThemeKindContainer::Root => {
                self.padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
            }
            ThemeKindContainer::ActionShortcutModifier => {
                self.style(ContainerStyle::Code).padding(Padding::from([0.0, 5.0]))
            }
            ThemeKindContainer::ActionShortcutModifiersInit => {
                let horizontal_spacing = 10.0;
                self.padding(Padding::from([0.0, horizontal_spacing, 0.0, 0.0]))
            }
            ThemeKindContainer::ActionPanel => {
                self
                    .style(ContainerStyle::Background)
                    .padding(Padding::new(10.0))
                    .height(Length::Fixed(200.0))
                    .width(Length::Fixed(300.0))

            }
            ThemeKindContainer::MetadataTagItem => {
                self.padding(Padding::new(5.0))
            }
            ThemeKindContainer::MetadataItemValue => {
                self.padding(Padding::new(5.0))
            }
            ThemeKindContainer::RootBottomPanel => {
                self.padding(Padding::new(5.0))
            }
            ThemeKindContainer::RootTopPanel => {
                self.padding(Padding::new(10.0))
            }
            ThemeKindContainer::ListItemSubtitle => {
                self.padding(Padding::new(3.0))
            }
            ThemeKindContainer::ListItemTitle => {
                self.padding(Padding::new(3.0))
            }
            ThemeKindContainer::ContentParagraph => {
                self.padding(Padding::new(5.0))
            }
            ThemeKindContainer::ContentHorizontalBreak => {
                self.padding(Padding::from([10.0, 0.0]))
            }
            ThemeKindContainer::ContentCodeBlockText => {
                self.padding(Padding::from([3.0, 5.0]))
            }
            ThemeKindContainer::MetadataSeparator => {
                self.padding(Padding::from([10.0, 0.0]))
            }
            ThemeKindContainer::DetailMetadata => {
                self.padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
            }
            ThemeKindContainer::DetailContent => {
                self.padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
            }
            ThemeKindContainer::FormInputLabel => {
                self.padding(Padding::from([5.0, 10.0]))
            }
            ThemeKindContainer::Inline => {
                self
                    .padding(Padding::new(5.0))
                    .height(100)
                    .max_height(100)
            }
            ThemeKindContainer::EmptyViewImage => {
                self.padding(Padding::new(10.0))
            }
        }.into()
    }
}



impl<'a, Message: 'a + Clone + 'static> ThemableWidget<'a, Message> for Grid<'a, Message, GauntletTheme, Renderer> {
    type Kind = ThemeKindGrid;

    fn themed(mut self, kind: ThemeKindGrid) -> Element<'a, Message> {
        match kind {
            ThemeKindGrid::Grid => {
                self.spacing(10.0)
            },
        }.into()
    }
}
