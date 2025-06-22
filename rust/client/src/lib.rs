use crate::ui::scenario_runner::ScenarioRunnerData;

mod model;
mod ui;

pub fn run_app(minimized: bool) {
    ui::run(minimized, None);
}

pub fn run_scenario(
    scenarios_dir: String,
    plugins_dir: String,
    screenshots_dir: String,
    only_plugin: Option<String>,
    only_entrypoint: Option<String>,
) {
    ui::run(
        false,
        Some(ScenarioRunnerData {
            scenarios_dir,
            plugins_dir,
            screenshots_dir,
            only_plugin,
            only_entrypoint,
        }),
    );
}
