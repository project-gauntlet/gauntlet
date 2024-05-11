use common::rpc::frontend_api::FrontendApi;

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
            let mut frontend_client = FrontendApi::new().await?;

            frontend_client.show_window().await?;

            anyhow::Ok(())
        })
        .unwrap();
}
