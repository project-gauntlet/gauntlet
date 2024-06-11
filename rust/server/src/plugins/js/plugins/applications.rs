use std::path::Path;
use deno_core::op;
use crate::plugins::applications::{DesktopEntry, get_apps};

#[op]
fn list_applications() -> Vec<DesktopEntry> {
    get_apps()
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
