use client::{open_window, start_client};
use common::rpc::backend_api::BackendApi;
use common::rpc::backend_server::start_backend_server;
use crate::plugins::ApplicationManager;
use crate::rpc::BackendServerImpl;
use crate::search::SearchIndex;

pub mod rpc;
pub(in crate) mod search;
pub(in crate) mod plugins;
pub(in crate) mod model;
mod dirs;

const SETTINGS_ENV: &'static str = "GAUNTLET_INTERNAL_SETTINGS";

pub fn start(minimized: bool) {
    #[cfg(feature = "scenario_runner")]
    run_scenario_runner();

    #[cfg(not(feature = "scenario_runner"))]
    {
        if is_server_running() {
            open_window()
        } else {
            std::thread::spawn(|| {
                start_server();
            });

            start_client(minimized)
        }
    }
}

#[cfg(feature = "scenario_runner")]
fn run_scenario_runner() {
    let runner_type = std::env::var("GAUNTLET_SCENARIO_RUNNER_TYPE")
        .expect("Unable to read GAUNTLET_SCENARIO_RUNNER_TYPE");

    match runner_type.as_str() {
        "screenshot_gen" => {
            std::thread::spawn(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("unable to start server tokio runtime")
                    .block_on(async {
                        if let Err(err) = scenario_runner::run_screenshot_gen_backend().await {
                            tracing::error!("error running screenshot gen backend: {:?}", err);
                        }
                    });
            });

            start_client(false)
        }
        "scenario_runner" => {
            start_server()
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


fn start_server() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("unable to start server tokio runtime")
        .block_on(async {
            run_server().await
        })
        .unwrap();
}

async fn run_server() -> anyhow::Result<()> {
    let search_index = SearchIndex::create_index()?;
    let mut application_manager = ApplicationManager::create(search_index.clone()).await?;

    application_manager.clear_all_icon_cache_dir()?;

    #[cfg(not(feature = "scenario_runner"))]
    if let Err(err) = application_manager.load_builtin_plugins().await {
        tracing::error!("error loading bundled plugin(s): {:?}", err);
    }

    #[cfg(feature = "scenario_runner")]
    tokio::spawn(async {
        if let Err(err) = scenario_runner::run_scenario_runner_frontend().await {
            tracing::error!("error running scenario runner frontend: {:?}", err);
        }
    });

    #[cfg(not(any(feature = "scenario_runner", feature = "release")))]
    {
        let plugin_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../dev_plugin/dist").to_owned();
        let plugin_path = std::fs::canonicalize(plugin_path).expect("valid path");
        let plugin_path = plugin_path.to_str().expect("valid utf8");

        if let Err(err) = application_manager.save_local_plugin(plugin_path).await {
            tracing::error!("error loading dev plugin: {:?}", err);
        }
    }

    application_manager.reload_all_plugins().await?; // TODO do not fail here ?

    tokio::spawn(async {
        start_backend_server(Box::new(BackendServerImpl::new(search_index, application_manager))).await
    });

    std::future::pending::<()>().await;

    Ok(())
}
