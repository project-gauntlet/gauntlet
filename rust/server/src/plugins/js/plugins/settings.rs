use deno_core::op;

#[op]
fn open_settings() -> anyhow::Result<()> {
    std::process::Command::new(std::env::current_exe()?)
        .args(["settings"])
        .spawn()?;

    Ok(())
}