use std::collections::{HashMap, HashSet};

use futures::stream::StreamExt;
use iced::{Alignment, Application, Command, Element, executor, font, futures, Length, Padding, Renderer, Settings, Subscription, subscription, theme, Theme, window};
use iced::theme::Palette;
use iced::widget::{button, checkbox, column, container, horizontal_space, row, scrollable, text, text_input, vertical_rule};
use iced_aw::graphics::icons;
use iced_table::table;
use zbus::Connection;

use common::dbus::DBusEntrypointType;
use common::model::{EntrypointId, PluginId};

use crate::dbus::{DbusManagementServerProxyProxy, RemotePluginDownloadFinishedSignalStream};

pub fn run() {
    ManagementAppModel::run(Settings {
        id: None,
        window: window::Settings {
            size: (900, 600),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
}


struct ManagementAppModel {
    dbus_connection: Connection,
    dbus_server: DbusManagementServerProxyProxy<'static>,
    columns: Vec<Column>,
    plugins: HashMap<PluginId, Plugin>,
    selected_item: SelectedItem,
    header: scrollable::Id,
    body: scrollable::Id,
    running_downloads: HashSet<PluginId>,
}

#[derive(Debug, Clone)]
enum ManagementAppMsg {
    TableSyncHeader(scrollable::AbsoluteOffset),
    FontLoaded(Result<(), font::Error>),
    PluginsReloaded(HashMap<PluginId, Plugin>),
    ToggleShowEntrypoints {
        plugin_id: PluginId,
    },
    SelectItem(SelectedItem),
    EnabledToggleItem(EnabledItem),
    AddPlugin {
        plugin_id: PluginId,
    },
    RemotePluginDownloadFinished {
        plugin_id: PluginId,
    },
    Noop
}

#[derive(Debug, Clone)]
enum SelectedItem {
    None,
    NewPlugin {
        repository_url: String
    },
    Plugin {
        plugin_id: PluginId
    },
    Entrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}

#[derive(Debug, Clone)]
enum EnabledItem {
    Plugin {
        enabled: bool,
        plugin_id: PluginId,
    },
    Entrypoint {
        enabled: bool,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}


#[derive(Debug, Clone)]
struct Plugin {
    plugin_id: PluginId,
    plugin_name: String,
    show_entrypoints: bool,
    enabled: bool,
    entrypoints: HashMap<EntrypointId, Entrypoint>,
}

#[derive(Debug, Clone)]
struct Entrypoint {
    entrypoint_id: EntrypointId,
    entrypoint_name: String,
    entrypoint_type: EntrypointType,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub enum EntrypointType {
    Command,
    View,
}

impl Application for ManagementAppModel {
    type Executor = executor::Default;
    type Message = ManagementAppMsg;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (dbus_connection, dbus_server) = futures::executor::block_on(async {
            let dbus_connection = zbus::ConnectionBuilder::session()?
                .build()
                .await?;

            let dbus_server = DbusManagementServerProxyProxy::new(&dbus_connection).await?;

            Ok::<(Connection, DbusManagementServerProxyProxy<'_>), anyhow::Error>((dbus_connection, dbus_server))
        }).unwrap();

        let dbus_server_clone = dbus_server.clone();
        (
            ManagementAppModel {
                dbus_connection,
                dbus_server,
                columns: vec![
                    Column::new(ColumnKind::ShowEntrypointsToggle),
                    Column::new(ColumnKind::Name),
                    Column::new(ColumnKind::Type),
                    Column::new(ColumnKind::EnableToggle),
                ],
                plugins: HashMap::new(),
                selected_item: SelectedItem::None,
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
                running_downloads: HashSet::new(),
            },
            Command::batch([
                font::load(icons::ICON_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
                Command::perform(async move {
                    reload_plugins(dbus_server_clone).await
                }, ManagementAppMsg::PluginsReloaded)
            ]),
        )
    }

    fn title(&self) -> String {
        "Gauntlet Settings".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            ManagementAppMsg::TableSyncHeader(offset) => {
                scrollable::scroll_to(self.header.clone(), offset)
            }
            ManagementAppMsg::FontLoaded(result) => {
                result.expect("unable to load font");
                Command::none()
            }
            ManagementAppMsg::ToggleShowEntrypoints { plugin_id } => {
                let plugin = self.plugins.get_mut(&plugin_id).unwrap();
                plugin.show_entrypoints = !plugin.show_entrypoints;
                Command::none()
            }
            ManagementAppMsg::PluginsReloaded(plugins) => {
                self.plugins = plugins;
                Command::none()
            }
            ManagementAppMsg::SelectItem(selected_item) => {
                self.selected_item = selected_item;
                Command::none()
            }
            ManagementAppMsg::EnabledToggleItem(item) => {
                match item {
                    EnabledItem::Plugin { enabled, plugin_id } => {
                        let dbus_server = self.dbus_server.clone();

                        Command::perform(async move {
                            dbus_server.set_plugin_state(&plugin_id.to_string(), enabled).await.unwrap();

                            reload_plugins(dbus_server).await
                        }, ManagementAppMsg::PluginsReloaded)
                    }
                    EnabledItem::Entrypoint { enabled, plugin_id, entrypoint_id } => {
                        let dbus_server = self.dbus_server.clone();

                        Command::perform(async move {
                            dbus_server.set_entrypoint_state(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled).await.unwrap();

                            reload_plugins(dbus_server).await
                        }, ManagementAppMsg::PluginsReloaded)
                    }
                }
            }
            ManagementAppMsg::AddPlugin { plugin_id } => {
                let dbus_server = self.dbus_server.clone();

                let exists = self.running_downloads.insert(plugin_id.clone());
                if !exists {
                    panic!("already downloading this plugins")
                }

                Command::perform(async move {
                    dbus_server.download_and_add_plugin(&plugin_id.to_string()).await.unwrap()
                }, |_| ManagementAppMsg::Noop)
            }
            ManagementAppMsg::RemotePluginDownloadFinished { plugin_id } => {
                self.running_downloads.remove(&plugin_id);
                let dbus_server = self.dbus_server.clone();
                Command::perform(async move {
                    reload_plugins(dbus_server).await
                }, ManagementAppMsg::PluginsReloaded)
            }
            ManagementAppMsg::Noop => {
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let mut plugins: Vec<_> = self.plugins
            .iter()
            .map(|(_, plugin)| plugin)
            .collect();

        plugins.sort_by_key(|plugin| &plugin.plugin_name);

        let rows: Vec<_> = plugins
            .iter()
            .flat_map(|plugin| {
                let mut result = vec![];

                result.push(Row::Plugin {
                    plugin
                });

                if plugin.show_entrypoints {
                    let mut entrypoints: Vec<_> = plugin.entrypoints
                        .iter()
                        .map(|(_, entrypoint)| entrypoint)
                        .collect();

                    entrypoints.sort_by_key(|entrypoint| &entrypoint.entrypoint_name);

                    let mut entrypoints: Vec<_> = entrypoints
                        .iter()
                        .map(|entrypoint| {
                            Row::Entrypoint {
                                plugin,
                                entrypoint,
                            }
                        })
                        .collect();

                    result.append(&mut entrypoints);
                }

                result
            })
            .collect();

        let table: Element<_> = table(self.header.clone(), self.body.clone(), &self.columns, &rows, ManagementAppMsg::TableSyncHeader)
            .into();

        let table: Element<_> = container(table)
            .padding(Padding::new(5.0))
            .into();

        let sidebar_content: Element<_> = match &self.selected_item {
            SelectedItem::None => {
                let text1: Element<_> = text("Select item from the list on the left").into();
                let text2: Element<_> = text("or").into();
                let text3: Element<_> = text("Click '+' to add new plugin").into();

                let text_column = column(vec![text1, text2, text3]).align_items(Alignment::Center);

                container(text_column)
                    .center_y()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
            SelectedItem::Plugin { plugin_id } => {
                let plugin = self.plugins.get(plugin_id).unwrap();

                let name = container(text(&plugin.plugin_name))
                    .padding(Padding::new(10.0))
                    .into();

                column(vec![
                    name,
                ]).into()
            }
            SelectedItem::Entrypoint { plugin_id, entrypoint_id } => {
                let plugin = self.plugins.get(plugin_id).unwrap();
                let entrypoint = plugin.entrypoints.get(entrypoint_id).unwrap();

                let name = container(text(&entrypoint.entrypoint_name))
                    .padding(Padding::new(10.0))
                    .into();

                column(vec![
                    name,
                ]).into()
            }
            SelectedItem::NewPlugin { repository_url } => {
                let url_input: Element<_> = text_input("Enter Git Repository URL", &repository_url)
                    .on_input(|value| ManagementAppMsg::SelectItem(SelectedItem::NewPlugin { repository_url: value }))
                    .on_submit(ManagementAppMsg::AddPlugin { plugin_id: PluginId::from_string(repository_url) })
                    .into();

                let content: Element<_> = column(vec![
                    url_input,
                    text("Supported protocols:").into(),
                    text("file, http(s), ssh").into(),
                ]).into();

                container(content)
                    .padding(Padding::new(10.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .into()
            }
        };


        let plugin_url = if let SelectedItem::NewPlugin { repository_url } = &self.selected_item {
            if !repository_url.is_empty() {
                Some(repository_url)
            } else {
                None
            }
        } else {
            None
        };

        let top_button_text = if plugin_url.is_some() {
            text("Download plugin")
        } else {
            text(icons::Icon::Plus)
                .font(icons::ICON_FONT)
        };

        let top_button_text_container: Element<_> = container(top_button_text)
            .width(Length::Fill)
            .center_y()
            .center_x()
            .into();

        let top_button_action = match plugin_url {
            Some(plugin_url) => ManagementAppMsg::AddPlugin { plugin_id: PluginId::from_string(plugin_url) },
            None => ManagementAppMsg::SelectItem(SelectedItem::NewPlugin { repository_url: Default::default() })
        };

        let top_button = button(top_button_text_container)
            .width(Length::Fill)
            .on_press(top_button_action)
            .into();

        let progress_bar_text: Element<_> = if self.running_downloads.is_empty() {
            horizontal_space(Length::Fill)
                .into()
        } else {
            let multiple = if self.running_downloads.len() > 1 { "s" } else { "" };
             text(format!("{} plugin{} downloading...", self.running_downloads.len(), multiple))
                .into()
        };

        let sidebar: Element<_> = column(vec![top_button, sidebar_content, progress_bar_text])
            .padding(Padding::new(4.0))
            .into();

        let separator: Element<_> = vertical_rule(1)
            .into();

        let content: Element<_> = row(vec![table, separator, sidebar])
            .into();

        container(content)
            .padding(Padding::new(3.0))
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {

        let (dbus_download_finished_stream, ) = futures::executor::block_on(async {
            let download_finished_stream = self.dbus_server.receive_remote_plugin_download_finished_signal().await?;

            Ok::<(RemotePluginDownloadFinishedSignalStream<'_>, ), anyhow::Error>((download_finished_stream, ))
        }).unwrap();

        let dbus_download_finished_stream = dbus_download_finished_stream
            .map(|signal| {
                let signal = signal.args().unwrap();
                ManagementAppMsg::RemotePluginDownloadFinished {
                    plugin_id: PluginId::from_string(signal.plugin_id),
                }
            });

        struct DownloadFinishedStream;

        Subscription::batch([
            subscription::run_with_id(std::any::TypeId::of::<DownloadFinishedStream>(), dbus_download_finished_stream)
        ])
    }

    fn theme(&self) -> Self::Theme {
        Theme::custom(Palette {
            background: iced::color!(0x2C323A),
            text: iced::color!(0xCAC2B6),
            primary: iced::color!(0xC79F60),
            success: iced::color!(0x659B5E),
            danger: iced::color!(0x6C1B1B),
        })
    }
}


enum Row<'a> {
    Plugin {
        plugin: &'a Plugin
    },
    Entrypoint {
        plugin: &'a Plugin,
        entrypoint: &'a Entrypoint,
    },
}

enum ColumnKind {
    ShowEntrypointsToggle,
    Name,
    Type,
    EnableToggle,
}

struct Column {
    kind: ColumnKind,
}

impl Column {
    fn new(kind: ColumnKind) -> Self {
        Self {
            kind
        }
    }
}

impl<'a, 'b> table::Column<'a, 'b, ManagementAppMsg, Renderer> for Column {
    type Row = Row<'b>;

    fn header(&'b self, _col_index: usize) -> Element<'a, ManagementAppMsg> {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => {
                horizontal_space(Length::Fill)
                    .into()
            }
            ColumnKind::Name => {
                container(text("Name"))
                    .center_y()
                    .into()
            }
            ColumnKind::Type => {
                container(text("Type"))
                    .center_y()
                    .into()
            }
            ColumnKind::EnableToggle => {
                container(text("Enabled"))
                    .center_y()
                    .into()
            }
        }
    }

    fn cell(
        &'b self,
        _col_index: usize,
        _row_index: usize,
        row_entry: &'b Self::Row,
    ) -> Element<'a, ManagementAppMsg> {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => {
                match row_entry {
                    Row::Plugin { plugin } => {
                        let icon = if plugin.show_entrypoints { icons::Icon::CaretDown } else { icons::Icon::CaretRight };

                        let icon: Element<_> = text(icon)
                            .font(icons::ICON_FONT)
                            .into();

                        button(icon)
                            .style(theme::Button::Text)
                            .on_press(ManagementAppMsg::ToggleShowEntrypoints { plugin_id: plugin.plugin_id.clone() })
                            .into()
                    }
                    Row::Entrypoint { .. } => {
                        horizontal_space(Length::Fill)
                            .into()
                    }
                }
            }
            ColumnKind::Name => {
                let content: Element<_> = match row_entry {
                    Row::Plugin { plugin } => {
                        container(text(&plugin.plugin_name))
                            .center_y()
                            .into()
                    }
                    Row::Entrypoint { entrypoint, .. } => {
                        let text: Element<_> = text(&entrypoint.entrypoint_name)
                            .into();

                        let text: Element<_> = row(vec![
                            horizontal_space(Length::Fixed(30.0)).into(),
                            text,
                        ]).into();

                        container(text)
                            .center_y()
                            .into()
                    }
                };

                let msg = match &row_entry {
                    Row::Plugin { plugin } => SelectedItem::Plugin {
                        plugin_id: plugin.plugin_id.clone()
                    },
                    Row::Entrypoint { entrypoint, plugin } => SelectedItem::Entrypoint {
                        plugin_id: plugin.plugin_id.clone(),
                        entrypoint_id: entrypoint.entrypoint_id.clone(),
                    }
                };

                button(content)
                    .style(theme::Button::Text)
                    .on_press(ManagementAppMsg::SelectItem(msg))
                    .width(Length::Fill)
                    .into()
            }
            ColumnKind::Type => {
                let content: Element<_> = match row_entry {
                    Row::Plugin { .. } => {
                        horizontal_space(Length::Fill)
                            .into()
                    }
                    Row::Entrypoint { entrypoint, .. } => {
                        let entrypoint_type = match entrypoint.entrypoint_type {
                            EntrypointType::Command => "Command",
                            EntrypointType::View => "View"
                        };

                        container(text(entrypoint_type))
                            .center_y()
                            .into()
                    }
                };

                let msg = match &row_entry {
                    Row::Plugin { plugin } => SelectedItem::Plugin {
                        plugin_id: plugin.plugin_id.clone()
                    },
                    Row::Entrypoint { entrypoint, plugin } => SelectedItem::Entrypoint {
                        plugin_id: plugin.plugin_id.clone(),
                        entrypoint_id: entrypoint.entrypoint_id.clone(),
                    }
                };

                button(content)
                    .style(theme::Button::Text)
                    .on_press(ManagementAppMsg::SelectItem(msg))
                    .width(Length::Fill)
                    .into()
            }
            ColumnKind::EnableToggle => {
                let (enabled, plugin_id, entrypoint_id) = match &row_entry {
                    Row::Plugin { plugin } => {
                        (
                            plugin.enabled,
                            plugin.plugin_id.clone(),
                            None
                        )
                    }
                    Row::Entrypoint { entrypoint, plugin } => {
                        (
                            entrypoint.enabled,
                            plugin.plugin_id.clone(),
                            Some(entrypoint.entrypoint_id.clone())
                        )
                    }
                };


                // TODO disable if plugin is disabled but preserve current state https://github.com/iced-rs/iced/pull/2109
                let checkbox: Element<_> = checkbox("", enabled, move |enabled| {
                    let enabled_item = match &entrypoint_id {
                        None => EnabledItem::Plugin {
                            enabled,
                            plugin_id: plugin_id.clone(),
                        },
                        Some(entrypoint_id) => EnabledItem::Entrypoint {
                            enabled,
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                        }
                    };
                    ManagementAppMsg::EnabledToggleItem(enabled_item)
                }).into();

                container(checkbox)
                    .width(Length::Fill)
                    .center_x()
                    .into()
            }
        }
    }

    fn width(&self) -> f32 {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => 35.0,
            ColumnKind::Name => 400.0,
            ColumnKind::Type => 100.0,
            ColumnKind::EnableToggle => 75.0
        }
    }

    fn resize_offset(&self) -> Option<f32> {
        None
    }
}


async fn reload_plugins(dbus_server: DbusManagementServerProxyProxy<'static>) -> HashMap<PluginId, Plugin> {
    let plugins = dbus_server.plugins().await.unwrap();

    plugins.into_iter()
        .map(|plugin| {
            let entrypoints: HashMap<_, _> = plugin.entrypoints
                .into_iter()
                .map(|entrypoint| {
                    let id = EntrypointId::new(entrypoint.entrypoint_id);
                    let entrypoint = Entrypoint {
                        enabled: entrypoint.enabled,
                        entrypoint_id: id.clone(),
                        entrypoint_name: entrypoint.entrypoint_name.clone(),
                        entrypoint_type: match entrypoint.entrypoint_type {
                            DBusEntrypointType::Command => EntrypointType::Command,
                            DBusEntrypointType::View => EntrypointType::View
                        }
                    };
                    (id, entrypoint)
                })
                .collect();

            let id = PluginId::from_string(plugin.plugin_id);
            let plugin = Plugin {
                plugin_id: id.clone(),
                plugin_name: plugin.plugin_name,
                show_entrypoints: true,
                enabled: plugin.enabled,
                entrypoints,
            };

            (id, plugin)
        })
        .collect()
}