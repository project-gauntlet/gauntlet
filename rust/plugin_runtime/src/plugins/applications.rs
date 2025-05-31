use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use deno_core::ToJsBuffer;
use deno_core::op2;
use image::ImageFormat;
use image::imageops::FilterType;
use serde::Serialize;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::gauntlet_internal_linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::gauntlet_internal_windows;

#[allow(unused)]
use crate::deno::GauntletJsError;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum DesktopPathAction {
    #[serde(rename = "add")]
    Add { id: String, data: DesktopApplication },
    #[serde(rename = "remove")]
    #[allow(unused)]
    Remove { id: String },
}

#[cfg(target_os = "linux")]
#[derive(Debug, Serialize)]
pub struct DesktopApplication {
    name: String,
    desktop_file_path: String,
    icon: Option<ToJsBuffer>,
    startup_wm_class: Option<String>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Serialize)]
pub struct DesktopApplication {
    name: String,
    path: String,
    icon: Option<ToJsBuffer>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Serialize)]
pub struct DesktopApplication {
    name: String,
    path: String,
    icon: Option<ToJsBuffer>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Serialize)]
pub struct DesktopSettingsPre13Data {
    name: String,
    path: String,
    icon: Option<ToJsBuffer>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Serialize)]
pub struct DesktopSettings13AndPostData {
    name: String,
    preferences_id: String,
    icon: Option<ToJsBuffer>,
}

#[op2]
#[string]
pub fn current_os() -> &'static str {
    std::env::consts::OS
}

#[op2(fast)]
pub fn wayland(state: Rc<RefCell<OpState>>) -> bool {
    let wayland = { state.borrow().borrow::<ApplicationContext>().desktop.is_wayland() };

    wayland
}

pub enum DesktopEnvironment {
    #[cfg(target_os = "linux")]
    Linux(linux::LinuxDesktopEnvironment),
    #[allow(unused)]
    None,
}

impl DesktopEnvironment {
    fn new() -> anyhow::Result<Self> {
        #[cfg(target_os = "linux")]
        let result = Ok(Self::Linux(linux::LinuxDesktopEnvironment::new()?));

        #[cfg(not(target_os = "linux"))]
        let result = Ok(Self::None);

        result
    }

    fn is_wayland(&self) -> bool {
        match self {
            #[cfg(target_os = "linux")]
            DesktopEnvironment::Linux(linux) => linux.is_wayland(),
            DesktopEnvironment::None => false,
        }
    }
}

pub struct ApplicationContext {
    desktop: DesktopEnvironment,
}

impl ApplicationContext {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            desktop: DesktopEnvironment::new()?,
        })
    }
}

#[cfg(target_os = "macos")]
#[op2(fast)]
pub fn macos_major_version() -> u8 {
    macos::macos_major_version()
}

#[cfg(target_os = "macos")]
#[op2(async)]
#[serde]
pub async fn macos_app_from_path(
    #[string] path: String,
    #[string] lang: Option<String>,
) -> Result<Option<DesktopPathAction>, GauntletJsError> {
    use std::path::PathBuf;

    use tokio::task::spawn_blocking;

    let result = spawn_blocking(|| macos::macos_app_from_path(&PathBuf::from(path), lang))
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    Ok(result)
}

#[cfg(target_os = "macos")]
#[op2(async)]
#[serde]
pub async fn macos_app_from_arbitrary_path(
    #[string] path: String,
    #[string] lang: Option<String>,
) -> Result<Option<DesktopPathAction>, GauntletJsError> {
    use std::path::PathBuf;

    use tokio::task::spawn_blocking;

    let result = spawn_blocking(|| macos::macos_app_from_arbitrary_path(PathBuf::from(path), lang))
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    Ok(result)
}

#[cfg(target_os = "macos")]
#[op2]
#[serde]
pub fn macos_system_applications() -> Vec<String> {
    macos::macos_system_applications()
        .into_iter()
        .map(|path| path.to_str().expect("non-utf8 paths are not supported").to_string())
        .collect()
}

#[cfg(target_os = "macos")]
#[op2]
#[serde]
pub fn macos_application_dirs() -> Vec<String> {
    macos::macos_application_dirs()
        .into_iter()
        .map(|path| path.to_str().expect("non-utf8 paths are not supported").to_string())
        .collect()
}

#[cfg(target_os = "macos")]
#[op2(fast)]
pub fn macos_open_application(#[string] app_path: String) -> Result<(), GauntletJsError> {
    use gauntlet_common::detached_process::CommandExt;
    std::process::Command::new("open")
        .args([app_path])
        .spawn_detached()
        .map_err(|err| anyhow::anyhow!(err))?;

    Ok(())
}

#[cfg(target_os = "macos")]
#[op2]
#[serde]
pub fn macos_settings_pre_13() -> Vec<DesktopSettingsPre13Data> {
    macos::macos_settings_pre_13()
}

#[cfg(target_os = "macos")]
#[op2]
#[serde]
pub fn macos_settings_13_and_post(#[string] lang: Option<String>) -> Vec<DesktopSettings13AndPostData> {
    macos::macos_settings_13_and_post(lang)
}

#[cfg(target_os = "macos")]
#[op2(fast)]
pub fn macos_open_setting_13_and_post(#[string] preferences_id: String) -> Result<(), GauntletJsError> {
    use gauntlet_common::detached_process::CommandExt;
    std::process::Command::new("open")
        .args([format!("x-apple.systempreferences:{}", preferences_id)])
        .spawn_detached()
        .map_err(|err| anyhow::anyhow!(err))?;

    Ok(())
}

#[cfg(target_os = "macos")]
#[op2(fast)]
pub fn macos_open_setting_pre_13(#[string] setting_path: String) -> Result<(), GauntletJsError> {
    use gauntlet_common::detached_process::CommandExt;
    std::process::Command::new("open")
        .args(["-b", "com.apple.systempreferences", &setting_path])
        .spawn_detached()
        .map_err(|err| anyhow::anyhow!(err))?;

    Ok(())
}

#[cfg(target_os = "macos")]
#[op2]
#[string]
pub fn macos_get_localized_language() -> Option<String> {
    sys_locale::get_locale()?
        .split("-")
        .collect::<Vec<&str>>()
        .get(0)
        .map(|s| s.to_string())
}

#[allow(unused)]
pub(in crate::plugins::applications) fn resize_icon(data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let data = image::load_from_memory_with_format(&data, ImageFormat::Png)?;
    let data = image::imageops::resize(&data, 48, 48, FilterType::Lanczos3);

    let mut buffer = std::io::Cursor::new(vec![]);

    data.write_to(&mut buffer, ImageFormat::Png)?;

    Ok(buffer.into_inner())
}
