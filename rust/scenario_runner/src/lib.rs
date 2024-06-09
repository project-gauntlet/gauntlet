pub mod frontend_mock;
pub mod backend_mock;
mod model;

pub async fn run_screenshot_gen_backend() -> anyhow::Result<()> {
    backend_mock::start_screenshot_gen_backend().await;

    Ok(())
}

pub async fn run_scenario_runner_frontend() -> anyhow::Result<()> {
    frontend_mock::start_scenario_runner_frontend().await?;

    Ok(())
}
