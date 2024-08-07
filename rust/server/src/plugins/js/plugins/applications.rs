use std::path::Path;

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

    std::process::Command::new(Path::new(path))
        .args(args)
        .spawn()?;

    Ok(())
}
