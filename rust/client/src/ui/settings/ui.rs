use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use gauntlet_common::model::DownloadStatus;
use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PluginId;
use gauntlet_common_ui::padding;
use gauntlet_server::global_hotkey::GlobalHotKeyManager;
use gauntlet_server::plugins::ApplicationManager;
use gauntlet_utils::channel::RequestError;
use gauntlet_utils::channel::RequestResult;
use iced::Alignment;
use iced::Length;
use iced::Padding;
use iced::Size;
use iced::Subscription;
use iced::Task;
use iced::advanced::text::Shaping;
use iced::alignment;
use iced::padding;
use iced::time;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::horizontal_space;
use iced::widget::mouse_area;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::stack;
use iced::widget::text;
use iced::window;
use iced_fonts::bootstrap::exclamation_triangle_fill;
use iced_fonts::bootstrap::gear_fill;
use iced_fonts::bootstrap::patch_check_fill;
use iced_fonts::bootstrap::puzzle_fill;
use itertools::Itertools;

use crate::ui::settings::components::spinner::Spinner;
use crate::ui::settings::theme::Element;
use crate::ui::settings::theme::button::ButtonStyle;
use crate::ui::settings::theme::container::ContainerStyle;
use crate::ui::settings::theme::text::TextStyle;
use crate::ui::settings::views::general::SettingsGeneralMsgIn;
use crate::ui::settings::views::general::SettingsGeneralMsgOut;
use crate::ui::settings::views::general::SettingsGeneralState;
use crate::ui::settings::views::plugins::SelectedItem;
use crate::ui::settings::views::plugins::SettingsPluginMsgIn;
use crate::ui::settings::views::plugins::SettingsPluginMsgOut;
use crate::ui::settings::views::plugins::SettingsPluginsState;

pub struct SettingsWindowState {
    pub settings_window_id: Option<window::Id>,
    application_manager: Arc<ApplicationManager>,
    wayland: bool,
    error_view: Option<ErrorView>,
    downloads_info: HashMap<PluginId, DownloadInfo>,
    download_info_shown: bool,
    current_settings_view: SettingsView,
    general_state: SettingsGeneralState,
    plugins_state: SettingsPluginsState,
}

impl SettingsWindowState {
    pub fn new(application_manager: Arc<ApplicationManager>, wayland: bool) -> SettingsWindowState {
        SettingsWindowState {
            settings_window_id: None,
            application_manager: application_manager.clone(),
            wayland,
            error_view: None,
            downloads_info: HashMap::new(),
            download_info_shown: false,
            current_settings_view: SettingsView::Plugins,
            general_state: SettingsGeneralState::new(application_manager.clone()),
            plugins_state: SettingsPluginsState::new(application_manager.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SettingsParams {
    Default,
    PluginPreferences {
        plugin_id: PluginId,
    },
    EntrypointPreferences {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}

#[derive(Debug, Clone)]
pub enum SettingsMsg {
    Refresh,
    OpenSettings(SettingsParams),
    WindowToBeCreated(window::Id),
    WindowCreated,
    WindowDestroyed,
    General(SettingsGeneralMsgIn),
    Plugin(SettingsPluginMsgIn),
    SwitchView(SettingsView),
    DownloadStatus { plugins: HashMap<PluginId, DownloadStatus> },
    HandleBackendError(RequestError),
    CheckDownloadStatus,
    DownloadPlugin { plugin_id: PluginId },
    Noop,
    ToggleDownloadInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsView {
    General,
    Plugins,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ErrorView {
    UnknownError { display: String },
    Timeout,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)] // ordering used in sorting items in ui
pub enum DownloadInfo {
    InProgress,
    Error { message: String },
    Successful,
}

pub fn update_settings(
    state: &mut SettingsWindowState,
    global_hotkey_manager: &Option<GlobalHotKeyManager>,
    message: SettingsMsg,
) -> Task<SettingsMsg> {
    match message {
        SettingsMsg::Plugin(message) => {
            state.plugins_state.update(global_hotkey_manager, message).map(|msg| {
                match msg {
                    SettingsPluginMsgOut::Inner(msg) => SettingsMsg::Plugin(msg),
                    SettingsPluginMsgOut::Outer(msg) => msg,
                }
            })
        }
        SettingsMsg::General(message) => {
            state.general_state.update(global_hotkey_manager, message).map(|msg| {
                match msg {
                    SettingsGeneralMsgOut::Inner(msg) => SettingsMsg::General(msg),
                    SettingsGeneralMsgOut::Outer(msg) => msg,
                }
            })
        }
        SettingsMsg::SwitchView(view) => {
            state.current_settings_view = view;

            Task::none()
        }
        SettingsMsg::HandleBackendError(err) => {
            state.error_view = Some(match err {
                RequestError::Timeout => ErrorView::Timeout,
                RequestError::Other { display } => ErrorView::UnknownError { display },
                RequestError::OtherSideWasDropped => {
                    ErrorView::UnknownError {
                        display: "The other side was dropped".to_string(),
                    }
                }
            });

            Task::none()
        }
        SettingsMsg::DownloadStatus { plugins } => {
            for (plugin, status) in plugins {
                match status {
                    DownloadStatus::InProgress => {
                        state.downloads_info.insert(plugin.clone(), DownloadInfo::InProgress);
                    }
                    DownloadStatus::Done => {
                        state.downloads_info.insert(plugin.clone(), DownloadInfo::Successful);
                    }
                    DownloadStatus::Failed { message } => {
                        state
                            .downloads_info
                            .insert(plugin.clone(), DownloadInfo::Error { message });
                    }
                }
            }

            let application_manager = state.application_manager.clone();

            Task::perform(
                async move {
                    let plugins = application_manager.plugins()?;
                    let global_entrypoint_shortcuts = application_manager.get_global_entrypoint_shortcuts()?;
                    let entrypoint_search_aliases = application_manager.get_entrypoint_search_aliases()?;

                    Ok((plugins, global_entrypoint_shortcuts, entrypoint_search_aliases))
                },
                |result| {
                    handle_backend_error(
                        result,
                        |(plugins, global_entrypoint_shortcuts, entrypoint_search_aliases)| {
                            SettingsMsg::Plugin(SettingsPluginMsgIn::PluginsReloaded(
                                plugins,
                                global_entrypoint_shortcuts,
                                entrypoint_search_aliases,
                            ))
                        },
                    )
                },
            )
        }
        SettingsMsg::CheckDownloadStatus => {
            if state.downloads_info.is_empty() {
                Task::none()
            } else {
                let plugins = state.application_manager.download_status();

                Task::done(SettingsMsg::DownloadStatus { plugins })
            }
        }
        SettingsMsg::DownloadPlugin { plugin_id } => {
            let backend_client = state.application_manager.clone();

            let already_downloading = state
                .downloads_info
                .insert(plugin_id.clone(), DownloadInfo::InProgress)
                .is_some();

            if already_downloading {
                Task::none()
            } else {
                backend_client.download_plugin(plugin_id);

                Task::none()
            }
        }
        SettingsMsg::Noop => Task::none(),
        SettingsMsg::ToggleDownloadInfo => {
            state.download_info_shown = !state.download_info_shown;
            Task::none()
        }
        SettingsMsg::Refresh => {
            fn run(state: &mut SettingsWindowState) -> anyhow::Result<Task<SettingsMsg>> {
                let (global_shortcut, global_shortcut_error) =
                    state.application_manager.get_global_shortcut().map(|data| {
                        data.map(|(shortcut, error)| (Some(shortcut), error))
                            .unwrap_or((None, None))
                    })?;

                let global_entrypoint_shortcuts = state.application_manager.get_global_entrypoint_shortcuts()?;

                let theme = state.application_manager.get_theme()?;

                let window_position_mode = state.application_manager.get_window_position_mode()?;
                let wayland_global_shortcuts_enabled = state.application_manager.config()?.wayland_use_legacy_x11_api;

                Ok(Task::batch([
                    Task::done(SettingsMsg::General(SettingsGeneralMsgIn::InitSetting {
                        theme,
                        window_position_mode,
                        shortcut: global_shortcut,
                        shortcut_error: global_shortcut_error,
                        global_shortcuts_unsupported: state.wayland && !wayland_global_shortcuts_enabled,
                    })),
                    Task::done(SettingsMsg::Plugin(SettingsPluginMsgIn::InitSetting {
                        global_entrypoint_shortcuts,
                        show_global_shortcuts: !state.wayland || wayland_global_shortcuts_enabled,
                    })),
                ]))
            }

            run(state).unwrap_or_else(|err| Task::done(SettingsMsg::HandleBackendError(err.into())))
        }
        SettingsMsg::WindowToBeCreated(window_id) => {
            state.settings_window_id = Some(window_id);

            Task::none()
        }
        SettingsMsg::WindowCreated => {
            if let Some(window_id) = state.settings_window_id {
                window::gain_focus(window_id)
            } else {
                Task::none()
            }
        }
        SettingsMsg::WindowDestroyed => {
            state.settings_window_id = None;

            Task::none()
        }
        SettingsMsg::OpenSettings(settings_params) => {
            let item = match settings_params {
                SettingsParams::Default => SelectedItem::None,
                SettingsParams::PluginPreferences { plugin_id } => SelectedItem::Plugin { plugin_id },
                SettingsParams::EntrypointPreferences {
                    plugin_id,
                    entrypoint_id,
                } => {
                    SelectedItem::Entrypoint {
                        plugin_id,
                        entrypoint_id,
                    }
                }
            };

            let open = match state.settings_window_id {
                None => {
                    let settings = window::Settings {
                        size: Size::new(1150.0, 700.0),
                        ..Default::default()
                    };
                    let (_, open) = window::open(settings);
                    open.map(SettingsMsg::WindowToBeCreated)
                }
                Some(_) => Task::none(),
            };

            Task::batch([
                open,
                Task::done(SettingsMsg::Plugin(SettingsPluginMsgIn::FetchPlugins)),
                Task::done(SettingsMsg::Refresh),
                Task::done(SettingsMsg::SwitchView(SettingsView::Plugins)),
                Task::done(SettingsMsg::Plugin(SettingsPluginMsgIn::SelectItem(item))),
            ])
        }
    }
}

pub fn view_settings(state: &SettingsWindowState) -> Element<'_, SettingsMsg> {
    if let Some(err) = &state.error_view {
        return match err {
            ErrorView::Timeout => {
                let description: Element<_> = text("Error occurred").into();

                let description = container(description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let sub_description: Element<_> =
                    text("Backend was unable to process message in a timely manner").into();

                let sub_description = container(sub_description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let content: Element<_> = column([description, sub_description]).into();

                let content: Element<_> = container(content)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();

                content
            }
            ErrorView::UnknownError { display } => {
                let description: Element<_> = text("Unknown error occurred").into();

                let description = container(description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let sub_description: Element<_> = text("Please report").into();

                let sub_description = container(sub_description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let error_description: Element<_> = text(display).shaping(Shaping::Advanced).into();

                let error_description = container(error_description)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(12)
                    .into();

                let content: Element<_> = column([description, sub_description, error_description]).into();

                let content: Element<_> = container(content)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();

                content
            }
        };
    }

    let content = match state.current_settings_view {
        SettingsView::General => state.general_state.view().map(|msg| SettingsMsg::General(msg)),
        SettingsView::Plugins => state.plugins_state.view().map(|msg| SettingsMsg::Plugin(msg)),
    };

    let icon_general: Element<_> = gear_fill()
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
        .on_press(SettingsMsg::SwitchView(SettingsView::General))
        .height(Length::Fill)
        .width(80)
        .class(
            if state.current_settings_view == SettingsView::General {
                ButtonStyle::ViewSwitcherSelected
            } else {
                ButtonStyle::ViewSwitcher
            },
        )
        .into();

    let general_button: Element<_> = container(general_button).padding(8.0).into();

    let icon_plugins: Element<_> = puzzle_fill()
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
        .on_press(SettingsMsg::SwitchView(SettingsView::Plugins))
        .height(Length::Fill)
        .width(80)
        .class(
            if state.current_settings_view == SettingsView::Plugins {
                ButtonStyle::ViewSwitcherSelected
            } else {
                ButtonStyle::ViewSwitcher
            },
        )
        .into();

    let plugins_button: Element<_> = container(plugins_button).padding(8.0).into();

    let top_bar_buttons: Element<_> = row(vec![general_button, plugins_button]).into();

    let top_bar_buttons: Element<_> = container(top_bar_buttons)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into();

    let top_bar_left_space: Element<_> = horizontal_space().width(Length::Fill).into();

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
            let spinner: Element<_> = Spinner::new().width(Length::Fixed(16.0)).height(Length::Fill).into();

            let spinner: Element<_> = container(spinner).height(Length::Fill).into();

            let text: Element<_> = text(in_progress_count)
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .into();

            let spinner: Element<_> = row(vec![text, spinner]).spacing(8.0).into();

            download_info_icons.push(spinner);
        }
        if successful_count > 0 {
            let icon: Element<_> = patch_check_fill()
                .size(16)
                .align_y(alignment::Vertical::Center)
                .height(Length::Fill)
                .class(TextStyle::Positive)
                .into();

            let icon: Element<_> = container(icon).height(Length::Fill).into();

            let text: Element<_> = text(successful_count)
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .into();

            let icon: Element<_> = row(vec![text, icon]).spacing(8.0).into();

            download_info_icons.push(icon);
        }
        if error_count > 0 {
            let icon: Element<_> = exclamation_triangle_fill()
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .size(16)
                .class(TextStyle::Destructive)
                .into();

            let icon: Element<_> = container(icon).height(Length::Fill).into();

            let text: Element<_> = text(error_count)
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .into();

            let icon: Element<_> = row(vec![text, icon]).spacing(8.0).into();

            download_info_icons.push(icon);
        }

        if download_info_icons.is_empty() {
            horizontal_space().width(Length::Fill).into()
        } else {
            let top_bar_right: Element<_> = row(download_info_icons)
                .spacing(12.0)
                .height(Length::Fill)
                .align_y(Alignment::Center)
                .into();

            let top_bar_right: Element<_> = button(top_bar_right)
                .class(ButtonStyle::DownloadInfo)
                .on_press(SettingsMsg::ToggleDownloadInfo)
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

    let separator: Element<_> = horizontal_rule(1).into();

    let content: Element<_> = column(vec![top_bar, separator, content]).into();

    let download_info_panel: Element<_> = {
        let downloads = state
            .downloads_info
            .iter()
            .sorted_by_key(|&(_, &ref info)| info)
            .map(|(plugin_id, info)| {
                match info {
                    DownloadInfo::InProgress => {
                        let kind_text: Element<_> = text("Download in progress").into();

                        let kind_text: Element<_> = container(kind_text).padding(padding(16, 0, 8, 0)).into();

                        let plugin_id: Element<_> = text(plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .class(TextStyle::Subtitle)
                            .size(14)
                            .into();

                        let plugin_id: Element<_> = container(plugin_id).padding(padding::bottom(16)).into();

                        let spinner: Element<_> = Spinner::new().width(Length::Fixed(32.0)).into();

                        let spinner: Element<_> = container(spinner).padding(16).into();

                        let content: Element<_> = column(vec![kind_text, plugin_id]).into();

                        let content: Element<_> = row(vec![spinner, content]).into();

                        container(content).width(Length::Fill).into()
                    }
                    DownloadInfo::Error { message } => {
                        let kind_text: Element<_> = text("Download failed").into();

                        let kind_text: Element<_> = container(kind_text).padding(padding(16, 0, 8, 0)).into();

                        let plugin_id: Element<_> = text(plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .class(TextStyle::Subtitle)
                            .size(14)
                            .into();

                        let icon: Element<_> = exclamation_triangle_fill()
                            .align_y(alignment::Vertical::Center)
                            .size(32)
                            .class(TextStyle::Destructive)
                            .into();

                        let icon: Element<_> = container(icon).padding(16).into();

                        let message: Element<_> = text(message.to_string()).shaping(Shaping::Advanced).into();

                        let message: Element<_> = container(message).padding(padding(8, 0, 16, 0)).into();

                        let content: Element<_> = column(vec![kind_text, plugin_id, message]).into();

                        let content: Element<_> = row(vec![icon, content]).into();

                        container(content).width(Length::Fill).into()
                    }
                    DownloadInfo::Successful => {
                        let kind_text: Element<_> = text("Download successful").into();

                        let kind_text: Element<_> = container(kind_text).padding(padding(16, 0, 8, 0)).into();

                        let plugin_id: Element<_> = text(plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .size(14)
                            .class(TextStyle::Subtitle)
                            .into();

                        let plugin_id: Element<_> = container(plugin_id).padding(padding::bottom(16)).into();

                        let icon: Element<_> = patch_check_fill()
                            .size(32)
                            .align_y(alignment::Vertical::Center)
                            .class(TextStyle::Positive)
                            .into();

                        let icon: Element<_> = container(icon).padding(16).into();

                        let content: Element<_> = column(vec![kind_text, plugin_id]).into();

                        let content: Element<_> = row(vec![icon, content]).into();

                        container(content).width(Length::Fill).into()
                    }
                }
            });

        let downloads = Itertools::intersperse_with(downloads, || horizontal_rule(1).into());

        let downloads: Vec<Element<_>> = downloads.collect();

        let downloads: Element<_> = column(downloads).into();

        let downloads: Element<_> = scrollable(downloads).width(Length::Fill).into();

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

    let content = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .class(ContainerStyle::WindowRoot);

    let content: Element<_> = mouse_area(content)
        .on_press(
            if state.download_info_shown {
                SettingsMsg::ToggleDownloadInfo
            } else {
                SettingsMsg::Noop
            },
        )
        .into();

    let mut content = vec![content];

    if state.download_info_shown {
        content.push(download_info_panel);
    }

    stack(content).into()
}

pub fn subscription_settings(state: &SettingsWindowState) -> Subscription<SettingsMsg> {
    match state.settings_window_id {
        None => Subscription::none(),
        Some(_) => time::every(Duration::from_millis(300)).map(|_| SettingsMsg::CheckDownloadStatus),
    }
}

pub fn handle_backend_error<T>(result: RequestResult<T>, convert: impl FnOnce(T) -> SettingsMsg) -> SettingsMsg {
    match result {
        Ok(val) => convert(val),
        Err(err) => SettingsMsg::HandleBackendError(err),
    }
}
