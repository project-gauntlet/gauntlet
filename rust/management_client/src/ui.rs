use std::collections::HashMap;
use std::time::Duration;

use iced::{Alignment, alignment, font, futures, Length, Padding, Size, Subscription, time, window, Task, Renderer, padding};
use iced::advanced::text::Shaping;
use iced::widget::{button, column, container, horizontal_rule, horizontal_space, mouse_area, row, scrollable, stack, text, value};
use iced_aw::Spinner;
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT, BOOTSTRAP_FONT_BYTES};
use itertools::Itertools;

use gauntlet_common::model::{DownloadStatus, PhysicalShortcut, PluginId, SettingsTheme};
use gauntlet_common::rpc::backend_api::{BackendApi, BackendApiError};
use gauntlet_common_ui::padding;
use crate::theme::{Element, GauntletSettingsTheme};
use crate::theme::button::ButtonStyle;
use crate::theme::container::ContainerStyle;
use crate::theme::text::TextStyle;
use crate::views::general::{ManagementAppGeneralMsgIn, ManagementAppGeneralMsgOut, ManagementAppGeneralState};
use crate::views::plugins::{ManagementAppPluginMsgIn, ManagementAppPluginMsgOut, ManagementAppPluginsState};

pub fn run() {
    iced::application::<ManagementAppModel, ManagementAppMsg, GauntletSettingsTheme, Renderer>("Gauntlet Settings", update, view)
        .window(window::Settings {
            size: Size::new(1000.0, 600.0),
            ..Default::default()
        })
        .subscription(subscription)
        .theme(|_| GauntletSettingsTheme::default())
        .run_with(new)
        .expect("Unable to start settings application");
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

fn new() -> (ManagementAppModel, Task<ManagementAppMsg>) {
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
        Task::batch([
            font::load(BOOTSTRAP_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
            Task::done(ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::FetchPlugins)),
            Task::perform(
                async {
                    match backend_api {
                        Some(backend_api) => Some(init_data(backend_api).await),
                        None => None
                    }
                },
                |shortcut| {
                    match shortcut {
                        None => ManagementAppMsg::General(ManagementAppGeneralMsgIn::Noop),
                        Some(init) => {
                            match init {
                                Ok(init) => {
                                    ManagementAppMsg::General(ManagementAppGeneralMsgIn::InitSetting {
                                        theme: init.theme,
                                        shortcut: init.global_shortcut,
                                        shortcut_error: init.global_shortcut_error
                                    })
                                },
                                Err(err) => ManagementAppMsg::HandleBackendError(err)
                            }
                        }
                    }
                }
            ),
        ]),
    )
}

struct InitSettingsData {
    global_shortcut: Option<PhysicalShortcut>,
    global_shortcut_error: Option<String>,
    theme: SettingsTheme
}

async fn init_data(mut backend_api: BackendApi) -> Result<InitSettingsData, BackendApiError> {
    let (global_shortcut, global_shortcut_error) = backend_api.get_global_shortcut()
        .await?;

    let theme = backend_api.get_theme()
        .await?;

    Ok(InitSettingsData {
        global_shortcut,
        global_shortcut_error,
        theme
    })
}

fn update(state: &mut ManagementAppModel, message: ManagementAppMsg) -> Task<ManagementAppMsg> {
    let backend_api = match &state.backend_api {
        Some(backend_api) => backend_api.clone(),
        None => {
            return Task::none()
        }
    };

    match message {
        ManagementAppMsg::Plugin(message) => {
            state.plugins_state.update(message)
                .map(|msg| {
                    match msg {
                        ManagementAppPluginMsgOut::PluginsReloaded(plugins) => {
                            ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::PluginsFetched(plugins))
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
            state.general_state.update(message)
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
            Task::none()
        }
        ManagementAppMsg::SwitchView(view) => {
            state.current_settings_view = view;

            Task::none()
        }
        ManagementAppMsg::HandleBackendError(err) => {
            state.error_view = Some(match err {
                BackendApiError::Timeout => ErrorView::Timeout,
                BackendApiError::Internal { display } => ErrorView::UnknownError { display }
            });

            Task::none()
        }
        ManagementAppMsg::DownloadStatus { plugins } => {
            for (plugin, status) in plugins {
                match status {
                    DownloadStatus::InProgress => {
                        state.downloads_info.insert(plugin.clone(), DownloadInfo::InProgress);
                    }
                    DownloadStatus::Done => {
                        state.downloads_info.insert(plugin.clone(), DownloadInfo::Successful);
                    }
                    DownloadStatus::Failed { message } => {
                        state.downloads_info.insert(plugin.clone(), DownloadInfo::Error { message });
                    }
                }
            }

            let mut backend_api = backend_api.clone();

            Task::perform(
                async move {
                    let plugins = backend_api.plugins()
                        .await?;

                    Ok(plugins)
                },
                |result| handle_backend_error(result, |plugins| ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::PluginsFetched(plugins)))
            )
        }
        ManagementAppMsg::CheckDownloadStatus => {
            if state.downloads_info.is_empty() {
                Task::none()
            } else {
                let mut backend_client = backend_api.clone();

                Task::perform(
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

            let already_downloading = state.downloads_info.insert(plugin_id.clone(), DownloadInfo::InProgress)
                .is_some();

            if already_downloading {
                Task::none()
            } else {
                Task::perform(
                    async move {
                        backend_client.download_plugin(plugin_id)
                            .await?;

                        Ok(())
                    },
                    |result| handle_backend_error(result, |()| ManagementAppMsg::Noop)
                )
            }
        }
        ManagementAppMsg::Noop => Task::none(),
        ManagementAppMsg::ToggleDownloadInfo => {
            state.download_info_shown = !state.download_info_shown;
            Task::none()
        }
    }
}

fn view(state: &ManagementAppModel) -> Element<'_, ManagementAppMsg> {
    if let None = &state.backend_api {
        let description: Element<_> = text("Unable to connect to server. Please check if you have Gauntlet running on your PC")
            .into();

        let content: Element<_> = container(description)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        return content
    }

    if let Some(err) = &state.error_view {
        return match err {
            ErrorView::Timeout => {
                let description: Element<_> = text("Error occurred")
                    .into();

                let description = container(description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let sub_description: Element<_> = text("Backend was unable to process message in a timely manner")
                    .into();

                let sub_description = container(sub_description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let content: Element<_> = column([
                    description,
                    sub_description,
                ]).into();

                let content: Element<_> = container(content)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
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
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let sub_description: Element<_> = text("Please report")
                    .into();

                let sub_description = container(sub_description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let error_description: Element<_> = text(display)
                    .shaping(Shaping::Advanced)
                    .into();

                let error_description = container(error_description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let content: Element<_> = column([
                    description,
                    sub_description,
                    error_description,
                ]).into();

                let content: Element<_> = container(content)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();

                content
            }
        }
    }


    let content = match state.current_settings_view {
        SettingsView::General => {
            state.general_state.view()
                .map(|msg| ManagementAppMsg::General(msg))
        }
        SettingsView::Plugins => {
            state.plugins_state.view()
                .map(|msg| ManagementAppMsg::Plugin(msg))
        }
    };

    let icon_general: Element<_> = value(Bootstrap::GearFill)
        .font(BOOTSTRAP_FONT)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(alignment::Vertical::Center)
        .align_x(alignment::Horizontal::Center)
        .into();

    let text_general: Element<_> = text("General")
        .height(Length::Fill)
        .align_y(alignment::Vertical::Center)
        .align_x(alignment::Horizontal::Center)
        .into();

    let general_button: Element<_> = column(vec![icon_general, text_general])
        .align_x(Alignment::Center)
        .height(Length::Fill)
        .width(Length::Fill)
        .into();

    let general_button: Element<_> = button(general_button)
        .on_press(ManagementAppMsg::SwitchView(SettingsView::General))
        .height(Length::Fill)
        .width(80)
        .class(if state.current_settings_view == SettingsView::General { ButtonStyle::ViewSwitcherSelected } else { ButtonStyle::ViewSwitcher })
        .into();

    let general_button: Element<_> = container(general_button)
        .padding(8.0)
        .into();

    let icon_plugins: Element<_> = value(Bootstrap::PuzzleFill)
        .font(BOOTSTRAP_FONT)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(alignment::Vertical::Center)
        .align_x(alignment::Horizontal::Center)
        .into();

    let text_plugins: Element<_> = text("Plugins")
        .height(Length::Fill)
        .align_y(alignment::Vertical::Center)
        .align_x(alignment::Horizontal::Center)
        .into();

    let plugins_button: Element<_> = column(vec![icon_plugins, text_plugins])
        .align_x(Alignment::Center)
        .height(Length::Fill)
        .width(Length::Fill)
        .into();

    let plugins_button: Element<_> = button(plugins_button)
        .on_press(ManagementAppMsg::SwitchView(SettingsView::Plugins))
        .height(Length::Fill)
        .width(80)
        .class(if state.current_settings_view == SettingsView::Plugins { ButtonStyle::ViewSwitcherSelected } else { ButtonStyle::ViewSwitcher })
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

        for (_, download_info) in state.downloads_info.iter() {
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
                .align_y(alignment::Vertical::Center)
                .into();

            let spinner: Element<_> = row(vec![text, spinner])
                .spacing(8.0)
                .into();

            download_info_icons.push(spinner);
        }
        if successful_count > 0 {
            let icon: Element<_> = value(Bootstrap::PatchCheckFill)
                .size(16)
                .align_y(alignment::Vertical::Center)
                .font(BOOTSTRAP_FONT)
                .height(Length::Fill)
                .class(TextStyle::Positive)
                .into();

            let icon: Element<_> = container(icon)
                .height(Length::Fill)
                .into();

            let text: Element<_> = text(successful_count)
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .into();

            let icon: Element<_> = row(vec![text, icon])
                .spacing(8.0)
                .into();

            download_info_icons.push(icon);
        }
        if error_count > 0 {
            let icon: Element<_> = value(Bootstrap::ExclamationTriangleFill)
                .font(BOOTSTRAP_FONT)
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .size(16)
                .class(TextStyle::Destructive)
                .into();

            let icon: Element<_> = container(icon)
                .height(Length::Fill)
                .into();

            let text: Element<_> = text(error_count)
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
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
                .align_y(Alignment::Center)
                .into();

            let top_bar_right: Element<_> = button(top_bar_right)
                .class(ButtonStyle::DownloadInfo)
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
                .center_y(Length::Fill)
                .align_x(alignment::Horizontal::Right)
                .into();

            top_bar_right
        }
    };

    let top_bar: Element<_> = row(vec![top_bar_left_space, top_bar_buttons, top_bar_right])
        .width(Length::Fill)
        .into();

    let top_bar: Element<_> = container(top_bar)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Shrink)
        .max_height(70)
        .into();

    let separator: Element<_> = horizontal_rule(1)
        .into();

    let content: Element<_> = column(vec![top_bar, separator, content])
        .into();

    let download_info_panel: Element<_> = {
        let downloads: Vec<Element<_>> = state.downloads_info.iter()
            .sorted_by_key(|(_, info)| info.clone())
            .map(|(plugin_id, info)| {
                match info {
                    DownloadInfo::InProgress => {
                        let kind_text: Element<_> = text("Download in progress")
                            .into();

                        let kind_text: Element<_> = container(kind_text)
                            .padding(padding(16, 0, 8, 0))
                            .into();

                        let plugin_id: Element<_> = text(plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .class(TextStyle::Subtitle)
                            .size(14)
                            .into();

                        let plugin_id: Element<_> = container(plugin_id)
                            .padding(padding::bottom(16))
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
                            .padding(padding(16, 0, 8, 0))
                            .into();

                        let plugin_id: Element<_> = text(plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .class(TextStyle::Subtitle)
                            .size(14)
                            .into();

                        let icon: Element<_> = value(Bootstrap::ExclamationTriangleFill)
                            .font(BOOTSTRAP_FONT)
                            .align_y(alignment::Vertical::Center)
                            .size(32)
                            .class(TextStyle::Destructive)
                            .into();

                        let icon: Element<_> = container(icon)
                            .padding(16)
                            .into();

                        let message: Element<_> = text(message.to_string())
                            .shaping(Shaping::Advanced)
                            .into();

                        let message: Element<_> = container(message)
                            .padding(padding(8, 0, 16, 0))
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
                            .padding(padding(16, 0, 8, 0))
                            .into();

                        let plugin_id: Element<_> = text(plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .size(14)
                            .class(TextStyle::Subtitle)
                            .into();

                        let plugin_id: Element<_> = container(plugin_id)
                            .padding(padding::bottom(16))
                            .into();

                        let icon: Element<_> = value(Bootstrap::PatchCheckFill)
                            .size(32)
                            .align_y(alignment::Vertical::Center)
                            .font(BOOTSTRAP_FONT)
                            .class(TextStyle::Positive)
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

        let content: Element<_> = container(downloads)
            .padding(4)
            .width(Length::Fixed(400.0))
            .max_height(500.0)
            .class(ContainerStyle::Box)
            .into();

        container(content)
            .padding(gauntlet_common_ui::padding(8.0, 60.0, 0.0, 0.0))
            .align_right(Length::Fill)
            .align_top(Length::Fill)
            .into()
    };

    let content: Element<_> = mouse_area(content)
        .on_press(if state.download_info_shown { ManagementAppMsg::ToggleDownloadInfo } else { ManagementAppMsg::Noop })
        .into();

    let mut content = vec![content];

    if state.download_info_shown {
        content.push(download_info_panel);
    }

    stack(content)
        .into()
}

fn subscription(_state: &ManagementAppModel) -> Subscription<ManagementAppMsg> {
    time::every(Duration::from_millis(300))
        .map(|_| ManagementAppMsg::CheckDownloadStatus)
}


pub fn handle_backend_error<T>(result: Result<T, BackendApiError>, convert: impl FnOnce(T) -> ManagementAppMsg) -> ManagementAppMsg {
    match result {
        Ok(val) => convert(val),
        Err(err) => ManagementAppMsg::HandleBackendError(err)
    }
}
