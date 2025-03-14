mod api;
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

use std::cell::RefCell;
use std::cell::RefMut;
use std::convert;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
pub use api::BackendForPluginRuntimeApi;
use bincode::Decode;
use bincode::Encode;
use deno_core::futures::SinkExt;
pub use events::JsEvent;
pub use events::JsKeyboardEventOrigin;
pub use events::JsUiPropertyValue;
use gauntlet_utils::channel::Payload;
use gauntlet_utils::channel::RequestReceiver;
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::RecvHalf;
use interprocess::local_socket::tokio::SendHalf;
use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::GenericFilePath;
use interprocess::local_socket::NameType;
use interprocess::local_socket::ToNsName;
pub use model::*;
use once_cell::sync::Lazy;
pub use permissions::PERMISSIONS_VARIABLE_PATTERN;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use tokio_util::sync::CancellationToken;

use crate::api::BackendForPluginRuntimeApiProxy;
use crate::deno::start_js_runtime;

pub fn run_plugin_runtime(socket_name: String) {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);


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

    let (request_sender, mut request_receiver) =
        gauntlet_utils::channel::channel::<JsRequest, Result<JsResponse, String>>();
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
        result @ _ = {
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
        result @ _ = {
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
        result @ _ = {
            run_new_tokio(handle, stop_token.clone(), init, event_receiver, api)
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

    drop((recver, sender));

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
    request_receiver: &mut RequestReceiver<JsRequest, Result<JsResponse, String>>,
    response_oneshot: &Mutex<Option<oneshot::Sender<Result<JsResponse, String>>>>,
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

        let (tx, rx) = oneshot::channel::<Result<JsResponse, String>>();

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

    responder.respond(response);

    Ok(())
}

async fn message_loop(
    recv: &mut RecvHalf,
    event_sender: &Sender<JsEvent>,
    response_oneshot: &Mutex<Option<oneshot::Sender<Result<JsResponse, String>>>>,
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
                        Some(mut oneshot) => {
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

#[derive(Debug)]
pub enum JsMessageSide {
    PluginRuntime,
    Backend,
}

static MESSAGE_ID: AtomicU32 = AtomicU32::new(0);

pub async fn send_message<T: Encode + Debug>(side: JsMessageSide, send: &mut SendHalf, value: T) -> anyhow::Result<()> {
    let encoded: Vec<u8> = bincode::encode_to_vec(&value, bincode::config::standard())?;

    let message_id = MESSAGE_ID.fetch_add(1, Ordering::SeqCst);

    tracing::trace!(
        side = debug(&side),
        "Sending message with id {} and size of {} bytes: {:?}",
        message_id,
        encoded.len(),
        &value
    );

    send.write_u32(message_id).await?;

    send.write_u32(encoded.len() as u32).await?;

    send.write_all(&encoded[..]).await?;

    tracing::trace!(
        side = debug(&side),
        "Message with id {} and size of {} bytes has been sent",
        message_id,
        encoded.len()
    );

    Ok(())
}

pub async fn recv_message<T: Decode + Debug>(side: JsMessageSide, recv: &mut RecvHalf) -> anyhow::Result<T> {
    tracing::trace!(side = debug(&side), "Waiting for next message...");

    let message_id = recv.read_u32().await?;

    tracing::trace!(side = debug(&side), "Reading message with id: {}", message_id);

    let buf_size = recv.read_u32().await?;

    let mut buffer = vec![0; buf_size as usize];

    recv.read_exact(&mut buffer).await?;

    let (decoded, _) = bincode::decode_from_slice(&buffer[..], bincode::config::standard())
        .context(format!("Unable to deserialize message with id: {}", message_id))?;

    tracing::trace!(
        side = debug(&side),
        "Received message with id {}: {:?}",
        message_id,
        &decoded
    );

    Ok(decoded)
}
