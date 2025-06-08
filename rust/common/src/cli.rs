use gauntlet_utils::channel::RequestError;

use crate::model::EntrypointId;
use crate::model::PluginId;
use crate::rpc::backend_api::BackendForCliApi;
use crate::rpc::backend_api::BackendForCliApiProxy;
use crate::rpc::backend_api::GrpcBackendApi;

pub fn is_server_running() -> bool {
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
