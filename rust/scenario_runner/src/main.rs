use tracing_subscriber::EnvFilter;

pub fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_thread_names(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let scenarios_dir = std::env::var("GAUNTLET_SCENARIOS_DIR")?;
    let plugins_dir = std::env::var("GAUNTLET_SCENARIOS_PLUGINS_DIR")?;
    let screenshots_dir = std::env::var("GAUNTLET_SCENARIOS_SCREENSHOTS_DIR")?;
    let only_plugin = std::env::var("GAUNTLET_SCENARIOS_ONLY_PLUGIN").ok();
    let only_entrypoint = std::env::var("GAUNTLET_SCENARIOS_ONLY_ENTRYPOINT").ok();

    gauntlet_client::run_scenario(
        scenarios_dir,
        plugins_dir,
        screenshots_dir,
        only_plugin,
        only_entrypoint,
    );

    Ok(())
}
