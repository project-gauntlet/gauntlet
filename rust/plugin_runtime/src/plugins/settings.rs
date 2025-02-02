use deno_core::op2;

#[op2(fast)]
pub fn open_settings() -> anyhow::Result<()> {
    std::process::Command::new(std::env::current_exe()?)
        .args(["settings"])
        .spawn()?;

    Ok(())
}
