use iced::window::{Level, Position, Settings};
use iced::{window, Size, Task};
use std::convert;
use std::time::Duration;
use crate::ui::AppMsg;

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
        #[cfg(target_os = "macos")]
        platform_specific: window::settings::PlatformSpecific {
            window_kind: window::settings::WindowKind::Popup,
            ..Default::default()
        },
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
    let settings = layer_shell_settings();

    Task::done(AppMsg::LayerShell(crate::ui::layer_shell::LayerShellAppMsg::NewLayerShell { id, settings }))
        .then(move |_| in_2_seconds(AppMsg::CloseHudWindow { id }))
}

pub fn close_hud_window(
    #[cfg(target_os = "linux")]
    wayland: bool,
    id: window::Id
) -> Task<AppMsg> {
    #[cfg(target_os = "linux")]
    if wayland {
        Task::done(AppMsg::LayerShell(crate::ui::layer_shell::LayerShellAppMsg::RemoveWindow(id)))
    } else {
        window::close(id)
    }

    #[cfg(not(target_os = "linux"))]
    window::close(id)
}

#[cfg(target_os = "linux")]
fn layer_shell_settings() -> iced_layershell::reexport::NewLayerShellSettings {
    iced_layershell::reexport::NewLayerShellSettings {
        layer: iced_layershell::reexport::Layer::Overlay,
        keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::None,
        use_last_output: false,
        events_transparent: true,
        anchor: iced_layershell::reexport::Anchor::empty(),
        margin: Default::default(),
        exclusive_zone: Some(0),
        size: Some((HUD_WINDOW_WIDTH as u32, HUD_WINDOW_HEIGHT as u32)),
    }
}

fn in_2_seconds(msg: AppMsg) -> Task<AppMsg> {
    Task::perform(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;

        msg
    }, convert::identity)
}
