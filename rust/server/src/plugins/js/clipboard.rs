use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use anyhow::{anyhow, Context, Error};
use arboard::ImageData;
use deno_core::{op2, OpState};
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use crate::plugins::js::permissions::PluginPermissionsClipboard;
use crate::plugins::js::{clipboard, PluginData};

#[derive(Clone)]
pub struct Clipboard {
    clipboard: Arc<RwLock<arboard::Clipboard>>,
}

impl Clipboard {
    pub fn new() -> anyhow::Result<Self> {
        let clipboard = arboard::Clipboard::new()
            .context("error while creating clipboard")?;

        Ok(Self {
            clipboard: Arc::new(RwLock::new(clipboard)),
        })
    }

    fn read(&self) -> anyhow::Result<ClipboardData> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        let png_data = match clipboard.get_image() {
            Ok(data) => {
                let rgba_image = RgbaImage::from_raw(data.width as u32, data.height as u32, data.bytes.into());
                let rgba_image = image::DynamicImage::ImageRgba8(rgba_image.unwrap());

                let mut result = Cursor::new(vec![]);

                rgba_image.write_to(&mut result, image::ImageFormat::Png)
                    .expect("should be able to convert to png");

                Some(result.into_inner())
            },
            Err(err) => {
                match err {
                    arboard::Error::ContentNotAvailable => None,
                    err @ _ => {
                        return Err(unknown_err_clipboard(err));
                    },
                }
            }
        };

        let text_data = match clipboard.get_text() {
            Ok(data) => Some(data),
            Err(err) => {
                match err {
                    arboard::Error::ContentNotAvailable => None,
                    err @ _ => {
                        return Err(unknown_err_clipboard(err));
                    },
                }
            }
        };

        Ok(ClipboardData {
            text_data,
            png_data,
        })
    }

    fn read_text(&self) -> anyhow::Result<Option<String>> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        let data = match clipboard.get_text() {
            Ok(data) => Some(data),
            Err(err) => {
                match err {
                    arboard::Error::ContentNotAvailable => None,
                    err @ _ => {
                        return Err(unknown_err_clipboard(err));
                    },
                }
            }
        };

        Ok(data)
    }

    fn write(&self, data: ClipboardData) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        if let Some(png_data) = data.png_data {

            let cursor = Cursor::new(&png_data);

            let mut reader = image::io::Reader::new(cursor);
            reader.set_format(image::ImageFormat::Png);

            let image = reader.decode()
                .map_err(|_err| unable_to_convert_image_err())?
                .into_rgba8();

            let (w, h) = image.dimensions();

            let image_data = ImageData {
                width: w as usize,
                height: h as usize,
                bytes: image.into_raw().into()
            };

            clipboard.set_image(image_data)
                .map_err(|err| unknown_err_clipboard(err))?;
        }

        if let Some(text_data) = data.text_data {
            clipboard.set_text(text_data)
                .map_err(|err| unknown_err_clipboard(err))?;
        }

        Ok(())
    }

    fn write_text(&self, data: String) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        clipboard.set_text(data)
            .map_err(|err| unknown_err_clipboard(err))?;

        Ok(())
    }

    fn clear(&self) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        clipboard.clear()
            .map_err(|err| unknown_err_clipboard(err))?;

        Ok(())
    }
}

fn unknown_err_clipboard(err: arboard::Error) -> Error {
    anyhow!("UNKNOWN_ERROR: {:?}", err)
}

fn unknown_err_image(err: image::ImageError) -> Error {
    anyhow!("UNKNOWN_ERROR: {:?}", err)
}

fn unable_to_convert_image_err() -> Error {
    anyhow!("UNABLE_TO_CONVERT_IMAGE")
}

#[derive(Debug, Serialize, Deserialize)]
struct ClipboardData {
    text_data: Option<String>,
    png_data: Option<Vec<u8>>
}

#[op2(async)]
#[serde]
pub async fn clipboard_read(state: Rc<RefCell<OpState>>) -> anyhow::Result<ClipboardData> {
    let clipboard = {
        let state = state.borrow();

        let plugin_data = state
            .borrow::<PluginData>();

        let allow = plugin_data
            .permissions()
            .clipboard
            .contains(&PluginPermissionsClipboard::Read);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'read' permission for clipboard"));
        }

        tracing::debug!("Reading from clipboard, plugin id: {:?}", plugin_data.plugin_id);

        let clipboard = state
            .borrow::<Clipboard>()
            .clone();

        clipboard
    };

    spawn_blocking(move || clipboard.read()).await?
}


#[op2(async)]
#[string]
pub async fn clipboard_read_text(state: Rc<RefCell<OpState>>) -> anyhow::Result<Option<String>> {
    let clipboard = {
        let state = state.borrow();

        let plugin_data = state
            .borrow::<PluginData>();

        let allow = plugin_data
            .permissions()
            .clipboard
            .contains(&PluginPermissionsClipboard::Read);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'read' permission for clipboard"));
        }

        tracing::debug!("Reading text from clipboard, plugin id: {:?}", plugin_data.plugin_id);

        let clipboard = state
            .borrow::<Clipboard>()
            .clone();

        clipboard
    };

    spawn_blocking(move || clipboard.read_text()).await?
}

#[op2(async)]
pub async fn clipboard_write(state: Rc<RefCell<OpState>>, #[serde] data: ClipboardData) -> anyhow::Result<()> { // TODO deserialization broken, fix when migrating to deno's op2
    let clipboard = {
        let state = state.borrow();

        let plugin_data = state
            .borrow::<PluginData>();

        let allow = plugin_data
            .permissions()
            .clipboard
            .contains(&PluginPermissionsClipboard::Write);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'write' permission for clipboard"));
        }

        tracing::debug!("Writing to clipboard, plugin id: {:?}", plugin_data.plugin_id);

        let clipboard = state
            .borrow::<Clipboard>()
            .clone();

        clipboard
    };

    spawn_blocking(move || clipboard.write(data)).await?
}

#[op2(async)]
pub async fn clipboard_write_text(state: Rc<RefCell<OpState>>, #[string] data: String) -> anyhow::Result<()> {
    let clipboard = {
        let state = state.borrow();

        let plugin_data = state
            .borrow::<PluginData>();

        let allow = plugin_data
            .permissions()
            .clipboard
            .contains(&PluginPermissionsClipboard::Write);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'write' permission for clipboard"));
        }

        tracing::debug!("Writing text to clipboard, plugin id: {:?}", plugin_data.plugin_id);

        let clipboard = state
            .borrow::<Clipboard>()
            .clone();

        clipboard
    };

    spawn_blocking(move || clipboard.write_text(data)).await?
}

#[op2(async)]
pub async fn clipboard_clear(state: Rc<RefCell<OpState>>) -> anyhow::Result<()> {
    let clipboard = {
        let state = state.borrow();

        let plugin_data = state
            .borrow::<PluginData>();

        let allow = plugin_data
            .permissions()
            .clipboard
            .contains(&PluginPermissionsClipboard::Clear);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'clear' permission for clipboard"));
        }

        tracing::debug!("Clearing clipboard, plugin id: {:?}", plugin_data.plugin_id);

        let clipboard = state
            .borrow::<Clipboard>()
            .clone();

        clipboard
    };

    spawn_blocking(move || clipboard.clear()).await?
}
