
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::get_apps;

#[cfg(not(target_os = "linux"))]
mod other;
#[cfg(not(target_os = "linux"))]
pub use other::get_apps;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: Option<Vec<u8>>,
    pub command: Vec<String>,
}