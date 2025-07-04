use std::fs;
use std::path::PathBuf;

use gauntlet_common::model::WindowPositionMode;
use iced::Point;
use iced::Size;
use iced::Task;
use iced::window;
use iced::window::Level;
use iced::window::Position;

use crate::ui::AppMsg;
use crate::ui::windows::hud::show_hud_window;

pub mod hud;
#[cfg(target_os = "linux")]
pub mod x11_focus;

pub struct WindowState {
    pub main_window_id: Option<window::Id>,
    focused: bool,
    #[cfg(target_os = "linux")]
    pub wayland: bool,
    #[cfg(target_os = "linux")]
    pub layer_shell: bool,
    window_position_mode: WindowPositionMode,
    close_on_unfocus: bool,
    window_position_file: Option<PathBuf>,
    #[cfg(target_os = "linux")]
    x11_active_window: Option<u32>,
    open_position: Position,
}

impl WindowState {
    pub fn new(
        window_position_file: Option<PathBuf>,
        close_on_unfocus: bool,
        window_position_mode: WindowPositionMode,
        #[cfg(target_os = "linux")] wayland: bool,
        #[cfg(target_os = "linux")] layer_shell: bool,
    ) -> WindowState {
        let open_position = window_position_file
            .as_ref()
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

        Self {
            main_window_id: None,
            focused: false,
            #[cfg(target_os = "linux")]
            layer_shell,
            #[cfg(target_os = "linux")]
            wayland,
            window_position_mode,
            close_on_unfocus,
            window_position_file,
            #[cfg(target_os = "linux")]
            x11_active_window: None,
            open_position,
        }
    }
}

impl WindowState {
    pub fn handle_action(&mut self, action: WindowActionMsg) -> Task<AppMsg> {
        match action {
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
                    self.layer_shell,
                )
                .map(AppMsg::WindowAction);

                Task::batch([Task::done(AppMsg::SetHudDisplay { display }), show_hud])
            }
            WindowActionMsg::SetMainWindowId(id) => {
                self.main_window_id = id;
                Task::none()
            }
        }
    }
    pub fn handle_unfocused_event(&mut self, window_id: window::Id) -> Task<AppMsg> {
        if !self.close_on_unfocus {
            return Task::none();
        }

        let Some(main_window_id) = self.main_window_id else {
            return Task::none();
        };

        if window_id != main_window_id {
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

        let Some(main_window_id) = self.main_window_id else {
            return Task::none();
        };

        if window_id != main_window_id {
            return Task::none();
        }

        self.on_focused()
    }

    pub fn handle_move_event(&mut self, window_id: window::Id, point: Point) -> Task<AppMsg> {
        let Some(main_window_id) = self.main_window_id else {
            return Task::none();
        };

        if window_id != main_window_id {
            return Task::none();
        }

        if let Some(window_position_file) = &self.window_position_file {
            let _ = fs::create_dir_all(window_position_file.parent().unwrap());
            let _ = fs::write(&window_position_file, format!("{}:{}", point.x, point.y));
        }

        self.open_position = Position::Specific(Point::new(point.x, point.y));

        Task::none()
    }

    fn on_focused(&mut self) -> Task<AppMsg> {
        self.focused = true;
        Task::none()
    }

    #[allow(unused)]
    fn on_unfocused(&mut self) -> Task<AppMsg> {
        // for some reason (on both macOS and linux x11, but x11 now uses separate impl) duplicate Unfocused fires right before Focus event
        if self.focused {
            self.hide_window(true)
        } else {
            Task::none()
        }
    }

    fn toggle_window(&mut self) -> Task<AppMsg> {
        match self.main_window_id {
            Some(_) => self.hide_window(false),
            None => self.show_window(),
        }
    }

    fn hide_window(&mut self, reset_state: bool) -> Task<AppMsg> {
        let Some(main_window_id) = self.main_window_id else {
            return Task::none();
        };

        self.focused = false;

        let mut commands = vec![];

        commands.push(window::close(main_window_id));
        commands.push(Task::done(AppMsg::WindowAction(WindowActionMsg::SetMainWindowId(None))));

        if reset_state {
            commands.push(Task::done(AppMsg::ClosePluginView));
            commands.push(Task::done(AppMsg::ResetWindowState));
        }

        commands.push(Task::done(AppMsg::ResetMainWindowScroll));

        #[cfg(target_os = "macos")]
        macos_focus_previous_app();

        Task::batch(commands)
    }

    fn show_window(&mut self) -> Task<AppMsg> {
        if let Some(_) = self.main_window_id {
            return Task::none();
        };

        let (main_window_id, open_task) = window::open(window_settings(
            #[cfg(target_os = "linux")]
            self.layer_shell,
            self.open_position,
        ));

        Task::batch([
            open_task.map(|id| WindowActionMsg::SetMainWindowId(Some(id))),
            #[cfg(target_os = "macos")]
            match self.window_position_mode {
                WindowPositionMode::Static => Task::none(),
                WindowPositionMode::ActiveMonitor => window::move_to_active_monitor(main_window_id),
            },
            window::gain_focus(main_window_id),
            window::set_level(main_window_id, Level::AlwaysOnTop),
        ])
        .map(AppMsg::WindowAction)
    }
}

#[derive(Debug, Clone)]
pub enum WindowActionMsg {
    SetMainWindowId(Option<window::Id>),
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

const WINDOW_WIDTH: f32 = 750.0;
const WINDOW_HEIGHT: f32 = 450.0;

#[cfg(not(target_os = "macos"))]
fn window_settings(#[cfg(target_os = "linux")] layer_shell: bool, position: Position) -> window::Settings {
    window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position,
        resizable: false,
        decorations: false,
        visible: true,
        transparent: true,
        closeable: false,
        minimizable: false,
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "gauntlet".to_string(),
            layer_shell: if layer_shell {
                layer_shell_settings()
            } else {
                Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(target_os = "macos")]
fn window_settings(position: Position) -> window::Settings {
    window::Settings {
        size: Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position,
        resizable: false,
        decorations: true,
        visible: true,
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

#[cfg(target_os = "macos")]
pub fn macos_focus_previous_app() {
    unsafe {
        // when closing NSPanel current active application doesn't automatically become key window
        // is there a proper way? without doing this manually
        let app = objc2_app_kit::NSWorkspace::sharedWorkspace().menuBarOwningApplication();

        if let Some(app) = app {
            app.activateWithOptions(objc2_app_kit::NSApplicationActivationOptions::empty());
        }
    }
}

#[cfg(target_os = "linux")]
fn layer_shell_settings() -> window::settings::LayerShellSettings {
    window::settings::LayerShellSettings {
        layer: Some(window::settings::Layer::Top),
        anchor: None,
        output: None,
        exclusive_zone: Some(0),
        margin: None,
        input_region: None,
        keyboard_interactivity: Some(window::settings::KeyboardInteractivity::OnDemand),
        namespace: Some("gauntlet".to_string()),
    }
}
