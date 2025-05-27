mod assets;
mod clipboard;
mod component_model;
mod deno;
mod entrypoint_generators;
mod environment;
mod events;
mod logs;
mod model;
mod permissions;
mod plugin_data;
mod plugins;
mod preferences;
mod search;
mod ui;

use std::ops::Deref;

use anyhow::anyhow;
use gauntlet_common_plugin_runtime::JsMessageSide;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiProxy;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiRequestData;
use gauntlet_common_plugin_runtime::api::BackendForPluginRuntimeApiResponseData;
use gauntlet_common_plugin_runtime::model::JsEvent;
use gauntlet_common_plugin_runtime::model::JsInit;
use gauntlet_common_plugin_runtime::model::JsMessage;
use gauntlet_common_plugin_runtime::model::JsPluginRuntimeMessage;
use gauntlet_common_plugin_runtime::recv_message;
use gauntlet_common_plugin_runtime::send_message;
use gauntlet_utils::channel::RequestReceiver;
use interprocess::local_socket::tokio::RecvHalf;
use interprocess::local_socket::tokio::SendHalf;
use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::tokio::prelude::*;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::deno::start_js_runtime;

pub fn run_plugin_runtime(socket_name: String) {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("unable to start tokio runtime for plugin")
        .block_on(run_outer(socket_name))
        .expect("plugin runtime crashed");
}

async fn run_outer(socket_name: String) -> anyhow::Result<()> {
    tracing::info!("Starting plugin runtime at socket: {}", &socket_name);

    let stop_token = CancellationToken::new();

    #[cfg(target_os = "windows")]
    let name = socket_name.to_ns_name::<interprocess::local_socket::GenericNamespaced>()?;

    #[cfg(unix)]
    let name = socket_name.to_fs_name::<interprocess::os::unix::local_socket::FilesystemUdSocket>()?;

    let conn = Stream::connect(name).await?;

    let (mut recver, mut sender) = conn.split();

    let (request_sender, mut request_receiver) = gauntlet_utils::channel::channel::<
        BackendForPluginRuntimeApiRequestData,
        BackendForPluginRuntimeApiResponseData,
    >();
    let (event_sender, event_receiver) = channel::<JsEvent>(10);
    let response_oneshot = Mutex::new(None);

    let init = recv_message::<JsInit>(JsMessageSide::PluginRuntime, &mut recver).await?;

    let plugin_id = init.plugin_id.clone();

    let api = BackendForPluginRuntimeApiProxy::new(request_sender);

    let handle = Handle::current();

    tokio::select! {
        _ = stop_token.cancelled() => {
            tracing::debug!("Plugin runtime outer loop will be stopped {:?}", plugin_id)
        }
        _ = {
             tokio::task::unconstrained(async {
                loop {
                    if let Err(err) = message_loop(&mut recver, &event_sender, &response_oneshot, stop_token.clone()).await {
                        tracing::error!("Message loop has returned an error: {:?}", err);
                        break;
                    }
                }
             })
        } => {
            tracing::error!("Message loop has unexpectedly stopped {:?}", plugin_id)
        }
        _ = {
             tokio::task::unconstrained(async {
                loop {
                    if let Err(err) = request_loop(&mut sender, &mut request_receiver, &response_oneshot).await {
                        tracing::error!("Request loop has returned an error: {:?}", err);
                        break;
                    }
                }
             })
        } => {
            tracing::error!("Request loop has unexpectedly stopped {:?}", plugin_id)
        }
        _ = {
            run_new_tokio(handle, stop_token.clone(), init, event_receiver, api.clone())
        } => {
            tracing::error!("Request loop has unexpectedly stopped {:?}", plugin_id)
        }
    }

    send_message(
        JsMessageSide::PluginRuntime,
        &mut sender,
        JsPluginRuntimeMessage::Stopped,
    )
    .await?;

    tracing::debug!("Plugin runtime outer loop has been stopped {:?}", plugin_id);

    drop((recver, sender, api));

    Ok(())
}

async fn run_new_tokio(
    outer_handle: Handle,
    stop_token: CancellationToken,
    init: JsInit,
    event_receiver: Receiver<JsEvent>,
    api: BackendForPluginRuntimeApiProxy,
) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("unable to start tokio runtime for plugin")
            .block_on(run(outer_handle, stop_token, init, event_receiver, api))
    })
    .await??;

    Ok(())
}

async fn run(
    outer_handle: Handle,
    stop_token: CancellationToken,
    init: JsInit,
    event_receiver: Receiver<JsEvent>,
    api: BackendForPluginRuntimeApiProxy,
) -> anyhow::Result<()> {
    let plugin_id = init.plugin_id.clone();

    tokio::select! {
        _ = stop_token.cancelled() => {
            tracing::debug!("Plugin runtime inner loop will be stopped {:?}", plugin_id)
        }
        result @ _ = {
            tokio::task::unconstrained(async {
                 start_js_runtime(outer_handle, init, event_receiver, api).await
            })
        } => {
            if let Err(err) = result {
                tracing::error!("Plugin runtime inner loop has failed {:?} - {:?}", plugin_id, err)
            }
        }
    }

    tracing::debug!("Plugin runtime inner loop has been stopped {:?}", plugin_id);

    Ok(())
}

async fn request_loop(
    send: &mut SendHalf,
    request_receiver: &mut RequestReceiver<
        BackendForPluginRuntimeApiRequestData,
        BackendForPluginRuntimeApiResponseData,
    >,
    response_oneshot: &Mutex<Option<oneshot::Sender<Result<BackendForPluginRuntimeApiResponseData, String>>>>,
) -> anyhow::Result<()> {
    let (request, responder) = request_receiver.recv().await;

    tracing::trace!("Received request {:?}", &request);

    let rx = {
        let mut response_oneshot = response_oneshot.lock().await;

        let None = response_oneshot.deref() else {
            return Err(anyhow!(
                "Trying to set response one shot while previous is not fulfilled"
            ));
        };

        let (tx, rx) = oneshot::channel::<Result<BackendForPluginRuntimeApiResponseData, String>>();

        *response_oneshot = Some(tx);

        rx
    };

    send_message(
        JsMessageSide::PluginRuntime,
        send,
        JsPluginRuntimeMessage::Request(request),
    )
    .await?;

    tracing::trace!("Waiting for oneshot response...");

    let response = rx.await?;

    tracing::trace!("Sending response request {:?}", &response);

    responder.respond(response.map_err(|err| anyhow::anyhow!("{}", err)));

    Ok(())
}

async fn message_loop(
    recv: &mut RecvHalf,
    event_sender: &Sender<JsEvent>,
    response_oneshot: &Mutex<Option<oneshot::Sender<Result<BackendForPluginRuntimeApiResponseData, String>>>>,
    stop_token: CancellationToken,
) -> anyhow::Result<()> {
    match recv_message::<JsMessage>(JsMessageSide::PluginRuntime, recv).await {
        Err(e) => {
            tracing::error!("Unable to handle message: {:?}", e);
            Err(e)
        }
        Ok(msg) => {
            match msg {
                JsMessage::Event(event) => {
                    tracing::trace!("Received plugin event from backend {:?}", event);

                    let event_sender = event_sender.clone();

                    tokio::spawn(async move {
                        event_sender.send(event).await.expect("event receiver was dropped");
                    });

                    Ok(())
                }
                JsMessage::Response(response) => {
                    let mut response_oneshot = response_oneshot.lock().await;

                    match response_oneshot.take() {
                        Some(oneshot) => {
                            match oneshot.send(response) {
                                Err(_) => {
                                    tracing::error!("Dropped oneshot receiving side");
                                }
                                Ok(_) => {
                                    tracing::trace!("Oneshot response sent");
                                }
                            }
                        }
                        None => {
                            tracing::error!("Received response without corresponding request: {:?}", response);
                        }
                    }

                    Ok(())
                }
                JsMessage::Stop => {
                    stop_token.cancel();

                    Ok(())
                }
            }
        }
    }
}
