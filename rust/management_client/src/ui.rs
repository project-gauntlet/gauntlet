use std::collections::HashMap;
use std::time::Duration;

use iced::{Alignment, alignment, Application, color, Command, executor, font, futures, Length, Padding, Settings, Size, Subscription, time, window};
use iced::advanced::text::Shaping;
use iced::advanced::Widget;
use iced::widget::{button, column, container, horizontal_rule, horizontal_space, mouse_area, row, scrollable, text};
use iced_aw::{floating_element, Spinner};
use iced_aw::core::icons;
use iced_aw::floating_element::{Anchor, Offset};
use itertools::Itertools;

use common::model::{DownloadStatus, PhysicalShortcut, PluginId};
use common::rpc::backend_api::{BackendApi, BackendApiError};

use crate::theme::{Element, GauntletSettingsTheme};
use crate::theme::button::ButtonStyle;
use crate::theme::container::ContainerStyle;
use crate::theme::text::TextStyle;
use crate::views::general::{ManagementAppGeneralMsgIn, ManagementAppGeneralMsgOut, ManagementAppGeneralState};
use crate::views::plugins::{ManagementAppPluginMsgIn, ManagementAppPluginMsgOut, ManagementAppPluginsState};

pub fn run() {
    ManagementAppModel::run(Settings {
        id: None,
        window: window::Settings {
            size: Size::new(1000.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    }).expect("Unable to start settings application");
}

struct ManagementAppModel {
    backend_api: Option<BackendApi>,
    error_view: Option<ErrorView>,
    downloads_info: HashMap<PluginId, DownloadInfo>,
    download_info_shown: bool,
    current_settings_view: SettingsView,
    general_state: ManagementAppGeneralState,
    plugins_state: ManagementAppPluginsState
}


#[derive(Debug, Clone)]
enum ManagementAppMsg {
    FontLoaded(Result<(), font::Error>),
    General(ManagementAppGeneralMsgIn),
    Plugin(ManagementAppPluginMsgIn),
    SwitchView(SettingsView),
    DownloadStatus { plugins: HashMap<PluginId, DownloadStatus> },
    HandleBackendError(BackendApiError),
    CheckDownloadStatus,
    DownloadPlugin { plugin_id: PluginId },
    Noop,
    ToggleDownloadInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SettingsView {
    General,
    Plugins
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ErrorView {
    UnknownError {
        display: String
    },
    Timeout,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)] // ordering used in sorting items in ui
pub enum DownloadInfo {
    InProgress,
    Error {
        message: String
    },
    Successful,
}

//noinspection RsSortImplTraitMembers
impl Application for ManagementAppModel {
    type Executor = executor::Default;
    type Message = ManagementAppMsg;
    type Theme = GauntletSettingsTheme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let backend_api = futures::executor::block_on(async {
            anyhow::Ok(BackendApi::new().await?)
        })
            .inspect_err(|err| tracing::error!("Unable to connect to server: {:?}", err))
            .ok();

        (
            ManagementAppModel {
                backend_api: backend_api.clone(),
                error_view: None,
                downloads_info: HashMap::new(),
                download_info_shown: false,
                current_settings_view: SettingsView::Plugins,
                general_state: ManagementAppGeneralState::new(backend_api.clone()),
                plugins_state: ManagementAppPluginsState::new(backend_api.clone()),
            },
            Command::batch([
                font::load(icons::BOOTSTRAP_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
                Command::perform(
                    async {},
                    |plugins| ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::RequestPluginReload)
                ),
                Command::perform(
                    async {
                        match backend_api {
                            Some(mut backend_api) => {
                                let shortcut = backend_api.get_global_shortcut()
                                    .await;

                                Some(shortcut)
                            }
                            None => None
                        }
                    },
                    |shortcut| {
                        match shortcut {
                            None => ManagementAppMsg::General(ManagementAppGeneralMsgIn::Noop),
                            Some(shortcut) => {
                                match shortcut {
                                    Ok(shortcut) => ManagementAppMsg::General(ManagementAppGeneralMsgIn::SetShortcut(shortcut)),
                                    Err(err) => ManagementAppMsg::HandleBackendError(err)
                                }
                            }
                        }
                    }
                ),
            ]),
        )
    }

    fn title(&self) -> String {
        "Gauntlet Settings".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let backend_api = match &self.backend_api {
            Some(backend_api) => backend_api.clone(),
            None => {
                return Command::none()
            }
        };

        match message {
            ManagementAppMsg::Plugin(message) => {
                self.plugins_state.update(message)
                    .map(|msg| {
                        match msg {
                            ManagementAppPluginMsgOut::PluginsReloaded(plugins) => {
                                ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::PluginsReloaded(plugins))
                            }
                            ManagementAppPluginMsgOut::Noop => {
                                ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::Noop)
                            }
                            ManagementAppPluginMsgOut::DownloadPlugin { plugin_id } => {
                                ManagementAppMsg::DownloadPlugin { plugin_id }
                            }
                            ManagementAppPluginMsgOut::SelectedItem(selected_item) => {
                                ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::SelectItem(selected_item))
                            }
                            ManagementAppPluginMsgOut::HandleBackendError(err) => {
                                ManagementAppMsg::HandleBackendError(err)
                            }
                        }
                    })
            }
            ManagementAppMsg::General(message) => {
                self.general_state.update(message)
                    .map(|msg| {
                        match msg {
                            ManagementAppGeneralMsgOut::Noop => {
                                ManagementAppMsg::General(ManagementAppGeneralMsgIn::Noop)
                            },
                            ManagementAppGeneralMsgOut::HandleBackendError(err) => {
                                ManagementAppMsg::HandleBackendError(err)
                            }
                        }
                    })
            }
            ManagementAppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
            ManagementAppMsg::SwitchView(view) => {
                self.current_settings_view = view;

                Command::none()
            }
            ManagementAppMsg::HandleBackendError(err) => {
                self.error_view = Some(match err {
                    BackendApiError::Timeout => ErrorView::Timeout,
                    BackendApiError::Internal { display } => ErrorView::UnknownError { display }
                });

                Command::none()
            }
            ManagementAppMsg::DownloadStatus { plugins } => {
                for (plugin, status) in plugins {
                    match status {
                        DownloadStatus::InProgress => {
                            self.downloads_info.insert(plugin.clone(), DownloadInfo::InProgress);
                        }
                        DownloadStatus::Done => {
                            self.downloads_info.insert(plugin.clone(), DownloadInfo::Successful);
                        }
                        DownloadStatus::Failed { message } => {
                            self.downloads_info.insert(plugin.clone(), DownloadInfo::Error { message });
                        }
                    }
                }

                let mut backend_api = backend_api.clone();

                Command::perform(
                    async move {
                        let plugins = backend_api.plugins()
                            .await?;

                        Ok(plugins)
                    },
                    |result| handle_backend_error(result, |plugins| ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::PluginsReloaded(plugins)))
                )
            }
            ManagementAppMsg::CheckDownloadStatus => {
                if self.downloads_info.is_empty() {
                    Command::none()
                } else {
                    let mut backend_client = backend_api.clone();

                    Command::perform(
                        async move {
                            let plugins = backend_client.download_status()
                                .await?;

                            Ok(plugins)
                        },
                        |result| handle_backend_error(result, |plugins| ManagementAppMsg::DownloadStatus { plugins }),
                    )
                }
            }
            ManagementAppMsg::DownloadPlugin { plugin_id } => {
                let mut backend_client = backend_api.clone();

                let already_downloading = self.downloads_info.insert(plugin_id.clone(), DownloadInfo::InProgress)
                    .is_some();

                if already_downloading {
                    Command::none()
                } else {
                    Command::perform(
                        async move {
                            backend_client.download_plugin(plugin_id)
                                .await?;

                            Ok(())
                        },
                        |result| handle_backend_error(result, |()| ManagementAppMsg::Noop)
                    )
                }
            }
            ManagementAppMsg::Noop => Command::none(),
            ManagementAppMsg::ToggleDownloadInfo => {
                self.download_info_shown = !self.download_info_shown;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if let None = &self.backend_api {
            let description: Element<_> = text("Unable to connect to server. Please check if you have Gauntlet running on your PC")
                .into();

            let content: Element<_> = container(description)
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill)
                .into();

            return content
        }

        if let Some(err) = &self.error_view {
            return match err {
                ErrorView::Timeout => {
                    let description: Element<_> = text("Error occurred")
                        .into();

                    let description = container(description)
                        .width(Length::Fill)
                        .center_x()
                        .padding(12)
                        .into();

                    let sub_description: Element<_> = text("Backend was unable to process message in a timely manner")
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .center_x()
                        .padding(12)
                        .into();

                    let content: Element<_> = column([
                        description,
                        sub_description,
                    ]).into();

                    let content: Element<_> = container(content)
                        .center_x()
                        .center_y()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into();

                    content
                }
                ErrorView::UnknownError { display } => {
                    let description: Element<_> = text("Unknown error occurred")
                        .into();

                    let description = container(description)
                        .width(Length::Fill)
                        .center_x()
                        .padding(12)
                        .into();

                    let sub_description: Element<_> = text("Please report")
                        .into();

                    let sub_description = container(sub_description)
                        .width(Length::Fill)
                        .center_x()
                        .padding(12)
                        .into();

                    let error_description: Element<_> = text(display)
                        .shaping(Shaping::Advanced)
                        .into();

                    let error_description = container(error_description)
                        .width(Length::Fill)
                        .center_x()
                        .padding(12)
                        .into();

                    let content: Element<_> = column([
                        description,
                        sub_description,
                        error_description,
                    ]).into();

                    let content: Element<_> = container(content)
                        .center_x()
                        .center_y()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into();

                    content
                }
            }
        }


        let content = match self.current_settings_view {
            SettingsView::General => {
                self.general_state.view()
                    .map(|msg| ManagementAppMsg::General(msg))
            }
            SettingsView::Plugins => {
                self.plugins_state.view()
                    .map(|msg| ManagementAppMsg::Plugin(msg))
            }
        };

        let icon_general: Element<_> = text(icons::Bootstrap::GearFill)
            .font(icons::BOOTSTRAP_FONT)
            .height(Length::Fill)
            .width(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
            .into();

        let text_general: Element<_> = text("General")
            .height(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
            .into();

        let general_button: Element<_> = column(vec![icon_general, text_general])
            .align_items(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();

        let general_button: Element<_> = button(general_button)
            .on_press(ManagementAppMsg::SwitchView(SettingsView::General))
            .height(Length::Fill)
            .width(80)
            .style(if self.current_settings_view == SettingsView::General { ButtonStyle::ViewSwitcherSelected } else { ButtonStyle::ViewSwitcher })
            .into();

        let general_button: Element<_> = container(general_button)
            .padding(8.0)
            .into();

        let icon_plugins: Element<_> = text(icons::Bootstrap::PuzzleFill)
            .font(icons::BOOTSTRAP_FONT)
            .height(Length::Fill)
            .width(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
            .into();

        let text_plugins: Element<_> = text("Plugins")
            .height(Length::Fill)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center)
            .into();

        let plugins_button: Element<_> = column(vec![icon_plugins, text_plugins])
            .align_items(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();

        let plugins_button: Element<_> = button(plugins_button)
            .on_press(ManagementAppMsg::SwitchView(SettingsView::Plugins))
            .height(Length::Fill)
            .width(80)
            .style(if self.current_settings_view == SettingsView::Plugins { ButtonStyle::ViewSwitcherSelected } else { ButtonStyle::ViewSwitcher })
            .into();

        let plugins_button: Element<_> = container(plugins_button)
            .padding(8.0)
            .into();

        let top_bar_buttons: Element<_> = row(vec![general_button, plugins_button])
            .into();

        let top_bar_buttons: Element<_> = container(top_bar_buttons)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .into();

        let top_bar_left_space: Element<_> = horizontal_space()
            .width(Length::Fill)
            .into();

        let top_bar_right = {
            let mut successful_count = 0;
            let mut in_progress_count = 0;
            let mut error_count = 0;

            for (_, download_info) in self.downloads_info.iter() {
                match download_info {
                    DownloadInfo::Successful => {
                        successful_count += 1;
                    }
                    DownloadInfo::InProgress => {
                        in_progress_count += 1;
                    }
                    DownloadInfo::Error { .. } => {
                        error_count += 1;
                    }
                }
            }

            let mut download_info_icons = vec![];

            if in_progress_count > 0 {
                let spinner: Element<_> = Spinner::new()
                    .width(Length::Fixed(16.0))
                    .height(Length::Fill)
                    .into();

                let spinner: Element<_> = container(spinner)
                    .height(Length::Fill)
                    .into();

                let text: Element<_> = text(in_progress_count)
                    .height(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center)
                    .into();

                let spinner: Element<_> = row(vec![text, spinner])
                    .spacing(8.0)
                    .into();

                download_info_icons.push(spinner);
            }
            if successful_count > 0 {
                let icon: Element<_> = text(icons::Bootstrap::PatchCheckFill)
                    .size(16)
                    .vertical_alignment(alignment::Vertical::Center)
                    .font(icons::BOOTSTRAP_FONT)
                    .height(Length::Fill)
                    .style(TextStyle::Positive)
                    .into();

                let icon: Element<_> = container(icon)
                    .height(Length::Fill)
                    .into();

                let text: Element<_> = text(successful_count)
                    .height(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center)
                    .into();

                let icon: Element<_> = row(vec![text, icon])
                    .spacing(8.0)
                    .into();

                download_info_icons.push(icon);
            }
            if error_count > 0 {
                let icon: Element<_> = text(icons::Bootstrap::ExclamationTriangleFill)
                    .font(icons::BOOTSTRAP_FONT)
                    .height(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center)
                    .size(16)
                    .style(TextStyle::Destructive)
                    .into();

                let icon: Element<_> = container(icon)
                    .height(Length::Fill)
                    .into();

                let text: Element<_> = text(error_count)
                    .height(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center)
                    .into();

                let icon: Element<_> = row(vec![text, icon])
                    .spacing(8.0)
                    .into();

                download_info_icons.push(icon);
            }

            if download_info_icons.is_empty() {
                horizontal_space()
                    .width(Length::Fill)
                    .into()
            } else {
                let top_bar_right: Element<_> = row(download_info_icons)
                    .spacing(12.0)
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .into();

                let top_bar_right: Element<_> = button(top_bar_right)
                    .style(ButtonStyle::DownloadInfo)
                    .on_press(ManagementAppMsg::ToggleDownloadInfo)
                    .padding(Padding::from([4, 8]))
                    .height(Length::Fill)
                    .into();

                let top_bar_right: Element<_> = container(top_bar_right)
                    .height(Length::Fill)
                    .padding(Padding::from([18.0, 12.0]))
                    .into();

                let top_bar_right: Element<_> = container(top_bar_right)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_y()
                    .align_x(alignment::Horizontal::Right)
                    .into();

                top_bar_right
            }
        };

        let top_bar: Element<_> = row(vec![top_bar_left_space, top_bar_buttons, top_bar_right])
            .width(Length::Fill)
            .into();

        let top_bar: Element<_> = container(top_bar)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Shrink)
            .max_height(70)
            .into();

        let separator: Element<_> = horizontal_rule(1)
            .into();

        let content: Element<_> = column(vec![top_bar, separator, content])
            .into();

        let download_info_panel: Element<_> = {
            let downloads: Vec<Element<_>> = self.downloads_info.iter()
                .sorted_by_key(|(_, info)| info.clone())
                .map(|(plugin_id, info)| {
                    match info {
                        DownloadInfo::InProgress => {
                            let kind_text: Element<_> = text("Download in progress")
                                .into();

                            let kind_text: Element<_> = container(kind_text)
                                .padding(Padding::from([16, 0, 8, 0]))
                                .into();

                            let plugin_id: Element<_> = text(plugin_id.to_string())
                                .shaping(Shaping::Advanced)
                                .style(TextStyle::Subtitle)
                                .size(14)
                                .into();

                            let plugin_id: Element<_> = container(plugin_id)
                                .padding(Padding::from([0, 0, 16, 0]))
                                .into();

                            let spinner: Element<_> = Spinner::new()
                                .width(Length::Fixed(32.0))
                                .into();

                            let spinner: Element<_> = container(spinner)
                                .padding(16)
                                .into();

                            let content: Element<_> = column(vec![kind_text, plugin_id])
                                .into();

                            let content: Element<_> = row(vec![spinner, content])
                                .into();

                            container(content)
                                .width(Length::Fill)
                                .into()
                        }
                        DownloadInfo::Error { message } => {
                            let kind_text: Element<_> = text("Download failed")
                                .into();

                            let kind_text: Element<_> = container(kind_text)
                                .padding(Padding::from([16, 0, 8, 0]))
                                .into();

                            let plugin_id: Element<_> = text(plugin_id.to_string())
                                .shaping(Shaping::Advanced)
                                .style(TextStyle::Subtitle)
                                .size(14)
                                .into();

                            let icon: Element<_> = text(icons::Bootstrap::ExclamationTriangleFill)
                                .font(icons::BOOTSTRAP_FONT)
                                .vertical_alignment(alignment::Vertical::Center)
                                .size(32)
                                .style(TextStyle::Destructive)
                                .into();

                            let icon: Element<_> = container(icon)
                                .padding(16)
                                .into();

                            let message: Element<_> = text(message.to_string())
                                .shaping(Shaping::Advanced)
                                .into();

                            let message: Element<_> = container(message)
                                .padding(Padding::from([8, 0, 16, 0]))
                                .into();

                            let content: Element<_> = column(vec![kind_text, plugin_id, message])
                                .into();

                            let content: Element<_> = row(vec![icon, content])
                                .into();

                            container(content)
                                .width(Length::Fill)
                                .into()
                        }
                        DownloadInfo::Successful => {
                            let kind_text: Element<_> = text("Download successful")
                                .into();

                            let kind_text: Element<_> = container(kind_text)
                                .padding(Padding::from([16, 0, 8, 0]))
                                .into();

                            let plugin_id: Element<_> = text(plugin_id.to_string())
                                .shaping(Shaping::Advanced)
                                .size(14)
                                .style(TextStyle::Subtitle)
                                .into();

                            let plugin_id: Element<_> = container(plugin_id)
                                .padding(Padding::from([0, 0, 16, 0]))
                                .into();

                            let icon: Element<_> = text(icons::Bootstrap::PatchCheckFill)
                                .size(32)
                                .vertical_alignment(alignment::Vertical::Center)
                                .font(icons::BOOTSTRAP_FONT)
                                .style(TextStyle::Positive)
                                .into();

                            let icon: Element<_> = container(icon)
                                .padding(16)
                                .into();

                            let content: Element<_> = column(vec![kind_text, plugin_id])
                                .into();

                            let content: Element<_> = row(vec![icon, content])
                                .into();

                            container(content)
                                .width(Length::Fill)
                                .into()
                        }
                    }
                })
                .intersperse_with(|| horizontal_rule(1).into())
                .collect();

            let downloads: Element<_> = column(downloads)
                .into();

            let downloads: Element<_> = scrollable(downloads)
                .width(Length::Fill)
                .into();

            container(downloads)
                .padding(4)
                .width(Length::Fixed(400.0))
                .max_height(500.0)
                .style(ContainerStyle::Box)
                .into()
        };

        let content = if !self.download_info_shown {
            content
        } else {
            mouse_area(content)
                .on_press(ManagementAppMsg::ToggleDownloadInfo)
                .into()
        };

        floating_element(content, download_info_panel)
            .offset(Offset::from([8.0, 60.0]))
            .anchor(Anchor::NorthEast)
            .hide(!self.download_info_shown)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_millis(300))
            .map(|_| ManagementAppMsg::CheckDownloadStatus)
    }

    fn theme(&self) -> Self::Theme {
        GauntletSettingsTheme::default()
    }
}

pub fn handle_backend_error<T>(result: Result<T, BackendApiError>, convert: impl FnOnce(T) -> ManagementAppMsg) -> ManagementAppMsg {
    match result {
        Ok(val) => convert(val),
        Err(err) => ManagementAppMsg::HandleBackendError(err)
    }
}
