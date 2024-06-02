use iced::{Alignment, Length};
use iced::advanced::image::Handle;
use iced::widget::{column, Component, container, horizontal_space};
use iced::widget::button;
use iced::widget::component;
use iced::widget::row;
use iced::widget::text;

use common::model::UiSearchResult;

use crate::ui::themable_widget::{ThemableWidget, ThemeKindButton, ThemeKindContainer, ThemeKindImage, ThemeKindSpace, ThemeKindText};
use crate::ui::theme::{Element, GauntletTheme};

pub struct SearchList<Message> {
    on_select: Box<dyn Fn(UiSearchResult) -> Message>,
    search_results: Vec<UiSearchResult>,
}

pub fn search_list<Message>(
    search_results: Vec<UiSearchResult>,
    on_select: impl Fn(UiSearchResult) -> Message + 'static,
) -> SearchList<Message> {
    SearchList::new(search_results, on_select)
}

#[derive(Debug, Clone)]
pub struct SelectItemEvent(UiSearchResult);

impl<Message> SearchList<Message> {
    pub fn new(
        search_results: Vec<UiSearchResult>,
        on_open_view: impl Fn(UiSearchResult) -> Message + 'static,
    ) -> Self {
        Self {
            search_results,
            on_select: Box::new(on_open_view),
        }
    }
}

impl<Message> Component<Message, GauntletTheme> for SearchList<Message> {
    type State = ();
    type Event = SelectItemEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        SelectItemEvent(event): SelectItemEvent,
    ) -> Option<Message> {
        Some((self.on_select)(event))
    }

    fn view(&self, _state: &Self::State) -> Element<SelectItemEvent> {
        let items: Vec<Element<_>> = self.search_results
            .iter()
            .map(|search_result| {
                let main_text: Element<_> = text(&search_result.entrypoint_name)
                    .into();
                let main_text: Element<_> = container(main_text)
                    .themed(ThemeKindContainer::MainListItemText);

                let spacer: Element<_> = horizontal_space()
                    .width(Length::Fill)
                    .into();

                let sub_text: Element<_> = text(&search_result.plugin_name)
                    .themed(ThemeKindText::Subtext);
                let sub_text: Element<_> = container(sub_text)
                    .themed(ThemeKindContainer::MainListItemSubText); // FIXME find a way to set padding based on whether the scroll bar is visible

                let mut button_content = vec![];

                if let Some(path) = &search_result.entrypoint_icon {
                    let image: Element<_> = iced::widget::image(Handle::from_path(path))
                        .themed(ThemeKindImage::MainListItemIcon);

                    let image: Element<_> = container(image)
                        .themed(ThemeKindContainer::MainListItemIcon);

                    button_content.push(image);
                } else {
                    let spacer: Element<_> = horizontal_space() // TODO replace with grayed out gauntlet icon
                        .themed(ThemeKindSpace::MainListItemIcon);

                    let spacer: Element<_> = container(spacer)
                        .themed(ThemeKindContainer::MainListItemIcon);

                    button_content.push(spacer);
                }

                button_content.push(main_text);
                button_content.push(spacer);
                button_content.push(sub_text);

                let button_content: Element<_> = row(button_content)
                    .align_items(Alignment::Center)
                    .into();

                button(button_content)
                    .width(Length::Fill)
                    .on_press(SelectItemEvent(search_result.clone()))
                    .themed(ThemeKindButton::MainListItem)
            })
            .collect();

        column(items).into()
    }
}

impl<'a, Message> From<SearchList<Message>> for Element<'a, Message>
    where
        Message: 'a,
{
    fn from(search_list: SearchList<Message>) -> Self {
        component(search_list)
    }
}