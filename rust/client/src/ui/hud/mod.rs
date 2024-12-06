use crate::ui::AppMsg;
use iced::window::{Level, Position, Settings};
use iced::{window, Limits, Size, Task};
use std::convert;
use std::time::Duration;

const HUD_WINDOW_WIDTH: f32 = 400.0;
const HUD_WINDOW_HEIGHT: f32 = 40.0;

pub fn show_hud_window(
    #[cfg(target_os = "linux")]
    wayland: bool,
) -> Task<AppMsg> {
    #[cfg(target_os = "linux")]
    if wayland {
        open_wayland()
    } else {
        open_non_wayland()
    }

    #[cfg(not(target_os = "linux"))]
    open_non_wayland()
}

fn open_non_wayland() -> Task<AppMsg> {
    let settings = Settings {
        size: Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: false,
        transparent: true,
        visible: true,
        level: Level::AlwaysOnTop,
        // TODO macos
        // #[cfg(target_os = "macos")]
        // platform_specific: iced::window::settings::PlatformSpecific {
        //     activation_policy: window::settings::ActivationPolicy::Accessory,
        //     activate_ignoring_other_apps: false,
        //     ..Default::default()
        // },
        exit_on_close_request: false,
        ..Default::default()
    };

    window::open(settings)
        .1
        .then(|id| in_2_seconds(AppMsg::CloseHudWindow { id }))
}

#[cfg(target_os = "linux")]
fn open_wayland() -> Task<AppMsg> {
    let id = window::Id::unique();

    iced::platform_specific::shell::commands::layer_surface::get_layer_surface(layer_shell_settings(id))
        .then(|id| in_2_seconds(AppMsg::CloseHudWindow { id }))
}

pub fn close_hud_window(
    #[cfg(target_os = "linux")]
    wayland: bool,
    id: window::Id
) -> Task<AppMsg> {
    #[cfg(target_os = "linux")]
    if wayland {
        iced::platform_specific::shell::commands::layer_surface::destroy_layer_surface(id)
    } else {
        window::close(id)
    }

    #[cfg(not(target_os = "linux"))]
    window::close(id)
}

#[cfg(target_os = "linux")]
fn layer_shell_settings(id: window::Id) -> iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
    iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
        id,
        layer: iced::platform_specific::shell::commands::layer_surface::Layer::Overlay,
        keyboard_interactivity: iced::platform_specific::shell::commands::layer_surface::KeyboardInteractivity::None,
        pointer_interactivity: false,
        anchor: iced::platform_specific::shell::commands::layer_surface::Anchor::empty(),
        output: Default::default(),
        namespace: "Gauntlet HUD".to_string(),
        margin: Default::default(),
        exclusive_zone: 0,
        size: Some((Some(HUD_WINDOW_WIDTH as u32), Some(HUD_WINDOW_HEIGHT as u32))),
        size_limits: Limits::new(Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT), Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT)),
    }
}

fn in_2_seconds(msg: AppMsg) -> Task<AppMsg> {
    Task::perform(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;

        msg
    }, convert::identity)
}
