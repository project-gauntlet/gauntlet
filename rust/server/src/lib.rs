use std::backtrace::Backtrace;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use gauntlet_client::start_client;
use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PluginId;
use gauntlet_common::rpc::backend_api::BackendForCliApi;
use gauntlet_common::rpc::backend_api::BackendForCliApiProxy;
use gauntlet_common::rpc::backend_api::BackendForFrontendApiRequestData;
use gauntlet_common::rpc::backend_api::BackendForFrontendApiResponseData;
use gauntlet_common::rpc::backend_api::GrpcBackendApi;
use gauntlet_common::rpc::backend_api::handle_proxy_message;
use gauntlet_common::rpc::backend_server::start_backend_server;
use gauntlet_common::rpc::frontend_api::FrontendApiRequestData;
use gauntlet_common::rpc::frontend_api::FrontendApiResponseData;
use gauntlet_utils::channel::RequestError;
use gauntlet_utils::channel::RequestReceiver;
use gauntlet_utils::channel::RequestSender;
use gauntlet_utils::channel::channel;
use vergen_pretty::vergen_pretty_env;

use crate::plugins::ApplicationManager;
use crate::rpc::BackendServerImpl;

pub(crate) mod model;
pub mod plugins;
pub mod rpc;
pub(crate) mod search;

const PLUGIN_CONNECT_ENV: &'static str = "__GAUNTLET_INTERNAL_PLUGIN_CONNECT__";
const PLUGIN_UUID_ENV: &'static str = "__GAUNTLET_INTERNAL_PLUGIN_UUID__";

pub fn start(#[cfg(not(feature = "scenario_runner"))] minimized: bool) {
    register_panic_hook(std::env::var(PLUGIN_UUID_ENV).ok());

    if let Ok(socket_name) = std::env::var(PLUGIN_CONNECT_ENV) {
        gauntlet_plugin_runtime::run_plugin_runtime(socket_name);

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
            gauntlet_client::open_window()
        } else {
            let (frontend_sender, frontend_receiver) = channel::<FrontendApiRequestData, FrontendApiResponseData>();
            let (backend_sender, backend_receiver) =
                channel::<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>();

            std::thread::Builder::new()
                .name("gauntlet-server".to_string())
                .spawn(|| {
                    start_server(frontend_sender, backend_receiver);
                })
                .expect("failed to spawn thread");

            start_client(minimized, frontend_receiver, backend_sender)
        }
    }
}

pub fn run_action(plugin_id: String, entrypoint_id: String, action_id: String) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let result = GrpcBackendApi::new().await;

            match result {
                Ok(backend_api) => {
                    let backend_api = BackendForCliApiProxy::new(backend_api);

                    let plugin_id = PluginId::from_string(plugin_id);
                    let entrypoint_id = EntrypointId::from_string(entrypoint_id);

                    if let Err(err) = backend_api.run_action(plugin_id, entrypoint_id, action_id).await {
                        match err {
                            RequestError::Timeout => {
                                tracing::error!("Timeout occurred when handling command");
                            }
                            RequestError::Other { display: value } => {
                                tracing::error!("Error occurred when handling command: {}", value);
                            }
                            RequestError::OtherSideWasDropped => {
                                tracing::error!("Error occurred when handling command: Other side was dropped");
                            }
                        }
                    }
                }
                Err(_) => {
                    tracing::error!("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                }
            }
        })
}

#[cfg(feature = "scenario_runner")]
fn run_scenario_runner() {
    let runner_type =
        std::env::var("GAUNTLET_SCENARIO_RUNNER_TYPE").expect("Unable to read GAUNTLET_SCENARIO_RUNNER_TYPE");

    match runner_type.as_str() {
        "screenshot_gen" => {
            let (frontend_sender, frontend_receiver) = channel::<FrontendApiRequestData, FrontendApiResponseData>();
            let (backend_sender, backend_receiver) =
                channel::<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>();

            std::thread::spawn(|| {
                let theme = crate::plugins::theme::BundledThemes::new().unwrap();

                start_mock_server(frontend_sender, backend_receiver, theme.macos_dark_theme)
            });

            start_client(false, frontend_receiver, backend_sender);
        }
        "scenario_runner" => {
            let (frontend_sender, frontend_receiver) = channel::<FrontendApiRequestData, FrontendApiResponseData>();
            let (backend_sender, backend_receiver) =
                channel::<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>();

            std::thread::spawn(|| start_server(frontend_sender, backend_receiver));

            start_frontend_mock(frontend_receiver, backend_sender);
        }
        _ => panic!("unknown type"),
    }
}

#[cfg(not(feature = "scenario_runner"))]
fn is_server_running() -> bool {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let test_fn = || {
                async {
                    let api = GrpcBackendApi::new().await?;

                    let api = BackendForCliApiProxy::new(api);

                    api.ping().await?;

                    anyhow::Ok(())
                }
            };

            test_fn().await.is_ok()
        })
}

fn start_server(
    request_sender: RequestSender<FrontendApiRequestData, FrontendApiResponseData>,
    backend_receiver: RequestReceiver<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async { run_server(request_sender, backend_receiver).await })
        .unwrap();
}

#[cfg(feature = "scenario_runner")]
fn start_mock_server(
    request_sender: RequestSender<FrontendApiRequestData, FrontendApiResponseData>,
    backend_receiver: RequestReceiver<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
    theme: gauntlet_common::model::UiTheme,
) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            gauntlet_scenario_runner::run_scenario_runner_mock_server(request_sender, backend_receiver, theme).await
        })
        .unwrap();
}

#[cfg(feature = "scenario_runner")]
fn start_frontend_mock(
    request_receiver: RequestReceiver<FrontendApiRequestData, FrontendApiResponseData>,
    backend_sender: RequestSender<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
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

async fn run_server(
    frontend_sender: RequestSender<FrontendApiRequestData, FrontendApiResponseData>,
    mut backend_receiver: RequestReceiver<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
) -> anyhow::Result<()> {
    let application_manager = ApplicationManager::create(frontend_sender).await?;

    let application_manager = Arc::new(application_manager);

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

    application_manager.reload_all_plugins().await?;

    tokio::spawn({
        let application_manager = application_manager.clone();

        async move {
            start_backend_server(
                Box::new(BackendServerImpl::new(application_manager.clone())),
                Box::new(BackendServerImpl::new(application_manager.clone())),
                Box::new(BackendServerImpl::new(application_manager.clone())),
            )
            .await
        }
    });

    loop {
        let (request_data, responder) = backend_receiver.recv().await;

        let response_data = handle_proxy_message(request_data, application_manager.as_ref()).await;

        responder.respond(response_data);
    }
}

fn register_panic_hook(plugin_runtime: Option<String>) {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "full");
    };

    let dirs = Dirs::new();

    let crash_file = match plugin_runtime {
        None => dirs.server_crash_log_file(),
        Some(plugin_uuid) => dirs.plugin_crash_log_file(&plugin_uuid),
    };

    let _ = std::fs::remove_file(&crash_file);

    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info.payload();

        let payload = if let Some(&s) = payload.downcast_ref::<&'static str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<dyn Any>"
        };

        let location = panic_info.location().map(|l| l.to_string());
        let backtrace = Backtrace::capture();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis().to_string())
            .unwrap_or("Unknown".to_string());

        let content = format!(
            "Panic on {}\nPayload: {}\nLocation: {:?}\nBacktrace:\n{}",
            now, payload, location, backtrace
        );

        let crash_file = File::options().create(true).append(true).open(&crash_file);

        if let Ok(mut crash_file) = crash_file {
            let _ = crash_file.write_all(content.as_bytes());
        }

        eprintln!("{}", content);

        exit(101); // poor man's abort on panic because actual setting makes v8 linking fail
    }));
}
