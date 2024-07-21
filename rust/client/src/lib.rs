use common::model::{BackendRequestData, BackendResponseData, UiRequestData, UiResponseData};
use common::rpc::backend_api::BackendApi;
use utils::channel::{RequestReceiver, RequestSender};

pub(in crate) mod ui;
pub(in crate) mod model;

pub fn start_client(
    minimized: bool,
    frontend_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>
) {
    ui::run(minimized, frontend_receiver, backend_sender);
}

pub fn open_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let mut backend_api = BackendApi::new().await?;

            backend_api.show_window().await?;

            anyhow::Ok(())
        })
        .unwrap();
}
