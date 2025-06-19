use std::fs;
use std::path::PathBuf;

use gauntlet_common::model::WindowPositionMode;
use iced::Point;
use iced::Size;
use iced::Task;
use iced::window;
use iced::window::Level;
use iced::window::Mode;
use iced::window::Position;

use crate::ui::AppMsg;

pub mod hud;
#[cfg(target_os = "linux")]
pub mod x11_focus;

#[derive(Debug, Clone)]
pub enum WindowActionMsg {
    #[cfg(target_os = "linux")]
    LayerShell(layer_shell::LayerShellAppMsg),
    SetWindowPositionMode {
        mode: WindowPositionMode,
    },
    #[cfg(target_os = "linux")]
    X11ActiveWindowChanged {
        window: u32,
        wm_name: Option<String>,
    },
    ShowWindow,
    HideWindow,
    ToggleWindow,
    ShowHud {
        display: String,
    },
}

#[cfg(target_os = "linux")]
mod layer_shell {
    #[iced_layershell::to_layer_message(multi)]
    #[derive(Debug, Clone)]
    pub enum LayerShellAppMsg {}
}

#[cfg(target_os = "linux")]
impl TryInto<iced_layershell::actions::LayershellCustomActionWithId> for AppMsg {
    type Error = Self;
    fn try_into(self) -> Result<iced_layershell::actions::LayershellCustomActionWithId, Self::Error> {
        match self {
            Self::WindowAction(WindowActionMsg::LayerShell(msg)) => {
                msg.try_into()
                    .map_err(|msg| Self::WindowAction(WindowActionMsg::LayerShell(msg)))
            }
            _ => Err(self),
        }
    }
}

const WINDOW_WIDTH: f32 = 750.0;
const WINDOW_HEIGHT: f32 = 450.0;

#[cfg(not(target_os = "macos"))]
fn window_settings(visible: bool, position: Position) -> window::Settings {
    window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position,
        resizable: false,
        decorations: false,
        visible,
        transparent: true,
        closeable: false,
        minimizable: false,
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "gauntlet".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(target_os = "macos")]
fn window_settings(visible: bool, position: Position) -> window::Settings {
    window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position,
        resizable: false,
        decorations: true,
        visible,
        transparent: false,
        closeable: false,
        minimizable: false,
        platform_specific: window::settings::PlatformSpecific {
            window_kind: window::settings::WindowKind::Popup,
            fullsize_content_view: true,
            title_hidden: true,
            titlebar_transparent: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn create_window(
    #[cfg(target_os = "linux")] wayland: bool,
    minimized: bool,
    window_position_file: Option<&PathBuf>,
) -> (window::Id, Task<WindowActionMsg>) {
    #[cfg(target_os = "linux")]
    let (main_window_id, open_task) = if wayland {
        let id = window::Id::unique();

        if minimized {
            (id, Task::none())
        } else {
            open_main_window_wayland(id)
        }
    } else {
        open_main_window_non_wayland(minimized, window_position_file)
    };

    #[cfg(not(target_os = "linux"))]
    let (main_window_id, open_task) = open_main_window_non_wayland(minimized, window_position_file);

    (main_window_id, open_task)
}

pub fn show_window(
    main_window_id: window::Id,
    #[cfg(target_os = "linux")] wayland: bool,
    #[cfg(target_os = "macos")] window_position_mode: WindowPositionMode,
) -> Task<WindowActionMsg> {
    #[cfg(target_os = "linux")]
    let open_task = if wayland {
        let (_, open_task) = open_main_window_wayland(main_window_id);
        open_task
    } else {
        Task::batch([
            window::gain_focus(main_window_id),
            window::set_mode(main_window_id, Mode::Windowed),
        ])
    };

    #[cfg(not(target_os = "linux"))]
    let open_task = Task::batch([
        window::gain_focus(main_window_id),
        #[cfg(target_os = "macos")]
        match window_position_mode {
            WindowPositionMode::Static => Task::none(),
            WindowPositionMode::ActiveMonitor => window::move_to_active_monitor(main_window_id),
        },
        window::set_mode(main_window_id, Mode::Windowed),
    ]);

    open_task
}

pub fn hide_window(#[cfg(target_os = "linux")] wayland: bool, main_window_id: window::Id) -> Task<WindowActionMsg> {
    let mut commands = vec![];

    #[cfg(target_os = "linux")]
    if wayland {
        commands.push(Task::done(WindowActionMsg::LayerShell(
            layer_shell::LayerShellAppMsg::RemoveWindow(main_window_id),
        )));
    } else {
        commands.push(window::set_mode(main_window_id, Mode::Hidden));
    };

    #[cfg(not(target_os = "linux"))]
    commands.push(window::set_mode(main_window_id, Mode::Hidden));

    #[cfg(target_os = "macos")]
    unsafe {
        // when closing NSPanel current active application doesn't automatically become key window
        // is there a proper way? without doing this manually
        let app = objc2_app_kit::NSWorkspace::sharedWorkspace().menuBarOwningApplication();

        if let Some(app) = app {
            app.activateWithOptions(objc2_app_kit::NSApplicationActivationOptions::empty());
        }
    }

    Task::batch(commands)
}

#[cfg(target_os = "linux")]
fn layer_shell_settings() -> iced_layershell::reexport::NewLayerShellSettings {
    iced_layershell::reexport::NewLayerShellSettings {
        layer: iced_layershell::reexport::Layer::Overlay,
        keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::Exclusive,
        events_transparent: false,
        anchor: iced_layershell::reexport::Anchor::empty(),
        margin: Default::default(),
        exclusive_zone: Some(0),
        size: Some((WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)),
        use_last_output: false,
        namespace: None,
    }
}

fn open_main_window_non_wayland(
    minimized: bool,
    window_position_file: Option<&PathBuf>,
) -> (window::Id, Task<WindowActionMsg>) {
    let position = window_position_file
        .map(|window_position_file| fs::read_to_string(window_position_file).ok())
        .flatten()
        .map(|data| {
            if let Some((x, y)) = data.split_once(":") {
                match (x.parse(), y.parse()) {
                    (Ok(x), Ok(y)) => Some(Position::Specific(Point::new(x, y))),
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap_or(None)
        .unwrap_or(Position::Centered);

    let (main_window_id, open_task) = window::open(window_settings(!minimized, position));

    (
        main_window_id,
        Task::batch([
            open_task.discard(),
            window::gain_focus(main_window_id),
            window::set_level(main_window_id, Level::AlwaysOnTop),
        ]),
    )
}

#[cfg(target_os = "linux")]
fn open_main_window_wayland(id: window::Id) -> (window::Id, Task<WindowActionMsg>) {
    let settings = layer_shell_settings();

    (
        id,
        Task::done(WindowActionMsg::LayerShell(
            layer_shell::LayerShellAppMsg::NewLayerShell { id, settings },
        )),
    )
}
