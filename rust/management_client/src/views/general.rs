use crate::components::shortcut_selector::ShortcutSelector;
use crate::theme::text::TextStyle;
use crate::theme::Element;
use gauntlet_common::model::{PhysicalShortcut, SettingsTheme, WindowPositionMode};
use gauntlet_common::rpc::backend_api::{BackendApi, BackendApiError};
use iced::alignment::Horizontal;
use iced::widget::text::Shaping;
use iced::widget::tooltip::Position;
use iced::widget::{column, container, pick_list, row, text, tooltip, value, Space};
use iced::{alignment, Alignment, Length, Padding, Task};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use crate::theme::container::ContainerStyle;

pub struct ManagementAppGeneralState {
    backend_api: Option<BackendApi>,
    theme: SettingsTheme,
    window_position_mode: WindowPositionMode,
    current_shortcut: Option<PhysicalShortcut>,
    current_shortcut_error: Option<String>,
    currently_capturing: bool
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsgIn {
    ShortcutCaptured(Option<PhysicalShortcut>),
    CapturingChanged(bool),
    ThemeChanged(SettingsTheme),
    WindowPositionModeChanged(WindowPositionMode),
    SetGlobalShortcutResponse {
        shortcut: Option<PhysicalShortcut>,
        shortcut_error: Option<String>
    },
    InitSetting {
        theme: SettingsTheme,
        window_position_mode: WindowPositionMode,
        shortcut: Option<PhysicalShortcut>,
        shortcut_error: Option<String>
    },
    Noop
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsgOut {
    Noop,
    SetGlobalShortcutResponse {
        shortcut: Option<PhysicalShortcut>,
        shortcut_error: Option<String>
    },
    HandleBackendError(BackendApiError)
}

impl ManagementAppGeneralState {
    pub fn new(backend_api: Option<BackendApi>) -> Self {
        Self {
            backend_api,
            theme: SettingsTheme::AutoDetect,
            window_position_mode: WindowPositionMode::Static,
            current_shortcut: None,
            current_shortcut_error: None,
            currently_capturing: false,
        }
    }

    pub fn update(&mut self, message: ManagementAppGeneralMsgIn) -> Task<ManagementAppGeneralMsgOut> {
        let backend_api = match &self.backend_api {
            Some(backend_api) => backend_api.clone(),
            None => {
                return Task::none()
            }
        };

        match message {
            ManagementAppGeneralMsgIn::ShortcutCaptured(shortcut) => {
                let mut backend_api = backend_api.clone();

                Task::perform(
                    {
                        let shortcut = shortcut.clone();

                        async move {
                            let error = backend_api.set_global_shortcut(shortcut)
                                .await?;

                            Ok(error)
                        }
                    },
                    move |result| {
                        let shortcut = shortcut.clone();

                        handle_backend_error(result, move |shortcut_error| ManagementAppGeneralMsgOut::SetGlobalShortcutResponse {
                            shortcut,
                            shortcut_error,
                        })
                    },
                )
            }
            ManagementAppGeneralMsgIn::Noop => {
                Task::none()
            }
            ManagementAppGeneralMsgIn::InitSetting { theme, window_position_mode, shortcut, shortcut_error } => {
                self.theme = theme;
                self.window_position_mode = window_position_mode;
                self.current_shortcut = shortcut;
                self.current_shortcut_error = shortcut_error;

                Task::done(ManagementAppGeneralMsgOut::Noop)
            }
            ManagementAppGeneralMsgIn::CapturingChanged(capturing) => {
                self.currently_capturing = capturing;

                Task::none()
            }
            ManagementAppGeneralMsgIn::ThemeChanged(theme) => {
                self.theme = theme.clone();

                let mut backend_api = backend_api.clone();

                Task::perform(async move {
                    backend_api.set_theme(theme)
                        .await?;

                    Ok(())
                }, |result| handle_backend_error(result, |()| ManagementAppGeneralMsgOut::Noop))

            }
            ManagementAppGeneralMsgIn::WindowPositionModeChanged(mode) => {
                self.window_position_mode = mode.clone();

                let mut backend_api = backend_api.clone();

                Task::perform(async move {
                    backend_api.set_window_position_mode(mode)
                        .await?;

                    Ok(())
                }, |result| handle_backend_error(result, |()| ManagementAppGeneralMsgOut::Noop))
            }
            ManagementAppGeneralMsgIn::SetGlobalShortcutResponse { shortcut, shortcut_error } => {
                self.current_shortcut = shortcut;
                self.current_shortcut_error = shortcut_error;

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<ManagementAppGeneralMsgIn> {

        let global_shortcut_selector: Element<_> = ShortcutSelector::new(
            &self.current_shortcut,
            move |value| { ManagementAppGeneralMsgIn::ShortcutCaptured(value) },
            move |value| { ManagementAppGeneralMsgIn::CapturingChanged(value) },
        ).into();

        let global_shortcut_field: Element<_> = container(global_shortcut_selector)
            .width(Length::Fill)
            .height(Length::Fixed(35.0))
            .into();

        let global_shortcut_field = self.view_field(
            "Global Shortcut",
            global_shortcut_field,
            Some(self.shortcut_capture_after())
        );

        let theme_field = self.theme_field();

        let mut content = vec![global_shortcut_field, theme_field];

        #[cfg(target_os = "macos")]
        {
            content.push(self.window_position_mode_field())
        }

        let content: Element<_> = column(content)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .into();

        content
    }

    fn theme_field(&self) -> Element<ManagementAppGeneralMsgIn> {
        let theme_field = match &self.theme {
            SettingsTheme::ThemeFile => {
                let theme_field: Element<_> = text("Unable to change because theme config file is present ")
                    .shaping(Shaping::Advanced)
                    .align_x(Horizontal::Center)
                    .width(Length::Fill)
                    .into();

                theme_field
            }
            SettingsTheme::Config => {
                let theme_field: Element<_> = text("Unable to change because value is defined in config")
                    .shaping(Shaping::Advanced)
                    .align_x(Horizontal::Center)
                    .width(Length::Fill)
                    .into();

                theme_field
            }
            _ => {
                let theme_items = [
                    SettingsTheme::AutoDetect,
                    SettingsTheme::MacOSLight,
                    SettingsTheme::MacOSDark,
                    SettingsTheme::Legacy,
                ];

                let theme_field: Element<_> = pick_list(
                    theme_items,
                    Some(self.theme.clone()),
                    move |item| ManagementAppGeneralMsgIn::ThemeChanged(item),
                ).into();

                theme_field
            }
        };

        let theme_field: Element<_> = container(theme_field)
            .width(Length::Fill)
            .into();

        let theme_field = self.view_field(
            "Theme",
            theme_field,
            None
        );

        theme_field
    }

    fn window_position_mode_field(&self) -> Element<ManagementAppGeneralMsgIn> {
        let items = [
            WindowPositionMode::Static,
            WindowPositionMode::ActiveMonitor,
        ];

        let field: Element<_> = pick_list(
            items,
            Some(self.window_position_mode.clone()),
            move |item| ManagementAppGeneralMsgIn::WindowPositionModeChanged(item),
        ).into();

        let field: Element<_> = container(field)
            .width(Length::Fill)
            .into();

        let field = self.view_field(
            "Window Position Mode",
            field,
            None
        );

        field
    }

    fn view_field<'a>(&'a self, label: &'a str, input: Element<'a, ManagementAppGeneralMsgIn>, after: Option<Element<'a, ManagementAppGeneralMsgIn>>) -> Element<'a, ManagementAppGeneralMsgIn> {
        let label: Element<_> = text(label)
            .shaping(Shaping::Advanced)
            .align_x(Horizontal::Right)
            .width(Length::Fill)
            .into();

        let label: Element<_> = container(label)
            .width(Length::FillPortion(3))
            .padding(4)
            .into();

        let input_field = container(input)
            .width(Length::FillPortion(3))
            .padding(4)
            .into();

        let after = after.unwrap_or_else(|| {
            Space::with_width(Length::FillPortion(3))
                .into()
        });

        let content = vec![
            label,
            input_field,
            after,
        ];

        let row: Element<_> = row(content)
            .align_y(Alignment::Center)
            .padding(12)
            .into();

        row
    }

    fn shortcut_capture_after(&self) -> Element<ManagementAppGeneralMsgIn> {
        if self.currently_capturing {
            let hint1: Element<_> = text("Backspace - Unset Shortcut")
                .width(Length::Fill)
                .class(TextStyle::Subtitle)
                .into();

            let hint2: Element<_> = text("Escape - Stop Capturing")
                .width(Length::Fill)
                .class(TextStyle::Subtitle)
                .into();

            column(vec![hint1, hint2])
                .width(Length::FillPortion(3))
                .align_x(Alignment::Center)
                .padding(Padding::from([0.0, 8.0]))
                .into()
        } else {
            if let Some(current_shortcut_error) = &self.current_shortcut_error {
                let error_icon: Element<_> = value(Bootstrap::ExclamationTriangleFill)
                    .font(BOOTSTRAP_FONT)
                    .class(TextStyle::Destructive)
                    .into();

                let error_text: Element<_> = text(current_shortcut_error)
                    .class(TextStyle::Destructive)
                    .into();

                let error_text: Element<_> = container(error_text)
                    .padding(16.0)
                    .max_width(300)
                    .class(ContainerStyle::Box)
                    .into();

                let tooltip: Element<_> = tooltip(error_icon, error_text, Position::Bottom)
                    .into();

                let content = container(tooltip)
                    .width(Length::FillPortion(3))
                    .align_y(alignment::Vertical::Center)
                    .padding(Padding::from([0.0, 8.0]))
                    .into();

                content
            } else {
                Space::with_width(Length::FillPortion(3))
                    .into()
            }
        }
    }
}

pub fn handle_backend_error<T>(result: Result<T, BackendApiError>, convert: impl FnOnce(T) -> ManagementAppGeneralMsgOut) -> ManagementAppGeneralMsgOut {
    match result {
        Ok(val) => convert(val),
        Err(err) => ManagementAppGeneralMsgOut::HandleBackendError(err)
    }
}
