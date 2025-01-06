use std::rc::Rc;
use std::sync::Arc;
use vergen_pretty::vergen_pretty_env;
use gauntlet_client::{open_window, start_client};
use gauntlet_common::model::{BackendRequestData, BackendResponseData, UiRequestData, UiResponseData};
use gauntlet_common::rpc::backend_api::BackendApi;
use gauntlet_common::rpc::backend_server::start_backend_server;
use gauntlet_common::{settings_env_data_from_string, settings_env_data_to_string, SettingsEnvData};
use gauntlet_plugin_runtime::run_plugin_runtime;
use gauntlet_utils::channel::{channel, RequestReceiver, RequestSender};
use crate::plugins::ApplicationManager;
use crate::rpc::BackendServerImpl;
use crate::search::SearchIndex;

pub mod rpc;
pub(in crate) mod search;
pub(in crate) mod plugins;
pub(in crate) mod model;

const SETTINGS_ENV: &'static str = "GAUNTLET_INTERNAL_SETTINGS";
const PLUGIN_RUNTIME_ENV: &'static str = "GAUNTLET_INTERNAL_PLUGIN_RUNTIME";

pub fn start(minimized: bool) {
    if let Ok(socket_name) = std::env::var(PLUGIN_RUNTIME_ENV) {
        run_plugin_runtime(socket_name);

        return;
    }

    tracing::info!("Gauntlet Build Information:");
    for (name, value) in vergen_pretty_env!() {
        if let Some(value) = value {
            tracing::info!("{}: {}", name, value);
        }
    }

    #[cfg(feature = "scenario_runner")]
    run_scenario_runner();

    #[cfg(not(feature = "scenario_runner"))]
    {
        if is_server_running() {
            open_window()
        } else {
            let (frontend_sender, frontend_receiver) = channel::<UiRequestData, UiResponseData>();
            let (backend_sender, backend_receiver) = channel::<BackendRequestData, BackendResponseData>();

            std::thread::spawn(|| {
                start_server(frontend_sender, backend_receiver);
            });

            start_client(minimized, frontend_receiver, backend_sender)
        }
    }
}

#[cfg(feature = "scenario_runner")]
fn run_scenario_runner() {
    let runner_type = std::env::var("GAUNTLET_SCENARIO_RUNNER_TYPE")
        .expect("Unable to read GAUNTLET_SCENARIO_RUNNER_TYPE");

    match runner_type.as_str() {
        "screenshot_gen" => {
            let (frontend_sender, frontend_receiver) = channel::<UiRequestData, UiResponseData>();
            let (backend_sender, backend_receiver) = channel::<BackendRequestData, BackendResponseData>();

            start_client(false, frontend_receiver, backend_sender);

            drop(frontend_sender);
            drop(backend_receiver);
        }
        "scenario_runner" => {
            let (frontend_sender, frontend_receiver) = channel::<UiRequestData, UiResponseData>();
            let (backend_sender, backend_receiver) = channel::<BackendRequestData, BackendResponseData>();

            std::thread::spawn(|| {
                start_server(frontend_sender, backend_receiver)
            });

            start_frontend_mock(frontend_receiver, backend_sender)
        }
        _ => panic!("unknown type")
    }
}


fn is_server_running() -> bool {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let test_fn = || async {
                let mut api = BackendApi::new().await?;

                api.ping().await?;

                anyhow::Ok(())
            };

            test_fn().await.is_ok()
        })
}

fn start_server(request_sender: RequestSender<UiRequestData, UiResponseData>, backend_receiver: RequestReceiver<BackendRequestData, BackendResponseData>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            run_server(request_sender, backend_receiver).await
        })
        .unwrap();
}

#[cfg(feature = "scenario_runner")]
fn start_frontend_mock(
    request_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>
) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start frontend mock tokio runtime")
        .block_on(async {
            gauntlet_scenario_runner::run_scenario_runner_frontend_mock(request_receiver, backend_sender).await
        })
        .unwrap();
}

async fn run_server(frontend_sender: RequestSender<UiRequestData, UiResponseData>, mut backend_receiver: RequestReceiver<BackendRequestData, BackendResponseData>) -> anyhow::Result<()> {
    let application_manager = ApplicationManager::create(frontend_sender).await?;

    let mut application_manager = Arc::new(application_manager);

    application_manager.clear_all_icon_cache_dir()?;

    #[cfg(not(feature = "scenario_runner"))]
    if let Err(err) = application_manager.load_bundled_plugins().await {
        tracing::error!("error loading bundled plugin(s): {:?}", err);
    }

    #[cfg(not(any(feature = "scenario_runner", feature = "release")))]
    {
        let plugin_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_plugin").to_owned();
        let plugin_path = std::fs::canonicalize(plugin_path).expect("valid path");
        let plugin_path = plugin_path.to_str().expect("valid utf8");

        if let Err(err) = application_manager.save_local_plugin(plugin_path).await {
            tracing::error!("error loading dev plugin: {:?}", err);
        }
    }

    application_manager.reload_all_plugins().await?; // TODO do not fail here ?

    tokio::spawn({
        let application_manager = application_manager.clone();

        async move {
            start_backend_server(Box::new(BackendServerImpl::new(application_manager.clone()))).await
        }
    });

    loop {
        let (request_data, responder) = backend_receiver.recv().await;

        let response_data = handle_request(application_manager.clone(), request_data)
            .await
            .unwrap(); // TODO error handling

        responder.respond(response_data);
    }
}

async fn handle_request(application_manager: Arc<ApplicationManager>, request_data: BackendRequestData) -> anyhow::Result<BackendResponseData> {
    let response_data = match request_data {
        BackendRequestData::Setup => {
            let data = application_manager.setup_data().await?;

            BackendResponseData::SetupData {
                data,
            }
        }
        BackendRequestData::SetupResponse { global_shortcut_error } => {
            application_manager.setup_response(global_shortcut_error).await?;

            BackendResponseData::Nothing
        }
        BackendRequestData::Search { text, render_inline_view } => {
            let results = application_manager.search(&text, render_inline_view)?;

            BackendResponseData::Search {
                results,
            }
        }
        BackendRequestData::RequestViewRender { plugin_id, entrypoint_id } => {
            let shortcuts = application_manager.handle_render_view(plugin_id.clone(), entrypoint_id.clone())
                .await?;

            BackendResponseData::RequestViewRender {
                shortcuts
            }
        }
        BackendRequestData::RequestViewClose { plugin_id } => {
            application_manager.handle_view_close(plugin_id);

            BackendResponseData::Nothing
        }
        BackendRequestData::RequestRunCommand { plugin_id, entrypoint_id } => {
            application_manager.handle_run_command(plugin_id, entrypoint_id)
                .await;

            BackendResponseData::Nothing
        }
        BackendRequestData::RequestRunGeneratedCommand { plugin_id, entrypoint_id, action_index } => {
            application_manager.handle_run_generated_command(plugin_id, entrypoint_id, action_index)
                .await;

            BackendResponseData::Nothing
        }
        BackendRequestData::SendViewEvent { plugin_id, widget_id, event_name, event_arguments } => {
            application_manager.handle_view_event(plugin_id, widget_id, event_name, event_arguments);

            BackendResponseData::Nothing
        }
        BackendRequestData::SendKeyboardEvent { plugin_id, entrypoint_id, origin, key, modifier_shift, modifier_control, modifier_alt, modifier_meta } => {
            application_manager.handle_keyboard_event(
                plugin_id,
                entrypoint_id,
                origin,
                key,
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            );

            BackendResponseData::Nothing
        }
        BackendRequestData::SendOpenEvent { plugin_id: _, href } => {
            application_manager.handle_open(href);

            BackendResponseData::Nothing
        }
        BackendRequestData::OpenSettingsWindow => {
            application_manager.handle_open_settings_window();

            BackendResponseData::Nothing
        }
        BackendRequestData::OpenSettingsWindowPreferences { plugin_id, entrypoint_id } => {
            application_manager.handle_open_settings_window_preferences(plugin_id, entrypoint_id);

            BackendResponseData::Nothing
        }
        BackendRequestData::InlineViewShortcuts => {
            let shortcuts = application_manager.inline_view_shortcuts()
                .await?;

            BackendResponseData::InlineViewShortcuts { shortcuts }
        }
    };

    Ok(response_data)
}
