use iced::advanced::image::Handle;
use iced::{Alignment, Length};
use iced::Padding;
use iced::widget::{column, Component, container, horizontal_space};
use iced::widget::button;
use iced::widget::component;
use iced::widget::row;
use iced::widget::text;

use crate::model::NativeUiSearchResult;
use crate::ui::theme::{ButtonStyle, Element, GauntletTheme, TextStyle};

pub struct SearchList<Message> {
    on_select: Box<dyn Fn(NativeUiSearchResult) -> Message>,
    search_results: Vec<NativeUiSearchResult>,
}

pub fn search_list<Message>(
    search_results: Vec<NativeUiSearchResult>,
    on_select: impl Fn(NativeUiSearchResult) -> Message + 'static,
) -> SearchList<Message> {
    SearchList::new(search_results, on_select)
}

#[derive(Debug, Clone)]
pub struct SelectItemEvent(NativeUiSearchResult);

impl<Message> SearchList<Message> {
    pub fn new(
        search_results: Vec<NativeUiSearchResult>,
        on_open_view: impl Fn(NativeUiSearchResult) -> Message + 'static,
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
                    .padding(Padding::new(3.0))
                    .into();

                let spacer: Element<_> = horizontal_space()
                    .width(Length::Fill)
                    .into();

                let sub_text: Element<_> = text(&search_result.plugin_name)
                    .style(TextStyle::Subtext)
                    .into();
                let sub_text: Element<_> = container(sub_text)
                    .padding(Padding::from([3.0, 10.0])) // FIXME find a way to set padding based on whether the scroll bar is visible
                    .into();

                let mut button_content = row(vec![])
                    .align_items(Alignment::Center);

                if let Some(path) = &search_result.entrypoint_icon {
                    let image: Element<_> = iced::widget::image(Handle::from_path(path))
                        .width(16)
                        .height(16)
                        .into();

                    let image: Element<_> = container(image)
                        .padding(Padding::from([0.0, 7.0, 0.0, 5.0]))
                        .into();

                    button_content = button_content.push(image);
                } else {
                    let spacer: Element<_> = horizontal_space() // TODO replace with grayed out gauntlet icon
                        .width(16)
                        .into();

                    let spacer: Element<_> = container(spacer)
                        .padding(Padding::from([0.0, 7.0, 0.0, 5.0]))
                        .into();

                    button_content = button_content.push(spacer);
                }

                button_content = button_content
                    .push(main_text)
                    .push(spacer)
                    .push(sub_text);

                let button_content: Element<_> = button_content.into();

                button(button_content)
                    .width(Length::Fill)
                    .style(ButtonStyle::GauntletButton)
                    .on_press(SelectItemEvent(search_result.clone()))
                    .padding(Padding::new(5.0))
                    .into()
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