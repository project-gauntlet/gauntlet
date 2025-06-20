use std::time::Duration;

use iced::Point;
use iced::Size;
use iced::Task;
use iced::window;
use iced::window::Level;
use iced::window::Position;
use iced::window::Settings;

use crate::ui::windows::WindowActionMsg;

const HUD_WINDOW_WIDTH: f32 = 400.0;
const HUD_WINDOW_HEIGHT: f32 = 40.0;

pub fn show_hud_window(#[cfg(target_os = "linux")] wayland: bool) -> Task<WindowActionMsg> {
    #[cfg(target_os = "linux")]
    if wayland {
        open_wayland()
    } else {
        open_non_wayland()
    }

    #[cfg(not(target_os = "linux"))]
    open_non_wayland()
}

fn open_non_wayland() -> Task<WindowActionMsg> {
    let settings = Settings {
        size: Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT),
        position: Position::SpecificWith(|window, screen| {
            Point::new(
                (screen.width - window.width) / 2.0,
                (screen.height - window.height) / 1.25,
            )
        }),
        resizable: false,
        decorations: false,
        transparent: true,
        visible: true,
        level: Level::AlwaysOnTop,
        #[cfg(target_os = "macos")]
        platform_specific: window::settings::PlatformSpecific {
            window_kind: window::settings::WindowKind::Panel,
            ..Default::default()
        },
        exit_on_close_request: false,
        ..Default::default()
    };

    window::open(settings)
        .1
        .then(|id| sleep_for_2_seconds(id).then(|id| window::close(id)))
}

#[cfg(target_os = "linux")]
fn open_wayland() -> Task<WindowActionMsg> {
    use crate::ui::windows::layer_shell;

    let id = window::Id::unique();
    let settings = layer_shell_settings();

    Task::batch([
        Task::done(WindowActionMsg::LayerShell(
            layer_shell::LayerShellAppMsg::NewLayerShell { id, settings },
        )),
        sleep_for_2_seconds(id).then(|id| {
            Task::done(WindowActionMsg::LayerShell(
                layer_shell::LayerShellAppMsg::RemoveWindow(id),
            ))
        }),
    ])
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
        namespace: Some("gauntlet-hud".to_string()),
    }
}

fn sleep_for_2_seconds(id: window::Id) -> Task<window::Id> {
    Task::future(async {
        tokio::time::sleep(Duration::from_secs(2)).await;
    })
    .then(move |_| Task::done(id))
}
