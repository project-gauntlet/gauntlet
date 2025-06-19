mod model;
pub mod plugins;
pub mod rpc;
mod search;

pub use global_hotkey;

pub const PLUGIN_CONNECT_ENV: &'static str = "__GAUNTLET_INTERNAL_PLUGIN_CONNECT__";
pub const PLUGIN_UUID_ENV: &'static str = "__GAUNTLET_INTERNAL_PLUGIN_UUID__";

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

            start_app(false, frontend_receiver, backend_sender);
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
