use common::model::{BackendRequestData, BackendResponseData, UiRequestData, UiResponseData};
use utils::channel::{RequestReceiver, RequestSender};

pub mod frontend_mock;
mod model;

pub async fn run_scenario_runner_frontend_mock(
    request_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>
) -> anyhow::Result<()> {
    frontend_mock::start_scenario_runner_frontend(request_receiver, backend_sender).await?;

    Ok(())
}
