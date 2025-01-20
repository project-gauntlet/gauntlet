use gauntlet_common::model::BackendRequestData;
use gauntlet_common::model::UiSetupData;
use gauntlet_common::model::UiTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_common::model::BackendResponseData;
use gauntlet_common::model::UiRequestData;
use gauntlet_common::model::UiResponseData;
use gauntlet_utils::channel::RequestReceiver;
use gauntlet_utils::channel::RequestSender;

pub mod frontend_mock;
mod model;

pub async fn run_scenario_runner_frontend_mock(
    request_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) -> anyhow::Result<()> {
    frontend_mock::start_scenario_runner_frontend(request_receiver, backend_sender).await?;

    Ok(())
}

pub async fn run_scenario_runner_mock_server(
    _request_sender: RequestSender<UiRequestData, UiResponseData>,
    mut backend_receiver: RequestReceiver<BackendRequestData, BackendResponseData>,
    theme: UiTheme
) -> anyhow::Result<()> {

    let (_data, responder) = backend_receiver.recv().await;
    responder.respond(BackendResponseData::SetupData {
        data: UiSetupData {
            window_position_file: None,
            theme,
            global_shortcut: None,
            close_on_unfocus: false,
            window_position_mode: WindowPositionMode::Static,
        },
    });

    let (_data, responder) = backend_receiver.recv().await;
    responder.respond(BackendResponseData::Nothing);

    std::thread::park();

    Ok(())
}
