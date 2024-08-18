use deno_core::op;
use tokio::task::spawn_blocking;
use crate::plugins::applications::{DesktopEntry, get_apps};

#[op]
async fn list_applications() -> anyhow::Result<Vec<DesktopEntry>> {
    Ok(spawn_blocking(|| get_apps()).await?)
}

#[op]
fn open_application(command: Vec<String>) -> anyhow::Result<()> {
    let path = &command[0];
    let args = &command[1..];

    #[cfg(not(windows))]
    spawn_detached(path, args)?;

    Ok(())
}

#[cfg(not(windows))]
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