use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use client_context::ClientContext;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::KeyboardEventOrigin;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::SearchResult;
use gauntlet_common::model::SearchResultEntrypointActionType;
use gauntlet_common::model::SearchResultEntrypointType;
use gauntlet_common::model::UiRenderLocation;
use gauntlet_common::model::UiTheme;
use gauntlet_common::model::UiWidgetId;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApiRequestData;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApiResponseData;
use gauntlet_common_ui::physical_key_model;
use gauntlet_server::global_hotkey::GlobalHotKeyManager;
use gauntlet_server::plugins::ApplicationManager;
use gauntlet_server::plugins::settings::global_shortcut::GlobalShortcutAction;
use gauntlet_server::plugins::settings::global_shortcut::GlobalShortcutPressedEvent;
use gauntlet_server::plugins::settings::global_shortcut::register_global_shortcut_listener;
use gauntlet_utils::channel::RequestError;
use gauntlet_utils::channel::Responder;
use iced::Event;
use iced::Length;
use iced::Renderer;
use iced::Settings;
use iced::Subscription;
use iced::Task;
use iced::advanced::graphics::core::SmolStr;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::event;
use iced::futures::StreamExt;
use iced::keyboard;
use iced::keyboard::Key;
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;
use iced::keyboard::key::Physical;
use iced::stream;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::horizontal_space;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::text::Shaping;
use iced::widget::text_input;
use iced::widget::text_input::focus;
use iced::window;
use iced_fonts::BOOTSTRAP_FONT_BYTES;

use crate::model::UiViewEvent;
use crate::ui::search_list::search_list;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::container::ContainerStyleInner;
use crate::ui::theme::text_input::TextInputStyle;

mod client_context;
mod custom_widgets;
mod grid_navigation;
mod scroll_handle;
mod search_list;
mod state;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod sys_tray;
mod theme;
mod widget;
mod widget_container;

pub mod scenario_runner;
mod server;
mod windows;

pub use theme::GauntletComplexTheme;

use crate::ui::custom_widgets::loading_bar::LoadingBar;
use crate::ui::scenario_runner::ScenarioRunnerData;
use crate::ui::scenario_runner::ScenarioRunnerMsg;
use crate::ui::scenario_runner::handle_scenario_runner_msg;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::server::handle_server_message;
use crate::ui::state::ErrorViewData;
use crate::ui::state::Focus;
use crate::ui::state::GlobalState;
use crate::ui::state::LoadingBarState;
use crate::ui::state::MainViewState;
use crate::ui::state::PluginViewData;
use crate::ui::state::PluginViewState;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::action_panel::ActionPanelItem;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::root::render_root;
use crate::ui::windows::WindowActionMsg;
use crate::ui::windows::WindowState;
#[cfg(target_os = "linux")]
use crate::ui::windows::x11_focus::x11_linux_focus_change_subscription;

pub struct AppModel {
    // logic
    application_manager: Arc<ApplicationManager>,
    global_hotkey_manager: GlobalHotKeyManager,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    _tray_icon: tray_icon::TrayIcon,
    theme: GauntletComplexTheme,
    window: WindowState,

    // ephemeral state
    prompt: String,

    // state
    client_context: ClientContext,
    global_state: GlobalState,
    search_results: Vec<SearchResult>,
    loading_bar_state: HashMap<(PluginId, EntrypointId), ()>,
    hud_display: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    OpenView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    OpenGeneratedView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    },
    ShowNewView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    ShowNewGeneratedView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    },
    RunCommand {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    RunGeneratedEntrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    },
    RunSearchItemAction(SearchResult, usize),
    RunPluginAction {
        render_location: UiRenderLocation,
        plugin_id: PluginId,
        widget_id: UiWidgetId,
        id: Option<String>,
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
        data: HashMap<UiWidgetId, Vec<u8>>,
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
    HandleGlobalShortcut(GlobalShortcutPressedEvent),
    ToggleActionPanel {
        keyboard: bool,
    },
    ShowPreferenceRequiredView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool,
    },
    OpenSettingsPreferences {
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
    },
    OnOpenView {
        action_shortcuts: HashMap<String, PhysicalShortcut>,
    },
    ShowPluginErrorView {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    ShowBackendError(RequestError),
    ClosePluginView,
    OpenPluginView(PluginId, EntrypointId),
    InlineViewShortcuts {
        shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>,
    },
    OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus,
    OnPrimaryActionMainViewNoPanel {
        search_result: SearchResult,
    },
    OnSecondaryActionMainViewNoPanelKeyboardWithFocus {
        search_result: SearchResult,
    },
    OnSecondaryActionMainViewNoPanelKeyboardWithoutFocus,
    OnAnyActionMainViewSearchResultPanelKeyboardWithFocus {
        search_result: SearchResult,
        widget_id: UiWidgetId,
    },
    OnAnyActionMainViewInlineViewPanelKeyboardWithFocus {
        widget_id: UiWidgetId,
    },
    OnAnyActionPluginViewNoPanelKeyboardWithFocus {
        widget_id: UiWidgetId,
        id: Option<String>,
    },
    OnAnyActionPluginViewAnyPanelKeyboardWithFocus {
        widget_id: UiWidgetId,
        id: Option<String>,
    },
    OnAnyActionPluginViewAnyPanel {
        widget_id: UiWidgetId,
        id: Option<String>,
    },
    OnAnyActionMainViewSearchResultPanelMouse {
        widget_id: UiWidgetId,
    },
    OnPrimaryActionMainViewActionPanelMouse,
    ResetMainViewState,
    OnAnyActionMainViewNoPanelKeyboardAtIndex {
        index: usize,
    },
    UpdateLoadingBar {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        show: bool,
    },
    PendingPluginViewLoadingBar,
    ShowPluginViewLoadingBar,
    FocusPluginViewSearchBar {
        widget_id: UiWidgetId,
    },
    ClearInlineView {
        plugin_id: PluginId,
    },
    SetTheme {
        theme: UiTheme,
    },
    RunEntrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    HandleServerRequest {
        request_data: Arc<ServerGrpcApiRequestData>,
        responder: Arc<Mutex<Option<Responder<ServerGrpcApiResponseData>>>>,
    },
    WindowAction(WindowActionMsg),
    ResetWindowState,
    ResetMainWindowScroll,
    SetHudDisplay {
        display: String,
    },
    HandleScenario(ScenarioRunnerMsg),
}

pub fn run(minimized: bool, scenario_runner_data: Option<ScenarioRunnerData>) {
    let boot = move || new(minimized, scenario_runner_data.clone());

    iced::daemon::<AppModel, AppMsg, GauntletComplexTheme, Renderer>(boot, update, view)
        .title(title)
        .settings(Settings {
            #[cfg(target_os = "macos")]
            platform_specific: iced::PlatformSpecific {
                activation_policy: iced::ActivationPolicy::Accessory,
                activate_ignoring_other_apps: true,
            },
            ..Default::default()
        })
        .font(BOOTSTRAP_FONT_BYTES)
        .subscription(subscription)
        .theme(|state, _| state.theme.clone())
        .run()
        .expect("Unable to start scenario application");
}

fn new(minimized: bool, #[allow(unused)] scenario_runner_data: Option<ScenarioRunnerData>) -> (AppModel, Task<AppMsg>) {
    let (application_manager, global_hotkey_manager, setup_data, setup_task) = server::setup();

    #[cfg(target_os = "linux")]
    let wayland = std::env::var("WAYLAND_DISPLAY")
        .or_else(|_| std::env::var("WAYLAND_SOCKET"))
        .is_ok(); // todo add config value for layer shell

    let theme = GauntletComplexTheme::new(setup_data.theme);
    GauntletComplexTheme::set_global(theme.clone());

    let mut tasks: Vec<Task<AppMsg>> = vec![];

    tasks.push(setup_task);

    if !minimized {
        tasks.push(Task::done(AppMsg::WindowAction(WindowActionMsg::ShowWindow)));
    }

    #[cfg(feature = "scenario_runner")]
    tasks.push(scenario_runner::run_scenario(
        scenario_runner_data.unwrap(),
        application_manager.get_scenarios_theme(),
    ));

    let client_context = ClientContext::new();
    let global_state = GlobalState::new(text_input::Id::unique());
    let window = WindowState::new(
        setup_data.window_position_file,
        setup_data.close_on_unfocus,
        setup_data.window_position_mode,
        #[cfg(target_os = "linux")]
        wayland,
    );

    (
        AppModel {
            // logic
            application_manager,
            global_hotkey_manager,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            _tray_icon: sys_tray::create_tray(),
            theme,
            window,

            // ephemeral state
            prompt: "".to_string(),

            // state
            global_state,
            client_context,
            search_results: vec![],
            loading_bar_state: HashMap::new(),
            hud_display: None,
        },
        Task::batch(tasks),
    )
}

fn title(state: &AppModel, window: window::Id) -> String {
    if Some(window) == state.window.main_window_id {
        "Gauntlet".to_owned()
    } else {
        "Gauntlet HUD".to_owned()
    }
}

fn update(state: &mut AppModel, message: AppMsg) -> Task<AppMsg> {
    match message {
        AppMsg::OpenView {
            plugin_id,
            entrypoint_id,
        } => {
            match &mut state.global_state {
                GlobalState::MainView {
                    pending_plugin_view_data,
                    ..
                } => {
                    *pending_plugin_view_data = Some(PluginViewData {
                        top_level_view: true,
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        action_shortcuts: HashMap::new(),
                    });

                    Task::batch([
                        Task::done(AppMsg::OpenPluginView(plugin_id, entrypoint_id)),
                        Task::done(AppMsg::PendingPluginViewLoadingBar),
                    ])
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { .. } => Task::none(),
                GlobalState::PendingPluginView { .. } => Task::none(),
            }
        }
        AppMsg::OpenGeneratedView {
            plugin_id,
            entrypoint_id,
            action_index,
        } => {
            match &mut state.global_state {
                GlobalState::MainView {
                    pending_plugin_view_data,
                    ..
                } => {
                    *pending_plugin_view_data = Some(PluginViewData {
                        top_level_view: true,
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        action_shortcuts: HashMap::new(),
                    });

                    Task::batch([
                        state.run_generated_entrypoint(plugin_id, entrypoint_id, action_index),
                        Task::done(AppMsg::PendingPluginViewLoadingBar),
                    ])
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { .. } => Task::none(),
                GlobalState::PendingPluginView { .. } => Task::none(),
            }
        }
        AppMsg::RunCommand {
            plugin_id,
            entrypoint_id,
        } => {
            Task::batch([
                Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow)),
                state.run_command(plugin_id, entrypoint_id),
            ])
        }
        AppMsg::RunGeneratedEntrypoint {
            plugin_id,
            entrypoint_id,
            action_index,
        } => {
            Task::batch([
                Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow)),
                state.run_generated_entrypoint(plugin_id, entrypoint_id, action_index),
            ])
        }
        AppMsg::RunPluginAction {
            render_location,
            plugin_id,
            widget_id,
            id,
        } => {
            let widget_event = ComponentWidgetEvent::RunAction { widget_id, id };

            match render_location {
                UiRenderLocation::InlineView => {
                    Task::batch([
                        Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow)),
                        Task::done(AppMsg::WidgetEvent {
                            widget_event,
                            plugin_id,
                            render_location,
                        }),
                    ])
                }
                UiRenderLocation::View => {
                    Task::done(AppMsg::WidgetEvent {
                        widget_event,
                        plugin_id,
                        render_location,
                    })
                }
            }
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
                }
                SearchResultEntrypointType::View => {
                    if action_index == 0 {
                        Task::done(AppMsg::OpenView {
                            plugin_id: search_result.plugin_id.clone(),
                            entrypoint_id: search_result.entrypoint_id.clone(),
                        })
                    } else {
                        Task::none()
                    }
                }
                SearchResultEntrypointType::Generated => {
                    let action = &search_result.entrypoint_actions[action_index];
                    match &action.action_type {
                        SearchResultEntrypointActionType::Command => {
                            Task::done(AppMsg::RunGeneratedEntrypoint {
                                entrypoint_id: search_result.entrypoint_id.clone(),
                                plugin_id: search_result.plugin_id.clone(),
                                action_index,
                            })
                        }
                        SearchResultEntrypointActionType::View => {
                            Task::done(AppMsg::OpenGeneratedView {
                                plugin_id: search_result.plugin_id.clone(),
                                entrypoint_id: search_result.entrypoint_id.clone(),
                                action_index,
                            })
                        }
                    }
                }
            }
        }
        AppMsg::PromptChanged(mut new_prompt) => {
            match &mut state.global_state {
                GlobalState::MainView {
                    focused_search_result,
                    sub_state,
                    ..
                } => {
                    new_prompt.truncate(100); // search query uses regex so just to be safe truncate the prompt

                    state.prompt = new_prompt.clone();

                    focused_search_result.reset(true);

                    MainViewState::initial(sub_state);
                }
                GlobalState::ErrorView { .. } => {}
                GlobalState::PluginView { .. } => {}
                GlobalState::PendingPluginView { .. } => {}
            }

            state.search(new_prompt, true)
        }
        AppMsg::UpdateSearchResults => {
            match &state.global_state {
                GlobalState::MainView { .. } => state.search(state.prompt.clone(), false),
                _ => Task::none(),
            }
        }
        AppMsg::PromptSubmit => state.global_state.primary(&state.client_context, &state.search_results),
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
            data: images,
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
                }),
            ])
        }
        AppMsg::HandleRenderPluginUI {
            top_level_view,
            has_children,
            render_location,
        } => {
            match &mut state.global_state {
                GlobalState::MainView {
                    pending_plugin_view_data,
                    focused_search_result,
                    pending_plugin_view_loading_bar,
                    ..
                } => {
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
                                false,
                            )
                        }
                    };

                    if let UiRenderLocation::InlineView = render_location {
                        Task::batch([command, state.inline_view_shortcuts()])
                    } else {
                        command
                    }
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { plugin_view_data, .. } => {
                    plugin_view_data.top_level_view = top_level_view;

                    Task::none()
                }
                GlobalState::PendingPluginView {
                    pending_plugin_view_data,
                } => {
                    let pending_plugin_view_data = pending_plugin_view_data.clone();
                    GlobalState::plugin(
                        &mut state.global_state,
                        PluginViewData {
                            top_level_view,
                            ..pending_plugin_view_data
                        },
                        true,
                    )
                }
            }
        }
        AppMsg::IcedEvent(window_id, Event::Keyboard(event)) => {
            let Some(main_window_id) = state.window.main_window_id else {
                return Task::none();
            };

            if window_id != main_window_id {
                return Task::none();
            }

            match event {
                keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    physical_key,
                    text,
                    ..
                } => {
                    tracing::debug!(
                        "Key pressed: {:?}. shift: {:?} control: {:?} alt: {:?} meta: {:?}",
                        key,
                        modifiers.shift(),
                        modifiers.control(),
                        modifiers.alt(),
                        modifiers.logo()
                    );
                    match key {
                        Key::Named(Named::ArrowUp) => {
                            state.global_state.up(&mut state.client_context, &state.search_results)
                        }
                        Key::Named(Named::ArrowDown) => {
                            state
                                .global_state
                                .down(&mut state.client_context, &state.search_results)
                        }
                        Key::Named(Named::ArrowLeft) => {
                            state
                                .global_state
                                .left(&mut state.client_context, &state.search_results)
                        }
                        Key::Named(Named::ArrowRight) => {
                            state
                                .global_state
                                .right(&mut state.client_context, &state.search_results)
                        }
                        Key::Named(Named::Escape) => state.global_state.back(&state.client_context),
                        Key::Named(Named::Tab) if !modifiers.shift() => state.global_state.next(&state.client_context),
                        Key::Named(Named::Tab) if modifiers.shift() => {
                            state.global_state.previous(&state.client_context)
                        }
                        Key::Named(Named::Enter) => {
                            if modifiers.logo() || modifiers.alt() || modifiers.control() {
                                Task::none() // to avoid not wanted "enter" presses
                            } else {
                                if modifiers.shift() {
                                    // for main view, also fired in cases where main text field is not focused
                                    state
                                        .global_state
                                        .secondary(&state.client_context, &state.search_results)
                                } else {
                                    state.global_state.primary(&state.client_context, &state.search_results)
                                }
                            }
                        }
                        Key::Named(Named::Backspace) => {
                            match &mut state.global_state {
                                GlobalState::MainView {
                                    sub_state,
                                    search_field_id,
                                    ..
                                } => {
                                    match sub_state {
                                        MainViewState::None => {
                                            AppModel::backspace_prompt(&mut state.prompt, search_field_id.clone())
                                        }
                                        MainViewState::SearchResultActionPanel { .. } => Task::none(),
                                        MainViewState::InlineViewActionPanel { .. } => Task::none(),
                                    }
                                }
                                GlobalState::ErrorView { .. } => Task::none(),
                                GlobalState::PluginView { sub_state, .. } => {
                                    match sub_state {
                                        PluginViewState::None => state.client_context.backspace_text(),
                                        PluginViewState::ActionPanel { .. } => Task::none(),
                                    }
                                }
                                GlobalState::PendingPluginView { .. } => Task::none(),
                            }
                        }
                        _ => state.handle_shortcut_key(physical_key, modifiers, text),
                    }
                }
                _ => Task::none(),
            }
        }
        AppMsg::IcedEvent(window_id, Event::Window(window::Event::Focused)) => {
            state.window.handle_focused_event(window_id)
        }
        AppMsg::IcedEvent(window_id, Event::Window(window::Event::Unfocused)) => {
            state.window.handle_unfocused_event(window_id)
        }
        AppMsg::IcedEvent(window_id, Event::Window(window::Event::Moved(point))) => {
            state.window.handle_move_event(window_id, point)
        }
        AppMsg::IcedEvent(_, _) => Task::none(),
        AppMsg::WidgetEvent {
            widget_event: ComponentWidgetEvent::Noop,
            ..
        } => Task::none(),
        AppMsg::WidgetEvent {
            widget_event: ComponentWidgetEvent::PreviousView,
            ..
        } => state.global_state.back(&state.client_context),
        AppMsg::WidgetEvent {
            widget_event,
            plugin_id,
            render_location,
        } => state.handle_plugin_event(widget_event, plugin_id, render_location),
        AppMsg::Noop => Task::none(),
        AppMsg::ShowPreferenceRequiredView {
            plugin_id,
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required,
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
        AppMsg::ShowPluginErrorView {
            plugin_id,
            entrypoint_id,
        } => {
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
                    RequestError::Timeout => ErrorViewData::BackendTimeout,
                    RequestError::Other { display } => ErrorViewData::UnknownError { display },
                    RequestError::OtherSideWasDropped => {
                        ErrorViewData::UnknownError {
                            display: "The other side was dropped".to_string(),
                        }
                    }
                },
            )
        }
        AppMsg::OpenSettingsPreferences {
            plugin_id,
            entrypoint_id,
        } => state.open_settings_window_preferences(plugin_id, entrypoint_id),
        AppMsg::OnOpenView { action_shortcuts } => {
            match &mut state.global_state {
                GlobalState::MainView {
                    pending_plugin_view_data,
                    ..
                } => {
                    match pending_plugin_view_data {
                        None => {}
                        Some(pending_plugin_view_data) => {
                            pending_plugin_view_data.action_shortcuts = action_shortcuts;
                        }
                    };
                }
                GlobalState::ErrorView { .. } => {}
                GlobalState::PluginView { plugin_view_data, .. } => {
                    plugin_view_data.action_shortcuts = action_shortcuts;
                }
                GlobalState::PendingPluginView { .. } => {}
            }

            Task::none()
        }
        AppMsg::ToggleActionPanel { keyboard } => {
            match &mut state.global_state {
                GlobalState::MainView {
                    sub_state,
                    focused_search_result,
                    ..
                } => {
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
                GlobalState::ErrorView { .. } => {}
                GlobalState::PluginView { sub_state, .. } => {
                    state.client_context.toggle_action_panel();

                    match sub_state {
                        PluginViewState::None => PluginViewState::action_panel(sub_state, keyboard),
                        PluginViewState::ActionPanel { .. } => PluginViewState::initial(sub_state),
                    }
                }
                GlobalState::PendingPluginView { .. } => {}
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
        AppMsg::OnAnyActionMainViewSearchResultPanelKeyboardWithFocus {
            search_result,
            widget_id,
        } => {
            let index = widget_id;

            Task::batch([
                Task::done(AppMsg::RunSearchItemAction(search_result, index)),
                Task::done(AppMsg::ResetMainViewState),
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
                            id: None,
                        }),
                    ])
                }
                None => Task::none(),
            }
        }
        AppMsg::OnAnyActionPluginViewNoPanelKeyboardWithFocus { widget_id, id } => {
            Task::done(AppMsg::OnAnyActionPluginViewAnyPanel { widget_id, id })
        }
        AppMsg::OnAnyActionPluginViewAnyPanelKeyboardWithFocus { widget_id, id } => {
            Task::batch([
                Task::done(AppMsg::ToggleActionPanel { keyboard: true }),
                Task::done(AppMsg::RunPluginAction {
                    render_location: UiRenderLocation::View,
                    plugin_id: state.client_context.get_view_plugin_id(),
                    widget_id,
                    id,
                }),
            ])
        }
        AppMsg::OnPrimaryActionMainViewActionPanelMouse => {
            match &state.global_state {
                GlobalState::MainView {
                    focused_search_result, ..
                } => {
                    if let Some(search_result) = focused_search_result.get(&state.search_results) {
                        let search_result = search_result.clone();
                        Task::done(AppMsg::OnPrimaryActionMainViewNoPanel { search_result })
                    } else {
                        Task::done(AppMsg::OnPrimaryActionMainViewNoPanelKeyboardWithoutFocus)
                    }
                }
                GlobalState::PluginView { .. } => Task::none(),
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PendingPluginView { .. } => Task::none(),
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
                            id: None,
                        })
                    }
                    None => Task::none(),
                }
            } else {
                Task::none()
            }
        }
        AppMsg::OnAnyActionMainViewSearchResultPanelMouse { widget_id } => {
            match &mut state.global_state {
                GlobalState::MainView {
                    focused_search_result,
                    sub_state,
                    ..
                } => {
                    match sub_state {
                        MainViewState::None => Task::none(),
                        MainViewState::SearchResultActionPanel { .. } => {
                            if let Some(search_result) = focused_search_result.get(&state.search_results) {
                                let search_result = search_result.clone();
                                Task::done(AppMsg::OnAnyActionMainViewSearchResultPanelKeyboardWithFocus {
                                    search_result,
                                    widget_id,
                                })
                            } else {
                                Task::none()
                            }
                        }
                        MainViewState::InlineViewActionPanel { .. } => Task::none(),
                    }
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { .. } => Task::none(),
                GlobalState::PendingPluginView { .. } => Task::none(),
            }
        }
        AppMsg::OnAnyActionPluginViewAnyPanel { widget_id, id } => {
            Task::done(AppMsg::RunPluginAction {
                render_location: UiRenderLocation::View,
                plugin_id: state.client_context.get_view_plugin_id(),
                widget_id,
                id,
            })
        }
        AppMsg::OpenPluginView(plugin_id, entrypoint_id) => state.open_plugin_view(plugin_id, entrypoint_id),
        AppMsg::ClosePluginView => {
            if let GlobalState::PluginView { plugin_view_data, .. } = &state.global_state {
                state.close_plugin_view(plugin_view_data.plugin_id.clone())
            } else {
                Task::none()
            }
        }
        AppMsg::InlineViewShortcuts { shortcuts } => {
            state.client_context.set_inline_view_shortcuts(shortcuts);

            Task::none()
        }
        AppMsg::ResetMainViewState => {
            match &mut state.global_state {
                GlobalState::MainView { sub_state, .. } => {
                    MainViewState::initial(sub_state);

                    Task::none()
                }
                GlobalState::ErrorView { .. } => Task::none(),
                GlobalState::PluginView { .. } => Task::none(),
                GlobalState::PendingPluginView { .. } => Task::none(),
            }
        }
        AppMsg::UpdateLoadingBar {
            plugin_id,
            entrypoint_id,
            show,
        } => {
            if show {
                state.loading_bar_state.insert((plugin_id, entrypoint_id), ());
            } else {
                state.loading_bar_state.remove(&(plugin_id, entrypoint_id));
            }

            Task::none()
        }
        AppMsg::PendingPluginViewLoadingBar => {
            if let GlobalState::MainView {
                pending_plugin_view_loading_bar,
                ..
            } = &mut state.global_state
            {
                *pending_plugin_view_loading_bar = LoadingBarState::Pending;
            }

            Task::perform(
                async move {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                    AppMsg::ShowPluginViewLoadingBar
                },
                std::convert::identity,
            )
        }
        AppMsg::ShowPluginViewLoadingBar => {
            if let GlobalState::MainView {
                pending_plugin_view_loading_bar,
                ..
            } = &mut state.global_state
            {
                if let LoadingBarState::Pending = pending_plugin_view_loading_bar {
                    *pending_plugin_view_loading_bar = LoadingBarState::On;
                }
            }

            Task::none()
        }
        AppMsg::FocusPluginViewSearchBar { widget_id } => state.client_context.focus_search_bar(widget_id),
        AppMsg::WindowAction(action) => state.window.handle_action(action),
        AppMsg::ClearInlineView { plugin_id } => {
            state.client_context.clear_inline_view(&plugin_id);

            Task::none()
        }
        AppMsg::SetTheme { theme } => {
            state.theme = GauntletComplexTheme::new(theme);

            GauntletComplexTheme::update_global(state.theme.clone());

            Task::none()
        }
        AppMsg::ShowNewView {
            plugin_id,
            entrypoint_id,
        } => {
            Task::batch([
                GlobalState::pending_plugin(
                    &mut state.global_state,
                    PluginViewData {
                        top_level_view: true,
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        action_shortcuts: HashMap::new(),
                    },
                ),
                Task::done(AppMsg::OpenPluginView(plugin_id, entrypoint_id)),
                Task::done(AppMsg::WindowAction(WindowActionMsg::ShowWindow)),
            ])
        }
        AppMsg::ShowNewGeneratedView {
            plugin_id,
            entrypoint_id,
            action_index,
        } => {
            Task::batch([
                GlobalState::pending_plugin(
                    &mut state.global_state,
                    PluginViewData {
                        top_level_view: true,
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        action_shortcuts: HashMap::new(),
                    },
                ),
                state.run_generated_entrypoint(plugin_id, entrypoint_id, action_index),
                Task::done(AppMsg::WindowAction(WindowActionMsg::ShowWindow)),
            ])
        }
        AppMsg::HandleGlobalShortcut(event) => {
            match state.application_manager.handle_global_shortcut_event(event) {
                Ok(action) => {
                    match action {
                        GlobalShortcutAction::ToggleWindow => {
                            Task::done(AppMsg::WindowAction(WindowActionMsg::ToggleWindow))
                        }
                        GlobalShortcutAction::RunEntrypoint {
                            plugin_id,
                            entrypoint_id,
                        } => {
                            Task::done(AppMsg::RunEntrypoint {
                                plugin_id,
                                entrypoint_id,
                            })
                        }
                        GlobalShortcutAction::Noop => Task::none(),
                    }
                }
                Err(err) => {
                    tracing::error!("Error happened while handling global shortcut: {:?}", err);

                    Task::none()
                }
            }
        }
        AppMsg::RunEntrypoint {
            plugin_id,
            entrypoint_id,
        } => {
            let application_manager = state.application_manager.clone();

            Task::future(async move {
                application_manager
                    .run_action(plugin_id, entrypoint_id, ":primary".to_string())
                    .await
                    .map(|()| AppMsg::Noop)
                    .unwrap_or_else(|err| AppMsg::ShowBackendError(err.into()))
            })
        }
        AppMsg::HandleServerRequest {
            request_data,
            responder,
        } => handle_server_message(state, request_data, responder),
        AppMsg::ResetWindowState => state.reset_window_state(),
        AppMsg::ResetMainWindowScroll => {
            if let GlobalState::MainView {
                focused_search_result, ..
            } = &state.global_state
            {
                focused_search_result.scroll_to(0)
            } else {
                Task::none()
            }
        }
        AppMsg::SetHudDisplay { display } => {
            state.hud_display = Some(display);

            Task::none()
        }
        AppMsg::HandleScenario(msg) => {
            handle_scenario_runner_msg(msg, state.application_manager.clone(), state.window.main_window_id)
                .map(AppMsg::HandleScenario)
        }
    }
}

fn view(state: &AppModel, window: window::Id) -> Element<'_, AppMsg> {
    if Some(window) == state.window.main_window_id {
        view_main(state)
    } else {
        view_hud(state)
    }
}

fn view_hud(state: &AppModel) -> Element<'_, AppMsg> {
    match &state.hud_display {
        Some(hud_display) => {
            let hud: Element<_> = text(hud_display.to_string()).shaping(Shaping::Advanced).into();

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
            container(horizontal_space()).themed(ContainerStyle::Hud)
        }
    }
}

fn view_main(state: &AppModel) -> Element<'_, AppMsg> {
    match &state.global_state {
        GlobalState::ErrorView { error_view } => {
            match error_view {
                ErrorViewData::PreferenceRequired {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required,
                } => {
                    let (description_text, msg) = match (plugin_preferences_required, entrypoint_preferences_required) {
                        (true, true) => {
                            // TODO do not show "entrypoint" name to user
                            let description_text =
                                "Before using, plugin and entrypoint preferences need to be specified";
                            // note:
                            // we open plugin view and not entrypoint even though both need to be specified
                            let msg = AppMsg::OpenSettingsPreferences {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: None,
                            };
                            (description_text, msg)
                        }
                        (false, true) => {
                            // TODO do not show "entrypoint" name to user
                            let description_text = "Before using, entrypoint preferences need to be specified";
                            let msg = AppMsg::OpenSettingsPreferences {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: Some(entrypoint_id.clone()),
                            };
                            (description_text, msg)
                        }
                        (true, false) => {
                            let description_text = "Before using, plugin preferences need to be specified";
                            let msg = AppMsg::OpenSettingsPreferences {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: None,
                            };
                            (description_text, msg)
                        }
                        (false, false) => unreachable!(),
                    };

                    let description: Element<_> = text(description_text).shaping(Shaping::Advanced).into();

                    let description = container(description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PreferenceRequiredViewDescription);

                    let button_label: Element<_> = text("Open Settings").into();

                    let button: Element<_> = button(button_label).on_press(msg).into();

                    let button = container(button).width(Length::Fill).align_x(Horizontal::Center).into();

                    let content: Element<_> = column([description, button]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .themed(ContainerStyle::Main);

                    content
                }
                ErrorViewData::PluginError { .. } => {
                    let description: Element<_> = text("Error occurred in plugin when trying to show the view").into();

                    let description = container(description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> = text("Please report this to plugin author").into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let button_label: Element<_> = text("Close").into();

                    let button: Element<_> = button(button_label)
                        .on_press(AppMsg::WindowAction(WindowActionMsg::HideWindow))
                        .into();

                    let button = container(button).width(Length::Fill).align_x(Horizontal::Center).into();

                    let content: Element<_> = column([description, sub_description, button]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .themed(ContainerStyle::Main);

                    content
                }
                ErrorViewData::UnknownError { display } => {
                    let description: Element<_> = text("Unknown error occurred").into();

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

                    let error_description: Element<_> = text(display).shaping(Shaping::Advanced).into();

                    let error_description = container(error_description)
                        .width(Length::Fill)
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let error_description = scrollable(error_description).width(Length::Fill).into();

                    let button_label: Element<_> = text("Close").into();

                    let button: Element<_> = button(button_label)
                        .on_press(AppMsg::WindowAction(WindowActionMsg::HideWindow))
                        .into();

                    let button = container(button).width(Length::Fill).align_x(Horizontal::Center).into();

                    let content: Element<_> = column([description, sub_description, error_description, button]).into();

                    let content: Element<_> = container(content)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .themed(ContainerStyle::Main);

                    content
                }
                ErrorViewData::BackendTimeout => {
                    let description: Element<_> = text("Error occurred").into();

                    let description = container(description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewTitle);

                    let sub_description: Element<_> =
                        text("Backend was unable to process message in a timely manner").into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .themed(ContainerStyle::PluginErrorViewDescription);

                    let button_label: Element<_> = text("Close").into();

                    let button: Element<_> = button(button_label)
                        .on_press(AppMsg::WindowAction(WindowActionMsg::HideWindow))
                        .into();

                    let button = container(button).width(Length::Fill).align_x(Horizontal::Center).into();

                    let content: Element<_> = column([description, sub_description, button]).into();

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
        GlobalState::MainView {
            focused_search_result,
            sub_state,
            search_field_id,
            pending_plugin_view_loading_bar,
            ..
        } => {
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

            let separator = if matches!(pending_plugin_view_loading_bar, LoadingBarState::On)
                || !state.loading_bar_state.is_empty()
            {
                LoadingBar::new().into()
            } else {
                horizontal_rule(1).into()
            };

            let inline_view = match state.client_context.get_all_inline_view_containers().first() {
                Some((plugin_id, container)) => {
                    let plugin_id = plugin_id.clone();
                    container.render_inline_root_widget().map(move |widget_event| {
                        AppMsg::WidgetEvent {
                            plugin_id: plugin_id.clone(),
                            render_location: UiRenderLocation::InlineView,
                            widget_event,
                        }
                    })
                }
                None => horizontal_space().into(),
            };

            let content: Element<_> = column(vec![inline_view, list]).into();

            let (primary_action, action_panel) =
                if let Some(search_item) = focused_search_result.get(&state.search_results) {
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

                    let create_static =
                        |label: &str, primary_shortcut: PhysicalShortcut, secondary_shortcut: PhysicalShortcut| {
                            let mut actions: Vec<_> = search_item
                                .entrypoint_actions
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
                                (
                                    Some((label.to_string(), primary_action_widget_id, primary_shortcut)),
                                    None,
                                )
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

                                (
                                    Some((label, primary_action_widget_id, primary_shortcut)),
                                    Some(action_panel),
                                )
                            }
                        };

                    let create_generated =
                        |label: &str, primary_shortcut: PhysicalShortcut, secondary_shortcut: PhysicalShortcut| {
                            let label = search_item
                                .entrypoint_actions
                                .first()
                                .map(|action| action.label.clone())
                                .unwrap_or_else(|| label.to_string()); // should never happen, because there is always at least one action

                            let actions: Vec<_> = search_item
                                .entrypoint_actions
                                .iter()
                                .enumerate()
                                .map(|(index, action)| {
                                    let physical_shortcut = match index {
                                        0 => Some(primary_shortcut.clone()),
                                        1 => Some(secondary_shortcut.clone()),
                                        _ => action.shortcut.clone(),
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

                            (
                                Some((label, primary_action_widget_id, primary_shortcut)),
                                Some(action_panel),
                            )
                        };

                    match search_item.entrypoint_type {
                        SearchResultEntrypointType::Command => {
                            create_static("Run Command", primary_shortcut, secondary_shortcut)
                        }
                        SearchResultEntrypointType::View => {
                            create_static("Open View", primary_shortcut, secondary_shortcut)
                        }
                        SearchResultEntrypointType::Generated => {
                            create_generated("Run Command", primary_shortcut, secondary_shortcut)
                        }
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
                                        modifier_meta: false,
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
                        None::<&ScrollHandle>,
                        "",
                        || AppMsg::ToggleActionPanel { keyboard: false },
                        |_widget_id| {
                            // widget_id here is always 0
                            AppMsg::OnPrimaryActionMainViewActionPanelMouse
                        },
                        |_widget_id| AppMsg::Noop,
                        || AppMsg::Noop,
                    )
                }
                MainViewState::SearchResultActionPanel {
                    focused_action_item, ..
                } => {
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
                        |_widget_id| {
                            // widget_id here is always 0
                            AppMsg::OnPrimaryActionMainViewActionPanelMouse
                        },
                        |widget_id| AppMsg::OnAnyActionMainViewSearchResultPanelMouse { widget_id },
                        || AppMsg::Noop,
                    )
                }
                MainViewState::InlineViewActionPanel {
                    focused_action_item, ..
                } => {
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
                        |_widget_id| {
                            // widget_id here is always 0
                            AppMsg::OnPrimaryActionMainViewActionPanelMouse
                        },
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
        GlobalState::PluginView {
            plugin_view_data,
            sub_state,
            ..
        } => {
            let PluginViewData {
                plugin_id,
                action_shortcuts,
                ..
            } = plugin_view_data;

            let view_container = state.client_context.get_view_container();

            let container_element =
                view_container
                    .render_root_widget(sub_state, action_shortcuts)
                    .map(|widget_event| {
                        AppMsg::WidgetEvent {
                            plugin_id: plugin_id.clone(),
                            render_location: UiRenderLocation::View,
                            widget_event,
                        }
                    });

            let element: Element<_> = container(container_element)
                .width(Length::Fill)
                .height(Length::Fill)
                .themed(ContainerStyle::Root);

            // let element = element.explain(color!(0xFF0000));

            element
        }
        GlobalState::PendingPluginView { .. } => {
            // this should never be shown, but in case it does, do not make it fully transparent
            container(horizontal_space()).themed(ContainerStyle::Hud)
        }
    }
}

fn subscription(#[allow(unused)] state: &AppModel) -> Subscription<AppMsg> {
    let events_subscription = event::listen_with(|event, status, window_id| {
        match status {
            event::Status::Ignored => Some(AppMsg::IcedEvent(window_id, event)),
            event::Status::Captured => {
                match &event {
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: Key::Named(Named::Escape),
                        ..
                    }) => Some(AppMsg::IcedEvent(window_id, event)),
                    _ => None,
                }
            }
        }
    });

    #[allow(unused_mut)]
    let mut subscriptions = vec![
        Subscription::run(|| {
            stream::channel(10, async move |sender| {
                register_global_shortcut_listener(sender.clone());

                std::future::pending::<()>().await;

                unreachable!()
            })
            .map(AppMsg::HandleGlobalShortcut)
        }),
        events_subscription,
    ];

    #[cfg(target_os = "linux")]
    if !state.window.wayland {
        subscriptions.push(x11_linux_focus_change_subscription())
    }

    Subscription::batch(subscriptions)
}

impl AppModel {
    fn reset_window_state(&mut self) -> Task<AppMsg> {
        self.prompt = "".to_string();

        self.client_context.clear_all_inline_views();

        GlobalState::initial(&mut self.global_state)
    }

    fn open_plugin_view(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Task<AppMsg> {
        let msg = self
            .application_manager
            .request_render_view(plugin_id, entrypoint_id)
            .map(|action_shortcuts| AppMsg::OnOpenView { action_shortcuts })
            .unwrap_or_else(|err| AppMsg::ShowBackendError(err.into()));

        Task::done(msg)
    }

    fn close_plugin_view(&self, plugin_id: PluginId) -> Task<AppMsg> {
        self.application_manager.request_view_close(plugin_id);

        Task::none()
    }

    fn run_command(&self, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Task<AppMsg> {
        self.application_manager.run_command(plugin_id, entrypoint_id);

        Task::none()
    }

    fn run_generated_entrypoint(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        action_index: usize,
    ) -> Task<AppMsg> {
        self.application_manager
            .request_run_generated_entrypoint(plugin_id, entrypoint_id, action_index);

        Task::none()
    }

    fn handle_plugin_event(
        &mut self,
        widget_event: ComponentWidgetEvent,
        plugin_id: PluginId,
        render_location: UiRenderLocation,
    ) -> Task<AppMsg> {
        let application_manager = self.application_manager.clone();

        let event = self
            .client_context
            .handle_event(render_location, &plugin_id, widget_event.clone());

        if let Some(event) = event {
            match event {
                UiViewEvent::View {
                    widget_id,
                    event_name,
                    event_arguments,
                } => {
                    let msg = match widget_event {
                        ComponentWidgetEvent::ActionClick { .. } => AppMsg::ToggleActionPanel { keyboard: false },
                        _ => AppMsg::Noop,
                    };

                    application_manager.send_view_event(plugin_id, widget_id, event_name, event_arguments);

                    Task::done(msg)
                }
                UiViewEvent::Open { href } => {
                    application_manager.handle_open(href);

                    Task::none()
                }
                UiViewEvent::AppEvent { event } => Task::done(event),
            }
        } else {
            Task::none()
        }
    }

    fn handle_main_view_keyboard_event(
        &self,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        physical_key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> Task<AppMsg> {
        self.application_manager.handle_keyboard_event(
            plugin_id,
            entrypoint_id,
            KeyboardEventOrigin::MainView,
            physical_key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        );

        Task::none()
    }

    fn handle_plugin_view_keyboard_event(
        &self,
        physical_key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> Task<AppMsg> {
        let (plugin_id, entrypoint_id) = {
            (
                self.client_context.get_view_plugin_id(),
                self.client_context.get_view_entrypoint_id(),
            )
        };

        self.application_manager.handle_keyboard_event(
            plugin_id,
            entrypoint_id,
            KeyboardEventOrigin::PluginView,
            physical_key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        );

        Task::none()
    }

    fn handle_inline_plugin_view_keyboard_event(
        &self,
        physical_key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    ) -> Task<AppMsg> {
        let (plugin_id, entrypoint_id) = {
            match self.client_context.get_first_inline_view_container() {
                None => return Task::none(),
                Some(container) => (container.get_plugin_id(), container.get_entrypoint_id()),
            }
        };

        self.application_manager.handle_keyboard_event(
            plugin_id,
            entrypoint_id,
            KeyboardEventOrigin::PluginView,
            physical_key,
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta,
        );

        Task::none()
    }

    fn search(&self, new_prompt: String, render_inline_view: bool) -> Task<AppMsg> {
        let msg = self
            .application_manager
            .search(&new_prompt, render_inline_view)
            .map(|search_result| AppMsg::SetSearchResults(search_result))
            .unwrap_or_else(|err| AppMsg::ShowBackendError(err.into()));

        Task::done(msg)
    }

    fn open_settings_window_preferences(
        &self,
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
    ) -> Task<AppMsg> {
        self.application_manager
            .open_settings_window_preferences(plugin_id, entrypoint_id);

        Task::none()
    }

    fn inline_view_shortcuts(&self) -> Task<AppMsg> {
        let result = self
            .application_manager
            .inline_view_shortcuts()
            .map(|shortcuts| AppMsg::InlineViewShortcuts { shortcuts })
            .unwrap_or_else(|err| AppMsg::ShowBackendError(err.into()));

        Task::done(result)
    }

    fn handle_shortcut_key(
        &mut self,
        physical_key: Physical,
        modifiers: Modifiers,
        text: Option<SmolStr>,
    ) -> Task<AppMsg> {
        let Physical::Code(physical_key) = physical_key else {
            return Task::none();
        };

        match &mut self.global_state {
            GlobalState::MainView {
                sub_state,
                search_field_id,
                focused_search_result,
                ..
            } => {
                match sub_state {
                    MainViewState::None => {
                        match physical_key_model(physical_key, modifiers) {
                            Some(PhysicalShortcut {
                                physical_key: PhysicalKey::Comma,
                                modifier_shift: false,
                                modifier_control: cfg!(any(target_os = "linux", target_os = "windows")),
                                modifier_alt: false,
                                modifier_meta: cfg!(target_os = "macos"),
                            }) => {
                                self.application_manager.handle_open_settings_window();

                                Task::none()
                            }
                            Some(PhysicalShortcut {
                                physical_key: PhysicalKey::KeyK,
                                modifier_shift: false,
                                modifier_control: false,
                                modifier_alt: true,
                                modifier_meta: false,
                            }) => Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true }),
                            Some(PhysicalShortcut {
                                physical_key,
                                modifier_shift,
                                modifier_control,
                                modifier_alt,
                                modifier_meta,
                            }) => {
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
                                                modifier_meta,
                                            )
                                        } else {
                                            Task::none()
                                        }
                                    } else {
                                        self.handle_inline_plugin_view_keyboard_event(
                                            physical_key,
                                            modifier_shift,
                                            modifier_control,
                                            modifier_alt,
                                            modifier_meta,
                                        )
                                    }
                                } else {
                                    AppModel::append_prompt(&mut self.prompt, text, search_field_id.clone(), modifiers)
                                }
                            }
                            _ => AppModel::append_prompt(&mut self.prompt, text, search_field_id.clone(), modifiers),
                        }
                    }
                    MainViewState::SearchResultActionPanel { .. } => {
                        match physical_key_model(physical_key, modifiers) {
                            Some(PhysicalShortcut {
                                physical_key: PhysicalKey::KeyK,
                                modifier_shift: false,
                                modifier_control: false,
                                modifier_alt: true,
                                modifier_meta: false,
                            }) => Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true }),
                            Some(PhysicalShortcut {
                                physical_key,
                                modifier_shift,
                                modifier_control,
                                modifier_alt,
                                modifier_meta,
                            }) => {
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
                                                modifier_meta,
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
                            _ => Task::none(),
                        }
                    }
                    MainViewState::InlineViewActionPanel { .. } => {
                        match physical_key_model(physical_key, modifiers) {
                            Some(PhysicalShortcut {
                                physical_key: PhysicalKey::KeyK,
                                modifier_shift: false,
                                modifier_control: false,
                                modifier_alt: true,
                                modifier_meta: false,
                            }) => Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true }),
                            Some(PhysicalShortcut {
                                physical_key,
                                modifier_shift,
                                modifier_control,
                                modifier_alt,
                                modifier_meta,
                            }) => {
                                if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                                    self.handle_inline_plugin_view_keyboard_event(
                                        physical_key,
                                        modifier_shift,
                                        modifier_control,
                                        modifier_alt,
                                        modifier_meta,
                                    )
                                } else {
                                    Task::none()
                                }
                            }
                            _ => Task::none(),
                        }
                    }
                }
            }
            GlobalState::ErrorView { .. } => Task::none(),
            GlobalState::PluginView { sub_state, .. } => {
                match physical_key_model(physical_key, modifiers) {
                    Some(PhysicalShortcut {
                        physical_key: PhysicalKey::KeyK,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: true,
                        modifier_meta: false,
                    }) => Task::perform(async {}, |_| AppMsg::ToggleActionPanel { keyboard: true }),
                    Some(PhysicalShortcut {
                        physical_key,
                        modifier_shift,
                        modifier_control,
                        modifier_alt,
                        modifier_meta,
                    }) => {
                        if modifier_shift || modifier_control || modifier_alt || modifier_meta {
                            self.handle_plugin_view_keyboard_event(
                                physical_key,
                                modifier_shift,
                                modifier_control,
                                modifier_alt,
                                modifier_meta,
                            )
                        } else {
                            match sub_state {
                                PluginViewState::None => {
                                    match text {
                                        None => Task::none(),
                                        Some(text) => self.client_context.append_text(text.as_str()),
                                    }
                                }
                                PluginViewState::ActionPanel { .. } => Task::none(),
                            }
                        }
                    }
                    _ => Task::none(),
                }
            }
            GlobalState::PendingPluginView { .. } => Task::none(),
        }
    }
}

// these are needed to force focus the text_input in main search view when
// the window is opened but text_input not focused
impl AppModel {
    fn append_prompt(
        prompt: &mut String,
        value: Option<SmolStr>,
        search_field_id: text_input::Id,
        modifiers: Modifiers,
    ) -> Task<AppMsg> {
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
                None => Task::none(),
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
