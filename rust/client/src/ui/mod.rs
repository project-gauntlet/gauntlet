use std::sync::{Arc, RwLock as StdRwLock};

use iced::{Application, Command, Element, Event, executor, futures, keyboard, Length, Padding, Renderer, Subscription, subscription};
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::KeyCode;
use iced::Settings;
use iced::widget::{column, container, horizontal_rule, scrollable, text_input};
use iced::window::Position;
use tokio::sync::RwLock as TokioRwLock;
use zbus::{Connection, InterfaceRef};

use common::dbus::DbusEventViewCreated;
use common::model::{EntrypointId, PluginId};
use utils::channel::{channel, RequestReceiver};

use crate::dbus::{DbusClient, DbusServerProxyProxy};
use crate::model::{NativeUiRequestData, NativeUiResponseData, NativeUiSearchResult};
use crate::ui::plugin_container::{ClientContext, plugin_container};
use crate::ui::search_list::search_list;
use crate::ui::widget::ComponentWidgetEvent;

mod plugin_container;
mod search_list;
mod widget;

pub struct AppModel {
    client_context: Arc<StdRwLock<ClientContext>>,
    dbus_connection: Connection,
    dbus_server: DbusServerProxyProxy<'static>,
    dbus_client: InterfaceRef<DbusClient>,
    state: Vec<NavState>,
    search_results: Vec<NativeUiSearchResult>,
    request_rx: Arc<TokioRwLock<RequestReceiver<(PluginId, NativeUiRequestData), NativeUiResponseData>>>,
}

enum NavState {
    SearchView {
        prompt: Option<String>,
    },
    PluginView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    OpenView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    PromptChanged(String),
    SetSearchResults(Vec<NativeUiSearchResult>),
    IcedEvent(Event),
    WidgetEvent {
        plugin_id: PluginId,
        widget_event: ComponentWidgetEvent,
    },
    Noop,
}

pub fn run() {
    AppModel::run(Settings {
        id: None,
        window: iced::window::Settings {
            size: (650, 400),
            position: Position::Centered,
            resizable: false,
            decorations: false,
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
}


impl Application for AppModel {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = iced::Theme;
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

        (
            AppModel {
                client_context: client_context.clone(),
                dbus_connection,
                dbus_server,
                dbus_client,
                request_rx: Arc::new(TokioRwLock::new(request_rx)),
                state: vec![NavState::SearchView { prompt: None }],
                search_results: vec![],
            },
            Command::perform(async {}, |_| AppMsg::PromptChanged("".to_owned())),
        )
    }

    fn title(&self) -> String {
        "Gauntlet".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::OpenView { plugin_id, entrypoint_id } => {
                self.state.push(NavState::PluginView {
                    plugin_id: plugin_id.clone(),
                    entrypoint_id: entrypoint_id.clone(),
                });

                let mut client_context = self.client_context.write().expect("lock is poisoned");
                client_context.create_view_container(plugin_id.clone());

                let dbus_client = self.dbus_client.clone();

                Command::perform(async move {
                    let event_view_created = DbusEventViewCreated {
                        reconciler_mode: "persistent".to_owned(),
                        view_name: entrypoint_id.to_string(), // TODO what was view_name supposed to be?
                    };

                    let signal_context = dbus_client.signal_context();

                    DbusClient::view_created_signal(signal_context, &plugin_id.to_string(), event_view_created)
                        .await
                        .unwrap();
                }, |_| AppMsg::Noop)
            }
            AppMsg::PromptChanged(new_prompt) => {
                match self.state.last_mut().expect("state is supposed to always have at least one item") {
                    NavState::SearchView { prompt } => {
                        prompt.replace(new_prompt.clone());

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
                                })
                                .collect();

                            search_result
                        }, AppMsg::SetSearchResults)
                    }
                    _ => {
                        Command::none()
                    }
                }
            }
            AppMsg::SetSearchResults(search_results) => {
                self.search_results = search_results;
                Command::none()
            }
            AppMsg::IcedEvent(Event::Keyboard(event)) => {
                match event {
                    keyboard::Event::KeyPressed { key_code, .. } => {
                        match key_code {
                            KeyCode::Up => iced::widget::focus_previous(),
                            KeyCode::Down => iced::widget::focus_next(),
                            KeyCode::Escape => {
                                if self.state.len() <= 1 {
                                    iced::window::close()
                                } else {
                                    self.state.pop();
                                    Command::none()
                                }
                            }
                            _ => Command::none()
                        }
                    }
                    _ => Command::none()
                }
            }
            AppMsg::IcedEvent(_) => Command::none(),
            AppMsg::WidgetEvent { widget_event, plugin_id } => {
                let dbus_client = self.dbus_client.clone();
                Command::perform(async move {
                    let signal_context = dbus_client.signal_context();

                    widget_event.handle(signal_context, plugin_id).await
                }, |_| AppMsg::Noop)
            }
            AppMsg::Noop => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let client_context = self.client_context.clone();

        match &self.state.last().expect("state is supposed to always have at least one item") {
            NavState::SearchView { prompt } => {
                let input: Element<_> = text_input("", prompt.as_ref().unwrap_or(&"".to_owned()))
                    .on_input(AppMsg::PromptChanged)
                    .width(Length::Fill)
                    .into();

                let search_results = self.search_results.iter().cloned().collect();

                let search_list = search_list(search_results, |event| {
                    AppMsg::OpenView {
                        plugin_id: event.plugin_id,
                        entrypoint_id: event.entrypoint_id,
                    }
                });

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

                // column.explain(Color::from_rgb(1f32, 0f32, 0f32))
                column
            }
            NavState::PluginView { plugin_id, entrypoint_id } => {
                let container: Element<ComponentWidgetEvent> = plugin_container(client_context, plugin_id.clone())
                    .into();

                container.map(|widget_event| AppMsg::WidgetEvent {
                    plugin_id: plugin_id.to_owned(),
                    widget_event,
                })
            }
        }
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

async fn request_loop(
    client_context: Arc<StdRwLock<ClientContext>>,
    request_rx: Arc<TokioRwLock<RequestReceiver<(PluginId, NativeUiRequestData), NativeUiResponseData>>>,
    mut sender: Sender<AppMsg>,
) {
    let mut request_rx = request_rx.write().await;
    loop {
        let ((plugin_id, request_data), responder) = request_rx.recv().await;

        {
            let mut client_context = client_context.write().expect("lock is poisoned");

            match request_data {
                NativeUiRequestData::GetRoot => {
                    let container = client_context.get_root(&plugin_id);

                    let response = NativeUiResponseData::GetRoot { container };

                    responder.respond(response)
                }
                NativeUiRequestData::CreateInstance { widget_type, properties } => {
                    let widget = client_context.create_instance(&plugin_id, &widget_type, properties);

                    let response = NativeUiResponseData::CreateInstance { widget };

                    responder.respond(response)
                }
                NativeUiRequestData::CreateTextInstance { text } => {
                    let widget = client_context.create_text_instance(&plugin_id, &text);

                    let response = NativeUiResponseData::CreateTextInstance { widget };

                    responder.respond(response)
                }
                NativeUiRequestData::AppendChild { parent, child } => {
                    let result = client_context.append_child(&plugin_id, parent, child);

                    let response = NativeUiResponseData::AppendChild { result };

                    responder.respond(response)
                }
                NativeUiRequestData::CloneInstance {
                    widget,
                    widget_type,
                    new_props,
                    keep_children
                } => {
                    let widget = client_context.clone_instance(&plugin_id, widget, &widget_type, new_props, keep_children);

                    let response = NativeUiResponseData::CloneInstance { widget };

                    responder.respond(response)
                }
                NativeUiRequestData::ReplaceContainerChildren { container, new_children } => {
                    let result = client_context.replace_container_children(&plugin_id, container, new_children);

                    let response = NativeUiResponseData::ReplaceContainerChildren { result };

                    responder.respond(response)
                }
            }
        }

        let _ = sender.send(AppMsg::Noop).await; // refresh ui
    }
}