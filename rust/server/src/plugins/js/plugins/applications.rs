use deno_core::op;
use std::path::PathBuf;
use image::ImageFormat;
use image::imageops::FilterType;
use serde::Serialize;

#[cfg(target_os = "linux")]
mod linux;

// TODO macos
// #[cfg(target_os = "macos")]
// mod macos;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum DesktopPathAction {
    #[serde(rename = "add")]
    Add {
        id: String,
        data: DesktopApplication
    },
    #[serde(rename = "remove")]
    Remove {
        id: String
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Serialize)]
pub struct DesktopApplication {
    name: String,
    icon: Option<Vec<u8>>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Serialize)]
pub struct DesktopApplication {
    name: String,
    icon: Option<Vec<u8>>,
}

#[cfg(all(not(target_os = "linux"), not(target_os = "macos")))]
#[derive(Debug, Serialize)]
pub struct DesktopApplication {

}

#[cfg(target_os = "linux")]
#[op]
pub fn linux_app_from_path(path: String) -> Option<DesktopPathAction> {
    linux::linux_app_from_path(PathBuf::from(path))
}

#[cfg(target_os = "linux")]
#[op]
pub fn linux_application_dirs() -> Vec<String> {
    linux::linux_application_dirs()
        .into_iter()
        .map(|path| path.to_str().expect("non-utf8 paths are not supported").to_string())
        .collect()
}

#[cfg(target_os = "linux")]
#[op]
pub fn linux_open_application(desktop_file_id: String) -> anyhow::Result<()> {

    #[cfg(not(windows))]
    spawn_detached("gtk-launch", &[desktop_file_id])?;

    Ok(())
}

#[cfg(unix)]
pub fn spawn_detached<I, S>(
    path: &str,
    args: I,
) -> std::io::Result<()>
where
    I: IntoIterator<Item = S> + Copy,
    S: AsRef<std::ffi::OsStr>,
{
    // from https://github.com/alacritty/alacritty/blob/5abb4b73937b17fe501b9ca20b602950f1218b96/alacritty/src/daemon.rs#L65
    use std::os::unix::prelude::CommandExt;
    use std::process::{Command, Stdio};

    let mut command = Command::new(path);

    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        command
            .pre_exec(|| {
                match libc::fork() {
                    -1 => return Err(std::io::Error::last_os_error()),
                    0 => (),
                    _ => libc::_exit(0),
                }

                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }

                Ok(())
            })
            .spawn()?
            .wait()
            .map(|_| ())
    }
}

pub(in crate::plugins::js::plugins::applications) fn resize_icon(data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let data = image::load_from_memory_with_format(&data, ImageFormat::Png)?;
    let data = image::imageops::resize(&data, 48, 48, FilterType::Lanczos3);

    let mut buffer = std::io::Cursor::new(vec![]);

    data.write_to(&mut buffer, ImageFormat::Png)?;

    Ok(buffer.into_inner())
}