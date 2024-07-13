use std::time::Duration;

use iced::{Alignment, alignment, Application, Command, executor, font, futures, Length, Settings, Size, Subscription, time, window};
use iced::widget::{button, column, container, horizontal_rule, row, text};
use iced_aw::core::icons;

use common::rpc::backend_api::BackendApi;

use crate::theme::{Element, GauntletSettingsTheme};
use crate::views::general::{ManagementAppGeneralMsg, ManagementAppGeneralState};
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
    general_state: ManagementAppGeneralState,
    plugins_state: ManagementAppPluginsState
}


#[derive(Debug, Clone)]
enum ManagementAppMsg {
    FontLoaded(Result<(), font::Error>),
    General(ManagementAppGeneralMsg),
    Plugin(ManagementAppPluginMsgIn),
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
                general_state: ManagementAppGeneralState::new(backend_api.clone()),
                plugins_state: ManagementAppPluginsState::new(backend_api.clone()),
            },
            Command::batch([
                font::load(icons::BOOTSTRAP_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
                Command::perform(
                    async {},
                    |plugins| ManagementAppMsg::Plugin(ManagementAppPluginMsgIn::RequestPluginReload)
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
            ManagementAppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
            ManagementAppMsg::General(message) => {
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

        let content = self.plugins_state.view()
            .map(|msg| ManagementAppMsg::Plugin(msg));

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
            .height(Length::Fill)
            .width(80)
            .into();

        let general_button: Element<_> = container(general_button)
            .padding(8.0)
            .into();

        let icon_plugins: Element<_> = text(icons::Bootstrap::Cpu)
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
            .height(Length::Fill)
            .width(80)
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
