use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use iced::{Alignment, Application, color, Command, Element, executor, font, futures, Length, Padding, Renderer, Settings, Size, Subscription, theme, Theme, time, window};
use iced::theme::Palette;
use iced::widget::{button, checkbox, column, container, horizontal_space, row, scrollable, Space, text, text_input, vertical_rule};
use iced_aw::graphics::icons;
use iced_table::table;
use tonic::Request;

use common::model::{EntrypointId, PluginId};
use common::rpc::{BackendClient, RpcDownloadPluginRequest, RpcDownloadStatus, RpcDownloadStatusRequest, RpcEntrypointType, RpcPluginPreference, RpcPluginPreferenceUserData, RpcPluginPreferenceValueType, RpcPluginsRequest, RpcSetEntrypointStateRequest, RpcSetPluginStateRequest};
use common::rpc::rpc_backend_client::RpcBackendClient;
use common::rpc::rpc_ui_property_value::Value;

pub fn run() {
    ManagementAppModel::run(Settings {
        id: None,
        window: window::Settings {
            size: Size::new(900.0, 600.0),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    }).unwrap();
}


struct ManagementAppModel {
    backend_client: BackendClient,
    columns: Vec<Column>,
    rows: Vec<Row>,
    plugins: Rc<RefCell<HashMap<PluginId, Plugin>>>,
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
    CheckDownloadStatus,
    DownloadStatus {
        plugins: Vec<PluginId>,
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
    preferences: HashMap<String, PluginPreference>,
    preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Clone)]
struct Entrypoint {
    entrypoint_id: EntrypointId,
    entrypoint_name: String,
    entrypoint_type: EntrypointType,
    enabled: bool,
    preferences: HashMap<String, PluginPreference>,
    preferences_user_data: HashMap<String, PluginPreferenceUserData>,
}

#[derive(Debug, Clone)]
pub enum EntrypointType {
    Command,
    View,
    InlineView,
}

#[derive(Debug, Clone)]
pub enum PluginPreferenceUserData {
    Number {
        value: Option<f64>,
    },
    String {
        value: Option<String>,
    },
    Enum {
        value: Option<String>,
    },
    Bool {
        value: Option<bool>,
    },
    ListOfStrings {
        value: Option<Vec<String>>,
    },
    ListOfNumbers {
        value: Option<Vec<f64>>,
    },
    ListOfEnums {
        value: Option<Vec<String>>,
    }
}

#[derive(Debug, Clone)]
pub enum PluginPreference {
    Number {
        default: Option<f64>,
        description: String,
    },
    String {
        default: Option<String>,
        description: String,
    },
    Enum {
        default: Option<String>,
        description: String,
        enum_values: Vec<PreferenceEnumValue>,
    },
    Bool {
        default: Option<bool>,
        description: String,
    },
    ListOfStrings {
        default: Option<Vec<String>>,
        description: String,
    },
    ListOfNumbers {
        default: Option<Vec<f64>>,
        description: String,
    },
    ListOfEnums {
        default: Option<Vec<String>>,
        enum_values: Vec<PreferenceEnumValue>,
        description: String,
    }
}

#[derive(Debug, Clone)]
pub struct PreferenceEnumValue {
    pub label: String,
    pub value: String,
}


impl Application for ManagementAppModel {
    type Executor = executor::Default;
    type Message = ManagementAppMsg;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let backend_client = futures::executor::block_on(async {
            anyhow::Ok(RpcBackendClient::connect("http://127.0.0.1:42320").await?)
        }).unwrap();

        (
            ManagementAppModel {
                backend_client: backend_client.clone(),
                columns: vec![
                    Column::new(ColumnKind::ShowEntrypointsToggle),
                    Column::new(ColumnKind::Name),
                    Column::new(ColumnKind::Type),
                    Column::new(ColumnKind::EnableToggle),
                ],
                rows: vec![],
                plugins: Rc::new(RefCell::new(HashMap::new())),
                selected_item: SelectedItem::None,
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
                running_downloads: HashSet::new(),
            },
            Command::batch([
                font::load(icons::BOOTSTRAP_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
                Command::perform(
                    reload_plugins(backend_client),
                    ManagementAppMsg::PluginsReloaded,
                )
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
                let mut plugins = self.plugins.borrow_mut();
                let plugin = plugins.get_mut(&plugin_id).unwrap();
                plugin.show_entrypoints = !plugin.show_entrypoints;
                Command::none()
            }
            ManagementAppMsg::PluginsReloaded(plugins) => {
                let plugins = Rc::new(RefCell::new(plugins));
                self.plugins = plugins.clone();

                let plugin_refs = plugins.borrow();

                let mut plugin_refs: Vec<_> = plugin_refs
                    .iter()
                    .map(|(_, plugin)| plugin)
                    .collect();

                plugin_refs.sort_by_key(|plugin| &plugin.plugin_name);

                self.rows = plugin_refs
                    .iter()
                    .flat_map(|plugin| {
                        let mut result = vec![];

                        result.push(Row::Plugin {
                            plugins: plugins.clone(),
                            plugin_id: plugin.plugin_id.clone()
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
                                        plugins: plugins.clone(),
                                        plugin_id: plugin.plugin_id.clone(),
                                        entrypoint_id: entrypoint.entrypoint_id.clone(),
                                    }
                                })
                                .collect();

                            result.append(&mut entrypoints);
                        }

                        result
                    })
                    .collect();

                Command::none()
            }
            ManagementAppMsg::SelectItem(selected_item) => {
                self.selected_item = selected_item;
                Command::none()
            }
            ManagementAppMsg::EnabledToggleItem(item) => {
                match item {
                    EnabledItem::Plugin { enabled, plugin_id } => {
                        let mut backend_client = self.backend_client.clone();

                        Command::perform(
                            async move {
                                let request = RpcSetPluginStateRequest {
                                    plugin_id: plugin_id.to_string(),
                                    enabled,
                                };

                                backend_client.set_plugin_state(Request::new(request)).await.unwrap();

                                reload_plugins(backend_client).await
                            },
                            ManagementAppMsg::PluginsReloaded,
                        )
                    }
                    EnabledItem::Entrypoint { enabled, plugin_id, entrypoint_id } => {
                        let mut backend_client = self.backend_client.clone();

                        Command::perform(
                            async move {
                                let request = RpcSetEntrypointStateRequest {
                                    plugin_id: plugin_id.to_string(),
                                    entrypoint_id: entrypoint_id.to_string(),
                                    enabled,
                                };

                                backend_client.set_entrypoint_state(Request::new(request)).await.unwrap();

                                reload_plugins(backend_client).await
                            },
                            ManagementAppMsg::PluginsReloaded,
                        )
                    }
                }
            }
            ManagementAppMsg::AddPlugin { plugin_id } => {
                let mut backend_client = self.backend_client.clone();

                let exists = self.running_downloads.insert(plugin_id.clone());
                if !exists {
                    panic!("already downloading this plugins")
                }

                Command::perform(
                    async move {
                        let request = RpcDownloadPluginRequest {
                            plugin_id: plugin_id.to_string()
                        };

                        backend_client.download_plugin(Request::new(request)).await.unwrap()
                    },
                    |_| ManagementAppMsg::Noop,
                )
            }
            ManagementAppMsg::DownloadStatus { plugins } => {
                for plugin in plugins {
                    self.running_downloads.remove(&plugin);
                }
                let backend_client = self.backend_client.clone();
                Command::perform(
                    reload_plugins(backend_client),
                    ManagementAppMsg::PluginsReloaded,
                )
            }
            ManagementAppMsg::CheckDownloadStatus => {
                let mut backend_client = self.backend_client.clone();

                Command::perform(
                    async move {
                        let plugins = backend_client.download_status(Request::new(RpcDownloadStatusRequest::default()))
                            .await
                            .unwrap()
                            .into_inner()
                            .status_per_plugin
                            .into_iter()
                            .filter_map(|(plugin_id, status)| {
                                let status: RpcDownloadStatus = status.status.try_into()
                                    .expect("download status failed");

                                match status {
                                    RpcDownloadStatus::InProgress => None,
                                    RpcDownloadStatus::Done => Some(PluginId::from_string(plugin_id)),
                                    RpcDownloadStatus::Failed => Some(PluginId::from_string(plugin_id))
                                }
                            })
                            .collect::<Vec<_>>();

                        ManagementAppMsg::DownloadStatus { plugins }
                    },
                    std::convert::identity,
                )
            }
            ManagementAppMsg::Noop => {
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme> {
        let table: Element<_> = table(self.header.clone(), self.body.clone(), &self.columns, &self.rows, ManagementAppMsg::TableSyncHeader)
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
                let plugins = self.plugins.borrow();
                let plugin = plugins.get(&plugin_id).unwrap();

                let name = container(text(&plugin.plugin_name))
                    .padding(Padding::new(10.0))
                    .into();

                column(vec![
                    name,
                ]).into()
            }
            SelectedItem::Entrypoint { plugin_id, entrypoint_id } => {
                let plugins = self.plugins.borrow();
                let plugin = plugins.get(&plugin_id).unwrap();
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
            text(icons::BootstrapIcon::Plus)
                .font(icons::BOOTSTRAP_FONT)
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
            horizontal_space()
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
        time::every(Duration::from_millis(300))
            .map(|_| ManagementAppMsg::CheckDownloadStatus)
    }

    fn theme(&self) -> Self::Theme {
        Theme::custom("gauntlet".to_string(), Palette {
            background: iced::color!(0x2C323A),
            text: iced::color!(0xCAC2B6),
            primary: iced::color!(0xC79F60),
            success: iced::color!(0x659B5E),
            danger: iced::color!(0x6C1B1B),
        })
    }
}


enum Row {
    Plugin {
        plugins: Rc<RefCell<HashMap<PluginId, Plugin>>>,
        plugin_id: PluginId
    },
    Entrypoint {
        plugins: Rc<RefCell<HashMap<PluginId, Plugin>>>,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
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

impl<'a> table::Column<'a, ManagementAppMsg, Theme, Renderer> for Column {
    type Row = Row;

    fn header(&'a self, _col_index: usize) -> Element<'a, ManagementAppMsg> {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => {
                horizontal_space()
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
        &'a self,
        _col_index: usize,
        _row_index: usize,
        row_entry: &'a Self::Row,
    ) -> Element<'a, ManagementAppMsg> {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => {
                match row_entry {
                    Row::Plugin { plugins, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();

                        let icon = if plugin.show_entrypoints { icons::BootstrapIcon::CaretDown } else { icons::BootstrapIcon::CaretRight };

                        let icon: Element<_> = text(icon)
                            .font(icons::BOOTSTRAP_FONT)
                            .into();

                        button(icon)
                            .style(theme::Button::Text)
                            .on_press(ManagementAppMsg::ToggleShowEntrypoints { plugin_id: plugin.plugin_id.clone() })
                            .into()
                    }
                    Row::Entrypoint { .. } => {
                        horizontal_space()
                            .into()
                    }
                }
            }
            ColumnKind::Name => {
                let content: Element<_> = match row_entry {
                    Row::Plugin { plugins, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();

                        container(text(&plugin.plugin_name))
                            .center_y()
                            .into()
                    }
                    Row::Entrypoint { plugins, plugin_id, entrypoint_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        let text: Element<_> = text(&entrypoint.entrypoint_name)
                            .into();

                        let text: Element<_> = row(vec![
                            Space::with_width(Length::Fixed(30.0)).into(),
                            text,
                        ]).into();

                        container(text)
                            .center_y()
                            .into()
                    }
                };

                let msg = match &row_entry {
                    Row::Plugin { plugins, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();

                        SelectedItem::Plugin {
                            plugin_id: plugin.plugin_id.clone()
                        }
                    },
                    Row::Entrypoint { plugins, entrypoint_id, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        SelectedItem::Entrypoint {
                            plugin_id: plugin.plugin_id.clone(),
                            entrypoint_id: entrypoint.entrypoint_id.clone(),
                        }
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
                        horizontal_space()
                            .into()
                    }
                    Row::Entrypoint { plugins, plugin_id, entrypoint_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        let entrypoint_type = match entrypoint.entrypoint_type {
                            EntrypointType::Command => "Command",
                            EntrypointType::View => "View",
                            EntrypointType::InlineView => "Inline View"
                        };

                        container(text(entrypoint_type))
                            .center_y()
                            .into()
                    }
                };

                let msg = match &row_entry {
                    Row::Plugin { plugins, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();

                        SelectedItem::Plugin {
                            plugin_id: plugin.plugin_id.clone()
                        }
                    },
                    Row::Entrypoint { plugins, entrypoint_id, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        SelectedItem::Entrypoint {
                            plugin_id: plugin.plugin_id.clone(),
                            entrypoint_id: entrypoint.entrypoint_id.clone(),
                        }
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
                    Row::Plugin { plugins, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();

                        (
                            plugin.enabled,
                            plugin.plugin_id.clone(),
                            None
                        )
                    }
                    Row::Entrypoint { plugins, entrypoint_id, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        (
                            entrypoint.enabled,
                            plugin.plugin_id.clone(),
                            Some(entrypoint.entrypoint_id.clone())
                        )
                    }
                };


                // TODO disable if plugin is disabled but preserve current state https://github.com/iced-rs/iced/pull/2109
                let checkbox: Element<_> = checkbox("", enabled)
                    .on_toggle(move |enabled| {
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
                    })
                    .into();

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


async fn reload_plugins(mut backend_client: BackendClient) -> HashMap<PluginId, Plugin> {
    backend_client.plugins(Request::new(RpcPluginsRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .plugins
        .into_iter()
        .map(|plugin| {
            let entrypoints: HashMap<_, _> = plugin.entrypoints
                .into_iter()
                .map(|entrypoint| {
                    let id = EntrypointId::new(entrypoint.entrypoint_id);
                    let entrypoint_type: RpcEntrypointType = entrypoint.entrypoint_type.try_into()
                        .expect("download status failed");

                    let entrypoint_type = match entrypoint_type {
                        RpcEntrypointType::Command => EntrypointType::Command,
                        RpcEntrypointType::View => EntrypointType::View,
                        RpcEntrypointType::InlineView => EntrypointType::InlineView
                    };

                    let entrypoint = Entrypoint {
                        enabled: entrypoint.enabled,
                        entrypoint_id: id.clone(),
                        entrypoint_name: entrypoint.entrypoint_name.clone(),
                        entrypoint_type,
                        preferences: entrypoint.preferences.into_iter()
                            .map(|(key, value)| (key, plugin_preference_from_grpc(value)))
                            .collect(),
                        preferences_user_data: entrypoint.preferences_user_data.into_iter()
                            .map(|(key, value)| (key, plugin_preference_user_data_from_grpc(value)))
                            .collect(),
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
                preferences: plugin.preferences.into_iter()
                    .map(|(key, value)| (key, plugin_preference_from_grpc(value)))
                    .collect(),
                preferences_user_data: plugin.preferences_user_data.into_iter()
                    .map(|(key, value)| (key, plugin_preference_user_data_from_grpc(value)))
                    .collect(),};

            (id, plugin)
        })
        .collect()
}


fn plugin_preference_from_grpc(value: RpcPluginPreference) -> PluginPreference {
    let value_type: RpcPluginPreferenceValueType = value.r#type.try_into().unwrap();
    match value_type {
        RpcPluginPreferenceValueType::Number => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Number(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::Number {
                default,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::String => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::String {
                default,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::Enum => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::Enum {
                default,
                description: value.description,
                enum_values: value.enum_values.into_iter()
                    .map(|value| PreferenceEnumValue { label: value.label, value: value.value })
                    .collect()
            }
        }
        RpcPluginPreferenceValueType::Bool => {
            let default = value.default
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Bool(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreference::Bool {
                default,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfStrings => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value.default_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(default_list)
                },
                false => None
            };

            PluginPreference::ListOfStrings {
                default: default_list,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfNumbers => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value.default_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::Number(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(default_list)
                },
                false => None
            };

            PluginPreference::ListOfNumbers {
                default: default_list,
                description: value.description,
            }
        }
        RpcPluginPreferenceValueType::ListOfEnums => {
            let default_list = match value.default_list_exists {
                true => {
                    let default_list = value.default_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(default_list)
                },
                false => None
            };

            PluginPreference::ListOfEnums {
                default: default_list,
                enum_values: value.enum_values.into_iter()
                    .map(|value| PreferenceEnumValue { label: value.label, value: value.value })
                    .collect(),
                description: value.description,
            }
        }
    }
}

fn plugin_preference_user_data_from_grpc(value: RpcPluginPreferenceUserData) -> PluginPreferenceUserData {
    let value_type: RpcPluginPreferenceValueType = value.r#type.try_into().unwrap();
    match value_type {
        RpcPluginPreferenceValueType::Number => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Number(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::Number {
                value
            }
        }
        RpcPluginPreferenceValueType::String => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::String {
                value
            }
        }
        RpcPluginPreferenceValueType::Enum => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::String(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::Enum {
                value
            }
        }
        RpcPluginPreferenceValueType::Bool => {
            let value = value.value
                .map(|value| {
                    match value.value.unwrap() {
                        Value::Bool(value) => value,
                        _ => unreachable!()
                    }
                });

            PluginPreferenceUserData::Bool {
                value
            }
        }
        RpcPluginPreferenceValueType::ListOfStrings => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value.value_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(value_list)
                },
                false => None
            };

            PluginPreferenceUserData::ListOfStrings {
                value
            }
        }
        RpcPluginPreferenceValueType::ListOfNumbers => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value.value_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::Number(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(value_list)
                },
                false => None
            };

            PluginPreferenceUserData::ListOfNumbers {
                value
            }
        }
        RpcPluginPreferenceValueType::ListOfEnums => {
            let value = match value.value_list_exists {
                true => {
                    let value_list = value.value_list
                        .into_iter()
                        .flat_map(|value| value.value.map(|value| {
                            match value {
                                Value::String(value) => value,
                                _ => unreachable!()
                            }
                        }))
                        .collect();

                    Some(value_list)
                },
                false => None
            };

            PluginPreferenceUserData::ListOfEnums {
                value
            }
        }
    }
}
