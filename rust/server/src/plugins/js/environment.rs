use crate::plugins::js::PluginData;
use deno_core::{op, OpState};

#[op]
fn environment_gauntlet_version() -> u16 {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../VERSION"))
        .parse()
        .expect("version is not a number?")
}

#[op]
fn environment_is_development(state: &mut OpState) -> bool {
    let plugin_id = state
        .borrow::<PluginData>()
        .plugin_id();

    plugin_id
        .to_string()
        .starts_with("file://")
}

#[op]
fn environment_plugin_data_dir(state: &mut OpState) -> String {
    state
        .borrow::<PluginData>()
        .plugin_data_dir()
        .to_string()
}

#[op]
fn environment_plugin_cache_dir(state: &mut OpState) -> String {
    state
        .borrow::<PluginData>()
        .plugin_cache_dir()
        .to_string()
}