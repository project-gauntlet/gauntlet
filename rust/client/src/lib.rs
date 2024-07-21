use common::model::{UiRequestData, UiResponseData};
use common::rpc::backend_api::BackendApi;
use utils::channel::RequestReceiver;

pub(in crate) mod ui;
pub(in crate) mod model;

pub fn start_client(minimized: bool, request_receiver: RequestReceiver<UiRequestData, UiResponseData>) {
    ui::run(minimized, request_receiver);
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
