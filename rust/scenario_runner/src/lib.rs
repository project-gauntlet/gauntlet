use gauntlet_common::model::UiSetupData;
use gauntlet_common::model::UiTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::rpc::backend_api::BackendForFrontendApiRequestData;
use gauntlet_common::rpc::backend_api::BackendForFrontendApiResponseData;
use gauntlet_common::rpc::frontend_api::FrontendApiRequestData;
use gauntlet_common::rpc::frontend_api::FrontendApiResponseData;
use gauntlet_utils::channel::RequestReceiver;
use gauntlet_utils::channel::RequestSender;

pub mod frontend_mock;
mod model;

pub async fn run_scenario_runner_frontend_mock(
    request_receiver: RequestReceiver<FrontendApiRequestData, FrontendApiResponseData>,
    backend_sender: RequestSender<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
) -> anyhow::Result<()> {
    frontend_mock::start_scenario_runner_frontend(request_receiver, backend_sender).await?;

    Ok(())
}

pub async fn run_scenario_runner_mock_server(
    _request_sender: RequestSender<FrontendApiRequestData, FrontendApiResponseData>,
    mut backend_receiver: RequestReceiver<BackendForFrontendApiRequestData, BackendForFrontendApiResponseData>,
    theme: UiTheme,
) -> anyhow::Result<()> {
    let (_data, responder) = backend_receiver.recv().await;
    responder.respond(Ok(BackendForFrontendApiResponseData::SetupData {
        data: UiSetupData {
            window_position_file: None,
            theme,
            close_on_unfocus: false,
            window_position_mode: WindowPositionMode::Static,
        },
    }));

    let (_data, responder) = backend_receiver.recv().await;
    responder.respond(Ok(BackendForFrontendApiResponseData::SetupResponse { data: () }));

    std::thread::park();

    Ok(())
}
