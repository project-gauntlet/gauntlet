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

pub fn show_hud_window(#[cfg(target_os = "linux")] layer_shell: bool) -> Task<WindowActionMsg> {
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
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            layer_shell: if layer_shell {
                layer_shell_settings()
            } else {
                Default::default()
            },
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
fn layer_shell_settings() -> window::settings::LayerShellSettings {
    window::settings::LayerShellSettings {
        layer: Some(window::settings::Layer::Overlay),
        keyboard_interactivity: Some(window::settings::KeyboardInteractivity::None),
        anchor: None,
        margin: None,
        exclusive_zone: Some(0),
        namespace: Some("gauntlet-hud".to_string()),
        output: None,
        input_region: Some((0, 0, 0, 0)), // mouse events transparency
    }
}

fn sleep_for_2_seconds(id: window::Id) -> Task<window::Id> {
    Task::future(async {
        tokio::time::sleep(Duration::from_secs(2)).await;
    })
    .then(move |_| Task::done(id))
}
