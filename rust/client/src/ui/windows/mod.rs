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
use crate::ui::windows::hud::show_hud_window;

pub mod hud;
#[cfg(target_os = "linux")]
pub mod x11_focus;

pub struct WindowState {
    pub main_window_id: window::Id,
    focused: bool,
    opened: bool,
    #[cfg(target_os = "linux")]
    pub wayland: bool,
    window_position_mode: WindowPositionMode,
    close_on_unfocus: bool,
    window_position_file: Option<PathBuf>,
    #[cfg(target_os = "linux")]
    x11_active_window: Option<u32>,
}

impl WindowState {
    pub fn new(
        main_window_id: window::Id,
        minimized: bool,
        window_position_file: Option<PathBuf>,
        close_on_unfocus: bool,
        window_position_mode: WindowPositionMode,
        #[cfg(target_os = "linux")] wayland: bool,
    ) -> WindowState {
        Self {
            main_window_id,
            focused: false,
            opened: !minimized,
            #[cfg(target_os = "linux")]
            wayland,
            window_position_mode,
            close_on_unfocus,
            window_position_file,
            #[cfg(target_os = "linux")]
            x11_active_window: None,
        }
    }
}

impl WindowState {
    pub fn handle_action(&mut self, action: WindowActionMsg) -> Task<AppMsg> {
        match action {
            #[cfg(target_os = "linux")]
            WindowActionMsg::LayerShell(_) => {
                // handled by library
                Task::none()
            }
            WindowActionMsg::SetWindowPositionMode { mode } => {
                self.window_position_mode = mode;

                Task::none()
            }
            #[cfg(target_os = "linux")]
            WindowActionMsg::X11ActiveWindowChanged { window, wm_name } => {
                if self.x11_active_window != Some(window) {
                    self.x11_active_window = Some(window);
                    if let Some(wm_name) = &wm_name {
                        if wm_name != "gauntlet" {
                            Task::done(AppMsg::WindowAction(WindowActionMsg::HideWindow))
                        } else {
                            Task::none()
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            WindowActionMsg::ToggleWindow => self.toggle_window(),
            WindowActionMsg::ShowWindow => self.show_window(),
            WindowActionMsg::HideWindow => self.hide_window(true),
            WindowActionMsg::ShowHud { display } => {
                let show_hud = show_hud_window(
                    #[cfg(target_os = "linux")]
                    self.wayland,
                )
                .map(AppMsg::WindowAction);

                Task::batch([Task::done(AppMsg::SetHudDisplay { display }), show_hud])
            }
        }
    }
    pub fn handle_unfocused_event(&mut self, window_id: window::Id) -> Task<AppMsg> {
        if !self.close_on_unfocus {
            return Task::none();
        }

        if window_id != self.main_window_id {
            return Task::none();
        }

        #[cfg(target_os = "linux")]
        if self.wayland {
            self.hide_window(true)
        } else {
            // x11 uses separate mechanism based on _NET_ACTIVE_WINDOW property
            Task::none()
        }

        #[cfg(not(target_os = "linux"))]
        self.on_unfocused()
    }

    pub fn handle_focused_event(&mut self, window_id: window::Id) -> Task<AppMsg> {
        if !self.close_on_unfocus {
            return Task::none();
        }

        if window_id != self.main_window_id {
            return Task::none();
        }

        self.on_focused()
    }

    pub fn handle_move_event(&mut self, window_id: window::Id, point: Point) -> Task<AppMsg> {
        if window_id != self.main_window_id {
            return Task::none();
        }

        if let Some(window_position_file) = &self.window_position_file {
            let _ = fs::create_dir_all(window_position_file.parent().unwrap());
            let _ = fs::write(&window_position_file, format!("{}:{}", point.x, point.y));
        }

        Task::none()
    }

    fn on_focused(&mut self) -> Task<AppMsg> {
        self.focused = true;
        Task::none()
    }

    #[allow(unused)]
    fn on_unfocused(&mut self) -> Task<AppMsg> {
        // for some reason (on both macOS and linux x11 but x11 now uses separate impl) duplicate Unfocused fires right before Focus event
        if self.focused {
            self.hide_window(true)
        } else {
            Task::none()
        }
    }

    fn toggle_window(&mut self) -> Task<AppMsg> {
        if self.opened {
            self.hide_window(false)
        } else {
            self.show_window()
        }
    }

    fn hide_window(&mut self, reset_state: bool) -> Task<AppMsg> {
        if !self.opened {
            return Task::none();
        }

        self.focused = false;
        self.opened = false;

        let mut commands = vec![];

        commands.push(
            hide_window(
                #[cfg(target_os = "linux")]
                self.wayland,
                self.main_window_id,
            )
            .map(AppMsg::WindowAction),
        );

        if reset_state {
            commands.push(Task::done(AppMsg::ClosePluginView));
            commands.push(Task::done(AppMsg::ResetWindowState));
        }

        commands.push(Task::done(AppMsg::ResetMainWindowScroll));

        Task::batch(commands)
    }

    fn show_window(&mut self) -> Task<AppMsg> {
        if self.opened {
            return Task::none();
        }

        self.opened = true;

        show_window(
            self.main_window_id,
            #[cfg(target_os = "linux")]
            self.wayland,
            #[cfg(target_os = "macos")]
            &self.window_position_mode,
        )
        .map(AppMsg::WindowAction)
    }
}

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
    #[cfg(target_os = "macos")] window_position_mode: &WindowPositionMode,
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
