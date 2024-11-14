use anyhow::anyhow;
use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;
use iced::advanced::graphics::core::SmolStr;
use iced::advanced::layout::Limits;
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::multi_window::Application;
use iced::widget::scrollable::{scroll_to, AbsoluteOffset};
use iced::widget::text::Shaping;
use iced::widget::text_input::focus;
use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input, Space};
use iced::window::settings::PlatformSpecific;
use iced::window::{Level, Position, Screenshot};
use iced::{event, executor, font, futures, keyboard, subscription, window, Alignment, Command, Event, Font, Length, Padding, Pixels, Settings, Size, Subscription};
use iced_aw::core::icons;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex as StdMutex, Mutex, RwLock as StdRwLock};
use serde::Deserialize;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};
use tonic::transport::Server;

use client_context::ClientContext;
use common::model::{BackendRequestData, BackendResponseData, EntrypointId, KeyboardEventOrigin, PhysicalKey, PhysicalShortcut, PluginId, RootWidget, RootWidgetMembers, SearchResult, SearchResultEntrypointAction, SearchResultEntrypointType, UiRenderLocation, UiRequestData, UiResponseData, UiWidgetId};
use common::rpc::backend_api::{BackendApi, BackendForFrontendApi, BackendForFrontendApiError};
use common::scenario_convert::{ui_render_location_from_scenario};
use common::scenario_model::{ScenarioFrontendEvent, ScenarioUiRenderLocation};
use common_ui::physical_key_model;
use utils::channel::{channel, RequestReceiver, RequestSender, Responder};

use crate::model::UiViewEvent;
use crate::ui::search_list::search_list;
use crate::ui::theme::container::{ContainerStyle, ContainerStyleInner};
use crate::ui::theme::text_input::TextInputStyle;
use crate::ui::theme::{Element, ThemableWidget};
use crate::ui::widget::{render_root, ActionPanel, ActionPanelItem, ComponentWidgetEvent};

mod search_list;
mod widget;
mod theme;
mod client_context;
mod widget_container;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod sys_tray;
mod custom_widgets;
mod scroll_handle;
mod state;
mod hud;
mod grid_navigation;

use crate::global_shortcut::{convert_physical_shortcut_to_hotkey, register_listener};
use crate::ui::custom_widgets::loading_bar::LoadingBar;
use crate::ui::hud::{close_hud_window, show_hud_window};
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::{ErrorViewData, Focus, GlobalState, LoadingBarState, MainViewState, PluginViewData, PluginViewState};
use crate::ui::widget_container::PluginWidgetContainer;
pub use theme::GauntletTheme;

pub struct AppModel {
    // logic
    backend_api: BackendForFrontendApi,
    global_hotkey_manager: Arc<StdRwLock<GlobalHotKeyManager>>,
    current_hotkey: Arc<StdMutex<Option<HotKey>>>,
    frontend_receiver: Arc<TokioRwLock<RequestReceiver<UiRequestData, UiResponseData>>>,
    focused: bool,
    theme: GauntletTheme,
    wayland: bool,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    tray_icon: tray_icon::TrayIcon,

    // ephemeral state
    prompt: String,

    // state
    client_context: Arc<StdRwLock<ClientContext>>,
    global_state: GlobalState,
    search_results: Vec<SearchResult>,
    loading_bar_state: HashMap<(PluginId, EntrypointId), ()>,
    hud_display: Option<String>
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
        has_children: bool,
        render_location: UiRenderLocation,
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
    ToggleActionPanel {
        keyboard: bool
    },
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
    OnOpenView {
        action_shortcuts: HashMap<String, PhysicalShortcut>
    },
    ShowPluginErrorView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation
    },
    RunSearchItemAction(SearchResult, Option<usize>),
    Screenshot {
        save_path: String
    },
    ScreenshotDone {
        save_path: String,
        screenshot: Screenshot
    },
    Close,
    ShowBackendError(BackendForFrontendApiError),
    ClosePluginView(PluginId),
    OpenPluginView(PluginId, EntrypointId),
    InlineViewShortcuts {
        shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>
    },
    ShowHud {
        display: String
    },
    CloseHudWindow {
        id: window::Id
    },
    OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus,
    OnPrimaryActionMainViewNoPanelKeyboardWithFocus { search_result: SearchResult },
    OnPrimaryActionMainViewSearchResultPanelKeyboardWithFocus { search_result: SearchResult, widget_id: UiWidgetId },
    OnPrimaryActionMainViewInlineViewPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnPrimaryActionPluginViewNoPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnPrimaryActionPluginViewAnyPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnAnyActionPluginViewAnyPanel { widget_id: UiWidgetId },
    OnSecondaryActionMainViewNoPanelKeyboardWithFocus { search_result: SearchResult },
    OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus,
    OnSecondaryActionPluginViewNoPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnAnyActionMainViewAnyPanelMouse { widget_id: UiWidgetId },
    OnPrimaryActionMainViewActionPanelMouse { widget_id: UiWidgetId },
    ResetMainViewState,
    OnAnyActionMainViewNoPanelKeyboardAtIndex { index: usize },
    SetGlobalShortcut {
        shortcut: PhysicalShortcut,
        responder: Arc<Mutex<Option<Responder<UiResponseData>>>>
    },
    UpdateLoadingBar {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        show: bool
    },
    PendingPluginViewLoadingBar,
    ShowPluginViewLoadingBar,
    FocusPluginViewSearchBar {
        widget_id: UiWidgetId
    },
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

        let global_hotkey_manager = GlobalHotKeyManager::new()
            .expect("unable to create global hot key manager");

        let mut commands = vec![
            font::load(icons::BOOTSTRAP_FONT_BYTES).map(AppMsg::FontLoaded),
        ];

        if !wayland {
            commands.push(
                window::gain_focus(window::Id::MAIN),
            );

            commands.push(
                window::change_level(window::Id::MAIN, Level::AlwaysOnTop),
            )
        }

        let (client_context, global_state) = if cfg!(feature = "scenario_runner") {
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
                ScenarioFrontendEvent::ReplaceView { entrypoint_id, render_location, top_level_view, container, images } => {
                    let plugin_id = PluginId::from_string("__SCREENSHOT_GEN___");
                    let entrypoint_id = EntrypointId::from_string(entrypoint_id);

                    let mut context = ClientContext::new();

                    let render_location = ui_render_location_from_scenario(render_location);
                    let container = RootWidget::deserialize(container).expect("should always be valid");
                    let has_children = container.content.is_some();

                    // ignore commands because screenshots are non-interactive
                    let _ = context.replace_view(
                        render_location,
                        container,
                        images,
                        &plugin_id,
                        "Screenshot Plugin",
                        &entrypoint_id,
                        "Screenshot Entrypoint",
                    );

                    let context = Arc::new(StdRwLock::new(context));

                    commands.push(Command::perform(async { }, move |_| AppMsg::ReplaceView { top_level_view, has_children, render_location }));

                    let state= match render_location {
                        UiRenderLocation::InlineView => GlobalState::new(text_input::Id::unique(), context.clone()),
                        UiRenderLocation::View => GlobalState::new_plugin(
                            PluginViewData {
                                top_level_view,
                                plugin_id,
                                plugin_name: "Screenshot Gen".to_string(),
                                entrypoint_id,
                                entrypoint_name: gen_name,
                                action_shortcuts: Default::default(),
                            },
                            context.clone()
                        )
                    };

                    (context, state)
                }
                ScenarioFrontendEvent::ShowPreferenceRequiredView { entrypoint_id, plugin_preferences_required, entrypoint_preferences_required } => {
                    let error_view = ErrorViewData::PreferenceRequired {
                        plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                        entrypoint_id: EntrypointId::from_string(entrypoint_id),
                        plugin_preferences_required,
                        entrypoint_preferences_required,
                    };

                    (Arc::new(StdRwLock::new(ClientContext::new())), GlobalState::new_error(error_view))
                }
                ScenarioFrontendEvent::ShowPluginErrorView { entrypoint_id, render_location: _ } => {
                    let error_view = ErrorViewData::PluginError {
                        plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                        entrypoint_id: EntrypointId::from_string(entrypoint_id),
                    };

                    (Arc::new(StdRwLock::new(ClientContext::new())), GlobalState::new_error(error_view))
                }
            }
        } else {
            let context = Arc::new(StdRwLock::new(ClientContext::new()));
            (context.clone(), GlobalState::new(text_input::Id::unique(), context.clone()))
        };

        (
            AppModel {
                // logic
                backend_api,
                global_hotkey_manager: Arc::new(StdRwLock::new(global_hotkey_manager)),
                current_hotkey: Arc::new(StdMutex::new(None)),
                frontend_receiver: Arc::new(TokioRwLock::new(frontend_receiver)),
                focused: false,
                theme: GauntletTheme::new(),
                wayland,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                tray_icon: sys_tray::create_tray(),

                // ephemeral state
                prompt: "".to_string(),

                // state
                global_state,
                client_context,
                search_results: vec![],
                loading_bar_state: HashMap::new(),
                hud_display: None,
            },
            Command::batch(commands),
        )
    }

    fn title(&self, window: window::Id) -> String {
        if window != window::Id::MAIN {
            "Gauntlet".to_owned()
        } else {
            "Gauntlet HUD".to_owned()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::OpenView { plugin_id, plugin_name, entrypoint_id, entrypoint_name } => {
                match &mut self.global_state {
                    GlobalState::MainView { pending_plugin_view_data, .. } => {
                        *pending_plugin_view_data = Some(PluginViewData {
                            top_level_view: true,
                            plugin_id: plugin_id.clone(),
                            plugin_name,
                            entrypoint_id: entrypoint_id.clone(),
                            entrypoint_name,
                            action_shortcuts: HashMap::new(),
                        });

                        Command::batch([
                            self.open_plugin_view(plugin_id, entrypoint_id),
                            Command::perform(async move { AppMsg::PendingPluginViewLoadingBar }, std::convert::identity)
                        ])
                    }
                    GlobalState::ErrorView { .. } => {
                        Command::none()
                    }
                    GlobalState::PluginView { .. } => {
                        Command::none()
                    }
                }
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
                    match &mut self.global_state {
                        GlobalState::MainView { focused_search_result, sub_state, ..} => {
                            new_prompt.truncate(100); // search query uses regex so just to be safe truncate the prompt

                            self.prompt = new_prompt.clone();

                            focused_search_result.reset(true);

                            MainViewState::initial(sub_state);
                        }
                        GlobalState::ErrorView { .. } => {}
                        GlobalState::PluginView { .. } => {}
                    }

                    self.search(new_prompt, true)
                }
            }
            AppMsg::UpdateSearchResults => {
                match &self.global_state {
                    GlobalState::MainView { .. } => {
                        self.search(self.prompt.clone(), false)
                    }
                    _ => Command::none()
                }
            }
            AppMsg::PromptSubmit => {
                self.global_state.primary(&self.search_results)
            },
            AppMsg::SetSearchResults(new_search_results) => {
                self.search_results = new_search_results;

                Command::none()
            }
            AppMsg::ReplaceView { top_level_view, render_location, has_children } => {
                match &mut self.global_state {
                    GlobalState::MainView { pending_plugin_view_data, focused_search_result, pending_plugin_view_loading_bar, .. } => {

                        if let LoadingBarState::Pending = pending_plugin_view_loading_bar {
                            *pending_plugin_view_loading_bar = LoadingBarState::Off;
                        }

                        if has_children {
                            if let UiRenderLocation::InlineView = render_location {
                                focused_search_result.unfocus();
                            }
                        }

                        let command = match pending_plugin_view_data {
                            None => Command::none(),
                            Some(pending_plugin_view_data) => {
                                let pending_plugin_view_data = pending_plugin_view_data.clone();
                                GlobalState::plugin(
                                    &mut self.global_state,
                                    PluginViewData {
                                        top_level_view,
                                        ..pending_plugin_view_data
                                    },
                                    self.client_context.clone()
                                )
                            }
                        };

                        if let UiRenderLocation::InlineView = render_location {
                            Command::batch([
                                command,
                                self.inline_view_shortcuts()
                            ])
                        } else {
                            command
                        }
                    }
                    GlobalState::ErrorView { .. } => Command::none(),
                    GlobalState::PluginView { plugin_view_data, ..} => {
                        plugin_view_data.top_level_view = top_level_view;

                        Command::none()
                    }
                }
            }
            AppMsg::IcedEvent(Event::Keyboard(event)) => {
                match event {
                    keyboard::Event::KeyPressed { key, modifiers, physical_key, text, .. } => {
                        tracing::debug!("Key pressed: {:?}. shift: {:?} control: {:?} alt: {:?} meta: {:?}", key, modifiers.shift(), modifiers.control(), modifiers.alt(), modifiers.logo());
                        match key {
                            Key::Named(Named::ArrowUp) => self.global_state.up(&self.search_results),
                            Key::Named(Named::ArrowDown) => self.global_state.down(&self.search_results),
                            Key::Named(Named::ArrowLeft) => self.global_state.left(&self.search_results),
                            Key::Named(Named::ArrowRight) => self.global_state.right(&self.search_results),
                            Key::Named(Named::Escape) => self.global_state.back(),
                            Key::Named(Named::Tab) if !modifiers.shift() => self.global_state.next(),
                            Key::Named(Named::Tab) if modifiers.shift() => self.global_state.previous(),
                            Key::Named(Named::Enter) => {
                                if modifiers.logo() || modifiers.alt() || modifiers.control() {
                                    Command::none() // to avoid not wanted "enter" presses
                                } else {
                                    if modifiers.shift() {
                                        // for main view, also fired in cases where main text field is not focused
                                        self.global_state.secondary(&self.search_results)
                                    } else {
                                        self.global_state.primary(&self.search_results)
                                    }
                                }
                            },
                            Key::Named(Named::Backspace) => {
                                match &mut self.global_state {
                                    GlobalState::MainView { sub_state, search_field_id, .. } => {
                                        match sub_state {
                                            MainViewState::None => Self::backspace_prompt(&mut self.prompt, search_field_id.clone()),
                                            MainViewState::SearchResultActionPanel { .. } => Command::none(),
                                            MainViewState::InlineViewActionPanel { .. } => Command::none()
                                        }
                                    }
                                    GlobalState::ErrorView { .. } => Command::none(),
                                    GlobalState::PluginView { sub_state, .. } => {
                                        match sub_state {
                                            PluginViewState::None => {
                                                let mut client_context = self.client_context.write().expect("lock is poisoned");

                                                client_context.backspace_text()
                                            }
                                            PluginViewState::ActionPanel { .. } => Command::none()
                                        }
                                    }
                                }
                            },
                            _ => {
                                match &mut self.global_state {
                                    GlobalState::MainView { sub_state, search_field_id, focused_search_result, .. } => {
                                        match sub_state {
                                            MainViewState::None => {
                                                match physical_key_model(physical_key, modifiers) {
                                                    Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                        Command::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                                    }
                                                    Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                        if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                            if let Some(search_item) = focused_search_result.get(&self.search_results) {
                                                                if search_item.entrypoint_actions.len() > 0 {
                                                                    self.handle_main_view_keyboard_event(
                                                                        search_item.plugin_id.clone(),
                                                                        search_item.entrypoint_id.clone(),
                                                                        physical_key,
                                                                        modifier_shift,
                                                                        modifier_control,
                                                                        modifier_alt,
                                                                        modifier_meta
                                                                    )
                                                                } else {
                                                                    Command::none()
                                                                }
                                                            } else {
                                                                self.handle_inline_plugin_view_keyboard_event(
                                                                    physical_key,
                                                                    modifier_shift,
                                                                    modifier_control,
                                                                    modifier_alt,
                                                                    modifier_meta
                                                                )
                                                            }
                                                        } else {
                                                            Self::append_prompt(&mut self.prompt, text, search_field_id.clone(), modifiers)
                                                        }
                                                    }
                                                    _ => Self::append_prompt(&mut self.prompt, text, search_field_id.clone(), modifiers)
                                                }
                                            }
                                            MainViewState::SearchResultActionPanel { .. } => {
                                                match physical_key_model(physical_key, modifiers) {
                                                    Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                        Command::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                                    }
                                                    Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                        if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                            if let Some(search_item) = focused_search_result.get(&self.search_results) {
                                                                if search_item.entrypoint_actions.len() > 0 {
                                                                    self.handle_main_view_keyboard_event(
                                                                        search_item.plugin_id.clone(),
                                                                        search_item.entrypoint_id.clone(),
                                                                        physical_key,
                                                                        modifier_shift,
                                                                        modifier_control,
                                                                        modifier_alt,
                                                                        modifier_meta
                                                                    )
                                                                } else {
                                                                    Command::none()
                                                                }
                                                            } else {
                                                                Command::none()
                                                            }
                                                        } else {
                                                            Command::none()
                                                        }
                                                    }
                                                    _ => Command::none()
                                                }
                                            }
                                            MainViewState::InlineViewActionPanel { .. } => {
                                                match physical_key_model(physical_key, modifiers) {
                                                    Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                        Command::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                                    }
                                                    Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                        if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                            self.handle_inline_plugin_view_keyboard_event(
                                                                physical_key,
                                                                modifier_shift,
                                                                modifier_control,
                                                                modifier_alt,
                                                                modifier_meta
                                                            )
                                                        } else {
                                                            Command::none()
                                                        }
                                                    }
                                                    _ => Command::none()
                                                }
                                            }
                                        }
                                    }
                                    GlobalState::ErrorView { .. } => Command::none(),
                                    GlobalState::PluginView { sub_state, .. } => {
                                        match physical_key_model(physical_key, modifiers) {
                                            Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                Command::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                            }
                                            Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                    self.handle_plugin_view_keyboard_event(physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                                                } else {
                                                    match sub_state {
                                                        PluginViewState::None => {
                                                            match text {
                                                                None => Command::none(),
                                                                Some(text) => {
                                                                    let mut client_context = self.client_context.write().expect("lock is poisoned");

                                                                    client_context.append_text(text.as_str())
                                                                }
                                                            }
                                                        }
                                                        PluginViewState::ActionPanel { .. } => Command::none()
                                                    }
                                                }
                                            }
                                            _ => Command::none()
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
            AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::Noop, .. } => Command::none(),
            AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::PreviousView, .. } => self.global_state.back(),
            AppMsg::WidgetEvent { widget_event, plugin_id, render_location } => {
                self.handle_plugin_event(widget_event, plugin_id, render_location)
            }
            AppMsg::Noop => Command::none(),
            AppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
            AppMsg::ShowWindow => self.show_window(),
            AppMsg::HideWindow => self.hide_window(),
            AppMsg::ShowPreferenceRequiredView {
                plugin_id,
                entrypoint_id,
                plugin_preferences_required,
                entrypoint_preferences_required
            } => {
                GlobalState::error(
                    &mut self.global_state,
                    ErrorViewData::PreferenceRequired {
                        plugin_id,
                        entrypoint_id,
                        plugin_preferences_required,
                        entrypoint_preferences_required,
                    },
                )
            }
            AppMsg::ShowPluginErrorView { plugin_id, entrypoint_id, .. } => {
                GlobalState::error(
                    &mut self.global_state,
                    ErrorViewData::PluginError {
                        plugin_id,
                        entrypoint_id,
                    },
                )
            }
            AppMsg::ShowBackendError(err) => {
                GlobalState::error(
                    &mut self.global_state,
                    match err {
                        BackendForFrontendApiError::TimeoutError => ErrorViewData::BackendTimeout,
                    }
                )
            }
            AppMsg::OpenSettingsPreferences { plugin_id, entrypoint_id, } => {
                self.open_settings_window_preferences(plugin_id, entrypoint_id)
            }
            AppMsg::OnOpenView { action_shortcuts } => {
                match &mut self.global_state {
                    GlobalState::MainView { pending_plugin_view_data, .. } => {
                        match pending_plugin_view_data {
                            None => {}
                            Some(pending_plugin_view_data) => {
                                pending_plugin_view_data.action_shortcuts = action_shortcuts;
                            }
                        };
                    }
                    GlobalState::ErrorView { .. } => { },
                    GlobalState::PluginView { plugin_view_data, ..} => {
                        plugin_view_data.action_shortcuts = action_shortcuts;
                    }
                }

                Command::none()
            }
            AppMsg::RunSearchItemAction(search_result, action_index) => {
                match search_result.entrypoint_type {
                    SearchResultEntrypointType::Command => {
                        match action_index {
                            None => {
                                let msg = AppMsg::RunCommand {
                                    entrypoint_id: search_result.entrypoint_id.clone(),
                                    plugin_id: search_result.plugin_id.clone()
                                };
                                Command::perform(async {}, |_| msg)
                            }
                            Some(_) => Command::none()
                        }
                    },
                    SearchResultEntrypointType::View => {
                        match action_index {
                            None => {
                                let msg = AppMsg::OpenView {
                                    plugin_id: search_result.plugin_id.clone(),
                                    plugin_name: search_result.plugin_name.clone(),
                                    entrypoint_id: search_result.entrypoint_id.clone(),
                                    entrypoint_name: search_result.entrypoint_name.clone(),
                                };
                                Command::perform(async {}, |_| msg)
                            }
                            Some(_) => Command::none()
                        }
                    },
                    SearchResultEntrypointType::GeneratedCommand => {
                        let msg = AppMsg::RunGeneratedCommandEvent {
                            entrypoint_id: search_result.entrypoint_id.clone(),
                            plugin_id: search_result.plugin_id.clone(),
                            action_index,
                        };

                        Command::perform(async {}, |_| msg)
                    },
                }
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
            AppMsg::ToggleActionPanel { keyboard } => {
                match &mut self.global_state {
                    GlobalState::MainView { sub_state, focused_search_result, .. } => {
                        match sub_state {
                            MainViewState::None => {
                                if let Some(search_item) = focused_search_result.get(&self.search_results) {
                                    if search_item.entrypoint_actions.len() > 0 {
                                        MainViewState::search_result_action_panel(sub_state, keyboard);
                                    }
                                } else {
                                    let client_context = self.client_context.read().expect("lock is poisoned");
                                    if let Some(_) = client_context.get_first_inline_view_container() {
                                        MainViewState::inline_result_action_panel(sub_state, keyboard);
                                    }
                                }
                            }
                            MainViewState::SearchResultActionPanel { .. } => {
                                MainViewState::initial(sub_state);
                            }
                            MainViewState::InlineViewActionPanel { .. } => {
                                MainViewState::initial(sub_state);
                            }
                        }
                    }
                    GlobalState::ErrorView { .. } => { },
                    GlobalState::PluginView { sub_state, .. } => {
                        let client_context = self.client_context.read().expect("lock is poisoned");

                        client_context.toggle_action_panel();

                        match sub_state {
                            PluginViewState::None => {
                                PluginViewState::action_panel(sub_state, keyboard)
                            }
                            PluginViewState::ActionPanel { .. } => {
                                PluginViewState::initial(sub_state)
                            }
                        }
                    }
                }

                Command::none()
            }
            AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus => {
                Command::perform(async {}, move |_| AppMsg::OnAnyActionMainViewNoPanelKeyboardAtIndex { index: 0 })
            }
            AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithFocus { search_result } => {
                Command::perform(async {}, |_| AppMsg::RunSearchItemAction(search_result, None))
            }
            AppMsg::OnPrimaryActionMainViewSearchResultPanelKeyboardWithFocus { search_result, widget_id } => {
                let run_action_command = if widget_id == 0 {
                    Command::perform(async {}, |_| AppMsg::RunSearchItemAction(search_result, None))
                } else {
                    Command::perform(async {}, move |_| AppMsg::RunSearchItemAction(search_result, Some(widget_id - 1)))
                };

                Command::batch([
                    run_action_command,
                    Command::perform(async {}, |_| AppMsg::ResetMainViewState)
                ])
            }
            AppMsg::OnPrimaryActionMainViewInlineViewPanelKeyboardWithFocus { widget_id } => {
                let client_context = self.client_context.read().expect("lock is poisoned");

                match client_context.get_first_inline_view_container() {
                    Some(container) => {
                        let plugin_id = container.get_plugin_id();

                        let widget_event = ComponentWidgetEvent::RunAction {
                            widget_id,
                        };
                        let render_location = UiRenderLocation::InlineView;

                        Command::batch([
                            Command::perform(async {}, move |_| AppMsg::ToggleActionPanel { keyboard: true }),
                            Command::perform(async {}, move |_| AppMsg::WidgetEvent { widget_event, plugin_id, render_location })
                        ])
                    }
                    None => Command::none()
                }
            }
            AppMsg::OnPrimaryActionPluginViewNoPanelKeyboardWithFocus { widget_id } => {
                Command::perform(async {}, move |_| AppMsg::OnAnyActionPluginViewAnyPanel { widget_id })
            }
            AppMsg::OnPrimaryActionPluginViewAnyPanelKeyboardWithFocus { widget_id } => {
                let client_context = self.client_context.read().expect("lock is poisoned");

                let plugin_id = client_context.get_view_plugin_id();

                let widget_event = ComponentWidgetEvent::RunAction {
                    widget_id,
                };
                let render_location = UiRenderLocation::View;

                Command::batch([
                    Command::perform(async {}, move |_| AppMsg::ToggleActionPanel { keyboard: true }),
                    Command::perform(async {}, move |_| AppMsg::WidgetEvent { widget_event, plugin_id, render_location })
                ])
            }
            AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id: _ } => {
                // widget_id here is always 0
                match &self.global_state {
                    GlobalState::MainView { focused_search_result, .. } => {
                        if let Some(search_result) = focused_search_result.get(&self.search_results) {
                            let search_result = search_result.clone();
                            Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithFocus { search_result })
                        } else {
                            Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus)
                        }
                    }
                    GlobalState::PluginView { .. } => Command::none(),
                    GlobalState::ErrorView { .. } => Command::none()
                }
            }
            AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus => {
                Command::perform(async {}, move |_| AppMsg::OnAnyActionMainViewNoPanelKeyboardAtIndex { index: 1 })
            }
            AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithFocus { search_result } => {
                Command::perform(async {}, |_| AppMsg::RunSearchItemAction(search_result, Some(0)))
            }
            AppMsg::OnSecondaryActionPluginViewNoPanelKeyboardWithFocus { widget_id } => {
                Command::perform(async {}, move |_| AppMsg::OnAnyActionPluginViewAnyPanel { widget_id })
            }
            AppMsg::OnAnyActionMainViewNoPanelKeyboardAtIndex { index } => {
                let client_context = self.client_context.read().expect("lock is poisoned");

                if let Some(container) = client_context.get_first_inline_view_container() {
                    let plugin_id = container.get_plugin_id();
                    let action_ids = container.get_action_ids();

                    match action_ids.get(index) {
                        Some(widget_id) => {
                            let widget_id = *widget_id;

                            let widget_event = ComponentWidgetEvent::RunAction {
                                widget_id,
                            };
                            let render_location = UiRenderLocation::InlineView;

                            Command::perform(async {}, move |_| AppMsg::WidgetEvent { widget_event, plugin_id, render_location })
                        }
                        None => Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            AppMsg::OnAnyActionMainViewAnyPanelMouse { widget_id } => {
                match &mut self.global_state {
                    GlobalState::MainView { focused_search_result, sub_state, .. } => {
                        match sub_state {
                            MainViewState::None => Command::none(),
                            MainViewState::SearchResultActionPanel { .. } => {
                                if let Some(search_result) = focused_search_result.get(&self.search_results) {
                                    let search_result = search_result.clone();
                                    Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewSearchResultPanelKeyboardWithFocus { search_result, widget_id })
                                } else {
                                    Command::none()
                                }
                            }
                            MainViewState::InlineViewActionPanel { .. } => {
                                Command::perform(async {}, move |_| AppMsg::OnPrimaryActionMainViewInlineViewPanelKeyboardWithFocus { widget_id })
                            }
                        }
                    }
                    GlobalState::ErrorView { .. } => Command::none(),
                    GlobalState::PluginView { .. } => Command::none()
                }
            }
            AppMsg::OnAnyActionPluginViewAnyPanel { widget_id } => {
                let client_context = self.client_context.read().expect("lock is poisoned");

                let plugin_id = client_context.get_view_plugin_id();

                let widget_event = ComponentWidgetEvent::RunAction {
                    widget_id,
                };

                let render_location = UiRenderLocation::View;

                Command::perform(async {}, move |_| AppMsg::WidgetEvent { widget_event, plugin_id, render_location })
            }
            AppMsg::OpenPluginView(plugin_id, entrypoint_id) => {
                self.open_plugin_view(plugin_id, entrypoint_id)
            }
            AppMsg::ClosePluginView(plugin_id) => {
                self.close_plugin_view(plugin_id)
            }
            AppMsg::InlineViewShortcuts { shortcuts } => {
                let mut client_context = self.client_context.write().expect("lock is poisoned");

                client_context.set_inline_view_shortcuts(shortcuts);

                Command::none()
            }
            AppMsg::ShowHud { display } => {
                self.hud_display = Some(display);

                show_hud_window(
                    #[cfg(target_os = "linux")]
                    self.wayland,
                )
            }
            AppMsg::CloseHudWindow { id } => {
                self.hud_display = None;

                close_hud_window(
                    #[cfg(target_os = "linux")]
                    self.wayland,
                    id
                )
            }
            AppMsg::ResetMainViewState => {
                match &mut self.global_state {
                    GlobalState::MainView { sub_state, .. } => {
                        MainViewState::initial(sub_state);

                        Command::none()
                    }
                    GlobalState::ErrorView { .. } => Command::none(),
                    GlobalState::PluginView { .. } => Command::none(),
                }
            }
            AppMsg::SetGlobalShortcut { shortcut, responder } => {
                tracing::info!("Registering new global shortcut: {:?}", shortcut);

                let run = || {
                    let global_hotkey_manager = self.global_hotkey_manager
                        .read()
                        .expect("lock is poisoned");

                    let mut hotkey_guard = self.current_hotkey
                        .lock()
                        .expect("lock is poisoned");

                    if let Some(current_hotkey) = *hotkey_guard {
                        global_hotkey_manager.unregister(current_hotkey)?;
                    }

                    let hotkey = convert_physical_shortcut_to_hotkey(shortcut);
                    *hotkey_guard = Some(hotkey);

                    global_hotkey_manager.register(hotkey)?;

                    Ok(())
                };

                // responder is not clone and send, and we need to consume it
                // so we wrap it in arc mutex option
                let mut responder = responder
                    .lock()
                    .expect("lock is poisoned")
                    .take()
                    .expect("there should always be a responder here");

                match run() {
                    Ok(()) => {
                        responder.respond(UiResponseData::Nothing);
                    }
                    Err(err) => {
                        responder.respond(UiResponseData::Err(err));
                    }
                }

                Command::none()
            }
            AppMsg::UpdateLoadingBar { plugin_id, entrypoint_id, show } => {
                if show {
                    self.loading_bar_state.insert((plugin_id, entrypoint_id), ());
                } else {
                    self.loading_bar_state.remove(&(plugin_id, entrypoint_id));
                }

                Command::none()
            }
            AppMsg::PendingPluginViewLoadingBar => {
                if let GlobalState::MainView { pending_plugin_view_loading_bar, .. } = &mut self.global_state {
                    *pending_plugin_view_loading_bar = LoadingBarState::Pending;
                }

                Command::perform(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                    AppMsg::ShowPluginViewLoadingBar
                }, std::convert::identity)
            }
            AppMsg::ShowPluginViewLoadingBar => {
                if let GlobalState::MainView { pending_plugin_view_loading_bar, .. } = &mut self.global_state {
                    if let LoadingBarState::Pending = pending_plugin_view_loading_bar {
                        *pending_plugin_view_loading_bar = LoadingBarState::On;
                    }
                }

                Command::none()
            }
            AppMsg::FocusPluginViewSearchBar { widget_id } => {
                let mut client_context = self.client_context.write().expect("lock is poisoned");

                client_context.focus_search_bar(widget_id)
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Self::Message> {
        if window != window::Id::MAIN {
            return match &self.hud_display {
                Some(hud_display) => {
                    let hud: Element<_> = text(&hud_display)
                        .shaping(Shaping::Advanced)
                        .into();

                    let hud = container(hud)
                        .center_x()
                        .center_y()
                        .height(Length::Fill)
                        .themed(ContainerStyle::HudInner);

                    let hud = container(hud)
                        .center_x()
                        .center_y()
                        .height(Length::Fill)
                        .themed(ContainerStyle::Hud);

                    let hud = container(hud)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .center_x()
                        .center_y()
                        .style(ContainerStyleInner::Transparent)
                        .into();

                    hud
                }
                None => {
                    horizontal_space()
                        .into()
                }
            }
        }


        match &self.global_state {
            GlobalState::ErrorView { error_view } => {
                match error_view {
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
                            .shaping(Shaping::Advanced)
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
                    ErrorViewData::PluginError { .. } => {
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
                            .shaping(Shaping::Advanced)
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
            GlobalState::MainView { focused_search_result, sub_state, search_field_id, pending_plugin_view_loading_bar, .. } => {
                let input: Element<_> = text_input("Search...", &self.prompt)
                    .on_input(AppMsg::PromptChanged)
                    .on_submit(AppMsg::PromptSubmit)
                    .ignore_with_modifiers(true)
                    .id(search_field_id.clone())
                    .width(Length::Fill)
                    .themed(TextInputStyle::MainSearch);

                let search_list = search_list(
                    &self.search_results,
                    &focused_search_result,
                    |search_result| AppMsg::RunSearchItemAction(search_result, None),
                );

                let search_list = container(search_list)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MainListInner);

                let list: Element<_> = scrollable(search_list)
                    .id(focused_search_result.scrollable_id.clone())
                    .width(Length::Fill)
                    .into();

                let list = container(list)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .themed(ContainerStyle::MainList);

                let input = container(input)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MainSearchBar);

                let separator = if matches!(pending_plugin_view_loading_bar, LoadingBarState::On) || !self.loading_bar_state.is_empty() {
                    LoadingBar::new()
                        .into()
                } else {
                    horizontal_rule(1)
                        .into()
                };

                let client_context = self.client_context.read().expect("lock is poisoned");

                let inline_view = match client_context.get_all_inline_view_containers().first() {
                    Some((plugin_id, container)) => {
                        let plugin_id = plugin_id.clone();
                        container.render_inline_root_widget()
                            .map(move |widget_event| {
                                AppMsg::WidgetEvent {
                                    plugin_id: plugin_id.clone(),
                                    render_location: UiRenderLocation::InlineView,
                                    widget_event,
                                }
                            })
                    }
                    None => {
                        horizontal_space()
                            .into()
                    }
                };

                let content: Element<_> = column(vec![
                    inline_view,
                    list,
                ]).into();

                let (primary_action, action_panel) = if let Some(search_item) = focused_search_result.get(&self.search_results) {
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
                        .map(|(index, action)| {
                            let physical_shortcut = if index == 0 {
                                Some(PhysicalShortcut { // secondary action
                                    physical_key: PhysicalKey::Enter,
                                    modifier_shift: true,
                                    modifier_control: false,
                                    modifier_alt: false,
                                    modifier_meta: false,
                                })
                            } else {
                                action.shortcut.clone()
                            };

                            ActionPanelItem::Action {
                                label: action.label.clone(),
                                widget_id: index + 1,
                                physical_shortcut,
                            }
                        })
                        .collect();

                    let primary_action_widget_id = 0;

                    if actions.len() == 0 {
                        (Some((label, primary_action_widget_id, default_shortcut)), None)
                    } else {
                        let primary_action = ActionPanelItem::Action {
                            label: label.clone(),
                            widget_id: primary_action_widget_id,
                            physical_shortcut: Some(default_shortcut.clone()),
                        };

                        actions.insert(0, primary_action);

                        let action_panel = ActionPanel {
                            title: Some(search_item.entrypoint_name.clone()),
                            items: actions,
                        };

                        (Some((label, primary_action_widget_id, default_shortcut)), Some(action_panel))
                    }
                } else {
                    let client_context = self.client_context.read().expect("lock is poisoned");

                    match client_context.get_first_inline_view_action_panel() {
                        None => (None, None),
                        Some(action_panel) => {
                            match action_panel.find_first() {
                                None => (None, None),
                                Some((label, widget_id)) => {
                                    let shortcut = PhysicalShortcut {
                                        physical_key: PhysicalKey::Enter,
                                        modifier_shift: false,
                                        modifier_control: false,
                                        modifier_alt: false,
                                        modifier_meta: false
                                    };

                                    (Some((label, widget_id, shortcut)), Some(action_panel))
                                }
                            }
                        }
                    }
                };

                let root = match sub_state {
                    MainViewState::None => {
                        render_root(
                            false,
                            input,
                            separator,
                            content,
                            primary_action,
                            action_panel,
                            None::<&ScrollHandle<SearchResultEntrypointAction>>,
                            "",
                            || AppMsg::ToggleActionPanel { keyboard: false },
                            |widget_id| AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id },
                            |widget_id| AppMsg::OnAnyActionMainViewAnyPanelMouse { widget_id },
                            || AppMsg::Noop,
                        )
                    }
                    MainViewState::SearchResultActionPanel { focused_action_item, .. } => {
                        render_root(
                            true,
                            input,
                            separator,
                            content,
                            primary_action,
                            action_panel,
                            Some(focused_action_item),
                            "",
                            || AppMsg::ToggleActionPanel { keyboard: false },
                            |widget_id| AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id },
                            |widget_id| AppMsg::OnAnyActionMainViewAnyPanelMouse { widget_id },
                            || AppMsg::Noop,
                        )
                    }
                    MainViewState::InlineViewActionPanel { focused_action_item, .. } => {
                        render_root(
                            true,
                            input,
                            separator,
                            content,
                            primary_action,
                            action_panel,
                            Some(focused_action_item),
                            "",
                            || AppMsg::ToggleActionPanel { keyboard: false },
                            |widget_id| AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id },
                            |widget_id| AppMsg::OnAnyActionMainViewAnyPanelMouse { widget_id },
                            || AppMsg::Noop,
                        )
                    }
                };

                let root: Element<_> = container(root)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .themed(ContainerStyle::Main);

                root
            }
            GlobalState::PluginView { plugin_view_data, sub_state, ..  } => {
                let PluginViewData { plugin_id, action_shortcuts, .. } = plugin_view_data;

                let client_context = self.client_context.read().expect("lock is poisoned");
                let view_container = client_context.get_view_container();

                let container_element = view_container
                    .render_root_widget(sub_state, action_shortcuts)
                    .map(|widget_event| AppMsg::WidgetEvent {
                        plugin_id: plugin_id.clone(),
                        render_location: UiRenderLocation::View,
                        widget_event,
                    });

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
        struct GlobalShortcutListener;

        let events_subscription = event::listen_with(|event, status| match status {
            event::Status::Ignored => Some(AppMsg::IcedEvent(event)),
            event::Status::Captured => match &event {
                Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(Named::Escape), .. }) => Some(AppMsg::IcedEvent(event)),
                _ => None
            }
        });

        Subscription::batch([
            subscription::channel(
                std::any::TypeId::of::<GlobalShortcutListener>(),
                10,
                |sender| async move {
                    register_listener(sender.clone());

                    std::future::pending::<()>().await;

                    unreachable!()
                },
            ),
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

impl AppModel {
    fn on_focused(&mut self) -> Command<AppMsg> {
        self.focused = true;
        Command::none()
    }

    fn on_unfocused(&mut self) -> Command<AppMsg> {
        // for some reason (on both macOS and linux x11) duplicate Unfocused fires right before Focus event
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

        match &self.global_state {
            GlobalState::PluginView { plugin_view_data: PluginViewData { plugin_id, .. }, .. } => {
                commands.push(self.close_plugin_view(plugin_id.clone()));
            }
            GlobalState::MainView { .. } => {}
            GlobalState::ErrorView { .. } => {}
        }

        Command::batch(commands)
    }

    fn show_window(&mut self) -> Command<AppMsg> {
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
        self.prompt = "".to_string();

        let mut client_context = self.client_context.write().expect("lock is poisoned");

        client_context.clear_all_inline_views();

        let mut commands = vec![
            GlobalState::initial(&mut self.global_state, self.client_context.clone()),
        ];

        if !self.wayland {
            commands.push(
                window::gain_focus(window::Id::MAIN),
            );
        }

        Command::batch(commands)
    }

    fn open_plugin_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(async move {
            let result = backend_client.request_view_render(plugin_id, entrypoint_id)
                .await?;

            Ok(result)
        }, |result| handle_backend_error(result, |action_shortcuts| AppMsg::OnOpenView { action_shortcuts }))
    }

    fn close_plugin_view(&self, plugin_id: PluginId) -> Command<AppMsg> {
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

    fn handle_plugin_event(&self, widget_event: ComponentWidgetEvent, plugin_id: PluginId, render_location: UiRenderLocation) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();
        let client_context = self.client_context.clone();

        Command::perform(async move {
            let event = {
                let client_context = client_context.read().expect("lock is poisoned");
                client_context.handle_event(render_location, &plugin_id, widget_event.clone())
            };

            if let Some(event) = event {
                match event {
                    UiViewEvent::View { widget_id, event_name, event_arguments } => {
                        let msg = match widget_event {
                            ComponentWidgetEvent::ActionClick { .. } => AppMsg::ToggleActionPanel { keyboard: false },
                            _ => AppMsg::Noop
                        };

                        backend_client.send_view_event(plugin_id, widget_id, event_name, event_arguments)
                            .await?;

                        Ok(msg)
                    }
                    UiViewEvent::Open { href } => {
                        backend_client.send_open_event(plugin_id, href)
                            .await?;

                        Ok(AppMsg::Noop)
                    }
                    UiViewEvent::AppEvent { event } => Ok(event)
                }
            } else {
                Ok(AppMsg::Noop)
            }
        }, |result| handle_backend_error(result, |msg| msg))
    }

    fn handle_main_view_keyboard_event(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, physical_key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Command::perform(
            async move {
                backend_client.send_keyboard_event(plugin_id, entrypoint_id, KeyboardEventOrigin::MainView, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                    .await?;

                Ok(())
            },
            |result| handle_backend_error(result, |()| AppMsg::Noop),
        )
    }

    fn handle_plugin_view_keyboard_event(&self, physical_key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        let (plugin_id, entrypoint_id) = {
            let client_context = self.client_context.read().expect("lock is poisoned");
            (client_context.get_view_plugin_id(), client_context.get_view_entrypoint_id())
        };

        Command::perform(
            async move {
                backend_client.send_keyboard_event(plugin_id, entrypoint_id, KeyboardEventOrigin::PluginView, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                    .await?;

                Ok(())
            },
            |result| handle_backend_error(result, |()| AppMsg::Noop),
        )
    }

    fn handle_inline_plugin_view_keyboard_event(&self, physical_key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> Command<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        let (plugin_id, entrypoint_id) = {
            let client_context = self.client_context.read().expect("lock is poisoned");
            match client_context.get_first_inline_view_container() {
                None => {
                    return Command::none()
                },
                Some(container) => (container.get_plugin_id(), container.get_entrypoint_id())
            }
        };

        Command::perform(
            async move {
                backend_client.send_keyboard_event(plugin_id, entrypoint_id, KeyboardEventOrigin::PluginView, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                    .await?;

                Ok(())
            },
            |result| handle_backend_error(result, |()| AppMsg::Noop),
        )
    }

    fn search(&self, new_prompt: String, render_inline_view: bool) -> Command<AppMsg> {
        let mut backend_api = self.backend_api.clone();

        Command::perform(async move {
            let search_results = backend_api.search(new_prompt, render_inline_view)
                .await?;

            Ok(search_results)
        }, |result| handle_backend_error(result, |search_results| AppMsg::SetSearchResults(search_results)))
    }

    fn open_settings_window_preferences(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> Command<AppMsg> {
        let mut backend_api = self.backend_api.clone();

        Command::perform(async move {
            backend_api.open_settings_window_preferences(plugin_id, entrypoint_id)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn inline_view_shortcuts(&self) -> Command<AppMsg> {
        let mut backend_api = self.backend_api.clone();

        Command::perform(async move {
            backend_api.inline_view_shortcuts().await
        }, |result| handle_backend_error(result, |shortcuts| AppMsg::InlineViewShortcuts { shortcuts }))
    }
}

// these are needed to force focus the text_input in main search view when
// the window is opened but text_input not focused
impl AppModel {
    fn append_prompt(prompt: &mut String, value: Option<SmolStr>, search_field_id: text_input::Id, modifiers: Modifiers) -> Command<AppMsg> {
        if modifiers.control() || modifiers.alt() || modifiers.logo() {
            Command::none()
        } else {
            match value {
                Some(value) => {
                    if let Some(value) = value.chars().next().filter(|c| !c.is_control()) {
                        *prompt = format!("{}{}", prompt, value);
                        focus(search_field_id.clone())
                    } else {
                        Command::none()
                    }
                }
                None => Command::none()
            }
        }
    }

    fn backspace_prompt(prompt: &mut String, search_field_id: text_input::Id) -> Command<AppMsg> {
        let mut chars = prompt.chars();
        chars.next_back();
        *prompt = chars.as_str().to_owned();

        focus(search_field_id.clone())
    }
}

fn handle_backend_error<T>(result: Result<T, BackendForFrontendApiError>, convert: impl FnOnce(T) -> AppMsg) -> AppMsg {
    match result {
        Ok(val) => convert(val),
        Err(err) => AppMsg::ShowBackendError(err)
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

        let app_msgs = {
            let mut client_context = client_context.write().expect("lock is poisoned");

            match request_data {
                UiRequestData::ReplaceView {
                    plugin_id,
                    plugin_name,
                    entrypoint_id,
                    entrypoint_name,
                    render_location,
                    top_level_view,
                    container,
                    #[cfg(feature = "scenario_runner")]
                    container_value: _,
                    images
                } => {
                    let has_children = container.content.is_some();

                    let message = client_context.replace_view(
                        render_location,
                        container,
                        images,
                        &plugin_id,
                        &plugin_name,
                        &entrypoint_id,
                        &entrypoint_name
                    );

                    responder.respond(UiResponseData::Nothing);

                    vec![
                        AppMsg::ReplaceView {
                            top_level_view,
                            has_children,
                            render_location,
                        },
                        message
                    ]
                }
                UiRequestData::ClearInlineView { plugin_id } => {
                    client_context.clear_inline_view(&plugin_id);

                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::Noop] // refresh ui
                }
                UiRequestData::ShowWindow => {
                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::ShowWindow]
                }
                UiRequestData::ShowPreferenceRequiredView {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required
                } => {
                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::ShowPreferenceRequiredView {
                        plugin_id,
                        entrypoint_id,
                        plugin_preferences_required,
                        entrypoint_preferences_required
                    }]
                }
                UiRequestData::ShowPluginErrorView { plugin_id, entrypoint_id, render_location } => {
                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::ShowPluginErrorView {
                        plugin_id,
                        entrypoint_id,
                        render_location,
                    }]
                }
                UiRequestData::RequestSearchResultUpdate => {
                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::UpdateSearchResults]
                }
                UiRequestData::ShowHud { display } => {
                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::ShowHud {
                        display
                    }]
                }
                UiRequestData::SetGlobalShortcut { shortcut } => {
                    vec![AppMsg::SetGlobalShortcut {
                        shortcut,
                        responder: Arc::new(Mutex::new(Some(responder)))
                    }]
                }
                UiRequestData::UpdateLoadingBar { plugin_id, entrypoint_id, show } => {
                    responder.respond(UiResponseData::Nothing);

                    vec![AppMsg::UpdateLoadingBar {
                        plugin_id,
                        entrypoint_id,
                        show
                    }]
                }
            }
        };

        for app_msg in app_msgs {
            let _ = sender.send(app_msg).await;
        }
    }
}
