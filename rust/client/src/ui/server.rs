use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use gauntlet_common::model::UiSetupData;
use gauntlet_common::rpc::frontend_api::FrontendApiRequestData;
use gauntlet_common::rpc::frontend_api::FrontendApiResponseData;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApiProxy;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApiRequestData;
use gauntlet_common::rpc::server_grpc_api::ServerGrpcApiResponseData;
use gauntlet_server::global_hotkey::GlobalHotKeyManager;
use gauntlet_server::plugins::ApplicationManager;
use gauntlet_server::rpc::run_grpc_server;
use gauntlet_utils::channel::Responder;
use gauntlet_utils::channel::channel;
use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::stream;
use tokio::sync::RwLock as TokioRwLock;

use crate::ui::AppModel;
use crate::ui::AppMsg;
#[cfg(target_os = "linux")]
use crate::ui::wayland::layer_shell_supported;
use crate::ui::windows::WindowActionMsg;

pub fn setup(
    #[cfg(target_os = "linux")] wayland: bool,
) -> (
    Arc<ApplicationManager>,
    Option<GlobalHotKeyManager>,
    UiSetupData,
    Task<AppMsg>,
) {
    let (frontend_sender, frontend_receiver) = channel::<FrontendApiRequestData, FrontendApiResponseData>();
    let (server_grpc_sender, server_grpc_receiver) = channel::<ServerGrpcApiRequestData, ServerGrpcApiResponseData>();

    #[cfg(target_os = "linux")]
    let layer_shell_supported = layer_shell_supported();
    #[cfg(not(target_os = "linux"))]
    let layer_shell_supported = false;

    let application_manager = ApplicationManager::create(frontend_sender, layer_shell_supported)
        .expect("Unable to setup application manager");

    let grpc_api = ServerGrpcApiProxy::new(server_grpc_sender);
    let frontend_receiver = Arc::new(TokioRwLock::new(frontend_receiver));
    let server_grpc_receiver = Arc::new(TokioRwLock::new(server_grpc_receiver));
    let application_manager = Arc::new(application_manager);

    let setup_data = application_manager.config().expect("Unable to setup");

    #[cfg(target_os = "linux")]
    let enable_global_hotkey_manager = !wayland || setup_data.wayland_use_legacy_x11_api;
    #[cfg(not(target_os = "linux"))]
    let enable_global_hotkey_manager = true;

    let global_hotkey_manager = if enable_global_hotkey_manager {
        let global_hotkey_manager = GlobalHotKeyManager::new().expect("Unable to setup shortcut manager");

        application_manager
            .setup_global_shortcuts(&global_hotkey_manager)
            .expect("Unable to setup");

        Some(global_hotkey_manager)
    } else {
        None
    };

    let mut tasks = vec![];

    tasks.push(Task::future(async move { run_grpc_server(grpc_api).await }).discard());

    tasks.push(Task::stream(stream::channel(10, |mut sender| {
        async move {
            let mut frontend_receiver = frontend_receiver.write().await;

            loop {
                let (request_data, responder) = frontend_receiver.recv().await;

                request_loop(request_data, &mut sender, responder).await;
            }
        }
    })));

    tasks.push(Task::stream(stream::channel(10, |mut sender: mpsc::Sender<AppMsg>| {
        async move {
            let mut server_grpc_receiver = server_grpc_receiver.write().await;

            loop {
                let (request_data, responder) = server_grpc_receiver.recv().await;

                let app_msg = AppMsg::HandleServerRequest {
                    request_data: Arc::new(request_data),
                    responder: Arc::new(Mutex::new(Some(responder))),
                };

                let _ = sender.send(app_msg).await;
            }
        }
    })));

    (
        application_manager,
        global_hotkey_manager,
        setup_data,
        Task::batch(tasks),
    )
}

async fn request_loop(
    request_data: FrontendApiRequestData,
    sender: &mut mpsc::Sender<AppMsg>,
    responder: Responder<FrontendApiResponseData>,
) {
    let app_msg = {
        match request_data {
            FrontendApiRequestData::ReplaceView {
                plugin_id,
                plugin_name,
                entrypoint_id,
                entrypoint_name,
                render_location,
                top_level_view,
                container,
                data: images,
            } => {
                responder.respond(Ok(FrontendApiResponseData::ReplaceView { data: () }));

                AppMsg::RenderPluginUI {
                    plugin_id,
                    plugin_name,
                    entrypoint_id,
                    entrypoint_name,
                    render_location,
                    top_level_view,
                    container: Arc::new(container),
                    data: images,
                }
            }
            FrontendApiRequestData::ClearInlineView { plugin_id } => {
                responder.respond(Ok(FrontendApiResponseData::ClearInlineView { data: () }));

                AppMsg::ClearInlineView { plugin_id }
            }
            FrontendApiRequestData::ToggleWindow {} => {
                responder.respond(Ok(FrontendApiResponseData::ToggleWindow { data: () }));

                AppMsg::WindowAction(WindowActionMsg::ToggleWindow)
            }
            FrontendApiRequestData::HideWindow {} => {
                responder.respond(Ok(FrontendApiResponseData::HideWindow { data: () }));

                AppMsg::WindowAction(WindowActionMsg::HideWindow)
            }
            FrontendApiRequestData::ShowPreferenceRequiredView {
                plugin_id,
                entrypoint_id,
                plugin_preferences_required,
                entrypoint_preferences_required,
            } => {
                responder.respond(Ok(FrontendApiResponseData::ShowPreferenceRequiredView { data: () }));

                AppMsg::ShowPreferenceRequiredView {
                    plugin_id,
                    entrypoint_id,
                    plugin_preferences_required,
                    entrypoint_preferences_required,
                }
            }
            FrontendApiRequestData::ShowPluginErrorView {
                plugin_id,
                entrypoint_id,
                render_location: _,
            } => {
                responder.respond(Ok(FrontendApiResponseData::ShowPluginErrorView { data: () }));

                AppMsg::ShowPluginErrorView {
                    plugin_id,
                    entrypoint_id,
                }
            }
            FrontendApiRequestData::RequestSearchResultsUpdate {} => {
                responder.respond(Ok(FrontendApiResponseData::RequestSearchResultsUpdate { data: () }));

                AppMsg::UpdateSearchResults
            }
            FrontendApiRequestData::ShowHud { display } => {
                responder.respond(Ok(FrontendApiResponseData::ShowHud { data: () }));

                AppMsg::WindowAction(WindowActionMsg::ShowHud { display })
            }
            FrontendApiRequestData::UpdateLoadingBar {
                plugin_id,
                entrypoint_id,
                show,
            } => {
                responder.respond(Ok(FrontendApiResponseData::UpdateLoadingBar { data: () }));

                AppMsg::UpdateLoadingBar {
                    plugin_id,
                    entrypoint_id,
                    show,
                }
            }
            FrontendApiRequestData::SetTheme { theme } => {
                responder.respond(Ok(FrontendApiResponseData::SetTheme { data: () }));

                AppMsg::SetTheme { theme }
            }
            FrontendApiRequestData::SetWindowPositionMode { mode } => {
                responder.respond(Ok(FrontendApiResponseData::SetWindowPositionMode { data: () }));

                AppMsg::WindowAction(WindowActionMsg::SetWindowPositionMode { mode })
            }
            FrontendApiRequestData::OpenPluginView {
                plugin_id,
                entrypoint_id,
            } => {
                responder.respond(Ok(FrontendApiResponseData::OpenPluginView { data: () }));

                AppMsg::ShowNewView {
                    plugin_id,
                    entrypoint_id,
                }
            }
            FrontendApiRequestData::OpenGeneratedPluginView {
                plugin_id,
                entrypoint_id,
                action_index,
            } => {
                responder.respond(Ok(FrontendApiResponseData::OpenGeneratedPluginView { data: () }));

                AppMsg::ShowNewGeneratedView {
                    plugin_id,
                    entrypoint_id,
                    action_index,
                }
            }
        }
    };

    let _ = sender.send(app_msg).await;
}

pub fn handle_server_message(
    state: &mut AppModel,
    request_data: Arc<ServerGrpcApiRequestData>,
    responder: Arc<Mutex<Option<Responder<ServerGrpcApiResponseData>>>>,
) -> Task<AppMsg> {
    let responder = responder
        .lock()
        .expect("lock is poisoned")
        .take()
        .expect("there should always be a responder here");

    match request_data.deref() {
        ServerGrpcApiRequestData::ShowWindow {} => {
            responder.respond(Ok(ServerGrpcApiResponseData::ShowWindow { data: () }));

            Task::done(AppMsg::WindowAction(WindowActionMsg::ShowWindow))
        }
        ServerGrpcApiRequestData::ShowSettingsWindow {} => {
            responder.respond(Ok(ServerGrpcApiResponseData::ShowSettingsWindow { data: () }));

            state.application_manager.open_settings_window();

            Task::none()
        }
        ServerGrpcApiRequestData::RunAction {
            plugin_id,
            entrypoint_id,
            action_id,
        } => {
            let application_manager = state.application_manager.clone();
            let plugin_id = plugin_id.clone();
            let entrypoint_id = entrypoint_id.clone();
            let action_id = action_id.clone();

            Task::future(async move {
                let result = application_manager
                    .run_action(plugin_id, entrypoint_id, action_id)
                    .await
                    .map(|data| ServerGrpcApiResponseData::RunAction { data });

                responder.respond(result);

                AppMsg::Noop
            })
        }
        ServerGrpcApiRequestData::SaveLocalPlugin { path } => {
            let result = state
                .application_manager
                .save_local_plugin(&path)
                .map(|data| ServerGrpcApiResponseData::SaveLocalPlugin { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::Plugins {} => {
            let result = state
                .application_manager
                .plugins()
                .map(|data| ServerGrpcApiResponseData::Plugins { data });

            responder.respond(result.into());

            Task::none()
        }
        ServerGrpcApiRequestData::SetPluginState { plugin_id, enabled } => {
            let result = state
                .application_manager
                .set_plugin_state(plugin_id.clone(), *enabled)
                .map(|data| ServerGrpcApiResponseData::SetPluginState { data });

            responder.respond(result.into());

            Task::none()
        }
        ServerGrpcApiRequestData::SetEntrypointState {
            plugin_id,
            entrypoint_id,
            enabled,
        } => {
            let result = state
                .application_manager
                .set_entrypoint_state(plugin_id.clone(), entrypoint_id.clone(), *enabled)
                .map(|data| ServerGrpcApiResponseData::SetEntrypointState { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::SetGlobalShortcut { shortcut } => {
            let Some(global_hotkey_manager) = &state.global_hotkey_manager else {
                responder.respond(Err(anyhow!("Global hotkey manager is disabled")));
                return Task::none();
            };

            let result = state
                .application_manager
                .set_global_shortcut(global_hotkey_manager, shortcut.clone());

            responder.respond(Ok(ServerGrpcApiResponseData::SetGlobalShortcut { data: result }));

            Task::none()
        }
        ServerGrpcApiRequestData::GetGlobalShortcut {} => {
            let result = state
                .application_manager
                .get_global_shortcut()
                .map(|data| ServerGrpcApiResponseData::GetGlobalShortcut { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::SetGlobalEntrypointShortcut {
            plugin_id,
            entrypoint_id,
            shortcut,
        } => {
            let Some(global_hotkey_manager) = &state.global_hotkey_manager else {
                responder.respond(Err(anyhow!("Global hotkey manager is disabled")));
                return Task::none();
            };

            let result = state
                .application_manager
                .set_global_entrypoint_shortcut(
                    global_hotkey_manager,
                    plugin_id.clone(),
                    entrypoint_id.clone(),
                    shortcut.clone(),
                )
                .map(|data| ServerGrpcApiResponseData::SetGlobalEntrypointShortcut { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::GetGlobalEntrypointShortcuts {} => {
            let result = state
                .application_manager
                .get_global_entrypoint_shortcut()
                .map(|data| ServerGrpcApiResponseData::GetGlobalEntrypointShortcuts { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::SetEntrypointSearchAlias {
            plugin_id,
            entrypoint_id,
            alias,
        } => {
            let result = state
                .application_manager
                .set_entrypoint_search_alias(plugin_id.clone(), entrypoint_id.clone(), alias.clone())
                .map(|data| ServerGrpcApiResponseData::SetEntrypointSearchAlias { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::GetEntrypointSearchAliases {} => {
            let result = state
                .application_manager
                .get_entrypoint_search_aliases()
                .map(|data| ServerGrpcApiResponseData::GetEntrypointSearchAliases { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::SetTheme { theme } => {
            let application_manager = state.application_manager.clone();
            let theme = theme.clone();

            Task::future(async move {
                let result = application_manager
                    .set_theme(theme)
                    .await
                    .map(|data| ServerGrpcApiResponseData::SetTheme { data });

                responder.respond(result);

                AppMsg::Noop
            })
        }
        ServerGrpcApiRequestData::GetTheme {} => {
            let result = state
                .application_manager
                .get_theme()
                .map(|data| ServerGrpcApiResponseData::GetTheme { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::SetWindowPositionMode { mode } => {
            let application_manager = state.application_manager.clone();
            let mode = mode.clone();

            Task::future(async move {
                let result = application_manager
                    .set_window_position_mode(mode)
                    .await
                    .map(|data| ServerGrpcApiResponseData::SetWindowPositionMode { data });

                responder.respond(result);

                AppMsg::Noop
            })
        }
        ServerGrpcApiRequestData::GetWindowPositionMode {} => {
            let result = state
                .application_manager
                .get_window_position_mode()
                .map(|data| ServerGrpcApiResponseData::GetWindowPositionMode { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::SetPreferenceValue {
            plugin_id,
            entrypoint_id,
            preference_id,
            preference_value,
        } => {
            let result = state
                .application_manager
                .set_preference_value(
                    plugin_id.clone(),
                    entrypoint_id.clone(),
                    preference_id.clone(),
                    preference_value.clone(),
                )
                .map(|data| ServerGrpcApiResponseData::SetPreferenceValue { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::DownloadPlugin { plugin_id } => {
            let result = state
                .application_manager
                .download_plugin(plugin_id.clone())
                .map(|data| ServerGrpcApiResponseData::DownloadPlugin { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::DownloadStatus {} => {
            let data = state.application_manager.download_status();

            responder.respond(Ok(ServerGrpcApiResponseData::DownloadStatus { data }));

            Task::none()
        }
        ServerGrpcApiRequestData::RemovePlugin { plugin_id } => {
            let result = state
                .application_manager
                .remove_plugin(plugin_id.clone())
                .map(|data| ServerGrpcApiResponseData::RemovePlugin { data });

            responder.respond(result);

            Task::none()
        }
        ServerGrpcApiRequestData::WaylandGlobalShortcutsEnabled {} => {
            let result = state.application_manager.config().map(|data| {
                ServerGrpcApiResponseData::WaylandGlobalShortcutsEnabled {
                    data: data.wayland_use_legacy_x11_api,
                }
            });

            responder.respond(result);

            Task::none()
        }
    }
}
