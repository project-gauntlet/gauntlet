use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ApplicationConfig {
    pub main_window: Option<ApplicationWindowConfig>,
    pub wayland: Option<WaylandConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ApplicationWindowConfig {
    pub close_on_unfocus: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct WaylandConfig {
    pub main_window_surface: Option<WaylandMainWindowConfig>,
    pub global_shortcuts_api: Option<WaylandGlobalShortcutConfig>,
}

#[derive(Deserialize, Debug)]
pub enum WaylandMainWindowConfig {
    #[serde(rename = "prefer_wlr_layer_shell")] // default
    PreferLayerShell,
    #[serde(rename = "xdg_shell")]
    XdgShell,
    #[serde(rename = "wlr_layer_shell")] // errors if not available
    LayerShell,
}

#[derive(Deserialize, Debug)]
pub enum WaylandGlobalShortcutConfig {
    #[serde(rename = "none")] // default
    None,
    #[serde(rename = "legacy_x11_api")]
    LegacyX11Api,
}

pub struct EffectiveConfig {
    pub close_on_unfocus: bool,
    pub layer_shell: bool,
    pub wayland_use_legacy_x11_api: bool,
}
