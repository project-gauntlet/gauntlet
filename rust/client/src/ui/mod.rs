use std::sync::{Arc, RwLock as StdRwLock};

use iced::{Command, Event, event, executor, font, futures, keyboard, Length, Padding, Settings, Size, Subscription, subscription, window};
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::multi_window::Application;
use iced::widget::{column, container, horizontal_rule, scrollable, text_input};
use iced::widget::text_input::focus;
use iced::window::Position;
use iced_aw::graphics::icons;
use tokio::sync::RwLock as TokioRwLock;
use tonic::Request;
use tonic::transport::Server;

use client_context::ClientContext;
use common::model::{EntrypointId, PluginId, PropertyValue, RenderLocation};
use common::rpc::{BackendClient, RpcEntrypointType, RpcEventRenderView, RpcEventRunCommand, RpcEventViewEvent, RpcRequestRunCommandRequest, RpcRequestViewRenderRequest, RpcSearchRequest, RpcSendViewEventRequest, RpcUiPropertyValue, RpcUiWidgetId};
use common::rpc::rpc_backend_client::RpcBackendClient;
use common::rpc::rpc_frontend_server::RpcFrontendServer;
use common::rpc::rpc_ui_property_value::Value;
use utils::channel::{channel, RequestReceiver};

use crate::model::{NativeUiRequestData, NativeUiResponseData, NativeUiSearchResult, SearchResultEntrypointType};
use crate::rpc::RpcFrontendServerImpl;
use crate::ui::inline_view_container::inline_view_container;
use crate::ui::search_list::search_list;
use crate::ui::theme::{ContainerStyle, Element, GauntletTheme};
use crate::ui::view_container::view_container;
use crate::ui::widget::ComponentWidgetEvent;

mod view_container;
mod search_list;
mod widget;
mod theme;
mod client_context;
mod widget_container;
mod inline_view_container;

pub struct AppModel {
    client_context: Arc<StdRwLock<ClientContext>>,
    backend_client: BackendClient,
    search_results: Vec<NativeUiSearchResult>,
    request_rx: Arc<TokioRwLock<RequestReceiver<NativeUiRequestData, NativeUiResponseData>>>,
    search_field_id: text_input::Id,
    view_data: Option<ViewData>,
    prompt: Option<String>,
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
        render_location: RenderLocation,
        widget_event: ComponentWidgetEvent,
    },
    Noop,
    FontLoaded(Result<(), font::Error>),
    ShowWindow,
    HideWindow,
}

const WINDOW_WIDTH: f32 = 650.0;
const WINDOW_HEIGHT: f32 = 400.0;
const SUB_VIEW_WINDOW_WIDTH: f32 = 850.0;
const SUB_VIEW_WINDOW_HEIGHT: f32 = 500.0;


fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: false,
        transparent: true,
        ..Default::default()
    }
}

pub fn run() {
    AppModel::run(Settings {
        id: None,
        window: window_settings(),
        ..Default::default()
    }).unwrap();
}

impl Application for AppModel {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = GauntletTheme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        register_shortcut();

        let (context_tx, request_rx) = channel::<NativeUiRequestData, NativeUiResponseData>();

        let client_context = Arc::new(StdRwLock::new(ClientContext::new()));

        tokio::spawn(async {
            let addr = "127.0.0.1:42321".parse().unwrap();

            Server::builder()
                .add_service(RpcFrontendServer::new(RpcFrontendServerImpl { context_tx }))
                .serve(addr)
                .await
                .expect("frontend server didn't start");
        });

        let backend_client = futures::executor::block_on(async {
            anyhow::Ok(RpcBackendClient::connect("http://127.0.0.1:42320").await?)
        }).unwrap();

        (
            AppModel {
                client_context: client_context.clone(),
                backend_client,
                request_rx: Arc::new(TokioRwLock::new(request_rx)),
                search_results: vec![],
                search_field_id: text_input::Id::unique(),
                prompt: None,
                view_data: None,
            },
            Command::batch([
                Command::perform(async {}, |_| AppMsg::ShowWindow),
                font::load(icons::BOOTSTRAP_FONT_BYTES).map(AppMsg::FontLoaded)
            ]),
        )
    }

    fn title(&self, _: window::Id) -> String {
        "Gauntlet".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::OpenView { plugin_id, entrypoint_id } => {
                self.view_data.replace(ViewData {
                    top_level_view: true,
                    plugin_id: plugin_id.clone(),
                    entrypoint_id: entrypoint_id.clone(),
                });

                Command::batch([
                    // TODO re-center the window
                    iced::window::resize(window::Id::MAIN, Size::new(SUB_VIEW_WINDOW_WIDTH, SUB_VIEW_WINDOW_HEIGHT)),
                    self.open_view(plugin_id, entrypoint_id)
                ])
            }
            AppMsg::RunCommand { plugin_id, entrypoint_id } => {
                Command::batch([
                    self.hide_window(),
                    self.run_command(plugin_id, entrypoint_id),
                ])
            }
            AppMsg::PromptChanged(new_prompt) => {
                self.prompt.replace(new_prompt.clone());

                let mut backend_client = self.backend_client.clone();

                Command::perform(async move {
                    let request = RpcSearchRequest {
                        text: new_prompt,
                    };

                    let search_result = backend_client.search(Request::new(request))
                        .await
                        .unwrap()
                        .into_inner()
                        .results
                        .into_iter()
                        .flat_map(|search_result| {
                            let entrypoint_type = search_result.entrypoint_type
                                .try_into()
                                .unwrap();

                            let entrypoint_type = match entrypoint_type {
                                RpcEntrypointType::Command => SearchResultEntrypointType::Command,
                                RpcEntrypointType::View => SearchResultEntrypointType::View,
                                RpcEntrypointType::InlineView => {
                                    return None;
                                }
                            };

                            Some(NativeUiSearchResult {
                                plugin_id: PluginId::from_string(search_result.plugin_id),
                                plugin_name: search_result.plugin_name,
                                entrypoint_id: EntrypointId::new(search_result.entrypoint_id),
                                entrypoint_name: search_result.entrypoint_name,
                                entrypoint_type,
                            })
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
                    keyboard::Event::KeyPressed { key, .. } => {
                        match key {
                            Key::Named(Named::ArrowUp) => iced::widget::focus_previous(),
                            Key::Named(Named::ArrowDown) => iced::widget::focus_next(),
                            Key::Named(Named::Escape) => self.previous_view(),
                            _ => Command::none()
                        }
                    }
                    _ => Command::none()
                }
            }
            AppMsg::IcedEvent(Event::Window(_, iced::window::Event::Unfocused)) => {
                self.hide_window()
            }
            AppMsg::IcedEvent(_) => Command::none(),
            AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::PreviousView, .. } => self.previous_view(),
            AppMsg::WidgetEvent { widget_event, plugin_id, render_location } => {
                let mut backend_client = self.backend_client.clone();
                let client_context = self.client_context.clone();

                Command::perform(async move {
                    let event = {
                        let client_context = client_context.read().expect("lock is poisoned");
                        client_context.handle_event(render_location, &plugin_id, widget_event)
                    };

                    if let Some(event) = event {
                        let widget_id = RpcUiWidgetId { value: event.widget_id };
                        let event_arguments = event.event_arguments
                            .into_iter()
                            .map(|value| match value {
                                PropertyValue::String(value) => RpcUiPropertyValue { value: Some(Value::String(value)) },
                                PropertyValue::Number(value) => RpcUiPropertyValue { value: Some(Value::Number(value)) },
                                PropertyValue::Bool(value) => RpcUiPropertyValue { value: Some(Value::Bool(value)) },
                                PropertyValue::Undefined => RpcUiPropertyValue { value: Some(Value::Undefined(0)) },
                            })
                            .collect();

                        let event = RpcEventViewEvent {
                            widget_id: Some(widget_id),
                            event_name: event.event_name,
                            event_arguments,
                        };

                        let request = RpcSendViewEventRequest {
                            plugin_id: plugin_id.to_string(),
                            event: Some(event),
                        };

                        backend_client.send_view_event(Request::new(request))
                            .await
                            .unwrap();
                    };
                }, |_| AppMsg::Noop)
            }
            AppMsg::Noop => Command::none(),
            AppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
            AppMsg::ShowWindow => self.show_window(),
            AppMsg::HideWindow => self.hide_window(),
        }
    }

    fn view(&self, _: window::Id) -> Element<'_, Self::Message> {
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

                let list = container(list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::new(5.0))
                    .into();

                let input = container(input)
                    .width(Length::Fill)
                    .padding(Padding::new(10.0))
                    .into();

                let separator = horizontal_rule(1)
                    .into();

                let inline_view_visible = {
                    let client_context = self.client_context.read().expect("lock is poisoned");
                    !client_context.get_all_inline_view_containers().is_empty()
                };

                let column: Element<_> = if inline_view_visible {
                    column(vec![
                        input,
                        separator,
                        inline_view_container(self.client_context.clone()).into(),
                        horizontal_rule(1).into(),
                        list,
                    ]).into()
                } else {
                    column(vec![
                        input,
                        separator,
                        list,
                    ]).into()
                };

                let element: Element<_> = container(column)
                    .style(ContainerStyle::Background)
                    .height(Length::Fixed(WINDOW_HEIGHT as f32))
                    .width(Length::Fixed(WINDOW_WIDTH as f32))
                    .into();

                // element.explain(iced::color!(0xFF0000))
                element
            }
            Some(ViewData { plugin_id, entrypoint_id: _, top_level_view: _ }) => {
                let container_element: Element<_> = view_container(self.client_context.clone(), plugin_id.to_owned())
                    .into();

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

    fn theme(&self, _: window::Id) -> Self::Theme {
        GauntletTheme::new()
    }

    fn subscription(&self) -> Subscription<AppMsg> {
        let client_context = self.client_context.clone();
        let request_rx = self.request_rx.clone();

        struct RequestLoop;
        struct GlobalShortcutListener;

        Subscription::batch([
            subscription::channel(
                std::any::TypeId::of::<GlobalShortcutListener>(),
                10,
                |sender| async move {
                    listen_on_shortcut(sender);

                    std::future::pending::<()>().await;

                    unreachable!()
                },
            ),
            event::listen().map(AppMsg::IcedEvent),
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
    fn hide_window(&mut self) -> Command<AppMsg> {
        self.prompt = None;
        self.view_data = None;
        self.search_results = vec![];

        window::change_mode(window::Id::MAIN, window::Mode::Hidden)
    }

    fn show_window(&mut self) -> Command<AppMsg> {
        Command::batch([
            window::change_mode(window::Id::MAIN, window::Mode::Windowed),
            Command::perform(async {}, |_| AppMsg::PromptChanged("".to_owned())),
            focus(self.search_field_id.clone())
        ])
    }

    fn previous_view(&mut self) -> Command<AppMsg> {
        match &self.view_data {
            None => {
                self.hide_window()
            }
            Some(ViewData { top_level_view: true, .. }) => {
                self.view_data.take();

                // TODO re-center the window
                iced::window::resize(window::Id::MAIN, Size::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            }
            Some(ViewData { top_level_view: false, plugin_id, entrypoint_id }) => {
                self.open_view(plugin_id.clone(), entrypoint_id.clone())
                // TODO re-center the window
            }
        }
    }

    fn open_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_client.clone();

        Command::perform(async move {
            let event = RpcEventRenderView {
                frontend: "default".to_owned(),
                entrypoint_id: entrypoint_id.to_string(),
            };

            let request = RpcRequestViewRenderRequest {
                plugin_id: plugin_id.to_string(),
                event: Some(event),
            };

            backend_client.request_view_render(Request::new(request))
                .await
                .unwrap();
        }, |_| AppMsg::Noop)
    }

    fn run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_client.clone();

        Command::perform(async move {
            let event = RpcEventRunCommand {
                entrypoint_id: entrypoint_id.to_string(),
            };

            let request = RpcRequestRunCommandRequest {
                plugin_id: plugin_id.to_string(),
                event: Some(event),
            };

            backend_client.request_run_command(Request::new(request))
                .await
                .unwrap();
        }, |_| AppMsg::Noop)
    }
}

fn register_shortcut() {
    use global_hotkey::{GlobalHotKeyManager, hotkey::{Code, HotKey, Modifiers}};

    let manager = GlobalHotKeyManager::new().unwrap();

    manager.register(HotKey::new(Some(Modifiers::ALT | Modifiers::CONTROL), Code::Space))
        .unwrap();
}

fn listen_on_shortcut(mut sender: Sender<AppMsg>) {
    use global_hotkey::GlobalHotKeyEvent;

    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();

        while let Ok(_) = receiver.recv() {
            let _ = sender.send(AppMsg::ShowWindow);
        }
    });
}


async fn request_loop(
    client_context: Arc<StdRwLock<ClientContext>>,
    request_rx: Arc<TokioRwLock<RequestReceiver<NativeUiRequestData, NativeUiResponseData>>>,
    mut sender: Sender<AppMsg>,
) {
    let mut request_rx = request_rx.write().await;
    loop {
        let (request_data, responder) = request_rx.recv().await;

        let mut app_msg = AppMsg::Noop; // refresh ui

        {
            let mut client_context = client_context.write().expect("lock is poisoned");

            match request_data {
                NativeUiRequestData::ReplaceView { plugin_id, render_location, top_level_view, container } => {
                    client_context.replace_view(render_location, container, &plugin_id);

                    app_msg = AppMsg::SetTopLevelView(top_level_view);

                    responder.respond(NativeUiResponseData::Nothing)
                }
                NativeUiRequestData::ClearInlineView { plugin_id } => {
                    client_context.clear_inline_view(&plugin_id);

                    responder.respond(NativeUiResponseData::Nothing)
                }
                NativeUiRequestData::ShowWindow => {
                    app_msg = AppMsg::ShowWindow;

                    responder.respond(NativeUiResponseData::Nothing)
                }
            }
        }

        let _ = sender.send(app_msg).await;
    }
}
