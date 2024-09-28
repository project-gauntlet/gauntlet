use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock as StdRwLock};
use anyhow::anyhow;
use iced::{Alignment, Command, Event, event, executor, font, Font, futures, keyboard, Length, Padding, Pixels, Settings, Size, Subscription, subscription, window};
use iced::advanced::graphics::core::SmolStr;
use iced::advanced::layout::Limits;
use iced::multi_window::Application;
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, scrollable, Space, text, text_input};
use iced::widget::scrollable::{AbsoluteOffset, scroll_to};
use iced::widget::text_input::focus;
use iced::window::{Level, Position, Screenshot};
use iced::window::settings::PlatformSpecific;
use iced_aw::core::icons;
use serde::Deserialize;
use tokio::runtime::Handle;
use tokio::sync::RwLock as TokioRwLock;
use tonic::transport::Server;

use client_context::ClientContext;
use common::model::{BackendRequestData, BackendResponseData, EntrypointId, PhysicalKey, PhysicalShortcut, PluginId, SearchResult, SearchResultEntrypointType, UiRenderLocation, UiRequestData, UiResponseData, UiWidgetId};
use common::rpc::backend_api::{BackendApi, BackendForFrontendApi, BackendForFrontendApiError};
use common::scenario_convert::{ui_render_location_from_scenario, ui_widget_from_scenario};
use common::scenario_model::{ScenarioFrontendEvent, ScenarioUiRenderLocation};
use common_ui::physical_key_model;
use utils::channel::{channel, RequestReceiver, RequestSender};

use crate::model::UiViewEvent;
use crate::ui::inline_view_container::inline_view_container;
use crate::ui::search_list::search_list;
use crate::ui::theme::{Element, ThemableWidget};
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::text_input::TextInputStyle;
use crate::ui::view_container::view_container;
use crate::ui::widget::{render_root, ActionPanel, ActionPanelItems, ComponentWidgetEvent};

mod view_container;
mod search_list;
mod widget;
mod theme;
mod client_context;
mod widget_container;
mod inline_view_container;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod sys_tray;
mod custom_widgets;

pub use theme::GauntletTheme;

pub struct AppModel {
    // logic
    backend_api: BackendForFrontendApi,
    frontend_receiver: Arc<TokioRwLock<RequestReceiver<UiRequestData, UiResponseData>>>,
    search_field_id: text_input::Id,
    focused: bool,
    theme: GauntletTheme,
    wayland: bool,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    tray_icon: tray_icon::TrayIcon,

    // ephemeral state
    prompt: String,
    focused_search_result: ScrollHandle,
    focused_action_item: ScrollHandle,

    // state
    client_context: Arc<StdRwLock<ClientContext>>,
    plugin_view_data: Option<PluginViewData>,
    error_view: Option<ErrorViewData>,
    search_results: Vec<SearchResult>,
    show_action_panel: bool,
}

#[derive(Clone, Debug)]
struct ScrollHandle {
    scrollable_id: scrollable::Id,
    index: usize,
    offset: usize,
}

impl ScrollHandle {
    fn new(scrollable_id: scrollable::Id) -> ScrollHandle {
        ScrollHandle {
            scrollable_id,
            index: 0,
            offset: 0,
        }
    }

    fn reset(&mut self) {
        self.index = 0;
        self.offset = 0;
    }

    fn get<'a, T>(&self, search_results: &'a Vec<T>) -> Option<&'a T> {
        search_results.get(self.index)
    }

    fn focus_next(&mut self, item_amount: usize) -> Command<AppMsg> {
        self.offset = if self.offset < 8 {
            self.offset + 1
        } else {
            8
        };

        if self.index < item_amount - 1 {
            self.index = self.index + 1;

            let pos_y = self.index as f32 * ESTIMATED_ITEM_SIZE - (self.offset as f32 * ESTIMATED_ITEM_SIZE);

            scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: pos_y })
        } else {
            Command::none()
        }
    }

    fn focus_previous(&mut self) -> Command<AppMsg> {
        self.offset = if self.offset > 1 {
            self.offset - 1
        } else {
            1
        };

        if self.index > 0 {
            self.index = self.index - 1;

            let pos_y = self.index as f32 * ESTIMATED_ITEM_SIZE - (self.offset as f32 * ESTIMATED_ITEM_SIZE);

            scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: pos_y })
        } else {
            Command::none()
        }
    }
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
    PreferenceRequired {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    PluginError {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    BackendTimeout,
    UnknownError {
        display: String
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
        action_index: Option<usize>
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
    ToggleActionPanel,
    OnEntrypointAction(UiWidgetId),
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
    RunSearchItemAction(SearchResult, Option<usize>),
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
    HandleBackendError(BackendForFrontendApiError),
}

pub struct AppFlags {
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
    wayland: bool,
}

impl Default for AppFlags {
    fn default() -> Self {
        panic!("not needed")
    }
}

const WINDOW_WIDTH: f32 = 750.0;
const WINDOW_HEIGHT: f32 = 450.0;

#[cfg(target_os = "linux")]
fn layer_shell_settings() -> iced::wayland::runtime::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings {
    iced::wayland::runtime::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings {
        id: window::Id::MAIN,
        layer: iced::wayland::commands::layer_surface::Layer::Overlay,
        keyboard_interactivity: iced::wayland::commands::layer_surface::KeyboardInteractivity::Exclusive,
        pointer_interactivity: true,
        anchor: iced::wayland::commands::layer_surface::Anchor::empty(),
        output: Default::default(),
        namespace: "Gauntlet".to_string(),
        margin: Default::default(),
        exclusive_zone: 0,
        size: Some((Some(WINDOW_WIDTH as u32), Some(WINDOW_HEIGHT as u32))),
        size_limits: Limits::new(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT), Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
    }
}

pub fn run(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) {
    let default_settings: Settings<()> = Settings::default();

    #[cfg(target_os = "linux")]
    let wayland = std::env::var("WAYLAND_DISPLAY")
        .or_else(|_| std::env::var("WAYLAND_SOCKET"))
        .is_ok();

    #[cfg(not(target_os = "linux"))]
    let wayland = false;

    let flags = AppFlags {
        frontend_receiver,
        backend_sender,
        wayland
    };

    let settings = Settings {
        id: None,
        window: window::Settings {
            size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            position: Position::Centered,
            resizable: false,
            decorations: false,
            transparent: true,
            visible: !minimized,
            #[cfg(target_os = "macos")]
            platform_specific: PlatformSpecific {
                activation_policy: window::settings::ActivationPolicy::Accessory,
                activate_ignoring_other_apps: false,
                ..Default::default()
            },
            ..Default::default()
        },
        #[cfg(target_os = "linux")]
        initial_surface: iced::wayland::settings::InitialSurface::LayerSurface(layer_shell_settings()),
        flags,
        fonts: default_settings.fonts,
        default_font: default_settings.default_font,
        default_text_size: default_settings.default_text_size,
        antialiasing: default_settings.antialiasing,
        #[cfg(target_os = "linux")]
        exit_on_close_request: false,
    };

    #[cfg(target_os = "linux")]
    let result = if wayland {
        AppModel::run_wayland(settings)
    } else {
        AppModel::run(settings)
    };

    #[cfg(not(target_os = "linux"))]
    let result = AppModel::run(settings);

    result.expect("Unable to start application")
}

impl Application for AppModel {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = GauntletTheme;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let frontend_receiver = flags.frontend_receiver;
        let backend_sender = flags.backend_sender;
        let wayland = flags.wayland;

        let backend_api = BackendForFrontendApi::new(backend_sender);

        let mut commands = vec![
            Command::perform(async {}, |_| AppMsg::ResetWindowState),
            font::load(icons::BOOTSTRAP_FONT_BYTES).map(AppMsg::FontLoaded),
        ];

        if !wayland {
            commands.push(
                window::change_level(window::Id::MAIN, Level::AlwaysOnTop),
            )
        }

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
                        "Screenshot Plugin",
                        &entrypoint_id,
                        "Screenshot Entrypoint",
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
                    let error_view = Some(ErrorViewData::PreferenceRequired {
                        plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                        entrypoint_id: EntrypointId::from_string(entrypoint_id),
                        plugin_preferences_required,
                        entrypoint_preferences_required,
                    });

                    (ClientContext::new(), None, error_view)
                }
                ScenarioFrontendEvent::ShowPluginErrorView { entrypoint_id, render_location: _ } => {
                    let error_view = Some(ErrorViewData::PluginError {
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
                frontend_receiver: Arc::new(TokioRwLock::new(frontend_receiver)),
                search_field_id: text_input::Id::unique(),
                focused: false,
                theme: GauntletTheme::new(),
                wayland,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                tray_icon: sys_tray::create_tray(),

                // ephemeral state
                prompt: "".to_string(),
                focused_search_result: ScrollHandle::new(scrollable::Id::unique()),
                focused_action_item: ScrollHandle::new(scrollable::Id::unique()),

                // state
                client_context: Arc::new(StdRwLock::new(client_context)),
                plugin_view_data,
                error_view,
                search_results: vec![],
                show_action_panel: false,
            },
            Command::batch(commands),
        )
    }

    fn title(&self, _window: window::Id) -> String {
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
            AppMsg::RunGeneratedCommandEvent { plugin_id, entrypoint_id, action_index } => {
                Command::batch([
                    self.hide_window(),
                    self.run_generated_command(plugin_id, entrypoint_id, action_index),
                ])
            }
            AppMsg::PromptChanged(mut new_prompt) => {
                if cfg!(feature = "scenario_runner") {
                    Command::none()
                } else {
                    new_prompt.truncate(100); // search query uses regex so just to be safe truncate the prompt

                    self.show_action_panel = false;
                    self.prompt = new_prompt.clone();
                    self.focused_search_result.reset();
                    self.focused_action_item.reset();

                    let mut backend_api = self.backend_api.clone();

                    Command::perform(async move {
                        let search_results = backend_api.search(new_prompt, true)
                            .await?;

                        Ok(search_results)
                    }, |result| handle_backend_error(result, |search_results| AppMsg::SetSearchResults(search_results)))
                }
            }
            AppMsg::UpdateSearchResults => {
                let prompt = self.prompt.clone();

                let mut backend_api = self.backend_api.clone();

                Command::perform(async move {
                    let search_results = backend_api.search(prompt, false)
                        .await?;

                    Ok(search_results)
                }, |result| handle_backend_error(result, |search_results| AppMsg::SetSearchResults(search_results)))
            }
            AppMsg::PromptSubmit => {
                if self.show_action_panel {
                    self.show_action_panel = false;
                    let widget_id = self.focused_action_item.index;
                    Command::perform(async {}, move |_| AppMsg::OnEntrypointAction(widget_id))
                } else {
                    self.show_action_panel = false;
                    if let Some(search_item) = self.focused_search_result.get(&self.search_results) {
                        let search_item = search_item.clone();
                        Command::perform(async {}, |_| AppMsg::RunSearchItemAction(search_item, None))
                    } else {
                        Command::none()
                    }
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
                        tracing::debug!("Key pressed: {:?}. shift: {:?} control: {:?} alt: {:?} meta: {:?}", key, modifiers.shift(), modifiers.control(), modifiers.alt(), modifiers.logo());
                        match key {
                            Key::Named(Named::ArrowUp) => self.focus_previous(),
                            Key::Named(Named::ArrowDown) => self.focus_next(),
                            Key::Named(Named::Escape) => self.previous_view(),
                            Key::Named(Named::Enter) => {
                                // fired in cases where main text field is not focused
                                Command ::perform(async {}, |_| AppMsg::PromptSubmit)
                            },
                            Key::Named(Named::Backspace) => {
                                self.backspace_prompt();
                                focus(self.search_field_id.clone())
                            },
                            _ => {
                                if self.plugin_view_data.is_none() {
                                    match physical_key_model(physical_key, modifiers) {
                                        Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                            Command::perform(async {}, |_| AppMsg::ToggleActionPanel)
                                        }
                                        _ => {
                                            match text {
                                                Some(text) => {
                                                    self.append_prompt(text.to_string());
                                                    focus(self.search_field_id.clone())
                                                }
                                                None => {
                                                    Command::none()
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    match physical_key_model(physical_key, modifiers) {
                                        Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                            let client_context = self.client_context.read().expect("lock is poisoned");

                                            client_context.show_action_panel();

                                            Command::none()
                                        }
                                        Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                            let (plugin_id, entrypoint_id) = {
                                                let client_context = self.client_context.read().expect("lock is poisoned");
                                                (client_context.get_view_plugin_id(), client_context.get_view_entrypoint_id())
                                            };

                                            Command::perform(
                                                async move {
                                                    backend_client.send_keyboard_event(plugin_id, entrypoint_id, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                                                        .await?;

                                                    Ok(())
                                                },
                                                |result| handle_backend_error(result, |()| AppMsg::Noop),
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
            AppMsg::IcedEvent(Event::Window(_, window::Event::Focused)) => {
                self.on_focused()
            }
            AppMsg::IcedEvent(Event::Window(_, window::Event::Unfocused)) => {
                self.on_unfocused()
            }
            #[cfg(target_os = "linux")]
            AppMsg::IcedEvent(
                Event::PlatformSpecific(
                    iced::wayland::core::event::PlatformSpecific::Wayland(
                        iced::wayland::core::event::wayland::Event::Layer(
                            iced::wayland::core::event::wayland::LayerEvent::Unfocused,
                            _,
                            _
                        )
                    )
                )
            ) => {
                // wayland layer shell doesn't have the same unfocused problem as the other platforms
                self.hide_window()
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
                                    .await?;
                            }
                            UiViewEvent::Open { href } => {
                                backend_client.send_open_event(plugin_id, href)
                                    .await?;
                            }
                        }
                    }

                    Ok(())
                }, |result| handle_backend_error(result, |()| AppMsg::Noop))
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
                self.error_view = Some(ErrorViewData::PreferenceRequired {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required,
                });
                Command::none()
            }
            AppMsg::ShowPluginErrorView { plugin_id, entrypoint_id, render_location } => {
                self.error_view = Some(ErrorViewData::PluginError {
                    plugin_id,
                    entrypoint_id,
                });
                Command::none()
            }
            AppMsg::OpenSettingsPreferences { plugin_id, entrypoint_id, } => {
                let mut backend_api = self.backend_api.clone();

                Command::perform(async move {
                    backend_api.open_settings_window_preferences(plugin_id, entrypoint_id)
                        .await?;

                    Ok(())
                }, |result| handle_backend_error(result, |()| AppMsg::Noop))
            }
            AppMsg::SaveActionShortcuts { action_shortcuts } => {
                if let Some(data) = self.plugin_view_data.as_mut() {
                    data.action_shortcuts = action_shortcuts;
                }
                Command::none()
            }
            AppMsg::RunSearchItemAction(search_result, action_index) => {
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
                        plugin_id: search_result.plugin_id.clone(),
                        action_index,
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
            AppMsg::Close => {
                #[cfg(target_os = "linux")]
                if self.wayland {
                    iced::wayland::commands::window::close_window(window::Id::MAIN)
                } else {
                    window::close(window::Id::MAIN)
                }

                #[cfg(not(target_os = "linux"))]
                window::close(window::Id::MAIN)
            }
            AppMsg::HandleBackendError(err) => {
                self.error_view = Some(match err {
                    BackendForFrontendApiError::TimeoutError => ErrorViewData::BackendTimeout,
                });

                Command::none()
            }
            AppMsg::ToggleActionPanel => {
                self.show_action_panel = !self.show_action_panel;

                Command::none()
            }
            AppMsg::OnEntrypointAction(widget_id) => {
                if let Some(search_item) = self.focused_search_result.get(&self.search_results) {
                    let search_item = search_item.clone();
                    if widget_id == 0 {
                        Command::perform(async {}, |_| AppMsg::RunSearchItemAction(search_item, None))
                    } else {
                        Command::perform(async {}, move |_| AppMsg::RunSearchItemAction(search_item, Some(widget_id - 1)))
                    }
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&self, _window: window::Id) -> Element<'_, Self::Message> {
        if let Some(view_data) = &self.error_view {
            return match view_data {
                ErrorViewData::PreferenceRequired { plugin_id, entrypoint_id, plugin_preferences_required, entrypoint_preferences_required } => {

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
                ErrorViewData::PluginError { plugin_id, entrypoint_id } => {
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
                ErrorViewData::UnknownError { display } => {
                    let description: Element<_> = text("Unknown error occurred")
                        .into();

                    let description = container(description)
                        .width(Length::Fill)
                        .center_x()
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Please report") // TODO link
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .center_x()
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let error_description: Element<_> = text(display)
                        .into();

                    let error_description = container(error_description)
                        .width(Length::Fill)
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let error_description = scrollable(error_description)
                        .width(Length::Fill)
                        .into();

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
                        error_description,
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
                ErrorViewData::BackendTimeout => {
                    let description: Element<_> = text("Error occurred")
                        .into();

                    let description = container(description)
                        .width(Length::Fill)
                        .center_x()
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Backend was unable to process message in a timely manner")
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
                    &self.focused_search_result,
                    |search_result| AppMsg::RunSearchItemAction(search_result, None)
                );

                let search_list = container(search_list)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MainListInner);

                let list: Element<_> = scrollable(search_list)
                    .id(self.focused_search_result.scrollable_id.clone())
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

                let content: Element<_> = column(vec![
                    inline_view_container(self.client_context.clone()).into(),
                    list,
                ]).into();

                let (default_action, action_panel) = if let Some(search_item) = self.focused_search_result.get(&self.search_results) {
                    let label = match search_item.entrypoint_type {
                        SearchResultEntrypointType::Command => "Run Command",
                        SearchResultEntrypointType::View => "Open View",
                        SearchResultEntrypointType::GeneratedCommand => "Run Command",
                    }.to_string();

                    let default_shortcut = PhysicalShortcut {
                        physical_key: PhysicalKey::Enter,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: false,
                    };

                    let mut actions: Vec<_> = search_item.entrypoint_actions
                        .iter()
                        .enumerate()
                        .map(|(index, action)| ActionPanelItems::Action {
                            label: action.label.clone(),
                            widget_id: index + 1,
                            physical_shortcut: action.shortcut.clone(),
                        })
                        .collect();

                    if actions.len() == 0 {
                        (Some((label, default_shortcut)), None)
                    } else {
                        let default_action = ActionPanelItems::Action {
                            label: label.clone(),
                            widget_id: 0,
                            physical_shortcut: Some(default_shortcut.clone()),
                        };

                        actions.insert(0, default_action);

                        let action_panel = ActionPanel {
                            title: Some(search_item.entrypoint_name.clone()),
                            items: actions,
                        };

                        (Some((label, default_shortcut)), Some(action_panel))
                    }
                } else {
                    (None, None)
                };

                let root = render_root(
                    self.show_action_panel,
                    input,
                    separator,
                    content,
                    default_action,
                    action_panel,
                    &self.focused_action_item,
                    "".to_string(),
                    || AppMsg::ToggleActionPanel,
                    AppMsg::OnEntrypointAction
                );

                let root: Element<_> = container(root)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .themed(ContainerStyle::Main);

                root
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

                // let element = element.explain(color!(0xFF0000));

                element
            }
        }
    }

    fn theme(&self, _window: window::Id) -> Self::Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<AppMsg> {
        let client_context = self.client_context.clone();
        let frontend_receiver = self.frontend_receiver.clone();

        struct RequestLoop;

        let events_subscription = event::listen_with(|event, status| match status {
            event::Status::Ignored => Some(AppMsg::IcedEvent(event)),
            event::Status::Captured => match &event {
                Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(Named::Escape), .. }) => Some(AppMsg::IcedEvent(event)),
                Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Character(char), modifiers, .. }) => {
                    if char == "k" && modifiers.alt() {
                        // TODO this still enters "k" into a search bar which is undesirable
                        Some(AppMsg::IcedEvent(event))
                    } else {
                        None
                    }
                },
                _ => None
            }
        });

        Subscription::batch([
            events_subscription,
            subscription::channel(
                std::any::TypeId::of::<RequestLoop>(),
                100,
                |sender| async move {
                    request_loop(client_context, frontend_receiver, sender).await;

                    panic!("request_rx was unexpectedly closed")
                },
            )
        ])
    }
}

const ESTIMATED_ITEM_SIZE: f32 = 38.8;

impl AppModel {
    fn on_focused(&mut self) -> Command<AppMsg> {
        self.focused = true;
        Command::none()
    }

    fn on_unfocused(&mut self) -> Command<AppMsg> {
        // for some reason (on both macos and linux x11) duplicate Unfocused fires right before Focus event
        if self.focused {
            self.focused = false;
            self.hide_window()
        } else {
            Command::none()
        }
    }

    fn hide_window(&mut self) -> Command<AppMsg> {
        let mut commands = vec![];

        #[cfg(target_os = "linux")]
        if self.wayland {
            use iced::wayland::commands::layer_surface::KeyboardInteractivity;

            commands.push(
                iced::wayland::commands::layer_surface::destroy_layer_surface(window::Id::MAIN),
            );
            commands.push(
                iced::wayland::commands::layer_surface::set_keyboard_interactivity(window::Id::MAIN, KeyboardInteractivity::None),
            );
        } else {
            commands.push(
                window::change_mode(window::Id::MAIN, window::Mode::Hidden)
            );
        };

        #[cfg(not(target_os = "linux"))]
        commands.push(
            window::change_mode(window::Id::MAIN, window::Mode::Hidden)
        );

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

        let mut commands = vec![];

        #[cfg(target_os = "linux")]
        if self.wayland {
            use iced::wayland::commands::layer_surface::KeyboardInteractivity;

            commands.push(
                iced::wayland::commands::layer_surface::get_layer_surface(layer_shell_settings()),
            );
            commands.push(
                iced::wayland::commands::layer_surface::set_keyboard_interactivity(window::Id::MAIN, KeyboardInteractivity::Exclusive),
            );
        } else {
            commands.push(
                window::change_mode(window::Id::MAIN, window::Mode::Windowed)
            );
        };

        #[cfg(not(target_os = "linux"))]
        commands.push(
            window::change_mode(window::Id::MAIN, window::Mode::Windowed)
        );

        commands.push(
            self.reset_window_state()
        );

        Command::batch(commands)
    }

    fn reset_window_state(&mut self) -> Command<AppMsg> {
        self.focused_action_item.reset();
        self.focused_search_result.reset();
        self.show_action_panel = false;

        let mut commands = vec![
            scroll_to(self.focused_action_item.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 }),
            scroll_to(self.focused_search_result.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: 0.0 }),
            Command::perform(async {}, |_| AppMsg::PromptChanged("".to_owned())),
            focus(self.search_field_id.clone())
        ];

        if !self.wayland {
            commands.push(
                window::gain_focus(window::Id::MAIN),
            );
        }

        Command::batch(commands)
    }

    fn focus_next(&mut self) -> Command<AppMsg> {
        if self.show_action_panel {
            if let Some(search_item) = self.focused_search_result.get(&self.search_results) {
                if search_item.entrypoint_actions.len() != 0 {
                    self.focused_action_item.focus_next(search_item.entrypoint_actions.len() + 1)
                } else {
                    self.show_action_panel = false;
                    Command::none()
                }
            } else {
                self.show_action_panel = false;
                Command::none()
            }
        } else {
            self.focused_search_result.focus_next(self.search_results.len())
        }
    }

    fn focus_previous(&mut self) -> Command<AppMsg> {
        if self.show_action_panel {
            if let Some(search_item) = self.focused_search_result.get(&self.search_results) {
                if search_item.entrypoint_actions.len() != 0 {
                    self.focused_action_item.focus_previous()
                } else {
                    self.show_action_panel = false;
                    Command::none()
                }
            } else {
                self.show_action_panel = false;
                Command::none()
            }
        } else {
            self.focused_search_result.focus_previous()
        }
    }

    fn close_error_view(&mut self) {
        self.error_view = None;
    }

    fn previous_view(&mut self) -> Command<AppMsg> {
        if self.show_action_panel {
            self.show_action_panel = false;
            Command::none()
        } else {
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
    }

    fn open_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            let result = backend_client.request_view_render(plugin_id, entrypoint_id)
                .await?;

            Ok(result)
        }, |result| handle_backend_error(result, |action_shortcuts| AppMsg::SaveActionShortcuts { action_shortcuts }))
    }

    fn close_view(&self, plugin_id: PluginId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_view_close(plugin_id)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_run_command(plugin_id, entrypoint_id)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn run_generated_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, action_index: Option<usize>) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            backend_client.request_run_generated_command(plugin_id, entrypoint_id, action_index)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
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

fn handle_backend_error<T>(result: Result<T, BackendForFrontendApiError>, convert: impl FnOnce(T) -> AppMsg) -> AppMsg {
    match result {
        Ok(val) => convert(val),
        Err(err) => AppMsg::HandleBackendError(err)
    }
}

async fn request_loop(
    client_context: Arc<StdRwLock<ClientContext>>,
    frontend_receiver: Arc<TokioRwLock<RequestReceiver<UiRequestData, UiResponseData>>>,
    mut sender: Sender<AppMsg>,
) {
    let mut frontend_receiver = frontend_receiver.write().await;
    loop {
        let (request_data, responder) = frontend_receiver.recv().await;

        let app_msg = {
            let mut client_context = client_context.write().expect("lock is poisoned");

            match request_data {
                UiRequestData::ReplaceView {
                    plugin_id,
                    plugin_name,
                    entrypoint_id,
                    entrypoint_name,
                    render_location,
                    top_level_view,
                    container
                } => {
                    client_context.replace_view(
                        render_location,
                        container,
                        &plugin_id,
                        &plugin_name,
                        &entrypoint_id,
                        &entrypoint_name
                    );

                    responder.respond(UiResponseData::Nothing);

                    AppMsg::ReplaceView {
                        top_level_view
                    }
                }
                UiRequestData::ClearInlineView { plugin_id } => {
                    client_context.clear_inline_view(&plugin_id);

                    responder.respond(UiResponseData::Nothing);

                    AppMsg::Noop // refresh ui
                }
                UiRequestData::ShowWindow => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::ShowWindow
                }
                UiRequestData::ShowPreferenceRequiredView {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required
                } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::ShowPreferenceRequiredView {
                        plugin_id,
                        entrypoint_id,
                        plugin_preferences_required,
                        entrypoint_preferences_required
                    }
                }
                UiRequestData::ShowPluginErrorView { plugin_id, entrypoint_id, render_location } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::ShowPluginErrorView {
                        plugin_id,
                        entrypoint_id,
                        render_location,
                    }
                }
                UiRequestData::RequestSearchResultUpdate => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::RequestSearchResultUpdate
                }
            }
        };

        let _ = sender.send(app_msg).await;
    }
}
