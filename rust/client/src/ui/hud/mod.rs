use crate::ui::AppMsg;
use iced::advanced::layout::Limits;
use iced::window::{Level, Position, Settings};
use iced::{window, Size, Task};
use std::convert;
use std::time::Duration;

const HUD_WINDOW_WIDTH: f32 = 400.0;
const HUD_WINDOW_HEIGHT: f32 = 40.0;

pub fn show_hud_window(
    // TODO
    // #[cfg(target_os = "linux")]
    // wayland: bool,
) -> Task<AppMsg> {

    let settings = Settings {
        size: Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT),
        position: Position::Centered,
        resizable: false,
        decorations: false,
        transparent: true,
        visible: true,
        level: Level::AlwaysOnTop,
        // #[cfg(target_os = "macos")]
        // platform_specific: iced::window::settings::PlatformSpecific {
        //     activation_policy: window::settings::ActivationPolicy::Accessory,
        //     activate_ignoring_other_apps: false,
        //     ..Default::default()
        // },
        exit_on_close_request: false,
        ..Default::default()
    };

    // #[cfg(target_os = "linux")]
    // if wayland {
    //     let id = window::Id::unique();
    //
    //     let show_command = iced::wayland::commands::layer_surface::get_layer_surface(layer_shell_settings(id));
    //     let close_command = in_2_seconds(AppMsg::CloseHudWindow { id });
    //
    //     Task::batch([
    //         show_command,
    //         close_command
    //     ])
    // } else {
    //     let (id, show_command) = window::spawn(settings);
    //     let close_command = in_2_seconds(AppMsg::CloseHudWindow { id });
    //
    //     Task::batch([
    //         show_command,
    //         close_command
    //     ])
    // }
    //
    // #[cfg(not(target_os = "linux"))]
    // {
    //     let (id, show_command) = window::spawn(settings);
    //     let close_command = in_2_seconds(AppMsg::CloseHudWindow { id });
    //
    //     Task::batch([
    //         show_command,
    //         close_command
    //     ])
    // }

    window::open(settings)
        .1
        .then(|id| in_2_seconds(AppMsg::CloseHudWindow { id }))
}

pub fn close_hud_window(
    // #[cfg(target_os = "linux")]
    // wayland: bool,
    id: window::Id
) -> Task<AppMsg> {
    // TODO
    // #[cfg(target_os = "linux")]
    // if wayland {
    //     iced::wayland::commands::layer_surface::destroy_layer_surface(id)
    // } else {
    //     window::close(id)
    // }
    //
    // #[cfg(not(target_os = "linux"))]
    // window::close(id)

    window::close(id)
}

// TODO
// #[cfg(target_os = "linux")]
// fn layer_shell_settings(id: window::Id) -> iced::wayland::runtime::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings {
//     iced::wayland::runtime::command::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings {
//         id,
//         layer: iced::wayland::commands::layer_surface::Layer::Overlay,
//         keyboard_interactivity: iced::wayland::commands::layer_surface::KeyboardInteractivity::None,
//         pointer_interactivity: false,
//         anchor: iced::wayland::commands::layer_surface::Anchor::empty(),
//         output: Default::default(),
//         namespace: "Gauntlet HUD".to_string(),
//         margin: Default::default(),
//         exclusive_zone: 0,
//         size: Some((Some(HUD_WINDOW_WIDTH as u32), Some(HUD_WINDOW_HEIGHT as u32))),
//         size_limits: Limits::new(Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT), Size::new(HUD_WINDOW_WIDTH, HUD_WINDOW_HEIGHT)),
//     }
// }

fn in_2_seconds(msg: AppMsg) -> Task<AppMsg> {
    Task::perform(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;

        msg
    }, convert::identity)
}
