use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ApplicationConfig {
    pub main_window: Option<ApplicationWindowConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ApplicationWindowConfig {
    pub close_on_unfocus: Option<bool>,
    pub wayland: Option<ApplicationWindowWaylandConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ApplicationWindowWaylandConfig {
    pub mode: Option<WaylandLayerShellConfig>,
}

#[derive(Deserialize, Debug)]
pub enum WaylandLayerShellConfig {
    #[serde(rename = "prefer_layer_shell")]
    PreferLayerShell,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "layer_shell")] // errors if not available
    LayerShell,
}

pub struct EffectiveConfig {
    pub close_on_unfocus: bool,
    pub layer_shell: bool,
}
