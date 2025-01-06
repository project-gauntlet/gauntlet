use anyhow::anyhow;
use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;
use iced::advanced::graphics::core::SmolStr;
use iced::advanced::layout::Limits;
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::keyboard::key::{Named, Physical};
use iced::keyboard::{Key, Modifiers};
use iced::widget::scrollable::{scroll_to, AbsoluteOffset};
use iced::widget::text::Shaping;
use iced::widget::text_input::focus;
use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input, Space};
use iced::window::{Level, Mode, Position, Screenshot};
use iced::{event, executor, font, futures, keyboard, stream, window, Alignment, Event, Font, Length, Padding, Pixels, Renderer, Settings, Size, Subscription, Task};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex as StdMutex, Mutex, RwLock as StdRwLock};
use iced::alignment::{Horizontal, Vertical};
use iced_fonts::BOOTSTRAP_FONT_BYTES;
use serde::Deserialize;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};

use client_context::ClientContext;
use gauntlet_common::model::{BackendRequestData, BackendResponseData, EntrypointId, UiTheme, KeyboardEventOrigin, PhysicalKey, PhysicalShortcut, PluginId, RootWidget, RootWidgetMembers, SearchResult, SearchResultEntrypointAction, SearchResultEntrypointActionType, SearchResultEntrypointType, UiRenderLocation, UiRequestData, UiResponseData, UiSetupData, UiWidgetId};
use gauntlet_common::rpc::backend_api::{BackendApi, BackendForFrontendApi, BackendForFrontendApiError};
use gauntlet_common::scenario_convert::{ui_render_location_from_scenario};
use gauntlet_common::scenario_model::{ScenarioFrontendEvent, ScenarioUiRenderLocation};
use gauntlet_common_ui::physical_key_model;
use gauntlet_utils::channel::{RequestReceiver, RequestSender, Responder};

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
use crate::ui::hud::show_hud_window;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::{ErrorViewData, Focus, GlobalState, LoadingBarState, MainViewState, PluginViewData, PluginViewState};
use crate::ui::widget_container::PluginWidgetContainer;
pub use theme::GauntletComplexTheme;

pub struct AppModel {
    // logic
    backend_api: BackendForFrontendApi,
    global_hotkey_manager: Arc<StdRwLock<GlobalHotKeyManager>>,
    current_hotkey: Arc<StdMutex<Option<HotKey>>>,
    frontend_receiver: Arc<TokioRwLock<RequestReceiver<UiRequestData, UiResponseData>>>,
    main_window_id: window::Id,
    focused: bool,
    wayland: bool,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    tray_icon: tray_icon::TrayIcon,
    theme: GauntletComplexTheme,

    // ephemeral state
    prompt: String,

    // state
    client_context: ClientContext,
    global_state: GlobalState,
    search_results: Vec<SearchResult>,
    loading_bar_state: HashMap<(PluginId, EntrypointId), ()>,
    hud_display: Option<String>
}

#[cfg(target_os = "linux")]
mod layer_shell {
    #[iced_layershell::to_layer_message(multi)]
    #[derive(Debug, Clone)]
    pub enum LayerShellAppMsg {}
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    OpenView {
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
    },
    OpenGeneratedView {
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        action_index: usize
    },
    RunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    RunGeneratedCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize
    },
    RunSearchItemAction(SearchResult, usize),
    RunPluginAction {
        render_location: UiRenderLocation,
        plugin_id: PluginId,
        widget_id: UiWidgetId
    },
    PromptChanged(String),
    PromptSubmit,
    UpdateSearchResults,
    SetSearchResults(Vec<SearchResult>),
    RenderPluginUI {
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_id: EntrypointId,
        entrypoint_name: String,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: Arc<RootWidget>,
        images: HashMap<UiWidgetId, Vec<u8>>,
    },
    HandleRenderPluginUI {
        top_level_view: bool,
        has_children: bool,
        render_location: UiRenderLocation,
    },
    IcedEvent(window::Id, Event),
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
    Screenshot {
        save_path: String
    },
    ScreenshotDone {
        save_path: String,
        screenshot: Screenshot
    },
    ShowBackendError(BackendForFrontendApiError),
    ClosePluginView(PluginId),
    OpenPluginView(PluginId, EntrypointId),
    InlineViewShortcuts {
        shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>
    },
    ShowHud {
        display: String
    },
    OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus,
    OnPrimaryActionMainViewNoPanel { search_result: SearchResult },
    OnSecondaryActionMainViewNoPanelKeyboardWithFocus { search_result: SearchResult },
    OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus,
    OnAnyActionMainViewSearchResultPanelKeyboardWithFocus { search_result: SearchResult, widget_id: UiWidgetId },
    OnAnyActionMainViewInlineViewPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnAnyActionPluginViewNoPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnAnyActionPluginViewAnyPanelKeyboardWithFocus { widget_id: UiWidgetId },
    OnAnyActionPluginViewAnyPanel { widget_id: UiWidgetId },
    OnAnyActionMainViewSearchResultPanelMouse { widget_id: UiWidgetId },
    OnPrimaryActionMainViewActionPanelMouse { widget_id: UiWidgetId },
    ResetMainViewState,
    OnAnyActionMainViewNoPanelKeyboardAtIndex { index: usize },
    SetGlobalShortcut {
        shortcut: Option<PhysicalShortcut>,
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
    #[cfg(target_os = "linux")]
    LayerShell(layer_shell::LayerShellAppMsg),
    ClearInlineView {
        plugin_id: PluginId,
    },
    SetTheme {
        theme: UiTheme
    },
}

#[cfg(target_os = "linux")]
impl TryInto<iced_layershell::actions::LayershellCustomActionsWithId> for AppMsg {
    type Error = Self;
    fn try_into(self) -> Result<iced_layershell::actions::LayershellCustomActionsWithId, Self::Error> {
        match self {
            Self::LayerShell(msg) => msg.try_into().map_err(|msg| Self::LayerShell(msg)),
            _ => Err(self)
        }
    }
}

const WINDOW_WIDTH: f32 = 750.0;
const WINDOW_HEIGHT: f32 = 450.0;

#[cfg(not(target_os = "macos"))]
fn window_settings(visible: bool) -> window::Settings {
    window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: false,
        visible,
        transparent: true,
        closeable: false,
        minimizable: false,
        ..Default::default()
    }
}

#[cfg(target_os = "macos")]
fn window_settings(visible: bool) -> window::Settings {
    window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: true,
        visible,
        transparent: false,
        closeable: false,
        minimizable: false,
        platform_specific: window::settings::PlatformSpecific {
            window_kind: window::settings::WindowKind::Popup,
            fullsize_content_view: true,
            title_hidden: true,
            titlebar_transparent: true,
            ..Default::default()
        },
        ..Default::default()
    }
}


#[cfg(target_os = "linux")]
fn layer_shell_settings() -> iced_layershell::reexport::NewLayerShellSettings {
    iced_layershell::reexport::NewLayerShellSettings {
        layer: iced_layershell::reexport::Layer::Overlay,
        keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::Exclusive,
        events_transparent: false,
        anchor: iced_layershell::reexport::Anchor::empty(),
        margin: Default::default(),
        exclusive_zone: Some(0),
        size: Some((WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)),
        use_last_output: false,
    }
}

fn open_main_window_non_wayland(minimized: bool) -> (window::Id, Task<AppMsg>) {
    let (main_window_id, open_task) = window::open(window_settings(!minimized));

    (main_window_id, Task::batch([
        open_task.map(|_| AppMsg::Noop),
        window::gain_focus(main_window_id),
        window::change_level(main_window_id, Level::AlwaysOnTop),
    ]))
}

#[cfg(target_os = "linux")]
fn open_main_window_wayland(id: window::Id) -> (window::Id, Task<AppMsg>) {
    let settings = layer_shell_settings();

    (id, Task::done(AppMsg::LayerShell(layer_shell::LayerShellAppMsg::NewLayerShell { id, settings })))
}


pub fn run(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) {
    #[cfg(target_os = "linux")]
    let result = {
        let wayland = std::env::var("WAYLAND_DISPLAY")
            .or_else(|_| std::env::var("WAYLAND_SOCKET"))
            .is_ok();

        if wayland {
            run_wayland(minimized, frontend_receiver, backend_sender)
        } else {
            run_non_wayland(minimized, frontend_receiver, backend_sender)
        }
    };

    #[cfg(not(target_os = "linux"))]
    let result = run_non_wayland(minimized, frontend_receiver, backend_sender);

    result.expect("Unable to start application")
}

fn run_non_wayland(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) -> anyhow::Result<()> {
    iced::daemon::<AppModel, AppMsg, GauntletComplexTheme, Renderer>(title, update, view)
        .settings(Settings {
            #[cfg(target_os = "macos")]
            platform_specific: iced::settings::PlatformSpecific {
                activation_policy: iced::settings::ActivationPolicy::Accessory,
                activate_ignoring_other_apps: true,
            },
            ..Default::default()
        })
        .subscription(subscription)
        .theme(|state, _| state.theme.clone())
        .run_with(move || new(frontend_receiver, backend_sender, false, minimized))?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_wayland(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) -> anyhow::Result<()> {
    iced_layershell::build_pattern::daemon("Gauntlet", update, view, wayland_remove_id_info)
        .layer_settings(iced_layershell::settings::LayerShellSettings {
            start_mode: iced_layershell::settings::StartMode::Background,
            events_transparent: true,
            keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::None,
            size: None,
            ..Default::default()
        })
        .subscription(subscription)
        .theme(|state| state.theme.clone())
        .run_with(move || new(frontend_receiver, backend_sender, true, minimized))?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn wayland_remove_id_info(_state: &mut AppModel, _id: window::Id) {
}

fn new(
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
    wayland: bool,
    minimized: bool,
) -> (AppModel, Task<AppMsg>) {
    let mut backend_api = BackendForFrontendApi::new(backend_sender);

    let setup_data = futures::executor::block_on(backend_api.setup_data())
        .expect("Unable to setup frontend");

    let theme = GauntletComplexTheme::new(setup_data.theme);

    GauntletComplexTheme::set_global(theme.clone());

    let current_hotkey = Arc::new(StdMutex::new(None));

    let global_hotkey_manager = GlobalHotKeyManager::new()
        .expect("unable to create global hot key manager");

    let assignment_result = assign_global_shortcut(&global_hotkey_manager, &current_hotkey, setup_data.global_shortcut);

    futures::executor::block_on(backend_api.setup_response(assignment_result.map_err(|err| format!("{:#}", err)).err()))
        .expect("Unable to setup frontend");

    let mut tasks = vec![
        font::load(BOOTSTRAP_FONT_BYTES).map(AppMsg::FontLoaded),
    ];

    #[cfg(target_os = "linux")]
    let (main_window_id, open_task) =  if wayland {
        let id = window::Id::unique();

        if minimized {
            (id, Task::none())
        } else {
            open_main_window_wayland(id)
        }
    } else {
        open_main_window_non_wayland(minimized)
    };

    #[cfg(not(target_os = "linux"))]
    let (main_window_id, open_task) = open_main_window_non_wayland(minimized);

    tasks.push(open_task);

    let global_state = if cfg!(feature = "scenario_runner") {
        let gen_in = std::env::var("GAUNTLET_SCREENSHOT_GEN_IN")
            .expect("Unable to read GAUNTLET_SCREENSHOT_GEN_IN");

       println!("Reading scenario file at: {}", gen_in);

        let gen_in = fs::read_to_string(gen_in)
            .expect("Unable to read file at GAUNTLET_SCREENSHOT_GEN_IN");

        let gen_out = std::env::var("GAUNTLET_SCREENSHOT_GEN_OUT")
            .expect("Unable to read GAUNTLET_SCREENSHOT_GEN_OUT");

        let gen_name = std::env::var("GAUNTLET_SCREENSHOT_GEN_NAME")
            .expect("Unable to read GAUNTLET_SCREENSHOT_GEN_NAME");

        let event: ScenarioFrontendEvent = serde_json::from_str(&gen_in)
            .expect("GAUNTLET_SCREENSHOT_GEN_IN is not valid json");

        tasks.push(
            Task::perform(
                async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                },
                move |_| AppMsg::Screenshot { save_path: gen_out.clone() },
            )
        );

        match event {
            ScenarioFrontendEvent::ReplaceView { entrypoint_id, render_location, top_level_view, container, images } => {
                let plugin_id = PluginId::from_string("__SCREENSHOT_GEN___");
                let entrypoint_id = EntrypointId::from_string(entrypoint_id);

                let render_location = ui_render_location_from_scenario(render_location);

                let msg = AppMsg::RenderPluginUI {
                    plugin_id: plugin_id.clone(),
                    plugin_name: "Screenshot Plugin".to_string(),
                    entrypoint_id: entrypoint_id.clone(),
                    entrypoint_name: "Screenshot Entrypoint".to_string(),
                    render_location,
                    top_level_view,
                    container: Arc::new(container),
                    images
                };

                tasks.push(Task::done(msg));

                match render_location {
                    UiRenderLocation::InlineView => GlobalState::new(text_input::Id::unique()),
                    UiRenderLocation::View => GlobalState::new_plugin(
                        PluginViewData {
                            top_level_view,
                            plugin_id,
                            plugin_name: "Screenshot Gen".to_string(),
                            entrypoint_id,
                            entrypoint_name: gen_name,
                            action_shortcuts: Default::default(),
                        },
                    )
                }
            }
            ScenarioFrontendEvent::ShowPreferenceRequiredView { entrypoint_id, plugin_preferences_required, entrypoint_preferences_required } => {
                let error_view = ErrorViewData::PreferenceRequired {
                    plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                    entrypoint_id: EntrypointId::from_string(entrypoint_id),
                    plugin_preferences_required,
                    entrypoint_preferences_required,
                };

                GlobalState::new_error(error_view)
            }
            ScenarioFrontendEvent::ShowPluginErrorView { entrypoint_id, render_location: _ } => {
                let error_view = ErrorViewData::PluginError {
                    plugin_id: PluginId::from_string("__SCREENSHOT_GEN___"),
                    entrypoint_id: EntrypointId::from_string(entrypoint_id),
                };

                GlobalState::new_error(error_view)
            }
        }
    } else {
        GlobalState::new(text_input::Id::unique())
    };

    (
        AppModel {
            // logic
            backend_api,
            global_hotkey_manager: Arc::new(StdRwLock::new(global_hotkey_manager)),
            current_hotkey,
            frontend_receiver: Arc::new(TokioRwLock::new(frontend_receiver)),
            main_window_id,
            focused: false,
            wayland,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            tray_icon: sys_tray::create_tray(),
            theme,

            // ephemeral state
            prompt: "".to_string(),

            // state
            global_state,
            client_context: ClientContext::new(),
            search_results: vec![],
            loading_bar_state: HashMap::new(),
            hud_display: None,
        },
        Task::batch(tasks),
    )
}

fn title(state: &AppModel, window: window::Id) -> String {
    if window == state.main_window_id {
        "Gauntlet".to_owned()
    } else {
        "Gauntlet HUD".to_owned()
    }
}

fn update(state: &mut AppModel, message: AppMsg) -> Task<AppMsg> {
    match message {
        AppMsg::OpenView { plugin_id, plugin_name, entrypoint_id, entrypoint_name } => {
            match &mut state.global_state {
                GlobalState::MainView { pending_plugin_view_data, .. } => {
                    *pending_plugin_view_data = Some(PluginViewData {
                        top_level_view: true,
                        plugin_id: plugin_id.clone(),
                        plugin_name,
                        entrypoint_id: entrypoint_id.clone(),
                        entrypoint_name,
                        action_shortcuts: HashMap::new(),
                    });

                    Task::batch([
                        state.open_plugin_view(plugin_id, entrypoint_id),
                        Task::done(AppMsg::PendingPluginViewLoadingBar)
                    ])
                }
                GlobalState::ErrorView { .. } => {
                    Task::none()
                }
                GlobalState::PluginView { .. } => {
                    Task::none()
                }
            }
        }
        AppMsg::OpenGeneratedView { plugin_id, plugin_name, entrypoint_id, entrypoint_name, action_index } => {
            match &mut state.global_state {
                GlobalState::MainView { pending_plugin_view_data, .. } => {
                    *pending_plugin_view_data = Some(PluginViewData {
                        top_level_view: true,
                        plugin_id: plugin_id.clone(),
                        plugin_name,
                        entrypoint_id: entrypoint_id.clone(),
                        entrypoint_name,
                        action_shortcuts: HashMap::new(),
                    });

                    Task::batch([
                        state.run_generated_command(plugin_id, entrypoint_id, action_index),
                        Task::done(AppMsg::PendingPluginViewLoadingBar)
                    ])
                }
                GlobalState::ErrorView { .. } => {
                    Task::none()
                }
                GlobalState::PluginView { .. } => {
                    Task::none()
                }
            }
        }
        AppMsg::RunCommand { plugin_id, entrypoint_id } => {
            Task::batch([
                state.hide_window(),
                state.run_command(plugin_id, entrypoint_id),
            ])
        }
        AppMsg::RunGeneratedCommand { plugin_id, entrypoint_id, action_index } => {
            Task::batch([
                state.hide_window(),
                state.run_generated_command(plugin_id, entrypoint_id, action_index),
            ])
        }
        AppMsg::RunPluginAction { render_location, plugin_id, widget_id } => {
            let widget_event = ComponentWidgetEvent::RunAction {
                widget_id,
            };

            Task::batch([
                state.hide_window(),
                Task::done(AppMsg::WidgetEvent { widget_event, plugin_id, render_location })
            ])
        }
        AppMsg::RunSearchItemAction(search_result, action_index) => {
            match search_result.entrypoint_type {
                SearchResultEntrypointType::Command => {
                    if action_index == 0 {
                        Task::done(AppMsg::RunCommand {
                            entrypoint_id: search_result.entrypoint_id.clone(),
                            plugin_id: search_result.plugin_id.clone(),
                        })
                    } else {
                        Task::none()
                    }
                },
                SearchResultEntrypointType::View => {
                    if action_index == 0 {
                        Task::done(AppMsg::OpenView {
                            plugin_id: search_result.plugin_id.clone(),
                            plugin_name: search_result.plugin_name.clone(),
                            entrypoint_id: search_result.entrypoint_id.clone(),
                            entrypoint_name: search_result.entrypoint_name.clone(),
                        })
                    } else {
                        Task::none()
                    }
                },
                SearchResultEntrypointType::Generated => {
                    let action = &search_result.entrypoint_actions[action_index];
                    match &action.action_type {
                        SearchResultEntrypointActionType::Command => {
                            Task::done(AppMsg::RunGeneratedCommand {
                                entrypoint_id: search_result.entrypoint_id.clone(),
                                plugin_id: search_result.plugin_id.clone(),
                                action_index,
                            })
                        }
                        SearchResultEntrypointActionType::View => {
                            Task::done(AppMsg::OpenGeneratedView {
                                plugin_id: search_result.plugin_id.clone(),
                                plugin_name: search_result.plugin_name.clone(),
                                entrypoint_id: search_result.entrypoint_id.clone(),
                                entrypoint_name: action.label.clone(),
                                action_index,
                            })
                        }
                    }
                },
            }
        }
        AppMsg::PromptChanged(mut new_prompt) => {
            if cfg!(feature = "scenario_runner") {
                Task::none()
            } else {
                match &mut state.global_state {
                    GlobalState::MainView { focused_search_result, sub_state, ..} => {
                        new_prompt.truncate(100); // search query uses regex so just to be safe truncate the prompt

                        state.prompt = new_prompt.clone();

                        focused_search_result.reset(true);

                        MainViewState::initial(sub_state);
                    }
                    GlobalState::ErrorView { .. } => {}
                    GlobalState::PluginView { .. } => {}
                }

                state.search(new_prompt, true)
            }
        }
        AppMsg::UpdateSearchResults => {
            match &state.global_state {
                GlobalState::MainView { .. } => {
                    state.search(state.prompt.clone(), false)
                }
                _ => Task::none()
            }
        }
        AppMsg::PromptSubmit => {
            state.global_state.primary(&state.client_context, &state.search_results)
        },
        AppMsg::SetSearchResults(new_search_results) => {
            state.search_results = new_search_results;

            Task::none()
        }
        AppMsg::RenderPluginUI {
            plugin_id,
            plugin_name,
            entrypoint_id,
            entrypoint_name,
            render_location,
            top_level_view,
            container,
            images
        } => {
            let has_children = container.content.is_some();

            Task::batch([
                Task::done(state.client_context.render_ui(
                    render_location,
                    container,
                    images,
                    &plugin_id,
                    &plugin_name,
                    &entrypoint_id,
                    &entrypoint_name,
                )),
                Task::done(AppMsg::HandleRenderPluginUI {
                    top_level_view,
                    has_children,
                    render_location,
                })
            ])
        }
        AppMsg::HandleRenderPluginUI {
            top_level_view,
            has_children,
            render_location
        } => {
            match &mut state.global_state {
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
                        None => Task::none(),
                        Some(pending_plugin_view_data) => {
                            let pending_plugin_view_data = pending_plugin_view_data.clone();
                            GlobalState::plugin(
                                &mut state.global_state,
                                PluginViewData {
                                    top_level_view,
                                    ..pending_plugin_view_data
                                },
                            )
                        }
                    };

                    if let UiRenderLocation::InlineView = render_location {
                        Task::batch([
                            command,
                            state.inline_view_shortcuts()
                        ])
                    } else {
                        command
                    }
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { plugin_view_data, ..} => {
                    plugin_view_data.top_level_view = top_level_view;

                    Task::none()
                }
            }
        }
        AppMsg::IcedEvent(window_id, Event::Keyboard(event)) => {
            if window_id != state.main_window_id {
                return Task::none()
            }

            match event {
                keyboard::Event::KeyPressed { key, modifiers, physical_key, text, .. } => {
                    tracing::debug!("Key pressed: {:?}. shift: {:?} control: {:?} alt: {:?} meta: {:?}", key, modifiers.shift(), modifiers.control(), modifiers.alt(), modifiers.logo());
                    match key {
                        Key::Named(Named::ArrowUp) => state.global_state.up(&state.client_context, &state.search_results),
                        Key::Named(Named::ArrowDown) => state.global_state.down(&state.client_context, &state.search_results),
                        Key::Named(Named::ArrowLeft) => state.global_state.left(&state.client_context, &state.search_results),
                        Key::Named(Named::ArrowRight) => state.global_state.right(&state.client_context, &state.search_results),
                        Key::Named(Named::Escape) => state.global_state.back(&state.client_context),
                        Key::Named(Named::Tab) if !modifiers.shift() => state.global_state.next(&state.client_context),
                        Key::Named(Named::Tab) if modifiers.shift() => state.global_state.previous(&state.client_context),
                        Key::Named(Named::Enter) => {
                            if modifiers.logo() || modifiers.alt() || modifiers.control() {
                                Task::none() // to avoid not wanted "enter" presses
                            } else {
                                if modifiers.shift() {
                                    // for main view, also fired in cases where main text field is not focused
                                    state.global_state.secondary(&state.client_context, &state.search_results)
                                } else {
                                    state.global_state.primary(&state.client_context, &state.search_results)
                                }
                            }
                        },
                        Key::Named(Named::Backspace) => {
                            match &mut state.global_state {
                                GlobalState::MainView { sub_state, search_field_id, .. } => {
                                    match sub_state {
                                        MainViewState::None => AppModel::backspace_prompt(&mut state.prompt, search_field_id.clone()),
                                        MainViewState::SearchResultActionPanel { .. } => Task::none(),
                                        MainViewState::InlineViewActionPanel { .. } => Task::none()
                                    }
                                }
                                GlobalState::ErrorView { .. } => Task::none(),
                                GlobalState::PluginView { sub_state, .. } => {
                                    match sub_state {
                                        PluginViewState::None => {
                                            state.client_context.backspace_text()
                                        }
                                        PluginViewState::ActionPanel { .. } => Task::none()
                                    }
                                }
                            }
                        },
                        _ => {
                            let Physical::Code(physical_key) = physical_key else {
                                return Task::none()
                            };

                            match &mut state.global_state {
                                GlobalState::MainView { sub_state, search_field_id, focused_search_result, .. } => {
                                    match sub_state {
                                        MainViewState::None => {
                                            match physical_key_model(physical_key, modifiers) {
                                                Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                    Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                                }
                                                Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                    if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                        if let Some(search_item) = focused_search_result.get(&state.search_results) {
                                                            if search_item.entrypoint_actions.len() > 0 {
                                                                state.handle_main_view_keyboard_event(
                                                                    search_item.plugin_id.clone(),
                                                                    search_item.entrypoint_id.clone(),
                                                                    physical_key,
                                                                    modifier_shift,
                                                                    modifier_control,
                                                                    modifier_alt,
                                                                    modifier_meta
                                                                )
                                                            } else {
                                                                Task::none()
                                                            }
                                                        } else {
                                                            state.handle_inline_plugin_view_keyboard_event(
                                                                physical_key,
                                                                modifier_shift,
                                                                modifier_control,
                                                                modifier_alt,
                                                                modifier_meta
                                                            )
                                                        }
                                                    } else {
                                                        AppModel::append_prompt(&mut state.prompt, text, search_field_id.clone(), modifiers)
                                                    }
                                                }
                                                _ => AppModel::append_prompt(&mut state.prompt, text, search_field_id.clone(), modifiers)
                                            }
                                        }
                                        MainViewState::SearchResultActionPanel { .. } => {
                                            match physical_key_model(physical_key, modifiers) {
                                                Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                    Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                                }
                                                Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                    if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                        if let Some(search_item) = focused_search_result.get(&state.search_results) {
                                                            if search_item.entrypoint_actions.len() > 0 {
                                                                state.handle_main_view_keyboard_event(
                                                                    search_item.plugin_id.clone(),
                                                                    search_item.entrypoint_id.clone(),
                                                                    physical_key,
                                                                    modifier_shift,
                                                                    modifier_control,
                                                                    modifier_alt,
                                                                    modifier_meta
                                                                )
                                                            } else {
                                                                Task::none()
                                                            }
                                                        } else {
                                                            Task::none()
                                                        }
                                                    } else {
                                                        Task::none()
                                                    }
                                                }
                                                _ => Task::none()
                                            }
                                        }
                                        MainViewState::InlineViewActionPanel { .. } => {
                                            match physical_key_model(physical_key, modifiers) {
                                                Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                                    Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                                }
                                                Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                                    if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                        state.handle_inline_plugin_view_keyboard_event(
                                                            physical_key,
                                                            modifier_shift,
                                                            modifier_control,
                                                            modifier_alt,
                                                            modifier_meta
                                                        )
                                                    } else {
                                                        Task::none()
                                                    }
                                                }
                                                _ => Task::none()
                                            }
                                        }
                                    }
                                }
                                GlobalState::ErrorView { .. } => Task::none(),
                                GlobalState::PluginView { sub_state, .. } => {
                                    match physical_key_model(physical_key, modifiers) {
                                        Some(PhysicalShortcut { physical_key: PhysicalKey::KeyK, modifier_shift: false, modifier_control: false, modifier_alt: true, modifier_meta: false }) => {
                                            Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true })
                                        }
                                        Some(PhysicalShortcut { physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta }) => {
                                            if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                                state.handle_plugin_view_keyboard_event(physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                                            } else {
                                                match sub_state {
                                                    PluginViewState::None => {
                                                        match text {
                                                            None => Task::none(),
                                                            Some(text) => {
                                                                state.client_context.append_text(text.as_str())
                                                            }
                                                        }
                                                    }
                                                    PluginViewState::ActionPanel { .. } => Task::none()
                                                }
                                            }
                                        }
                                        _ => Task::none()
                                    }
                                }
                            }
                        }
                    }
                }
                _ => Task::none()
            }
        }
        AppMsg::IcedEvent(window_id, Event::Window(window::Event::Focused)) => {
            if window_id != state.main_window_id {
                return Task::none()
            }

            state.on_focused()
        }
        AppMsg::IcedEvent(window_id, Event::Window(window::Event::Unfocused)) => {
            if window_id != state.main_window_id {
                return Task::none()
            }

            if state.wayland {
                state.hide_window()
            } else {
                state.on_unfocused()
            }
        }
        AppMsg::IcedEvent(_, _) => Task::none(),
        AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::Noop, .. } => Task::none(),
        AppMsg::WidgetEvent { widget_event: ComponentWidgetEvent::PreviousView, .. } => state.global_state.back(&state.client_context),
        AppMsg::WidgetEvent { widget_event, plugin_id, render_location } => {
            state.handle_plugin_event(widget_event, plugin_id, render_location)
        }
        AppMsg::Noop => Task::none(),
        AppMsg::FontLoaded(result) => {
            result.expect("unable to load font");
            Task::none()
        }
        AppMsg::ShowWindow => state.show_window(),
        AppMsg::HideWindow => state.hide_window(),
        AppMsg::ShowPreferenceRequiredView {
            plugin_id,
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required
        } => {
            GlobalState::error(
                &mut state.global_state,
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
                &mut state.global_state,
                ErrorViewData::PluginError {
                    plugin_id,
                    entrypoint_id,
                },
            )
        }
        AppMsg::ShowBackendError(err) => {
            GlobalState::error(
                &mut state.global_state,
                match err {
                    BackendForFrontendApiError::TimeoutError => ErrorViewData::BackendTimeout,
                    BackendForFrontendApiError::Internal { display } => ErrorViewData::UnknownError { display }
                }
            )
        }
        AppMsg::OpenSettingsPreferences { plugin_id, entrypoint_id, } => {
            state.open_settings_window_preferences(plugin_id, entrypoint_id)
        }
        AppMsg::OnOpenView { action_shortcuts } => {
            match &mut state.global_state {
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

            Task::none()
        }
        AppMsg::Screenshot { save_path } => {
            println!("Creating screenshot at: {}", save_path);

            fs::create_dir_all(Path::new(&save_path).parent().expect("no parent?"))
                .expect("unable to create scenario out directories");

            window::screenshot(state.main_window_id)
                .map(move |screenshot| AppMsg::ScreenshotDone {
                    save_path: save_path.clone(),
                    screenshot,
                })
        }
        AppMsg::ScreenshotDone { save_path, screenshot } => {
            println!("Saving screenshot at: {}", save_path);

            Task::perform(
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
                |_| (),
            ).then(|_| iced::exit())
        }
        AppMsg::ToggleActionPanel { keyboard } => {
            match &mut state.global_state {
                GlobalState::MainView { sub_state, focused_search_result, .. } => {
                    match sub_state {
                        MainViewState::None => {
                            if let Some(search_item) = focused_search_result.get(&state.search_results) {
                                if search_item.entrypoint_actions.len() > 0 {
                                    MainViewState::search_result_action_panel(sub_state, keyboard);
                                }
                            } else {
                                if let Some(_) = state.client_context.get_first_inline_view_container() {
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
                    state.client_context.toggle_action_panel();

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

            Task::none()
        }
        AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus => {
            Task::done(AppMsg::OnAnyActionMainViewNoPanelKeyboardAtIndex { index: 0 })
        }
        AppMsg::OnPrimaryActionMainViewNoPanel { search_result } => {
            Task::done(AppMsg::RunSearchItemAction(search_result, 0))
        }
        AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus => {
            Task::done(AppMsg::OnAnyActionMainViewNoPanelKeyboardAtIndex { index: 1 })
        }
        AppMsg::OnSecondaryActionMainViewNoPanelKeyboardWithFocus { search_result } => {
            Task::done(AppMsg::RunSearchItemAction(search_result, 1))
        }
        AppMsg::OnAnyActionMainViewSearchResultPanelKeyboardWithFocus { search_result, widget_id } => {
            let index = widget_id;

            Task::batch([
                Task::done(AppMsg::RunSearchItemAction(search_result, index)),
                Task::done(AppMsg::ResetMainViewState)
            ])
        }
        AppMsg::OnAnyActionMainViewInlineViewPanelKeyboardWithFocus { widget_id } => {
            match state.client_context.get_first_inline_view_container() {
                Some(container) => {
                    let plugin_id = container.get_plugin_id();

                    Task::batch([
                        Task::done(AppMsg::ToggleActionPanel { keyboard: true }),
                        Task::done(AppMsg::RunPluginAction {
                            render_location: UiRenderLocation::InlineView,
                            plugin_id,
                            widget_id,
                        })
                    ])
                }
                None => Task::none()
            }
        }
        AppMsg::OnAnyActionPluginViewNoPanelKeyboardWithFocus { widget_id } => {
            Task::done(AppMsg::OnAnyActionPluginViewAnyPanel { widget_id })
        }
        AppMsg::OnAnyActionPluginViewAnyPanelKeyboardWithFocus { widget_id } => {
            Task::batch([
                Task::done(AppMsg::ToggleActionPanel { keyboard: true }),
                Task::done(AppMsg::RunPluginAction {
                    render_location: UiRenderLocation::View,
                    plugin_id: state.client_context.get_view_plugin_id(),
                    widget_id,
                })
            ])
        }
        AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id: _ } => {
            // widget_id here is always 0
            match &state.global_state {
                GlobalState::MainView { focused_search_result, .. } => {
                    if let Some(search_result) = focused_search_result.get(&state.search_results) {
                        let search_result = search_result.clone();
                        Task::done(AppMsg::OnPrimaryActionMainViewNoPanel { search_result })
                    } else {
                        Task::done(AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus)
                    }
                }
                GlobalState::PluginView { .. } => Task::none(),
                GlobalState::ErrorView { .. } => Task::none()
            }
        }
        AppMsg::OnAnyActionMainViewNoPanelKeyboardAtIndex { index } => {
            if let Some(container) = state.client_context.get_first_inline_view_container() {
                let plugin_id = container.get_plugin_id();
                let action_ids = container.get_action_ids();

                match action_ids.get(index) {
                    Some(widget_id) => {
                        let widget_id = *widget_id;

                        Task::done(AppMsg::RunPluginAction {
                            render_location: UiRenderLocation::InlineView,
                            plugin_id,
                            widget_id,
                        })
                    }
                    None => Task::none()
                }
            } else {
                Task::none()
            }
        }
        AppMsg::OnAnyActionMainViewSearchResultPanelMouse { widget_id } => {
            match &mut state.global_state {
                GlobalState::MainView { focused_search_result, sub_state, .. } => {
                    match sub_state {
                        MainViewState::None => Task::none(),
                        MainViewState::SearchResultActionPanel { .. } => {
                            if let Some(search_result) = focused_search_result.get(&state.search_results) {
                                let search_result = search_result.clone();
                                Task::done(AppMsg::OnAnyActionMainViewSearchResultPanelKeyboardWithFocus { search_result, widget_id })
                            } else {
                                Task::none()
                            }
                        }
                        MainViewState::InlineViewActionPanel { .. } => {
                            Task::none()
                        }
                    }
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { .. } => Task::none()
            }
        }
        AppMsg::OnAnyActionPluginViewAnyPanel { widget_id } => {
            Task::done(AppMsg::RunPluginAction {
                render_location: UiRenderLocation::View,
                plugin_id: state.client_context.get_view_plugin_id(),
                widget_id,
            })
        }
        AppMsg::OpenPluginView(plugin_id, entrypoint_id) => {
            state.open_plugin_view(plugin_id, entrypoint_id)
        }
        AppMsg::ClosePluginView(plugin_id) => {
            state.close_plugin_view(plugin_id)
        }
        AppMsg::InlineViewShortcuts { shortcuts } => {
            state.client_context.set_inline_view_shortcuts(shortcuts);

            Task::none()
        }
        AppMsg::ShowHud { display } => {
            state.hud_display = Some(display);

            show_hud_window(
                #[cfg(target_os = "linux")]
                state.wayland,
            )
        }
        AppMsg::ResetMainViewState => {
            match &mut state.global_state {
                GlobalState::MainView { sub_state, .. } => {
                    MainViewState::initial(sub_state);

                    Task::none()
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { .. } => Task::none(),
            }
        }
        AppMsg::SetGlobalShortcut { shortcut, responder } => {
            tracing::info!("Registering new global shortcut: {:?}", shortcut);

            let run = || {
                let global_hotkey_manager = state.global_hotkey_manager
                    .read()
                    .expect("lock is poisoned");

                assign_global_shortcut(
                    &global_hotkey_manager,
                    &state.current_hotkey,
                    shortcut
                )
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

            Task::none()
        }
        AppMsg::UpdateLoadingBar { plugin_id, entrypoint_id, show } => {
            if show {
                state.loading_bar_state.insert((plugin_id, entrypoint_id), ());
            } else {
                state.loading_bar_state.remove(&(plugin_id, entrypoint_id));
            }

            Task::none()
        }
        AppMsg::PendingPluginViewLoadingBar => {
            if let GlobalState::MainView { pending_plugin_view_loading_bar, .. } = &mut state.global_state {
                *pending_plugin_view_loading_bar = LoadingBarState::Pending;
            }

            Task::perform(async move {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                AppMsg::ShowPluginViewLoadingBar
            }, std::convert::identity)
        }
        AppMsg::ShowPluginViewLoadingBar => {
            if let GlobalState::MainView { pending_plugin_view_loading_bar, .. } = &mut state.global_state {
                if let LoadingBarState::Pending = pending_plugin_view_loading_bar {
                    *pending_plugin_view_loading_bar = LoadingBarState::On;
                }
            }

            Task::none()
        }
        AppMsg::FocusPluginViewSearchBar { widget_id } => {
            state.client_context.focus_search_bar(widget_id)
        }
        #[cfg(target_os = "linux")]
        AppMsg::LayerShell(_) => {
            // handled by library
            Task::none()
        }
        AppMsg::ClearInlineView { plugin_id } => {
            state.client_context.clear_inline_view(&plugin_id);

            Task::none()
        }
        AppMsg::SetTheme { theme } => {
            state.theme = GauntletComplexTheme::new(theme);

            GauntletComplexTheme::update_global(state.theme.clone());

            Task::none()
        }
    }
}

fn view(state: &AppModel, window: window::Id) -> Element<'_, AppMsg> {
    if window != state.main_window_id {
        view_hud(state)
    } else {
        view_main(state)
    }
}

fn view_hud(state: &AppModel) -> Element<'_, AppMsg> {
    match &state.hud_display {
        Some(hud_display) => {
            let hud: Element<_> = text(hud_display.to_string())
                .shaping(Shaping::Advanced)
                .into();

            let hud = container(hud)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .height(Length::Fill)
                .themed(ContainerStyle::HudInner);

            let hud = container(hud)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .height(Length::Fill)
                .themed(ContainerStyle::Hud);

            let hud = container(hud)
                .height(Length::Fill)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .class(ContainerStyleInner::Transparent)
                .into();

            hud
        }
        None => {
            // this should never be shown, but in case it does, do not make it fully transparent
            container(horizontal_space())
                .themed(ContainerStyle::Hud)
        }
    }
}

fn view_main(state: &AppModel) -> Element<'_, AppMsg> {
    match &state.global_state {
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
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PreferenceRequiredViewDescription);

                    let button_label: Element<_> = text("Open Settings")
                        .into();

                    let button: Element<_> = button(button_label)
                        .on_press(msg)
                        .into();

                    let button = container(button)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .into();

                    let content: Element<_> = column([
                        description,
                        button
                    ]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
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
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Please report this to plugin author")
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let button_label: Element<_> = text("Close")
                        .into();

                    let button: Element<_> = button(button_label)
                        .on_press(AppMsg::HideWindow)
                        .into();

                    let button = container(button)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .into();

                    let content: Element<_> = column([
                        description,
                        sub_description,
                        button
                    ]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
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
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Please report") // TODO link
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
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
                        .align_x(Horizontal::Center)
                        .into();

                    let content: Element<_> = column([
                        description,
                        sub_description,
                        error_description,
                        button
                    ]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
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
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Backend was unable to process message in a timely manner")
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let button_label: Element<_> = text("Close")
                        .into();

                    let button: Element<_> = button(button_label)
                        .on_press(AppMsg::HideWindow)
                        .into();

                    let button = container(button)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .into();

                    let content: Element<_> = column([
                        description,
                        sub_description,
                        button
                    ]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .themed(ContainerStyle::Main);

                    content
                }
            }
        }
        GlobalState::MainView { focused_search_result, sub_state, search_field_id, pending_plugin_view_loading_bar, .. } => {
            let input: Element<_> = text_input("Search...", &state.prompt)
                .on_input(AppMsg::PromptChanged)
                .on_submit(AppMsg::PromptSubmit)
                .ignore_with_modifiers(true)
                .id(search_field_id.clone())
                .width(Length::Fill)
                .themed(TextInputStyle::MainSearch);

            let search_list = search_list(&state.search_results, &focused_search_result)
                .map(|search_result| AppMsg::OnPrimaryActionMainViewNoPanel { search_result });

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

            let separator = if matches!(pending_plugin_view_loading_bar, LoadingBarState::On) || !state.loading_bar_state.is_empty() {
                LoadingBar::new()
                    .into()
            } else {
                horizontal_rule(1)
                    .into()
            };

            let inline_view = match state.client_context.get_all_inline_view_containers().first() {
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

            let (primary_action, action_panel) = if let Some(search_item) = focused_search_result.get(&state.search_results) {
                let primary_shortcut = PhysicalShortcut {
                    physical_key: PhysicalKey::Enter,
                    modifier_shift: false,
                    modifier_control: false,
                    modifier_alt: false,
                    modifier_meta: false,
                };

                let secondary_shortcut = PhysicalShortcut {
                    physical_key: PhysicalKey::Enter,
                    modifier_shift: true,
                    modifier_control: false,
                    modifier_alt: false,
                    modifier_meta: false,
                };

                let create_static = |label: &str, primary_shortcut: PhysicalShortcut, secondary_shortcut: PhysicalShortcut| {
                    let mut actions: Vec<_> = search_item.entrypoint_actions
                        .iter()
                        .enumerate()
                        .map(|(index, action)| {
                            let physical_shortcut = if index == 0 {
                                Some(secondary_shortcut.clone())
                            } else {
                                action.shortcut.clone()
                            };

                            ActionPanelItem::Action {
                                label: action.label.clone(),
                                widget_id: index,
                                physical_shortcut,
                            }
                        })
                        .collect();

                    let primary_action_widget_id = 0;

                    if actions.len() == 0 {
                        (Some((label.to_string(), primary_action_widget_id, primary_shortcut)), None)
                    } else {
                        let label = label.to_string();

                        let primary_action = ActionPanelItem::Action {
                            label: label.clone(),
                            widget_id: primary_action_widget_id,
                            physical_shortcut: Some(primary_shortcut.clone()),
                        };

                        actions.insert(0, primary_action);

                        let action_panel = ActionPanel {
                            title: Some(search_item.entrypoint_name.clone()),
                            items: actions,
                        };

                        (Some((label, primary_action_widget_id, primary_shortcut)), Some(action_panel))
                    }
                };

                let create_generated = |label: &str, primary_shortcut: PhysicalShortcut, secondary_shortcut: PhysicalShortcut| {
                    let label = search_item.entrypoint_actions
                        .first()
                        .map(|action| action.label.clone())
                        .unwrap_or_else(|| label.to_string()); // should never happen, because there is always at least one action

                    let mut actions: Vec<_> = search_item.entrypoint_actions
                        .iter()
                        .enumerate()
                        .map(|(index, action)| {
                            let physical_shortcut = match index {
                                0 => Some(primary_shortcut.clone()),
                                1 => Some(secondary_shortcut.clone()),
                                _ => action.shortcut.clone()
                            };

                            ActionPanelItem::Action {
                                label: action.label.clone(),
                                widget_id: index,
                                physical_shortcut,
                            }
                        })
                        .collect();

                    let primary_action_widget_id = 0;

                    let action_panel = ActionPanel {
                        title: Some(search_item.entrypoint_name.clone()),
                        items: actions,
                    };

                    (Some((label, primary_action_widget_id, primary_shortcut)), Some(action_panel))
                };

                match search_item.entrypoint_type {
                    SearchResultEntrypointType::Command => create_static("Run Command", primary_shortcut, secondary_shortcut),
                    SearchResultEntrypointType::View => create_static("Open View", primary_shortcut, secondary_shortcut),
                    SearchResultEntrypointType::Generated => create_generated("Run Command", primary_shortcut, secondary_shortcut),
                }
            } else {
                match state.client_context.get_first_inline_view_action_panel() {
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

            let toast_text = if !state.loading_bar_state.is_empty() {
                Some("Indexing...")
            } else {
                None
            };

            let root = match sub_state {
                MainViewState::None => {
                    render_root(
                        false,
                        input,
                        separator,
                        toast_text,
                        content,
                        primary_action,
                        action_panel,
                        None::<&ScrollHandle<SearchResultEntrypointAction>>,
                        "",
                        || AppMsg::ToggleActionPanel { keyboard: false },
                        |widget_id| AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id },
                        |widget_id| AppMsg::Noop,
                        || AppMsg::Noop,
                    )
                }
                MainViewState::SearchResultActionPanel { focused_action_item, .. } => {
                    render_root(
                        true,
                        input,
                        separator,
                        toast_text,
                        content,
                        primary_action,
                        action_panel,
                        Some(focused_action_item),
                        "",
                        || AppMsg::ToggleActionPanel { keyboard: false },
                        |widget_id| AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id },
                        |widget_id| AppMsg::OnAnyActionMainViewSearchResultPanelMouse { widget_id },
                        || AppMsg::Noop,
                    )
                }
                MainViewState::InlineViewActionPanel { focused_action_item, .. } => {
                    render_root(
                        true,
                        input,
                        separator,
                        toast_text,
                        content,
                        primary_action,
                        action_panel,
                        Some(focused_action_item),
                        "",
                        || AppMsg::ToggleActionPanel { keyboard: false },
                        |widget_id| AppMsg::OnPrimaryActionMainViewActionPanelMouse { widget_id },
                        |widget_id| AppMsg::OnAnyActionMainViewInlineViewPanelKeyboardWithFocus { widget_id },
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

            let view_container = state.client_context.get_view_container();

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

fn subscription(state: &AppModel) -> Subscription<AppMsg> {
    let frontend_receiver = state.frontend_receiver.clone();

    struct RequestLoop;
    struct GlobalShortcutListener;

    let events_subscription = event::listen_with(|event, status, window_id| match status {
        event::Status::Ignored => Some(AppMsg::IcedEvent(window_id, event)),
        event::Status::Captured => match &event {
            Event::Keyboard(keyboard::Event::KeyPressed { key: Key::Named(Named::Escape), .. }) => Some(AppMsg::IcedEvent(window_id, event)),
            _ => None
        }
    });

    Subscription::batch([
        Subscription::run_with_id(
            std::any::TypeId::of::<GlobalShortcutListener>(),
            stream::channel(
                10,
                |sender| async move {
                    register_listener(sender.clone());

                    std::future::pending::<()>().await;

                    unreachable!()
                },
            )
        ),
        events_subscription,
        Subscription::run_with_id(
            std::any::TypeId::of::<RequestLoop>(),
            stream::channel(
                100,
                |sender| async move {
                    request_loop(frontend_receiver, sender).await;

                    panic!("request_rx was unexpectedly closed")
                },
            )
        )
    ])
}

fn assign_global_shortcut(
    global_hotkey_manager: &GlobalHotKeyManager,
    current_hotkey: &Arc<StdMutex<Option<HotKey>>>,
    shortcut: Option<PhysicalShortcut>,
) -> anyhow::Result<()> {

    let mut hotkey_guard = current_hotkey
        .lock()
        .expect("lock is poisoned");

    if let Some(current_hotkey) = *hotkey_guard {
        global_hotkey_manager.unregister(current_hotkey)?;
    }

    if let Some(shortcut) = shortcut {
        let hotkey = convert_physical_shortcut_to_hotkey(shortcut);

        *hotkey_guard = Some(hotkey);

        global_hotkey_manager.register(hotkey)?;
    }

    Ok(())
}

impl AppModel {
    fn on_focused(&mut self) -> Task<AppMsg> {
        self.focused = true;
        Task::none()
    }

    fn on_unfocused(&mut self) -> Task<AppMsg> {
        // for some reason (on both macOS and linux x11) duplicate Unfocused fires right before Focus event
        if self.focused {
            self.hide_window()
        } else {
            Task::none()
        }
    }

    fn hide_window(&mut self) -> Task<AppMsg> {
        self.focused = false;

        let mut commands = vec![];

        #[cfg(target_os = "linux")]
        if self.wayland {
            commands.push(
                Task::done(AppMsg::LayerShell(layer_shell::LayerShellAppMsg::RemoveWindow(self.main_window_id)))
            );
        } else {
            commands.push(
                window::change_mode(self.main_window_id, Mode::Hidden)
            );
        };

        #[cfg(not(target_os = "linux"))]
        commands.push(
            window::change_mode(self.main_window_id, Mode::Hidden)
        );

        #[cfg(target_os = "macos")]
        unsafe {
            // when closing NSPanel current active application doesn't automatically become key window
            // is there a proper way? without doing this manually
            let app = objc2_app_kit::NSWorkspace::sharedWorkspace()
                .menuBarOwningApplication();

            if let Some(app) = app {
                app.activateWithOptions(objc2_app_kit::NSApplicationActivationOptions::empty());
            }
        }

        match &self.global_state {
            GlobalState::PluginView { plugin_view_data: PluginViewData { plugin_id, .. }, .. } => {
                commands.push(self.close_plugin_view(plugin_id.clone()));
            }
            GlobalState::MainView { .. } => {}
            GlobalState::ErrorView { .. } => {}
        }

        Task::batch(commands)
    }

    fn show_window(&mut self) -> Task<AppMsg> {
        #[cfg(target_os = "linux")]
        let open_task =  if self.wayland {
            let (_, open_task) = open_main_window_wayland(self.main_window_id);
            open_task
        } else {
            Task::batch([
                window::gain_focus(self.main_window_id),
                window::change_mode(self.main_window_id, Mode::Windowed)
            ])
        };

        #[cfg(not(target_os = "linux"))]
        let open_task = Task::batch([
            window::gain_focus(self.main_window_id),
            window::change_mode(self.main_window_id, Mode::Windowed)
        ]);

        Task::batch([
            open_task,
            self.reset_window_state()
        ])
    }

    fn reset_window_state(&mut self) -> Task<AppMsg> {
        self.prompt = "".to_string();

        self.client_context.clear_all_inline_views();

        GlobalState::initial(&mut self.global_state)
    }

    fn open_plugin_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Task::perform(async move {
            let result = backend_client.request_view_render(plugin_id, entrypoint_id)
                .await?;

            Ok(result)
        }, |result| handle_backend_error(result, |action_shortcuts| AppMsg::OnOpenView { action_shortcuts }))
    }

    fn close_plugin_view(&self, plugin_id: PluginId) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Task::perform(async move {
            backend_client.request_view_close(plugin_id)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Task::perform(async move {
            backend_client.request_run_command(plugin_id, entrypoint_id)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn run_generated_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, action_index: usize) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Task::perform(async move {
            backend_client.request_run_generated_command(plugin_id, entrypoint_id, action_index)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn handle_plugin_event(&self, widget_event: ComponentWidgetEvent, plugin_id: PluginId, render_location: UiRenderLocation) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        let event = self.client_context.handle_event(render_location, &plugin_id, widget_event.clone());

        Task::perform(async move {
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

    fn handle_main_view_keyboard_event(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, physical_key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        Task::perform(
            async move {
                backend_client.send_keyboard_event(plugin_id, entrypoint_id, KeyboardEventOrigin::MainView, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                    .await?;

                Ok(())
            },
            |result| handle_backend_error(result, |()| AppMsg::Noop),
        )
    }

    fn handle_plugin_view_keyboard_event(&self, physical_key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        let (plugin_id, entrypoint_id) = {
            (self.client_context.get_view_plugin_id(), self.client_context.get_view_entrypoint_id())
        };

        Task::perform(
            async move {
                backend_client.send_keyboard_event(plugin_id, entrypoint_id, KeyboardEventOrigin::PluginView, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                    .await?;

                Ok(())
            },
            |result| handle_backend_error(result, |()| AppMsg::Noop),
        )
    }

    fn handle_inline_plugin_view_keyboard_event(&self, physical_key: PhysicalKey, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) -> Task<AppMsg> {
        let mut backend_client = self.backend_api.clone();

        let (plugin_id, entrypoint_id) = {
            match self.client_context.get_first_inline_view_container() {
                None => {
                    return Task::none()
                },
                Some(container) => (container.get_plugin_id(), container.get_entrypoint_id())
            }
        };

        Task::perform(
            async move {
                backend_client.send_keyboard_event(plugin_id, entrypoint_id, KeyboardEventOrigin::PluginView, physical_key, modifier_shift, modifier_control, modifier_alt, modifier_meta)
                    .await?;

                Ok(())
            },
            |result| handle_backend_error(result, |()| AppMsg::Noop),
        )
    }

    fn search(&self, new_prompt: String, render_inline_view: bool) -> Task<AppMsg> {
        let mut backend_api = self.backend_api.clone();

        Task::perform(async move {
            let search_results = backend_api.search(new_prompt, render_inline_view)
                .await?;

            Ok(search_results)
        }, |result| handle_backend_error(result, |search_results| AppMsg::SetSearchResults(search_results)))
    }

    fn open_settings_window_preferences(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>) -> Task<AppMsg> {
        let mut backend_api = self.backend_api.clone();

        Task::perform(async move {
            backend_api.open_settings_window_preferences(plugin_id, entrypoint_id)
                .await?;

            Ok(())
        }, |result| handle_backend_error(result, |()| AppMsg::Noop))
    }

    fn inline_view_shortcuts(&self) -> Task<AppMsg> {
        let mut backend_api = self.backend_api.clone();

        Task::perform(async move {
            backend_api.inline_view_shortcuts().await
        }, |result| handle_backend_error(result, |shortcuts| AppMsg::InlineViewShortcuts { shortcuts }))
    }
}

// these are needed to force focus the text_input in main search view when
// the window is opened but text_input not focused
impl AppModel {
    fn append_prompt(prompt: &mut String, value: Option<SmolStr>, search_field_id: text_input::Id, modifiers: Modifiers) -> Task<AppMsg> {
        if modifiers.control() || modifiers.alt() || modifiers.logo() {
            Task::none()
        } else {
            match value {
                Some(value) => {
                    if let Some(value) = value.chars().next().filter(|c| !c.is_control()) {
                        *prompt = format!("{}{}", prompt, value);
                        focus(search_field_id.clone())
                    } else {
                        Task::none()
                    }
                }
                None => Task::none()
            }
        }
    }

    fn backspace_prompt(prompt: &mut String, search_field_id: text_input::Id) -> Task<AppMsg> {
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
    frontend_receiver: Arc<TokioRwLock<RequestReceiver<UiRequestData, UiResponseData>>>,
    mut sender: Sender<AppMsg>,
) {
    let mut frontend_receiver = frontend_receiver.write().await;
    loop {
        let (request_data, responder) = frontend_receiver.recv().await;

        let app_msg = {
            match request_data {
                UiRequestData::ReplaceView {
                    plugin_id,
                    plugin_name,
                    entrypoint_id,
                    entrypoint_name,
                    render_location,
                    top_level_view,
                    container,
                    images
                } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::RenderPluginUI {
                        plugin_id,
                        plugin_name,
                        entrypoint_id,
                        entrypoint_name,
                        render_location,
                        top_level_view,
                        container: Arc::new(container),
                        images
                    }
                }
                UiRequestData::ClearInlineView { plugin_id } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::ClearInlineView {
                        plugin_id
                    }
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

                    AppMsg::UpdateSearchResults
                }
                UiRequestData::ShowHud { display } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::ShowHud {
                        display
                    }
                }
                UiRequestData::SetGlobalShortcut { shortcut } => {
                    AppMsg::SetGlobalShortcut {
                        shortcut,
                        responder: Arc::new(Mutex::new(Some(responder)))
                    }
                }
                UiRequestData::UpdateLoadingBar { plugin_id, entrypoint_id, show } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::UpdateLoadingBar {
                        plugin_id,
                        entrypoint_id,
                        show
                    }
                }
                UiRequestData::SetTheme { theme } => {
                    responder.respond(UiResponseData::Nothing);

                    AppMsg::SetTheme {
                        theme,
                    }
                }
            }
        };

        let _ = sender.send(app_msg).await;
    }
}
