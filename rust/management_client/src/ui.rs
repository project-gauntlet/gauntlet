use std::time::Duration;

use iced::{Alignment, alignment, Application, Command, executor, font, futures, Length, Settings, Size, Subscription, time, window};
use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced_aw::core::icons;
use common::model::PhysicalShortcut;
use common::rpc::backend_api::BackendApi;

use crate::theme::{Element, GauntletSettingsTheme};
use crate::theme::button::ButtonStyle;
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
    }).unwrap();
}

struct ManagementAppModel {
    backend_api: Option<BackendApi>,
    current_settings_view: SettingsView,
    general_state: ManagementAppGeneralState,
    plugins_state: ManagementAppPluginsState
}


#[derive(Debug, Clone)]
enum ManagementAppMsg {
    FontLoaded(Result<(), font::Error>),
    General(ManagementAppGeneralMsgIn),
    Plugin(ManagementAppPluginMsgIn),
    SwitchView(SettingsView)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SettingsView {
    General,
    Plugins
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
            .inspect_err(|err| tracing::error!("Unable to connect to backend: {:?}", err))
            .ok();

        (
            ManagementAppModel {
                backend_api: backend_api.clone(),
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
                                    .await
                                    .unwrap(); // TODO proper error handling

                                Some(shortcut)
                            }
                            None => None
                        }
                    },
                    |shortcut| {
                        match shortcut {
                            None => ManagementAppMsg::General(ManagementAppGeneralMsgIn::Noop),
                            Some(shortcut) => ManagementAppMsg::General(ManagementAppGeneralMsgIn::SetShortcut(shortcut))
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
                            ManagementAppPluginMsgOut::DownloadStatus { plugins } => {
                                ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::DownloadStatus { plugins })
                            }
                            ManagementAppPluginMsgOut::SelectedItem(selected_item) => {
                                ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::SelectItem(selected_item))
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

        let top_bar: Element<_> = row(vec![general_button, plugins_button])
            .into();

        let top_bar = container(top_bar)
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

        container(content)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_millis(300))
            .map(|_| ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::CheckDownloadStatus))
    }

    fn theme(&self) -> Self::Theme {
        GauntletSettingsTheme::default()
    }
}
