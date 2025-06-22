mod model;
pub mod plugins;
pub mod rpc;
mod search;

pub use global_hotkey;

pub const PLUGIN_CONNECT_ENV: &'static str = "__GAUNTLET_INTERNAL_PLUGIN_CONNECT__";
pub const PLUGIN_UUID_ENV: &'static str = "__GAUNTLET_INTERNAL_PLUGIN_UUID__";
