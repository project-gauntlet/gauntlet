use std::io::Cursor;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Context;
use anyhow::Error;
use anyhow::anyhow;
use arboard::ImageData;
use gauntlet_common_plugin_runtime::model::JsClipboardData;
use image::RgbaImage;

#[derive(Clone)]
pub struct Clipboard {
    clipboard: Arc<RwLock<arboard::Clipboard>>,
}

impl Clipboard {
    pub fn new() -> anyhow::Result<Self> {
        let clipboard = arboard::Clipboard::new().context("error while creating clipboard")?;

        Ok(Self {
            clipboard: Arc::new(RwLock::new(clipboard)),
        })
    }

    pub fn read(&self) -> anyhow::Result<JsClipboardData> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        let png_data = match clipboard.get_image() {
            Ok(data) => {
                let rgba_image = RgbaImage::from_raw(data.width as u32, data.height as u32, data.bytes.into());
                let rgba_image = image::DynamicImage::ImageRgba8(rgba_image.unwrap());

                let mut result = Cursor::new(vec![]);

                rgba_image
                    .write_to(&mut result, image::ImageFormat::Png)
                    .expect("should be able to convert to png");

                Some(result.into_inner())
            }
            Err(err) => {
                match err {
                    arboard::Error::ContentNotAvailable => None,
                    err @ _ => {
                        return Err(unknown_err_clipboard(err));
                    }
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
                    }
                }
            }
        };

        Ok(JsClipboardData { text_data, png_data })
    }

    pub fn read_text(&self) -> anyhow::Result<Option<String>> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        let data = match clipboard.get_text() {
            Ok(data) => Some(data),
            Err(err) => {
                match err {
                    arboard::Error::ContentNotAvailable => None,
                    err @ _ => {
                        return Err(unknown_err_clipboard(err));
                    }
                }
            }
        };

        Ok(data)
    }

    pub fn write(&self, data: JsClipboardData) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        if let Some(png_data) = data.png_data {
            let cursor = Cursor::new(&png_data);

            let mut reader = image::ImageReader::new(cursor);
            reader.set_format(image::ImageFormat::Png);

            let image = reader
                .decode()
                .map_err(|_err| unable_to_convert_image_err())?
                .into_rgba8();

            let (w, h) = image.dimensions();

            let image_data = ImageData {
                width: w as usize,
                height: h as usize,
                bytes: image.into_raw().into(),
            };

            clipboard
                .set_image(image_data)
                .map_err(|err| unknown_err_clipboard(err))?;
        }

        if let Some(text_data) = data.text_data {
            clipboard
                .set_text(text_data)
                .map_err(|err| unknown_err_clipboard(err))?;
        }

        Ok(())
    }

    pub fn write_text(&self, data: String) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        clipboard.set_text(data).map_err(|err| unknown_err_clipboard(err))?;

        Ok(())
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        let mut clipboard = self.clipboard.write().expect("lock is poisoned");

        clipboard.clear().map_err(|err| unknown_err_clipboard(err))?;

        Ok(())
    }
}

fn unknown_err_clipboard(err: arboard::Error) -> Error {
    anyhow!("UNKNOWN_ERROR: {}", err)
}

fn unable_to_convert_image_err() -> Error {
    anyhow!("UNABLE_TO_CONVERT_IMAGE")
}
