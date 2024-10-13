use crate::ui::AppMsg;
use iced::advanced::layout::Limits;
use iced::window::{Level, Position, Settings};
use iced::{window, Command, Size};
use std::convert;
use std::time::Duration;
use iced::window::settings::PlatformSpecific;

const HUD_WINDOW_WIDTH: f32 = 400.0;
const HUD_WINDOW_HEIGHT: f32 = 40.0;

pub fn show_hud_window(
    #[cfg(target_os = "linux")]
    wayland: bool,
) -> Command<AppMsg> {

    let settings = Settings {
        size: Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: false,
        transparent: true,
        visible: true,
        level: Level::AlwaysOnTop,
        #[cfg(target_os = "macos")]
        platform_specific: PlatformSpecific {
            activation_policy: window::settings::ActivationPolicy::Accessory,
            activate_ignoring_other_apps: false,
            ..Default::default()
        },
        exit_on_close_request: false,
        ..Default::default()
    };

    let (id, show_command) = window::spawn(settings);

    let close_command = Command::perform(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;

        AppMsg::CloseHudWindow { id }
    }, convert::identity);


    #[cfg(target_os = "linux")]
    if wayland {
        iced::wayland::commands::layer_surface::get_layer_surface(layer_shell_settings())
    } else {
        Command::batch([
            show_command,
            close_command
        ])
    }

    #[cfg(not(target_os = "linux"))]
    Command::batch([
        show_command,
        close_command
    ])
}

pub fn close_hud_window(
    #[cfg(target_os = "linux")]
    wayland: bool,
    id: window::Id
) -> Command<AppMsg> {
    let command = window::close(id);

    #[cfg(target_os = "linux")]
    if wayland {
        iced::wayland::commands::layer_surface::destroy_layer_surface(id)
    } else {
        command
    }

    #[cfg(not(target_os = "linux"))]
    command
}

#[cfg(target_os = "linux")]
fn layer_shell_settings() -> iced::wayland::runtime::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings {
    iced::wayland::runtime::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings {
        id: window::Id::unique(),
        layer: iced::wayland::commands::layer_surface::Layer::Overlay,
        keyboard_interactivity: iced::wayland::commands::layer_surface::KeyboardInteractivity::None,
        pointer_interactivity: false,
        anchor: iced::wayland::commands::layer_surface::Anchor::empty(),
        output: Default::default(),
        namespace: "Gauntlet HUD".to_string(),
        margin: Default::default(),
        exclusive_zone: 0,
        size: Some((Some(HUD_WINDOW_WIDTH as u32), Some(HUD_WINDOW_HEIGHT as u32))),
        size_limits: Limits::new(Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT), Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT)),
    }
}
