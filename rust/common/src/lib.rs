use serde::Deserialize;
use serde::Serialize;

pub mod cli;
pub mod detached_process;
pub mod dirs;
pub mod model;
pub mod rpc;
pub mod scenario_convert;
pub mod scenario_model;

pub const SETTINGS_ENV: &'static str = "__GAUNTLET_INTERNAL_SETTINGS__";

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SettingsEnvData {
    OpenPluginPreferences { plugin_id: String },
    OpenEntrypointPreferences { plugin_id: String, entrypoint_id: String },
}

pub fn settings_env_data_to_string(data: SettingsEnvData) -> String {
    serde_json::to_string(&data).expect("unable to serialize settings env data")
}

pub fn settings_env_data_from_string(data: String) -> SettingsEnvData {
    serde_json::from_str(&data).expect("unable to serialize settings env data")
}
