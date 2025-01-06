use crate::components::shortcut_selector::ShortcutSelector;
use crate::theme::text::TextStyle;
use crate::theme::Element;
use gauntlet_common::model::{PhysicalShortcut, SettingsTheme};
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
    current_shortcut: Option<PhysicalShortcut>,
    current_shortcut_error: Option<String>,
    currently_capturing: bool
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsgIn {
    ShortcutCaptured(Option<PhysicalShortcut>),
    CapturingChanged(bool),
    ThemeChanged(SettingsTheme),
    InitSetting {
        theme: SettingsTheme,
        shortcut: Option<PhysicalShortcut>,
        shortcut_error: Option<String>
    },
    Noop
}

#[derive(Debug, Clone)]
pub enum ManagementAppGeneralMsgOut {
    Noop,
    HandleBackendError(BackendApiError)
}

impl ManagementAppGeneralState {
    pub fn new(backend_api: Option<BackendApi>) -> Self {
        Self {
            backend_api,
            theme: SettingsTheme::AutoDetect,
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
                self.current_shortcut = shortcut.clone();

                let mut backend_api = backend_api.clone();

                Task::perform(async move {
                    backend_api.set_global_shortcut(shortcut)
                        .await?;

                    Ok(())
                }, |result| handle_backend_error(result, |()| ManagementAppGeneralMsgOut::Noop))
            }
            ManagementAppGeneralMsgIn::Noop => {
                Task::none()
            }
            ManagementAppGeneralMsgIn::InitSetting { theme, shortcut, shortcut_error } => {
                self.theme = theme;
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

        let theme_field: Element<_> = container(theme_field)
            .width(Length::Fill)
            .into();

        let theme_field = self.view_field(
            "Theme",
            theme_field,
            None
        );

        let content: Element<_> = column(vec![global_shortcut_field, theme_field])
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .into();

        let content: Element<_> = container(content)
            .width(Length::Fill)
            .into();

        content
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
