use deno_core::op2;
use gauntlet_common::detached_process::CommandExt;

#[op2(fast)]
pub fn open_settings() -> anyhow::Result<()> {
    std::process::Command::new(std::env::current_exe()?)
        .args(["settings"])
        .spawn_detached()?;

    Ok(())
}
