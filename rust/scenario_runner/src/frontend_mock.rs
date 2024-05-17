use std::fs;
use std::path::Path;
use std::time::Duration;

use common::model::{EntrypointId, PluginId, UiRenderLocation, UiWidget};
use common::rpc::backend_api::BackendApi;
use common::rpc::frontend_server::{FrontendServer, start_frontend_server};

use crate::model::{ScenarioBackendEvent, ScenarioFrontendEvent, ui_render_location_to_scenario, ui_widget_to_scenario};

pub async fn start_mock_frontend() -> anyhow::Result<()> {
    let scenario_dir = std::env::var("GAUNTLET_SCENARIOS_DIR")
        .expect("Unable to read GAUNTLET_SCENARIOS_DIR");

    let plugin_name = std::env::var("GAUNTLET_SCENARIO_PLUGIN_NAME")
        .expect("Unable to read GAUNTLET_SCENARIO_PLUGIN_NAME");

    let scenario_dir = Path::new(&scenario_dir);

    let scenario_plugin_dir = scenario_dir
        .join("plugins")
        .join(&plugin_name)
        .join("dist")
        .to_str()
        .expect("scenario_plugin_dir is invalid UTF-8")
        .to_string();

    let scenario_data_dir = scenario_dir
        .join("data")
        .join(&plugin_name)
        .to_str()
        .expect("scenario_data_dir is invalid UTF-8")
        .to_string();

    let scenario_out_dir = scenario_dir
        .join("out")
        .join(&plugin_name)
        .to_str()
        .expect("scenario_out_dir is invalid UTF-8")
        .to_string();

    fs::create_dir_all(&scenario_out_dir)
        .expect("unable to create scenario_out_dir");

    tokio::spawn(async {
        start_frontend_server(Box::new(RpcFrontendSaveToJson::new(scenario_out_dir))).await;
    });

    let mut client = BackendApi::new().await?;

    client.save_local_plugin(scenario_plugin_dir.clone()).await?;

    for entrypoint in fs::read_dir(&scenario_data_dir)? {
        let entrypoint = entrypoint?;
        if !entrypoint.metadata()?.is_dir() {
            panic!("unexpected file {:?} at {:?}", &entrypoint, &scenario_data_dir);
        }

        let entrypoint_name = entrypoint.file_name()
            .to_str()
            .expect("entrypoint name is invalid UTF-8")
            .to_string();

        for scenario in fs::read_dir(&entrypoint.path())? {
            let scenario = scenario?;
            if !scenario.metadata()?.is_file() {
                panic!("unexpected file {:?} at {:?}", &scenario, &scenario_data_dir);
            }

            let scenario_path = scenario.path();

            let scenario_data = fs::read(&scenario_path)
                .expect("unable to read scenario scenario from file");

            let event: ScenarioBackendEvent = serde_json::from_slice(&scenario_data)
                .expect("unable to deserialize scenario event");

            match event {
                ScenarioBackendEvent::Search { text } => {
                    client.search(text).await?;
                }
                ScenarioBackendEvent::RequestViewRender => {
                    let plugin_id = PluginId::from_string(format!("file://{scenario_plugin_dir}"));
                    let entrypoint_id = EntrypointId::from_string(&entrypoint_name);

                    client.request_view_render(plugin_id, entrypoint_id).await?;
                }
            }
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}

struct RpcFrontendSaveToJson {
    scenario_out_dir: String,
    counter: usize
}

impl RpcFrontendSaveToJson {
    fn new(scenario_out_dir: String) -> Self {
        Self {
            scenario_out_dir,
            counter: 0,
        }
    }

    fn save_event(&self, event: ScenarioFrontendEvent) {
        let json = serde_json::to_string_pretty(&event)
            .expect("unable to serialize scenario event");

        let entrypoint_id = match event {
            ScenarioFrontendEvent::ReplaceView { entrypoint_id, .. } => entrypoint_id,
            ScenarioFrontendEvent::ClearInlineView => "inline".to_string(),
            ScenarioFrontendEvent::ShowPreferenceRequiredView { entrypoint_id, .. } => entrypoint_id,
            ScenarioFrontendEvent::ShowPluginErrorView { entrypoint_id, .. } => entrypoint_id,
        };

        let out_dir = Path::new(&self.scenario_out_dir)
            .join(entrypoint_id);

        fs::create_dir_all(&out_dir)
            .expect("Unable to create scenario out dir");

        let out_path = out_dir
            .join(format!("{}.json", self.counter));

        fs::write(&out_path, json)
            .expect("unable to write scenario event to file");
    }
}

#[tonic::async_trait]
impl FrontendServer for RpcFrontendSaveToJson {
    async fn replace_view(&self, _plugin_id: PluginId, entrypoint_id: EntrypointId, container: UiWidget, top_level_view: bool, render_location: UiRenderLocation) {
        let event = ScenarioFrontendEvent::ReplaceView {
            entrypoint_id: entrypoint_id.to_string(),
            render_location: ui_render_location_to_scenario(render_location),
            top_level_view,
            container: ui_widget_to_scenario(container),
        };

        self.save_event(event);
    }

    async fn clear_inline_view(&self, _plugin_id: PluginId) {
        let event = ScenarioFrontendEvent::ClearInlineView;

        self.save_event(event);
    }

    async fn show_window(&self) {
        unreachable!()
    }

    async fn show_preference_required_view(&self, _plugin_id: PluginId, entrypoint_id: EntrypointId, plugin_preferences_required: bool, entrypoint_preferences_required: bool) {
        let event = ScenarioFrontendEvent::ShowPreferenceRequiredView {
            entrypoint_id: entrypoint_id.to_string(),
            plugin_preferences_required,
            entrypoint_preferences_required,
        };

        self.save_event(event);
    }

    async fn show_plugin_error_view(&self, _plugin_id: PluginId, entrypoint_id: EntrypointId, render_location: UiRenderLocation) {
        let event = ScenarioFrontendEvent::ShowPluginErrorView {
            entrypoint_id: entrypoint_id.to_string(),
            render_location: ui_render_location_to_scenario(render_location)
        };

        self.save_event(event);
    }
}

