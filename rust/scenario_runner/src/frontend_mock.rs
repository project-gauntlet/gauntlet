use std::fs;
use std::path::Path;

use gauntlet_common::model::BackendRequestData;
use gauntlet_common::model::BackendResponseData;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::UiRequestData;
use gauntlet_common::model::UiResponseData;
use gauntlet_common::rpc::backend_api::BackendApi;
use gauntlet_common::rpc::backend_api::BackendForFrontendApi;
use gauntlet_common::rpc::backend_server::wait_for_backend_server;
use gauntlet_common::scenario_convert::ui_render_location_to_scenario;
use gauntlet_common::scenario_model::ScenarioFrontendEvent;
use gauntlet_utils::channel::RequestReceiver;
use gauntlet_utils::channel::RequestSender;

use crate::model::ScenarioBackendEvent;

pub async fn start_scenario_runner_frontend(
    request_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    backend_sender: RequestSender<BackendRequestData, BackendResponseData>,
) -> anyhow::Result<()> {
    let scenario_dir = std::env::var("GAUNTLET_SCENARIOS_DIR").expect("Unable to read GAUNTLET_SCENARIOS_DIR");

    let plugin_name =
        std::env::var("GAUNTLET_SCENARIO_PLUGIN_NAME").expect("Unable to read GAUNTLET_SCENARIO_PLUGIN_NAME");

    let scenario_dir = Path::new(&scenario_dir);

    let scenario_plugin_dir = scenario_dir
        .join("plugins")
        .join(&plugin_name)
        .to_str()
        .expect("scenario_plugin_dir is invalid UTF-8")
        .to_string();

    let scenario_data_dir = scenario_dir
        .join("data")
        .join(&plugin_name)
        .to_str()
        .expect("scenario_data_dir is invalid UTF-8")
        .to_string();

    let scenario_out_dir = scenario_dir.join("out").join(&plugin_name);

    fs::create_dir_all(&scenario_out_dir).expect("unable to create scenario_out_dir");

    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move { request_loop(request_receiver, sender).await });

    println!("waiting for backend");

    wait_for_backend_server().await;

    println!("backend started");

    let mut backend_for_frontend_client = BackendForFrontendApi::new(backend_sender);
    let mut backend_client = BackendApi::new().await?;

    println!("saving local plugin");

    backend_client.save_local_plugin(scenario_plugin_dir.clone()).await?;

    println!("local plugin saved");

    for entrypoint in fs::read_dir(&scenario_data_dir)? {
        let entrypoint = entrypoint?;
        if !entrypoint.metadata()?.is_dir() {
            panic!("unexpected file {:?} at {:?}", &entrypoint, &scenario_data_dir);
        }

        let entrypoint_name = entrypoint
            .file_name()
            .to_str()
            .expect("entrypoint name is invalid UTF-8")
            .to_string();

        println!("entrypoint: {}", &entrypoint_name);

        for scenario in fs::read_dir(&entrypoint.path())? {
            let scenario = scenario?;
            if !scenario.metadata()?.is_file() {
                panic!("unexpected file {:?} at {:?}", &scenario, &scenario_data_dir);
            }

            let scenario_path = scenario.path();

            println!("scenario: {:?}", &scenario_path);

            let scenario_name = scenario_path.file_stem().unwrap().to_str().unwrap().to_string();

            let scenario_data = fs::read(&scenario_path).expect("unable to read scenario scenario from file");

            let event: ScenarioBackendEvent =
                serde_json::from_slice(&scenario_data).expect("unable to deserialize scenario event");

            match event {
                ScenarioBackendEvent::Search { text } => {
                    backend_for_frontend_client.search(text, true).await?;
                }
                ScenarioBackendEvent::RequestViewRender => {
                    let plugin_id = PluginId::from_string(format!("file://{scenario_plugin_dir}"));
                    let entrypoint_id = EntrypointId::from_string(&entrypoint_name);

                    backend_for_frontend_client
                        .request_view_render(plugin_id, entrypoint_id)
                        .await?;
                }
            }

            println!("waiting for scenario to finish");

            match receiver.recv().await {
                None => unreachable!(),
                Some(event) => save_event(&scenario_out_dir, scenario_name, event),
            }

            println!("scenario finished");
        }
    }

    println!("all scenarios done");

    std::process::exit(0)
}

fn save_event(scenario_out_dir: &Path, scenario_name: String, event: ScenarioFrontendEvent) {
    let json = serde_json::to_string_pretty(&event).expect("unable to serialize scenario event");

    let entrypoint_id = match event {
        ScenarioFrontendEvent::ReplaceView { entrypoint_id, .. } => entrypoint_id,
        ScenarioFrontendEvent::ShowPreferenceRequiredView { entrypoint_id, .. } => entrypoint_id,
        ScenarioFrontendEvent::ShowPluginErrorView { entrypoint_id, .. } => entrypoint_id,
    };

    let out_dir = Path::new(scenario_out_dir).join(entrypoint_id);

    fs::create_dir_all(&out_dir).expect("Unable to create scenario out dir");

    let out_path = out_dir.join(format!("{}.json", scenario_name));

    fs::write(&out_path, json).expect("unable to write scenario event to file");
}

async fn request_loop(
    mut request_receiver: RequestReceiver<UiRequestData, UiResponseData>,
    scenario_sender: tokio::sync::mpsc::Sender<ScenarioFrontendEvent>,
) {
    loop {
        let (request_data, responder) = request_receiver.recv().await;

        match request_data {
            UiRequestData::UpdateLoadingBar { .. }
            | UiRequestData::ShowHud { .. }
            | UiRequestData::ShowWindow
            | UiRequestData::HideWindow
            | UiRequestData::ClearInlineView { .. }
            | UiRequestData::SetTheme { .. } => {
                unreachable!()
            }
            UiRequestData::SetGlobalShortcut { .. } | UiRequestData::RequestSearchResultUpdate => {
                // noop
            }
            UiRequestData::ReplaceView {
                plugin_id: _,
                plugin_name: _,
                entrypoint_id,
                entrypoint_name: _,
                render_location,
                top_level_view,
                container,
                images,
            } => {
                let event = ScenarioFrontendEvent::ReplaceView {
                    entrypoint_id: entrypoint_id.to_string(),
                    render_location: ui_render_location_to_scenario(render_location),
                    top_level_view,
                    container,
                    images,
                };

                scenario_sender.send(event).await.expect("send failed")
            }
            UiRequestData::ShowPluginErrorView {
                plugin_id: _,
                entrypoint_id,
                render_location,
            } => {
                let event = ScenarioFrontendEvent::ShowPluginErrorView {
                    entrypoint_id: entrypoint_id.to_string(),
                    render_location: ui_render_location_to_scenario(render_location),
                };

                scenario_sender.send(event).await.expect("send failed")
            }
            UiRequestData::ShowPreferenceRequiredView {
                plugin_id: _,
                entrypoint_id,
                plugin_preferences_required,
                entrypoint_preferences_required,
            } => {
                let event = ScenarioFrontendEvent::ShowPreferenceRequiredView {
                    entrypoint_id: entrypoint_id.to_string(),
                    plugin_preferences_required,
                    entrypoint_preferences_required,
                };

                scenario_sender.send(event).await.expect("send failed")
            }
        }

        responder.respond(UiResponseData::Nothing);
    }
}
