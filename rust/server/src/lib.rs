use tonic::transport::Server;
use client::start_client;
use common::rpc::rpc_backend_server::RpcBackendServer;
use crate::rpc::RpcBackendServerImpl;
use crate::plugins::ApplicationManager;
use crate::search::SearchIndex;

pub mod rpc;
pub(in crate) mod search;
pub(in crate) mod plugins;
pub(in crate) mod model;
mod dirs;

const FRONTEND_ENV: &'static str = "GAUNTLET_INTERNAL_FRONTEND";

pub fn start_server() {
    if std::env::var(FRONTEND_ENV).is_ok() {
        start_client()
    } else {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("unable to start server tokio runtime")
            .block_on(async {
                run_server().await
            })
            .unwrap();
    }
}

async fn run_server() -> anyhow::Result<()> {
    let search_index = SearchIndex::create_index()?;
    let mut application_manager = ApplicationManager::create(search_index.clone()).await?;

    if let Err(err) = application_manager.load_builtin_plugins().await {
        tracing::error!("error loading bundled plugin(s): {:?}", err);
    }

    if cfg!(feature = "dev") {
        let plugin_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_plugin/dist").to_owned();
        let plugin_path = std::fs::canonicalize(plugin_path).expect("valid path");
        let plugin_path = plugin_path.to_str().expect("valid utf8");

        if let Err(err) = application_manager.save_local_plugin(plugin_path).await {
            tracing::error!("error loading dev plugin: {:?}", err);
        }
    }

    if !cfg!(feature = "dev") {
        std::process::Command::new(std::env::current_exe()?)
            .args(["server"])
            .env(FRONTEND_ENV, "")
            .spawn()
            .expect("failed to execute client process");
    }

    application_manager.reload_all_plugins().await?; // TODO do not fail here ?

    tokio::spawn(async {
        let addr = "127.0.0.1:42320".parse().unwrap();

        Server::builder()
            .add_service(RpcBackendServer::new(RpcBackendServerImpl { search_index, application_manager }))
            .serve(addr)
            .await
            .expect("unable to start backend server");
    });

    std::future::pending::<()>().await;

    Ok(())
}
