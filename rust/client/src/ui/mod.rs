use std::sync::{Arc, RwLock as StdRwLock};

use iced::{Application, Command, Event, executor, font, futures, keyboard, Length, Padding, Size, Subscription, subscription};
use iced::advanced::Widget;
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::KeyCode;
use iced::Settings;
use iced::widget::{column, container, horizontal_rule, scrollable, text_input};
use iced::widget::text_input::focus;
use iced::window::Position;
use iced_aw::graphics::icons;
use tokio::sync::RwLock as TokioRwLock;
use zbus::{Connection, InterfaceRef};

use common::dbus::{DBusEntrypointType, DbusEventRenderView, DbusEventRunCommand};
use common::model::{EntrypointId, PluginId};
use utils::channel::{channel, RequestReceiver};

use crate::dbus::{DbusClient, DbusServerProxyProxy};
use crate::model::{NativeUiRequestData, NativeUiResponseData, NativeUiSearchResult, SearchResultEntrypointType};
use crate::ui::plugin_container::{ClientContext, plugin_container};
use crate::ui::search_list::search_list;
use crate::ui::theme::{ContainerStyle, Element, GauntletTheme};
use crate::ui::widget::ComponentWidgetEvent;

mod plugin_container;
mod search_list;
mod widget;
mod theme;

pub struct AppModel {
    client_context: Arc<StdRwLock<ClientContext>>,
    dbus_connection: Connection,
    dbus_server: DbusServerProxyProxy<'static>,
    dbus_client: InterfaceRef<DbusClient>,
    search_results: Vec<NativeUiSearchResult>,
    request_rx: Arc<TokioRwLock<RequestReceiver<(PluginId, NativeUiRequestData), NativeUiResponseData>>>,
    search_field_id: text_input::Id,
    view_data: Option<ViewData>,
    prompt: Option<String>,
    waiting_for_next_unfocus: bool
}

struct ViewData {
    top_level_view: bool,
    plugin_id: PluginId,
    entrypoint_id: EntrypointId,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    OpenView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    RunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    PromptChanged(String),
    SetSearchResults(Vec<NativeUiSearchResult>),
    SetTopLevelView(bool),
    IcedEvent(Event),
    WidgetEvent {
        plugin_id: PluginId,
        widget_event: ComponentWidgetEvent,
    },
    Noop,
    FontLoaded(Result<(), font::Error>),
}

const WINDOW_WIDTH: u32 = 650;
const WINDOW_HEIGHT: u32 = 400;
const SUB_VIEW_WINDOW_WIDTH: u32 = 850;
const SUB_VIEW_WINDOW_HEIGHT: u32 = 500;

pub fn run() {
    AppModel::run(Settings {
        id: None,
        window: iced::window::Settings {
            size: (WINDOW_WIDTH, WINDOW_HEIGHT),
            position: Position::Centered,
            resizable: false,
            decorations: false,
            transparent: true,
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
}

impl Application for AppModel {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = GauntletTheme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (context_tx, request_rx) = channel::<(PluginId, NativeUiRequestData), NativeUiResponseData>();

        let client_context = Arc::new(StdRwLock::new(
            ClientContext { containers: Default::default() }
        ));

        let (dbus_connection, dbus_server, dbus_client) = futures::executor::block_on(async {
            let path = "/dev/projectgauntlet/Client";

            let dbus_connection = zbus::ConnectionBuilder::session()?
                .name("dev.projectgauntlet.Gauntlet.Client")?
                .serve_at(path, DbusClient { context_tx })?
                .build()
                .await?;

            let dbus_server = DbusServerProxyProxy::new(&dbus_connection).await?;

            let dbus_client = dbus_connection
                .object_server()
                .interface::<_, DbusClient>(path)
                .await?;

            Ok::<(Connection, DbusServerProxyProxy<'_>, InterfaceRef<DbusClient>), anyhow::Error>((dbus_connection, dbus_server, dbus_client))
        }).unwrap();

        let search_field_id = text_input::Id::unique();

        (
            AppModel {
                client_context: client_context.clone(),
                dbus_connection,
                dbus_server,
                dbus_client,
                request_rx: Arc::new(TokioRwLock::new(request_rx)),
                search_results: vec![],
                search_field_id: search_field_id.clone(),
                prompt: None,
                view_data: None,
                waiting_for_next_unfocus: false,
            },
            Command::batch([
                Command::perform(async {}, |_| AppMsg::PromptChanged("".to_owned())),
                font::load(icons::ICON_FONT_BYTES).map(AppMsg::FontLoaded),
                focus(search_field_id)
            ])
        )
    }

    fn title(&self) -> String {
        "Gauntlet".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::OpenView { plugin_id, entrypoint_id } => {
                self.view_data.replace(ViewData {
                    top_level_view:  true,
                    plugin_id: plugin_id.clone(),
                    entrypoint_id: entrypoint_id.clone(),
                });

                let mut client_context = self.client_context.write().expect("lock is poisoned");
                client_context.create_view_container(plugin_id.clone());

                Command::batch([
                    // TODO re-center the window
                    iced::window::resize(Size::new(SUB_VIEW_WINDOW_WIDTH, SUB_VIEW_WINDOW_HEIGHT)),
                    self.open_view(plugin_id, entrypoint_id)
                ])
            }
            AppMsg::RunCommand { plugin_id,  entrypoint_id } => {
                Command::batch([
                    self.run_command(plugin_id, entrypoint_id),
                    iced::window::close(),
                ])
            }
            AppMsg::PromptChanged(new_prompt) => {
                self.prompt.replace(new_prompt.clone());

                let dbus_server = self.dbus_server.clone();

                Command::perform(async move {
                    let search_result = dbus_server.search(&new_prompt)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|search_result| NativeUiSearchResult {
                            plugin_id: PluginId::from_string(search_result.plugin_id),
                            plugin_name: search_result.plugin_name,
                            entrypoint_id: EntrypointId::new(search_result.entrypoint_id),
                            entrypoint_name: search_result.entrypoint_name,
                            entrypoint_type: match search_result.entrypoint_type {
                                DBusEntrypointType::Command => SearchResultEntrypointType::Command,
                                DBusEntrypointType::View => SearchResultEntrypointType::View,
                            },
                        })
                        .collect();

                    search_result
                }, AppMsg::SetSearchResults)
            }
            AppMsg::SetSearchResults(search_results) => {
                self.search_results = search_results;
                Command::none()
            }
            AppMsg::SetTopLevelView(top_level_view) => {
                match &mut self.view_data {
                    None => Command::none(),
                    Some(view_data) => {
                        view_data.top_level_view = top_level_view;
                        Command::none()
                    }
                }
            }
            AppMsg::IcedEvent(Event::Keyboard(event)) => {
                match event {
                    keyboard::Event::KeyPressed { key_code, .. } => {
                        match key_code {
                            KeyCode::Up => iced::widget::focus_previous(),
                            KeyCode::Down => iced::widget::focus_next(),
                            KeyCode::Escape => self.previous_view(),
                            _ => Command::none()
                        }
                    }
                    _ => Command::none()
                }
            }
            AppMsg::IcedEvent(Event::Window(iced::window::Event::Unfocused)) => {
                // for some reason Unfocused fires right at the application start
                // and second time on actual window unfocus
                if self.waiting_for_next_unfocus {
                    iced::window::close()
                } else {
                    self.waiting_for_next_unfocus = true;
                    Command::none()
                }
            }
            AppMsg::IcedEvent(_) => Command::none(),
            AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::PreviousView, .. } => self.previous_view(),
            AppMsg::WidgetEvent { widget_event, plugin_id } => {
                let dbus_client = self.dbus_client.clone();
                let client_context = self.client_context.clone();

                Command::perform(async move {
                    let signal_context = dbus_client.signal_context();
                    let future = {
                        let client_context = client_context.read().expect("lock is poisoned");
                        client_context.handle_event(signal_context, &plugin_id, widget_event)
                    };

                    future.await;
                }, |_| AppMsg::Noop)
            }
            AppMsg::Noop => Command::none(),
            AppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let client_context = self.client_context.clone();

        match &self.view_data {
            None => {
                let input: Element<_> = text_input("Search...", self.prompt.as_ref().unwrap_or(&"".to_owned()))
                    .on_input(AppMsg::PromptChanged)
                    .id(self.search_field_id.clone())
                    .width(Length::Fill)
                    .into();

                let search_results = self.search_results.iter().cloned().collect();

                let search_list = search_list(
                    search_results,
                    |event| AppMsg::OpenView {
                        plugin_id: event.plugin_id,
                        entrypoint_id: event.entrypoint_id,
                    },
                    |event| AppMsg::RunCommand {
                        plugin_id: event.plugin_id,
                        entrypoint_id: event.entrypoint_id,
                    },
                );

                let list: Element<_> = scrollable(search_list)
                    .width(Length::Fill)
                    .into();

                let column: Element<_> = column(vec![
                    container(input)
                        .width(Length::Fill)
                        .padding(Padding::new(10.0))
                        .into(),
                    horizontal_rule(1)
                        .into(),
                    container(list)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(Padding::new(5.0))
                        .into(),
                ])
                    .into();

                let element: Element<_> = container(column)
                    .style(ContainerStyle::Background)
                    .height(Length::Fixed(WINDOW_HEIGHT as f32))
                    .width(Length::Fixed(WINDOW_WIDTH as f32))
                    .into();

                // element.explain(iced::color!(0xFF0000))
                element
            }
            Some(ViewData{ plugin_id, entrypoint_id: _, top_level_view: _ }) => {
                let container_element: Element<ComponentWidgetEvent> = plugin_container(client_context, plugin_id.clone())
                    .into();

                let container_element = container_element.map(|widget_event| AppMsg::WidgetEvent {
                    plugin_id: plugin_id.to_owned(),
                    widget_event,
                });

                let element: Element<_> = container(container_element)
                    .style(ContainerStyle::Background)
                    .height(Length::Fixed(SUB_VIEW_WINDOW_HEIGHT as f32))
                    .width(Length::Fixed(SUB_VIEW_WINDOW_WIDTH as f32))
                    .into();

                // element.explain(iced::color!(0xFF0000))
                element
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        GauntletTheme::new()
    }

    fn subscription(&self) -> Subscription<AppMsg> {
        let client_context = self.client_context.clone();
        let request_rx = self.request_rx.clone();

        struct RequestLoop;

        Subscription::batch([
            subscription::events().map(AppMsg::IcedEvent),
            subscription::channel(
                std::any::TypeId::of::<RequestLoop>(),
                100,
                |sender| async move {
                    request_loop(client_context, request_rx, sender).await;

                    panic!("request_rx was unexpectedly closed")
                },
            )
        ])
    }
}

impl AppModel {
    fn previous_view(&mut self) -> Command<AppMsg> {
        match &self.view_data {
            None => {
                iced::window::close()
            }
            Some(ViewData { top_level_view: true, .. }) => {
                self.view_data.take();

                // TODO re-center the window
                iced::window::resize(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            }
            Some(ViewData { top_level_view: false, plugin_id, entrypoint_id }) => {
                self.open_view(plugin_id.clone(), entrypoint_id.clone())
                // TODO re-center the window
            }
        }
    }

    fn open_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let dbus_client = self.dbus_client.clone();

        Command::perform(async move {
            let event_react_view = DbusEventRenderView {
                frontend: "default".to_owned(),
                entrypoint_id: entrypoint_id.to_string(),
            };

            let signal_context = dbus_client.signal_context();

            DbusClient::render_view_signal(signal_context, &plugin_id.to_string(), event_react_view)
                .await
                .unwrap();
        }, |_| AppMsg::Noop)
    }

    fn run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let dbus_client = self.dbus_client.clone();

        Command::perform(async move {
            let event_run_command = DbusEventRunCommand {
                entrypoint_id: entrypoint_id.to_string(),
            };

            let signal_context = dbus_client.signal_context();

            DbusClient::run_command_signal(signal_context, &plugin_id.to_string(), event_run_command)
                .await
                .unwrap();
        }, |_| AppMsg::Noop)
    }
}


async fn request_loop(
    client_context: Arc<StdRwLock<ClientContext>>,
    request_rx: Arc<TokioRwLock<RequestReceiver<(PluginId, NativeUiRequestData), NativeUiResponseData>>>,
    mut sender: Sender<AppMsg>,
) {
    let mut request_rx = request_rx.write().await;
    loop {
        let ((plugin_id, request_data), responder) = request_rx.recv().await;

        let mut app_msg = AppMsg::Noop; // refresh ui

        {
            let mut client_context = client_context.write().expect("lock is poisoned");

            match request_data {
                NativeUiRequestData::ReplaceContainerChildren { top_level_view, container } => {
                    client_context.replace_container_children(&plugin_id, container);

                    app_msg = AppMsg::SetTopLevelView(top_level_view);

                    responder.respond(NativeUiResponseData::ReplaceContainerChildren)
                }
            }
        }

        let _ = sender.send(app_msg).await;
    }
}