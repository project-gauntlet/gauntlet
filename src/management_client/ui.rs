use std::collections::HashMap;

use deno_core::error::AnyError;
use iced::{Application, Command, Element, executor, font, futures, Length, Padding, Renderer, Settings, theme, window};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced_aw::graphics::icons;
use iced_table::table;
use zbus::Connection;

use crate::common::model::{EntrypointId, PluginId};
use crate::management_client::dbus::DbusManagementServerProxyProxy;

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
}

#[derive(Debug, Clone)]
enum ManagementAppMsg {
    TableSyncHeader(scrollable::AbsoluteOffset),
    FontLoaded(Result<(), font::Error>),
    PluginsLoaded(HashMap<PluginId, Plugin>),
    ToggleShowEntrypoints {
        plugin_id: PluginId,
    },
    SelectItem(SelectedItem)
}

#[derive(Debug, Clone)]
enum SelectedItem {
    None,
    Plugin {
        plugin_id: PluginId
    },
    Entrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId
    }
}


#[derive(Debug, Clone)]
struct Plugin {
    plugin_id: PluginId,
    plugin_name: String,
    show_entrypoints: bool,
    entrypoints: HashMap<EntrypointId, Entrypoint>,
}

#[derive(Debug, Clone)]
struct Entrypoint {
    entrypoint_id: EntrypointId,
    entrypoint_name: String,
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
                    Column::new(ColumnKind::EntrypointToggle),
                    Column::new(ColumnKind::Name),
                ],
                plugins: HashMap::new(),
                selected_item: SelectedItem::None,
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
            },
            Command::batch([
                font::load(icons::ICON_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
                Command::perform(async move {
                    let plugins = dbus_server_clone.plugins().await.unwrap();

                    let plugins: HashMap<_, _> = plugins.into_iter()
                        .map(|plugin| {
                            let entrypoints: HashMap<_, _> = plugin.entrypoints
                                .into_iter()
                                .map(|entrypoint| {
                                    let id = EntrypointId::new(entrypoint.entrypoint_id);
                                    let entrypoint = Entrypoint {
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
                                entrypoints,
                            };

                            (id, plugin)
                        })
                        .collect();

                    plugins
                }, ManagementAppMsg::PluginsLoaded)
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
            ManagementAppMsg::PluginsLoaded(plugins) => {
                self.plugins = plugins;
                Command::none()
            }
            ManagementAppMsg::SelectItem(selected_item) => {
                self.selected_item = selected_item;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let mut plugins: Vec<_> = self.plugins
            .iter()
            .map(|(_, plugin)| plugin)
            .collect();

        plugins.sort_by_key(|plugins| &plugins.plugin_name);

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
                                entrypoint
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

        let sidebar: Element<_> = match &self.selected_item {
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
        };

        let content: Element<_> = row(vec![table, sidebar])
            .into();

        container(content)
            .padding(Padding::new(3.0))
            .into()
    }
}

enum Row<'a> {
    Plugin {
        plugin: &'a Plugin
    },
    Entrypoint {
        plugin: &'a Plugin,
        entrypoint: &'a Entrypoint
    },
}

enum ColumnKind {
    EntrypointToggle,
    Name,
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
            ColumnKind::EntrypointToggle => {
                horizontal_space(Length::Fill)
                    .into()
            }
            ColumnKind::Name => {
                container(text("Name"))
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
            ColumnKind::EntrypointToggle => {
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
                    },
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
                    Row::Plugin { plugin } => SelectedItem::Plugin { plugin_id: plugin.plugin_id.clone() },
                    Row::Entrypoint { entrypoint, plugin } => SelectedItem::Entrypoint {
                        plugin_id: plugin.plugin_id.clone(),
                        entrypoint_id: entrypoint.entrypoint_id.clone()
                    }
                };

                button(content)
                    .style(theme::Button::Text)
                    .on_press(ManagementAppMsg::SelectItem(msg))
                    .width(Length::Fill)
                    .into()
            }
        }
    }

    fn width(&self) -> f32 {
        match self.kind {
            ColumnKind::EntrypointToggle => 35.0,
            ColumnKind::Name => 550.0,
        }
    }

    fn resize_offset(&self) -> Option<f32> {
        None
    }
}
