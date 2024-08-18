use image::ImageFormat;
use image::imageops::FilterType;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::get_apps;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::get_apps;

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
mod other;
#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
pub use other::get_apps;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    pub command: Vec<String>,
}

pub(in crate::plugins::applications) fn resize_icon(data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let data = image::load_from_memory_with_format(&data, ImageFormat::Png)?;
    let data = image::imageops::resize(&data, 48, 48, FilterType::Lanczos3);

    let mut buffer = std::io::Cursor::new(vec![]);

    data.write_to(&mut buffer, ImageFormat::Png)?;

    Ok(buffer.into_inner())
}