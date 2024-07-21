use common::model::{UiRequestData, UiResponseData};
use utils::channel::RequestReceiver;

pub mod frontend_mock;
mod model;

pub async fn run_scenario_runner_frontend_mock(request_receiver: RequestReceiver<UiRequestData, UiResponseData>) -> anyhow::Result<()> {
    frontend_mock::start_scenario_runner_frontend(request_receiver).await?;

    Ok(())
}
