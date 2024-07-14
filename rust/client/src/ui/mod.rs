use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock as StdRwLock};

use global_hotkey::{GlobalHotKeyManager, HotKeyState};
use iced::{Alignment, Command, Event, event, executor, font, futures, keyboard, Length, Padding, Pixels, Settings, Size, Subscription, subscription, window};
use iced::advanced::graphics::core::SmolStr;
use iced::Application;
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, scrollable, Space, text, text_input};
use iced::widget::scrollable::{AbsoluteOffset, scroll_to};
use iced::widget::text_input::focus;
use iced::window::{change_level, Level, Position, Screenshot};
use iced_aw::core::icons;
use serde::Deserialize;
use tokio::runtime::Handle;
use tokio::sync::RwLock as TokioRwLock;
use tonic::transport::Server;

use client_context::ClientContext;
use common::model::{EntrypointId, PhysicalShortcut, PluginId, SearchResult, SearchResultEntrypointType, UiRenderLocation};
use common::rpc::backend_api::BackendApi;
use common::rpc::backend_server::wait_for_backend_server;
use common::rpc::frontend_server::start_frontend_server;
use common::scenario_convert::{ui_render_location_from_scenario, ui_widget_from_scenario};
use common::scenario_model::{ScenarioFrontendEvent, ScenarioUiRenderLocation};
use common_ui::physical_key_model;
use utils::channel::{channel, RequestReceiver};

use crate::model::{NativeUiResponseData, UiRequestData, UiViewEvent};
use crate::rpc::FrontendServerImpl;
use crate::ui::inline_view_container::inline_view_container;
use crate::ui::search_list::search_list;
use crate::ui::theme::{Element, GauntletTheme, ThemableWidget};
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::text_input::TextInputStyle;
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
    // logic
    backend_api: BackendApi,
    request_rx: Arc<TokioRwLock<RequestReceiver<UiRequestData, NativeUiResponseData>>>,
    search_field_id: text_input::Id,
    scrollable_id: scrollable::Id,
    waiting_for_next_unfocus: bool,
    _global_hotkey_manager: GlobalHotKeyManager,
    theme: GauntletTheme,

    // ephemeral state
    prompt: String,

    // state
    client_context: Arc<StdRwLock<ClientContext>>,
    plugin_view_data: Option<PluginViewData>,
    error_view: Option<ErrorViewData>,
    search_results: Vec<SearchResult>,
}

struct PluginViewData {
    top_level_view: bool,
    plugin_id: PluginId,
    plugin_name: String,
    entrypoint_id: EntrypointId,
    entrypoint_name: String,
    action_shortcuts: HashMap<String, PhysicalShortcut>,
    waiting_for_first_render: bool,
}

enum ErrorViewData {
    PreferenceRequiredViewData {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    PluginErrorViewData {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    }
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    OpenView {
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
    },
    RunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    RunGeneratedCommandEvent {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    PromptChanged(String),
    PromptSubmit,
    UpdateSearchResults,
    SetSearchResults(Vec<SearchResult>),
    ReplaceView {
        top_level_view: bool,
    },
    IcedEvent(Event),
    WidgetEvent {
        plugin_id: PluginId,
        render_location: UiRenderLocation,
        widget_event: ComponentWidgetEvent,
    },
    Noop,
    FontLoaded(Result<(), font::Error>),
    ShowWindow,
    HideWindow,
    ShowPreferenceRequiredView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    },
    OpenSettingsPreferences {
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
    },
    SaveActionShortcuts {
        action_shortcuts: HashMap<String, PhysicalShortcut>
    },
    ShowPluginErrorView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation
    },
    SelectSearchItem(SearchResult),
    RequestSearchResultUpdate,
    Screenshot {
        save_path: String
    },
    ScreenshotDone {
        save_path: String,
        screenshot: Screenshot
    },
    Close,
    ResetWindowState,
}

const WINDOW_WIDTH: f32 = 750.0;
const WINDOW_HEIGHT: f32 = 450.0;

fn window_settings(minimized: bool) -> iced::window::Settings {
    iced::window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: false,
        transparent: true,
        visible: !minimized,
        ..Default::default()
    }
}

pub fn run(minimized: bool) {
    AppModel::run(Settings {
        id: None,
        window: window_settings(minimized),
        ..Default::default()
    }).unwrap();
}

impl Application for AppModel {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = GauntletTheme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let global_hotkey_manager = register_shortcut();

        let (context_tx, request_rx) = channel::<UiRequestData, NativeUiResponseData>();

        tokio::spawn(async {
            start_frontend_server(Box::new(FrontendServerImpl::new(context_tx))).await;
        });

        let backend_api = futures::executor::block_on(async {
            wait_for_backend_server().await;

            BackendApi::new().await
        }).unwrap();

        let mut commands = vec![
            change_level(window::Id::MAIN, Level::AlwaysOnTop),
            Command::perform(async {}, |_| AppMsg::ResetWindowState),
            font::load(icons::BOOTSTRAP_FONT_BYTES).map(AppMsg::FontLoaded),
        ];

        let (client_context, plugin_view_data, error_view) = if cfg!(feature = "scenario_runner") {
            let gen_in = std::env::var("GAUNTLET_SCREENSHOT_GEN_IN")
                .expect("Unable to read GAUNTLET_SCREENSHOT_GEN_IN");

            let gen_in = fs::read_to_string(gen_in)
                .expect("Unable to read file at GAUNTLET_SCREENSHOT_GEN_IN");

            let gen_out = std::env::var("GAUNTLET_SCREENSHOT_GEN_OUT")
                .expect("Unable to read GAUNTLET_SCREENSHOT_GEN_OUT");

            let gen_name = std::env::var("GAUNTLET_SCREENSHOT_GEN_NAME")
                .expect("Unable to read GAUNTLET_SCREENSHOT_GEN_NAME");

            let event: ScenarioFrontendEvent = serde_json::from_str(&gen_in)
                .expect("GAUNTLET_SCREENSHOT_GEN_IN is not valid json");

            commands.push(
                Command::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    },
                    |_| AppMsg::Screenshot { save_path: gen_out },
                )
            );

            match event {
                ScenarioFrontendEvent::ReplaceView { entrypoint_id, render_location, top_level_view, container } => {
                    let plugin_id = PluginId::from_string("__SCREENSHOT_GEN___");
                    let entrypoint_id = EntrypointId::from_string(entrypoint_id);

                    let mut context = ClientContext::new();
                    context.replace_view(
                        ui_render_location_from_scenario(render_location),
                        ui_widget_from_scenario(container),
                        &plugin_id,
                        &entrypoint_id
                    );

                    commands.push(Command::perform(async move { top_level_view }, |top_level_view| AppMsg::ReplaceView { top_level_view }));

                    let plugin_view_data= match render_location {
                        ScenarioUiRenderLocation::InlineView => None,
                        ScenarioUiRenderLocation::View => Some(PluginViewData {
                            top_level_view,
                            plugin_id,
                            plugin_name: "Screenshot Gen".to_string(),
                            entrypoint_id,
                            entrypoint_name: gen_name,
                            action_shortcuts: Default::default(),
                            waiting_for_first_render: false,
                        })
                    };

                    (context, plugin_view_data, None)
                }
                ScenarioFrontendEvent::ShowPreferenceRequiredView { entrypoint_id, plugin_preferences_required, entrypoint_preferences_required } => {
                    let error_view = Some(ErrorViewData::PreferenceRequiredViewData {
                        plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                        entrypoint_id: EntrypointId::from_string(entrypoint_id),
                        plugin_preferences_required,
                        entrypoint_preferences_required,
                    });

                    (ClientContext::new(), None, error_view)
                }
                ScenarioFrontendEvent::ShowPluginErrorView { entrypoint_id, render_location: _ } => {
                    let error_view = Some(ErrorViewData::PluginErrorViewData {
                        plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                        entrypoint_id: EntrypointId::from_string(entrypoint_id),
                    });

                    (ClientContext::new(), None, error_view)
                }
            }
        } else {
            (ClientContext::new(), None, None)
        };

        (
            AppModel {
                // logic
                backend_api,
                request_rx: Arc::new(TokioRwLock::new(request_rx)),
                _global_hotkey_manager: global_hotkey_manager,
                search_field_id: text_input::Id::unique(),
                scrollable_id: scrollable::Id::unique(),
                waiting_for_next_unfocus: false,
                theme: GauntletTheme::new(),

                // ephemeral state
                prompt: "".to_string(),

                // state
                client_context: Arc::new(StdRwLock::new(client_context)),
                plugin_view_data,
                error_view,
                search_results: vec![],
            },
            Command::batch(commands),
        )
    }

    fn title(&self) -> String {
        "Gauntlet".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::OpenView { plugin_id, plugin_name, entrypoint_id, entrypoint_name } => {
                self.plugin_view_data.replace(PluginViewData {
                    top_level_view: true,
                    plugin_id: plugin_id.clone(),
                    plugin_name,
                    entrypoint_id: entrypoint_id.clone(),
                    entrypoint_name,
                    action_shortcuts: HashMap::new(),
                    waiting_for_first_render: true,
                });

                self.open_view(plugin_id, entrypoint_id)
            }
            AppMsg::RunCommand { plugin_id, entrypoint_id } => {
                Command::batch([
                    self.hide_window(),
                    self.run_command(plugin_id, entrypoint_id),
                ])
            }
            AppMsg::RunGeneratedCommandEvent { plugin_id, entrypoint_id } => {
                Command::batch([
                    self.hide_window(),
                    self.run_generated_command(plugin_id, entrypoint_id),
                ])
            }
            AppMsg::PromptChanged(mut new_prompt) => {
                if cfg!(feature = "scenario_runner") {
                    Command::none()
                } else {
                    new_prompt.truncate(100); // search query uses regex so just to be safe truncate the prompt

                    self.prompt = new_prompt.clone();

                    let mut backend_api = self.backend_api.clone();

                    Command::perform(async move {
                        backend_api.search(new_prompt, true)
                            .await
                            .unwrap() // TODO proper error handling
                    }, AppMsg::SetSearchResults)
                }
            }
            AppMsg::UpdateSearchResults => {
                let prompt = self.prompt.clone();

                let mut backend_api = self.backend_api.clone();

                Command::perform(async move {
                    backend_api.search(prompt, false)
                        .await
                        .unwrap() // TODO proper error handling
                }, AppMsg::SetSearchResults)
            }
            AppMsg::PromptSubmit => {
                if let Some(search_item) = self.search_results.first() {
                    let search_item = search_item.clone();
                    Command::perform(async {}, |_| AppMsg::SelectSearchItem(search_item))
                } else {
                    Command::none()
                }
            }
            AppMsg::SetSearchResults(search_results) => {
                self.search_results = search_results;
                Command::none()
            }
            AppMsg::ReplaceView { top_level_view } => {
                match &mut self.plugin_view_data {
                    None => Command::none(),
                    Some(view_data) => {
                        view_data.top_level_view = top_level_view;
                        view_data.waiting_for_first_render = false;

                        Command::none()
                    }
                }
            }
            AppMsg::IcedEvent(Event::Keyboard(event)) => {
                let mut backend_client = self.backend_api.clone();

                match event {
                    keyboard::Event::KeyPressed { key, modifiers, physical_key, text, .. } => {
                        match key {
                            Key::Named(Named::ArrowUp) => iced::widget::focus_previous(),
                            Key::Named(Named::ArrowDown) => iced::widget::focus_next(),
                            Key::Named(Named::Escape) => self.previous_view(),
                            Key::Named(Named::Backspace) => {
                                self.backspace_prompt();
                                focus(self.search_field_id.clone())
                            },
                            _ => {
                                if self.plugin_view_data.is_none() {
                                    match text {
                                        Some(text) => {
                                            self.append_prompt(text.to_string());
                                            focus(self.search_field_id.clone())
                                        }
                                        None => {
                                            Command::none()
                                        }
                                    }
                                } else {
                                    let (plugin_id, entrypoint_id) = {
                                        let client_context = self.client_context.read().expect("lock is poisoned");
                                        (client_context.get_view_plugin_id(), client_context.get_view_entrypoint_id())
                                    };

                                    match physical_key_model(physical_key) {
                                        Some(name) => {
                                            tracing::debug!("physical key pressed: {:?}. shift: {:?} control: {:?} alt: {:?} meta: {:?}", name, modifiers.shift(), modifiers.control(), modifiers.alt(), modifiers.logo());

                                            Command::perform(
                                                async move {
                                                    let modifier_shift = modifiers.shift();
                                                    let modifier_control = modifiers.control();
                                                    let modifier_alt = modifiers.alt();
                                                    let modifier_meta = modifiers.logo();

                                                    backend_client.send_keyboard_event(plugin_id, entrypoint_id, name, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                                                        .await
                                                        .unwrap(); // TODO proper error handling
                                                },
                                                |_| AppMsg::Noop,
                                            )
                                        }
                                        None => {
                                            Command::none()
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => Command::none()
                }
            }
            AppMsg::IcedEvent(Event::Window(_, iced::window::Event::Unfocused)) => {
                // for some reason (on both macos and linux) Unfocused fires right at the application start
                // and second time on actual window unfocus
                if self.waiting_for_next_unfocus {
                    if cfg!(target_os = "linux") { // gnome requires double global shortcut press (probably gnome bug, because works on kde).
                        self.waiting_for_next_unfocus = false;
                    }
                    self.hide_window()
                } else {
                    self.waiting_for_next_unfocus = true;
                    Command::none()
                }
            }
            AppMsg::IcedEvent(_) => Command::none(),
            AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::PreviousView, .. } => self.previous_view(),
            AppMsg::WidgetEvent { widget_event, plugin_id, render_location } => {
                let mut backend_client = self.backend_api.clone();
                let client_context = self.client_context.clone();

                Command::perform(async move {
                    let event = {
                        let client_context = client_context.read().expect("lock is poisoned");
                        client_context.handle_event(render_location, &plugin_id, widget_event)
                    };

                    if let Some(event) = event {
                        match event {
                            UiViewEvent::View { widget_id, event_name, event_arguments } => {
                                backend_client.send_view_event(plugin_id, widget_id, event_name, event_arguments)
                                    .await
                                    .unwrap(); // TODO proper error handling
                            }
                            UiViewEvent::Open { href } => {
                                backend_client.send_open_event(plugin_id, href)
                                    .await
                                    .unwrap(); // TODO proper error handling
                            }
                        }
                    };
                }, |_| AppMsg::Noop)
            }
            AppMsg::Noop => Command::none(),
            AppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
            AppMsg::ResetWindowState => self.reset_window_state(),
            AppMsg::ShowWindow => self.show_window(),
            AppMsg::HideWindow => self.hide_window(),
            AppMsg::ShowPreferenceRequiredView {
                plugin_id,
                entrypoint_id,
                plugin_preferences_required,
                entrypoint_preferences_required
            } => {
                self.error_view = Some(ErrorViewData::PreferenceRequiredViewData {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required,
                });
                Command::none()
            }
            AppMsg::ShowPluginErrorView { plugin_id, entrypoint_id, render_location } => {
                self.error_view = Some(ErrorViewData::PluginErrorViewData {
                    plugin_id,
                    entrypoint_id,
                });
                Command::none()
            }
            AppMsg::OpenSettingsPreferences { plugin_id, entrypoint_id, } => {
                let mut backend_api = self.backend_api.clone();

                Command::perform(async move {
                    backend_api.open_settings_window_preferences(plugin_id, entrypoint_id)
                        .await
                        .unwrap(); // TODO proper error handling
                }, |_| AppMsg::Noop)
            }
            AppMsg::SaveActionShortcuts { action_shortcuts } => {
                if let Some(data) = self.plugin_view_data.as_mut() {
                    data.action_shortcuts = action_shortcuts;
                }
                Command::none()
            }
            AppMsg::SelectSearchItem(search_result) => {
                let event = match search_result.entrypoint_type {
                    SearchResultEntrypointType::Command => AppMsg::RunCommand {
                        entrypoint_id: search_result.entrypoint_id.clone(),
                        plugin_id: search_result.plugin_id.clone()
                    },
                    SearchResultEntrypointType::View => AppMsg::OpenView {
                        plugin_id: search_result.plugin_id.clone(),
                        plugin_name: search_result.plugin_name.clone(),
                        entrypoint_id: search_result.entrypoint_id.clone(),
                        entrypoint_name: search_result.entrypoint_name.clone(),
                    },
                    SearchResultEntrypointType::GeneratedCommand => AppMsg::RunGeneratedCommandEvent {
                        entrypoint_id: search_result.entrypoint_id.clone(),
                        plugin_id: search_result.plugin_id.clone()
                    },
                };

                Command::perform(async {}, |_| event)
            }
            AppMsg::RequestSearchResultUpdate => {
                Command::perform(async {}, move |_| AppMsg::UpdateSearchResults)
            }
            AppMsg::Screenshot { save_path } => {
                println!("Creating screenshot at: {}", save_path);

                fs::create_dir_all(Path::new(&save_path).parent().expect("no parent?"))
                    .expect("unable to create scenario out directories");

                window::screenshot(
                    window::Id::MAIN,
                    |screenshot| AppMsg::ScreenshotDone {
                        save_path,
                        screenshot,
                    }
                )
            }
            AppMsg::ScreenshotDone { save_path, screenshot } => {
                println!("Saving screenshot at: {}", save_path);

                Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let save_dir = Path::new(&save_path);

                            let save_parent_dir = save_dir
                                .parent()
                                .expect("save_path has no parent");

                            fs::create_dir_all(save_parent_dir)
                                .expect("unable to create save_parent_dir");

                            image::save_buffer_with_format(
                                &save_path,
                                &screenshot.bytes,
                                screenshot.size.width,
                                screenshot.size.height,
                                image::ColorType::Rgba8,
                                image::ImageFormat::Png
                            )
                        }).await
                            .expect("Unable to save screenshot")
                    },
                    |_| AppMsg::Close,
                )
            }
            AppMsg::Close => window::close(window::Id::MAIN)
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if let Some(view_data) = &self.error_view {
            return match view_data {
                ErrorViewData::PreferenceRequiredViewData { plugin_id, entrypoint_id, plugin_preferences_required, entrypoint_preferences_required } => {

                    let (description_text, msg) = match (plugin_preferences_required, entrypoint_preferences_required) {
                        (true, true) => {
                            // TODO do not show "entrypoint" name to user
                            let description_text = "Before using, plugin and entrypoint preferences need to be specified";
                            // note:
                            // we open plugin view and not entrypoint even though both need to be specified
                            let msg = AppMsg::OpenSettingsPreferences { plugin_id: plugin_id.clone(), entrypoint_id: None };
                            (description_text, msg)
                        }
                        (false, true) => {
                            // TODO do not show "entrypoint" name to user
                            let description_text = "Before using, entrypoint preferences need to be specified";
                            let msg = AppMsg::OpenSettingsPreferences { plugin_id: plugin_id.clone(), entrypoint_id: Some(entrypoint_id.clone()) };
                            (description_text, msg)
                        }
                        (true, false) => {
                            let description_text = "Before using, plugin preferences need to be specified";
                            let msg = AppMsg::OpenSettingsPreferences { plugin_id: plugin_id.clone(), entrypoint_id: None };
                            (description_text, msg)
                        }
                        (false, false) => unreachable!()
                    };

                    let description: Element<_> = text(description_text)
                        .into();

                    let description = container(description)
                        .width(Length::Fill)
                        .center_x()
                        .themed(ContainerStyle::PreferenceRequiredViewDescription);

                    let button_label: Element<_> = text("Open Settings")
                        .into();

                    let button: Element<_> = button(button_label)
                        .on_press(msg)
                        .into();

                    let button = container(button)
                        .width(Length::Fill)
                        .center_x()
                        .into();

                    let content: Element<_> = column([
                        description,
                        button
                    ]).into();

                    let content: Element<_> = container(content)
                        .center_x()
                        .center_y()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .themed(ContainerStyle::Main);

                    content
                }
                ErrorViewData::PluginErrorViewData { plugin_id, entrypoint_id } => {
                    let description: Element<_> = text("Error occurred in plugin when trying to show the view")
                        .into();

                    let description = container(description)
                        .width(Length::Fill)
                        .center_x()
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Please report this to plugin author")
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .center_x()
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let button_label: Element<_> = text("Close")
                        .into();

                    let button: Element<_> = button(button_label)
                        .on_press(AppMsg::HideWindow)
                        .into();

                    let button = container(button)
                        .width(Length::Fill)
                        .center_x()
                        .into();

                    let content: Element<_> = column([
                        description,
                        sub_description,
                        button
                    ]).into();

                    let content: Element<_> = container(content)
                        .center_x()
                        .center_y()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .themed(ContainerStyle::Main);

                    content
                }
            }
        }

        match &self.plugin_view_data {
            None | Some(PluginViewData { waiting_for_first_render: true, .. }) => {
                let input: Element<_> = text_input("Search...", &self.prompt)
                    .on_input(AppMsg::PromptChanged)
                    .on_submit(AppMsg::PromptSubmit)
                    .id(self.search_field_id.clone())
                    .width(Length::Fill)
                    .themed(TextInputStyle::MainSearch);

                let search_results = self.search_results.iter().cloned().collect();

                let search_list = search_list(
                    search_results,
                    AppMsg::SelectSearchItem
                );

                let search_list = container(search_list)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MainListInner);

                let list: Element<_> = scrollable(search_list)
                    .id(self.scrollable_id.clone())
                    .width(Length::Fill)
                    .into();

                let list = container(list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .themed(ContainerStyle::MainList);

                let input = container(input)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MainSearchBar);

                let separator = horizontal_rule(1)
                    .into();

                let column: Element<_> = column(vec![
                    input,
                    separator,
                    inline_view_container(self.client_context.clone()).into(),
                    list,
                ]).into();

                let element: Element<_> = container(column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .themed(ContainerStyle::Main);

                element
            }
            Some(data) => {
                let PluginViewData {
                    top_level_view: _,
                    plugin_id,
                    plugin_name,
                    entrypoint_id,
                    entrypoint_name,
                    action_shortcuts,
                    waiting_for_first_render: _
                } = data;

                let container_element: Element<_> = view_container(
                    self.client_context.clone(),
                    plugin_id.to_owned(),
                    plugin_name.to_owned(),
                    entrypoint_id.to_owned(),
                    entrypoint_name.to_owned(),
                    action_shortcuts.to_owned(),
                ).into();

                let element: Element<_> = container(container_element)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .themed(ContainerStyle::Root);

                // let element = element.explain(iced::color!(0xFF0000));

                element
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<AppMsg> {
        let client_context = self.client_context.clone();
        let request_rx = self.request_rx.clone();

        struct RequestLoop;
        struct GlobalShortcutListener;

        let events_subscription = event::listen_with(|event, status| match status {
            event::Status::Ignored => Some(AppMsg::IcedEvent(event)),
            event::Status::Captured => match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(Named::Escape), .. }) => Some(AppMsg::IcedEvent(event)),
                _ => None
            }
        });

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
            events_subscription,
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
        let mut commands = vec![
            window::change_mode(window::Id::MAIN, window::Mode::Hidden)
        ];

        if let Some(PluginViewData { plugin_id, .. }) = &self.plugin_view_data {
            commands.push(self.close_view(plugin_id.clone()));
        }

        self.prompt = "".to_string();
        self.plugin_view_data = None;
        self.search_results = vec![];
        self.close_error_view();

        Command::batch(commands)
    }

    fn show_window(&mut self) -> Command<AppMsg> {
        self.close_error_view();

        Command::batch([
            window::change_mode(window::Id::MAIN, window::Mode::Windowed),
            self.reset_window_state()
        ])
    }

    fn reset_window_state(&mut self) -> Command<AppMsg> {
        Command::batch([
            window::gain_focus(window::Id::MAIN),
            scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 }),
            Command::perform(async {}, |_| AppMsg::PromptChanged("".to_owned())),
            focus(self.search_field_id.clone())
        ])
    }

    fn close_error_view(&mut self) {
        self.error_view = None;
    }

    fn previous_view(&mut self) -> Command<AppMsg> {
        match &self.plugin_view_data {
            None => {
                self.hide_window()
            }
            Some(PluginViewData { top_level_view: true, plugin_id, .. }) => {
                let plugin_id = plugin_id.clone();

                self.plugin_view_data.take();

                Command::batch([
                    self.close_view(plugin_id),
                    focus(self.search_field_id.clone()),
                ])
            }
            Some(PluginViewData { top_level_view: false, plugin_id, entrypoint_id, .. }) => {
                self.open_view(plugin_id.clone(), entrypoint_id.clone())
            }
        }
    }

    fn open_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_view_render(plugin_id, entrypoint_id)
                .await
                .unwrap() // TODO proper error handling
        }, |action_shortcuts| AppMsg::SaveActionShortcuts { action_shortcuts })
    }

    fn close_view(&self, plugin_id: PluginId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_view_close(plugin_id)
                .await
                .unwrap() // TODO proper error handling
        }, |_| AppMsg::Noop)
    }

    fn run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_run_command(plugin_id, entrypoint_id)
                .await
                .unwrap(); // TODO proper error handling
        }, |_| AppMsg::Noop)
    }

    fn run_generated_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_run_generated_command(plugin_id, entrypoint_id)
                .await
                .unwrap(); // TODO proper error handling
        }, |_| AppMsg::Noop)
    }

    fn append_prompt(&mut self, value: String) {
        self.prompt = format!("{}{}", self.prompt, value);
    }

    fn backspace_prompt(&mut self) {
        let mut chars = self.prompt.chars();
        chars.next_back();
        self.prompt = chars.as_str().to_owned();
    }
}

fn register_shortcut() -> GlobalHotKeyManager {
    use global_hotkey::hotkey::{Code, HotKey, Modifiers};

    let manager = GlobalHotKeyManager::new().unwrap();

    let key = if cfg!(target_os = "windows") {
        HotKey::new(Some(Modifiers::ALT), Code::Space)
    } else {
        HotKey::new(Some(Modifiers::META), Code::Space)
    };

    let result = manager.register(key);

    if let Err(err) = &result {
        tracing::warn!(target = "rpc", "error occurred when registering shortcut {:?}", err)
    }

    manager
}

fn listen_on_shortcut(sender: Sender<AppMsg>) {
    use global_hotkey::GlobalHotKeyEvent;

    let handle = Handle::current();

    GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
        let mut sender = sender.clone();
        if let HotKeyState::Released = e.state() {
            handle.spawn(async move {
                sender.send(AppMsg::ShowWindow).await
                    .unwrap();
            });
        }
    }));
}


async fn request_loop(
    client_context: Arc<StdRwLock<ClientContext>>,
    request_rx: Arc<TokioRwLock<RequestReceiver<UiRequestData, NativeUiResponseData>>>,
    mut sender: Sender<AppMsg>,
) {
    let mut request_rx = request_rx.write().await;
    loop {
        let (request_data, responder) = request_rx.recv().await;

        let app_msg = {
            let mut client_context = client_context.write().expect("lock is poisoned");

            match request_data {
                UiRequestData::ReplaceView { plugin_id, entrypoint_id, render_location, top_level_view, container } => {
                    client_context.replace_view(render_location, container, &plugin_id, &entrypoint_id);

                    responder.respond(NativeUiResponseData::Nothing);

                    AppMsg::ReplaceView {
                        top_level_view
                    }
                }
                UiRequestData::ClearInlineView { plugin_id } => {
                    client_context.clear_inline_view(&plugin_id);

                    responder.respond(NativeUiResponseData::Nothing);

                    AppMsg::Noop // refresh ui
                }
                UiRequestData::ShowWindow => {
                    responder.respond(NativeUiResponseData::Nothing);

                    AppMsg::ShowWindow
                }
                UiRequestData::ShowPreferenceRequiredView {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required
                } => {
                    responder.respond(NativeUiResponseData::Nothing);

                    AppMsg::ShowPreferenceRequiredView {
                        plugin_id,
                        entrypoint_id,
                        plugin_preferences_required,
                        entrypoint_preferences_required
                    }
                }
                UiRequestData::ShowPluginErrorView { plugin_id, entrypoint_id, render_location } => {
                    responder.respond(NativeUiResponseData::Nothing);

                    AppMsg::ShowPluginErrorView {
                        plugin_id,
                        entrypoint_id,
                        render_location,
                    }
                }
                UiRequestData::RequestSearchResultUpdate => {
                    responder.respond(NativeUiResponseData::Nothing);

                    AppMsg::RequestSearchResultUpdate
                }
            }
        };

        let _ = sender.send(app_msg).await;
    }
}
