use anyhow::anyhow;
use deno_core::op2;
use gauntlet_common::detached_process::CommandExt;

use crate::deno::GauntletJsError;

#[op2(fast)]
pub fn open_settings() -> Result<(), GauntletJsError> {
    let current_exe = std::env::current_exe().map_err(|err| anyhow!(err))?;

    std::process::Command::new(current_exe)
        .args(["settings"])
        .spawn_detached()
        .map_err(|err| anyhow!(err))?;

    Ok(())
}
