use std::io::Cursor;

use anyhow::anyhow;
use arboard::ImageData;
use deno_core::op;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

fn unknown_err_clipboard(err: arboard::Error) -> anyhow::Error {
    anyhow!("UNKNOWN_ERROR: {:?}", err)
}

fn unknown_err_image(err: image::ImageError) -> anyhow::Error {
    anyhow!("UNKNOWN_ERROR: {:?}", err)
}

fn unable_to_convert_image_err() -> anyhow::Error {
    anyhow!("UNABLE_TO_CONVERT_IMAGE")
}

#[derive(Debug, Serialize, Deserialize)]
struct ClipboardData {
    text_data: Option<String>,
    png_data: Option<Vec<u8>>
}

#[op]
async fn clipboard_read() -> anyhow::Result<ClipboardData> {
    spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| unknown_err_clipboard(err))?;

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
    }).await?
}


#[op]
async fn clipboard_read_text() -> anyhow::Result<Option<String>> {
    spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| unknown_err_clipboard(err))?;

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
    }).await?
}

#[op]
async fn clipboard_write(data: ClipboardData) -> anyhow::Result<()> { // TODO deserialization broken, fix when migrating to deno's op2
    spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| unknown_err_clipboard(err))?;

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
    }).await?
}

#[op]
async fn clipboard_write_text(data: String) -> anyhow::Result<()> {
    spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| unknown_err_clipboard(err))?;

        clipboard.set_text(data)
            .map_err(|err| unknown_err_clipboard(err))?;

        Ok(())
    }).await?
}

#[op]
async fn clipboard_clear() -> anyhow::Result<()> {
    spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| unknown_err_clipboard(err))?;

        clipboard.clear()?;

        Ok(())
    }).await?
}
