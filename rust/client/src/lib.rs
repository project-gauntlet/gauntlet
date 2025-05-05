use gauntlet_common::dirs::Dirs;
use gauntlet_common::rpc::backend_api::BackendForCliApi;
use gauntlet_common::rpc::backend_api::BackendForCliApiProxy;
use gauntlet_common::rpc::backend_api::BackendForFrontendApiRequestData;
use gauntlet_common::rpc::backend_api::BackendForFrontendApiResponseData;
use gauntlet_common::rpc::backend_api::GrpcBackendApi;
use gauntlet_common::rpc::frontend_api::FrontendApiRequestData;
use gauntlet_common::rpc::frontend_api::FrontendApiResponseData;
use gauntlet_utils::channel::RequestReceiver;
use gauntlet_utils::channel::RequestSender;

use crate::ui::GauntletComplexTheme;

pub mod global_shortcut;
pub(crate) mod model;
pub(crate) mod ui;

pub fn start_client(
    minimized: bool,
    frontend_receiver: RequestReceiver<FrontendApiRequestData, FrontendApiResponseData>,
    backend_sender: RequestSender<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
) {
    ui::run(minimized, frontend_receiver, backend_sender);
}

pub fn open_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let result = GrpcBackendApi::new().await;

            match result {
                Ok(backend_api) => {
                    let backend_api = BackendForCliApiProxy::new(backend_api);

                    tracing::info!("Server is already running, opening window...");

                    backend_api.show_window().await.expect("Unknown error")
                }
                Err(_) => {
                    tracing::error!("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                }
            }
        })
}

pub fn open_settings_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let result = GrpcBackendApi::new().await;

            match result {
                Ok(backend_api) => {
                    let backend_api = BackendForCliApiProxy::new(backend_api);

                    backend_api.show_settings_window().await.expect("Unknown error")
                }
                Err(_) => {
                    tracing::error!("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                }
            }
        })
}
