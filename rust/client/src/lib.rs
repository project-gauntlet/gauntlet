use tonic::Request;

use common::rpc::rpc_frontend_client::RpcFrontendClient;
use common::rpc::RpcShowWindowRequest;

pub(in crate) mod rpc;
pub(in crate) mod ui;
pub(in crate) mod model;

pub fn start_client() {
    ui::run();
}

pub fn open_window() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            let mut frontend_client = RpcFrontendClient::connect("http://127.0.0.1:42321").await?;

            frontend_client.show_window(Request::new(RpcShowWindowRequest::default())).await?;

            anyhow::Ok(())
        })
        .unwrap();
}
