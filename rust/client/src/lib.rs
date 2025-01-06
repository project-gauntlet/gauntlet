use gauntlet_common::dirs::Dirs;
use gauntlet_common::model::{BackendRequestData, BackendResponseData, UiRequestData, UiResponseData};
use gauntlet_common::rpc::backend_api::BackendApi;
use gauntlet_utils::channel::{RequestReceiver, RequestSender};
use crate::ui::GauntletComplexTheme;

pub(in crate) mod ui;
pub(in crate) mod model;
pub mod global_shortcut;

pub fn start_client(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) {
    ui::run(minimized, frontend_receiver, backend_sender);
}

pub fn open_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let result = BackendApi::new().await;

            match result {
                Ok(mut backend_api) => {
                    tracing::info!("Server is already running, opening window...");

                    backend_api.show_window()
                        .await
                        .expect("Unknown error")
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
            let result = BackendApi::new().await;

            match result {
                Ok(mut backend_api) => {
                    backend_api.show_settings_window()
                        .await
                        .expect("Unknown error")
                }
                Err(_) => {
                    tracing::error!("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                }
            }
        })
}

