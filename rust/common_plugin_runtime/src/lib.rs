use std::fmt::Debug;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

use anyhow::Context;
use bincode::Decode;
use bincode::Encode;
use interprocess::local_socket::tokio::RecvHalf;
use interprocess::local_socket::tokio::SendHalf;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub mod api;
pub mod model;

pub static PERMISSIONS_VARIABLE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{(?<namespace>.+?):(?<name>.+?)}").expect("invalid regex"));

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

pub async fn recv_message<T: Decode<()> + Debug>(side: JsMessageSide, recv: &mut RecvHalf) -> anyhow::Result<T> {
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
