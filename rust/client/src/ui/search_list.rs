use iced::Length;
use iced::Padding;
use iced::widget::{column, Component, container};
use iced::widget::button;
use iced::widget::component;
use iced::widget::row;
use iced::widget::text;

use common::model::{EntrypointId, PluginId};

use crate::model::{NativeUiSearchResult, SearchResultEntrypointType};
use crate::ui::theme::{ButtonStyle, Element, GauntletTheme, TextStyle};

pub struct SearchList<Message> {
    on_open_view: Box<dyn Fn(OpenViewEvent) -> Message>,
    on_run_command: Box<dyn Fn(RunCommandEvent) -> Message>,
    search_results: Vec<NativeUiSearchResult>,
}

pub fn search_list<Message>(
    search_results: Vec<NativeUiSearchResult>,
    on_open_view: impl Fn(OpenViewEvent) -> Message + 'static,
    on_run_command: impl Fn(RunCommandEvent) -> Message + 'static
) -> SearchList<Message> {
    SearchList::new(search_results, on_open_view, on_run_command)
}

pub struct OpenViewEvent {
    pub plugin_id: PluginId,
    pub entrypoint_id: EntrypointId,
}

pub struct RunCommandEvent {
    pub plugin_id: PluginId,
    pub entrypoint_id: EntrypointId,
}

#[derive(Debug, Clone)]
pub enum Event {
    RunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    OpenView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}

impl<Message> SearchList<Message> {
    pub fn new(
        search_results: Vec<NativeUiSearchResult>,
        on_open_view: impl Fn(OpenViewEvent) -> Message + 'static,
        on_run_command: impl Fn(RunCommandEvent) -> Message + 'static
    ) -> Self {
        Self {
            search_results,
            on_open_view: Box::new(on_open_view),
            on_run_command: Box::new(on_run_command),
        }
    }
}

impl<Message> Component<Message, GauntletTheme> for SearchList<Message> {
    type State = ();
    type Event = Event;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Event,
    ) -> Option<Message> {
        match event {
            Event::OpenView { plugin_id, entrypoint_id } => {
                let event = OpenViewEvent { plugin_id, entrypoint_id, };
                Some((self.on_open_view)(event))
            }
            Event::RunCommand { plugin_id, entrypoint_id } => {
                let event = RunCommandEvent { plugin_id, entrypoint_id, };
                Some((self.on_run_command)(event))
            }
        }
    }

    fn view(&self, _state: &Self::State) -> Element<Event> {
        let items: Vec<Element<_>> = self.search_results
            .iter()
            .map(|search_result| {
                let main_text: Element<_> = text(&search_result.entrypoint_name)
                    .into();
                let main_text: Element<_> = container(main_text)
                    .padding(Padding::new(3.0))
                    .into();

                let sub_text: Element<_> = text(&search_result.plugin_name)
                    .style(TextStyle::Subtext)
                    .into();
                let sub_text: Element<_> = container(sub_text)
                    .padding(Padding::new(3.0))
                    .into();

                let button_content: Element<_> = row(vec![
                    main_text,
                    sub_text,
                ]).into();

                let event = match search_result.entrypoint_type {
                    SearchResultEntrypointType::Command => Event::RunCommand {
                        entrypoint_id: search_result.entrypoint_id.clone(),
                        plugin_id: search_result.plugin_id.clone()
                    },
                    SearchResultEntrypointType::View => Event::OpenView {
                        entrypoint_id: search_result.entrypoint_id.clone(),
                        plugin_id: search_result.plugin_id.clone()
                    }
                };

                button(button_content)
                    .width(Length::Fill)
                    .style(ButtonStyle::EntrypointItem)
                    .on_press(event)
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