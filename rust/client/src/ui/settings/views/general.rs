use std::sync::Arc;

use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::SettingsTheme;
use gauntlet_common::model::WindowPositionMode;
use gauntlet_server::global_hotkey::GlobalHotKeyManager;
use gauntlet_server::plugins::ApplicationManager;
use gauntlet_utils::channel::RequestResult;
use iced::Alignment;
use iced::Font;
use iced::Length;
use iced::Padding;
use iced::Task;
use iced::alignment;
use iced::alignment::Horizontal;
use iced::font::Style;
use iced::widget::Space;
use iced::widget::column;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text::Shaping;

use crate::ui::settings::components::shortcut_selector::ShortcutData;
use crate::ui::settings::components::shortcut_selector::render_shortcut_error;
use crate::ui::settings::components::shortcut_selector::shortcut_selector;
use crate::ui::settings::theme::Element;
use crate::ui::settings::theme::container::ContainerStyle;
use crate::ui::settings::ui::SettingsMsg;

pub struct SettingsGeneralState {
    application_manager: Arc<ApplicationManager>,
    theme: SettingsTheme,
    window_position_mode: WindowPositionMode,
    current_shortcut: ShortcutData,
    global_shortcuts_unsupported: bool,
}

#[derive(Debug, Clone)]
pub enum SettingsGeneralMsgIn {
    ShortcutCaptured(Option<PhysicalShortcut>),
    ThemeChanged(SettingsTheme),
    WindowPositionModeChanged(WindowPositionMode),
    HandleShortcutResponse {
        shortcut: Option<PhysicalShortcut>,
        shortcut_error: Option<String>,
    },
    InitSetting {
        theme: SettingsTheme,
        window_position_mode: WindowPositionMode,
        shortcut: Option<PhysicalShortcut>,
        shortcut_error: Option<String>,
        global_shortcuts_unsupported: bool,
    },
}

#[derive(Debug, Clone)]
pub enum SettingsGeneralMsgOut {
    Inner(SettingsGeneralMsgIn),
    Outer(SettingsMsg),
}

impl SettingsGeneralState {
    pub fn new(application_manager: Arc<ApplicationManager>) -> Self {
        Self {
            application_manager,
            theme: SettingsTheme::AutoDetect,
            window_position_mode: WindowPositionMode::Static,
            current_shortcut: ShortcutData {
                shortcut: None,
                error: None,
            },
            global_shortcuts_unsupported: false,
        }
    }

    pub fn update(
        &mut self,
        global_hotkey_manager: &Option<GlobalHotKeyManager>,
        message: SettingsGeneralMsgIn,
    ) -> Task<SettingsGeneralMsgOut> {
        match message {
            SettingsGeneralMsgIn::ShortcutCaptured(shortcut) => {
                let Some(global_hotkey_manager) = &global_hotkey_manager else {
                    return Task::none();
                };

                let error = self
                    .application_manager
                    .set_global_shortcut(global_hotkey_manager, shortcut.clone());

                Task::done(SettingsGeneralMsgOut::Inner(
                    SettingsGeneralMsgIn::HandleShortcutResponse {
                        shortcut,
                        shortcut_error: error,
                    },
                ))
            }
            SettingsGeneralMsgIn::InitSetting {
                theme,
                window_position_mode,
                shortcut,
                shortcut_error,
                global_shortcuts_unsupported,
            } => {
                self.theme = theme;
                self.window_position_mode = window_position_mode;
                self.current_shortcut = ShortcutData {
                    shortcut,
                    error: shortcut_error,
                };
                self.global_shortcuts_unsupported = global_shortcuts_unsupported;

                Task::none()
            }
            SettingsGeneralMsgIn::ThemeChanged(theme) => {
                self.theme = theme.clone();

                let application_manager = self.application_manager.clone();

                Task::perform(
                    async move {
                        application_manager.set_theme(theme).await?;

                        Ok(())
                    },
                    |result| handle_backend_error(result, |()| SettingsGeneralMsgOut::Outer(SettingsMsg::Noop)),
                )
            }
            SettingsGeneralMsgIn::WindowPositionModeChanged(mode) => {
                self.window_position_mode = mode.clone();

                let application_manager = self.application_manager.clone();

                Task::perform(
                    async move {
                        application_manager.set_window_position_mode(mode).await?;

                        Ok(())
                    },
                    |result| handle_backend_error(result, |()| SettingsGeneralMsgOut::Outer(SettingsMsg::Noop)),
                )
            }
            SettingsGeneralMsgIn::HandleShortcutResponse {
                shortcut,
                shortcut_error,
            } => {
                self.current_shortcut = ShortcutData {
                    shortcut,
                    error: shortcut_error,
                };

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<SettingsGeneralMsgIn> {
        let global_shortcut_selector: Element<_> = if self.global_shortcuts_unsupported {
            let text = text("Not supported").font(Font {
                style: Style::Italic,
                ..Font::DEFAULT
            });

            container(text)
                .width(Length::Fill)
                .height(Length::Fill)
                .center(Length::Fill)
                .class(ContainerStyle::Box)
                .into()
        } else {
            shortcut_selector(
                &self.current_shortcut,
                move |shortcut| SettingsGeneralMsgIn::ShortcutCaptured(shortcut),
                ContainerStyle::Box,
                false,
            )
            .into()
        };

        let global_shortcut_field: Element<_> = container(global_shortcut_selector)
            .width(Length::Fill)
            .height(Length::Fixed(35.0))
            .into();

        let global_shortcut_field = self.view_field(
            "Global Shortcut",
            global_shortcut_field,
            Some(self.shortcut_capture_after()),
        );

        let theme_field = self.theme_field();

        #[allow(unused_mut)]
        let mut content = vec![global_shortcut_field, theme_field];

        #[cfg(target_os = "macos")]
        {
            content.push(self.window_position_mode_field())
        }

        let content: Element<_> = column(content).into();

        let content: Element<_> = container(content).width(Length::Fill).into();

        content
    }

    fn theme_field(&self) -> Element<SettingsGeneralMsgIn> {
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

                let theme_field: Element<_> = pick_list(theme_items, Some(self.theme.clone()), move |item| {
                    SettingsGeneralMsgIn::ThemeChanged(item)
                })
                .into();

                theme_field
            }
        };

        let theme_field: Element<_> = container(theme_field).width(Length::Fill).into();

        let theme_field = self.view_field("Theme", theme_field, None);

        theme_field
    }

    #[allow(unused)]
    fn window_position_mode_field(&self) -> Element<SettingsGeneralMsgIn> {
        let items = [WindowPositionMode::Static, WindowPositionMode::ActiveMonitor];

        let field: Element<_> = pick_list(items, Some(self.window_position_mode.clone()), move |item| {
            SettingsGeneralMsgIn::WindowPositionModeChanged(item)
        })
        .into();

        let field: Element<_> = container(field).width(Length::Fill).into();

        let field = self.view_field("Window Position Mode", field, None);

        field
    }

    fn view_field<'a>(
        &'a self,
        label: &'a str,
        input: Element<'a, SettingsGeneralMsgIn>,
        after: Option<Element<'a, SettingsGeneralMsgIn>>,
    ) -> Element<'a, SettingsGeneralMsgIn> {
        let label: Element<_> = text(label)
            .shaping(Shaping::Advanced)
            .align_x(Horizontal::Right)
            .width(Length::Fill)
            .into();

        let label: Element<_> = container(label).width(Length::FillPortion(3)).padding(4).into();

        let input_field = container(input).width(Length::FillPortion(3)).padding(4).into();

        let after = after.unwrap_or_else(|| Space::with_width(Length::FillPortion(3)).into());

        let content = vec![label, input_field, after];

        let row: Element<_> = row(content).align_y(Alignment::Center).padding(12).into();

        row
    }

    fn shortcut_capture_after(&self) -> Element<SettingsGeneralMsgIn> {
        if let Some(current_shortcut_error) = &self.current_shortcut.error {
            let content = render_shortcut_error(current_shortcut_error.clone());

            let content = container(content)
                .width(Length::FillPortion(3))
                .align_y(alignment::Vertical::Center)
                .padding(Padding::from([0.0, 8.0]))
                .into();

            content
        } else {
            Space::with_width(Length::FillPortion(3)).into()
        }
    }
}

fn handle_backend_error<T>(
    result: RequestResult<T>,
    convert: impl FnOnce(T) -> SettingsGeneralMsgOut,
) -> SettingsGeneralMsgOut {
    match result {
        Ok(val) => convert(val),
        Err(err) => SettingsGeneralMsgOut::Outer(SettingsMsg::HandleBackendError(err)),
    }
}
