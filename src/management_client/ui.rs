use std::collections::hash_map::Entry;
use std::collections::HashMap;

use deno_core::error::AnyError;
use futures::stream::StreamExt;
use iced::{Application, Command, Element, executor, font, futures, Length, Padding, Renderer, Settings, Subscription, subscription, theme, window};
use iced::widget::{button, checkbox, column, container, horizontal_space, progress_bar, row, scrollable, text, text_input};
use iced_aw::graphics::icons;
use iced_table::table;
use zbus::Connection;

use crate::common::model::{EntrypointId, PluginId};
use crate::management_client::dbus::{DbusManagementServerProxyProxy, PluginDownloadFinishedSignalStream, PluginDownloadStatusSignalStream};

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
    running_downloads: HashMap<String, RunningDownload>,
}

struct RunningDownload {
    percent: f32,
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
    NewDownload {
        repository_url: String,
    },
    RunningDownloadStatus {
        download_id: String,
        percent: f32,
    },
    RunningDownloadFinished {
        download_id: String,
    },
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
    enabled: bool,
}

impl Application for ManagementAppModel {
    type Executor = executor::Default;
    type Message = ManagementAppMsg;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (dbus_connection, dbus_server) = futures::executor::block_on(async {
            let dbus_connection = zbus::ConnectionBuilder::session()?
                .name("org.placeholdername.PlaceHolderName.ManagementClient")?
                .build()
                .await?;

            let dbus_server = DbusManagementServerProxyProxy::new(&dbus_connection).await?;

            Ok::<(Connection, DbusManagementServerProxyProxy<'_>), AnyError>((dbus_connection, dbus_server))
        }).unwrap();

        let dbus_server_clone = dbus_server.clone();
        (
            ManagementAppModel {
                dbus_connection,
                dbus_server,
                columns: vec![
                    Column::new(ColumnKind::ShowEntrypointsToggle),
                    Column::new(ColumnKind::Name),
                    Column::new(ColumnKind::EnableToggle),
                ],
                plugins: HashMap::new(),
                selected_item: SelectedItem::None,
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
                running_downloads: HashMap::new(),
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
        "PlaceHolderName Settings".to_owned()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            ManagementAppMsg::TableSyncHeader(offset) => {
                scrollable::scroll_to(self.header.clone(), offset)
            }
            ManagementAppMsg::FontLoaded(result) => {
                result.unwrap();
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
            ManagementAppMsg::NewDownload { repository_url } => {
                let dbus_server = self.dbus_server.clone();

                Command::perform(async move {
                    dbus_server.start_plugin_download(&repository_url).await.unwrap()
                }, |download_id| ManagementAppMsg::RunningDownloadStatus { download_id, percent: 0.0 })
            }
            ManagementAppMsg::RunningDownloadStatus { download_id, percent } => {
                match self.running_downloads.entry(download_id) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().percent = percent;
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(RunningDownload { percent });
                    }
                }
                Command::none()
            }
            ManagementAppMsg::RunningDownloadFinished { download_id } => {
                self.running_downloads.remove(&download_id).unwrap();
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

        let sidebar_content: Element<_> = match &self.selected_item {
            SelectedItem::None => {
                container(text("Select item from the list"))
                    .center_y()
                    .center_x()
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
                    .on_submit(ManagementAppMsg::NewDownload { repository_url: repository_url.to_string() })
                    .into();

                let content: Element<_> = column(vec![
                    url_input,
                    text("Supported protocols: file, http(s), ssh").into(),
                ]).into();

                container(content)
                    .padding(Padding::new(10.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .into()
            }
        };

        let new_plugin_button_text = text(icons::Icon::Plus)
            .font(icons::ICON_FONT);

        let new_plugin_button_text_container: Element<_> = container(new_plugin_button_text)
            .width(Length::Fill)
            .center_y()
            .center_x()
            .into();

        let new_plugin_button = button(new_plugin_button_text_container)
            .width(Length::Fill)
            .on_press(ManagementAppMsg::SelectItem(SelectedItem::NewPlugin { repository_url: Default::default() }))
            .into();

        let multiple = if self.running_downloads.len() > 1 { "s" } else { "" };
        let progress_bar_text: Element<_> = text(format!("{} plugin{} downloading...", self.running_downloads.len(), multiple))
            .into();

        let progress_bar: Element<_> = progress_bar(0.0..=100.0, 50.0)
            .into();

        let sidebar: Element<_> = column(vec![new_plugin_button, sidebar_content, progress_bar_text, progress_bar])
            .padding(Padding::new(4.0))
            .into();

        let content: Element<_> = row(vec![table, sidebar])
            .into();

        container(content)
            .padding(Padding::new(3.0))
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {

        let (dbus_download_status_stream, dbus_download_finished_stream) = futures::executor::block_on(async {
            let download_status_stream = self.dbus_server.receive_plugin_download_status_signal().await?;
            let download_finished_stream = self.dbus_server.receive_plugin_download_finished_signal().await?;

            Ok::<(PluginDownloadStatusSignalStream<'_>, PluginDownloadFinishedSignalStream<'_>), AnyError>((download_status_stream, download_finished_stream))
        }).unwrap();

        let dbus_download_status_stream = dbus_download_status_stream
            .map(|signal| {
                let signal = signal.args().unwrap();
                ManagementAppMsg::RunningDownloadStatus {
                    download_id: signal.download_id.to_owned(),
                    percent: signal.percent,
                }
            });

        let dbus_download_finished_stream = dbus_download_finished_stream
            .map(|signal| {
                let signal = signal.args().unwrap();
                ManagementAppMsg::RunningDownloadFinished {
                    download_id: signal.download_id.to_owned(),
                }
            });

        struct DownloadStatusStream;
        struct DownloadFinishedStream;

        Subscription::batch([
            subscription::run_with_id(std::any::TypeId::of::<DownloadStatusStream>(), dbus_download_status_stream),
            subscription::run_with_id(std::any::TypeId::of::<DownloadFinishedStream>(), dbus_download_finished_stream)
        ])
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

                        let icon: Element<_> = text(format!("{}", icon))
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
            ColumnKind::Name => 500.0,
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
                    };
                    (id, entrypoint)
                })
                .collect();

            let id = PluginId::new(plugin.plugin_id);
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