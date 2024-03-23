use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::rc::Rc;
use std::time::Duration;

use iced::{Alignment, Application, color, Command, Element, executor, font, Font, futures, Length, Padding, Renderer, Settings, Size, Subscription, theme, Theme, time, window};
use iced::font::Weight;
use iced::theme::Palette;
use iced::widget::{button, checkbox, column, container, horizontal_space, pick_list, row, scrollable, Space, text, text_input, vertical_rule};
use iced_aw::graphics::icons;
use iced_aw::helpers::number_input;
use iced_table::table;
use tonic::Request;

use common::model::{EntrypointId, PluginId};
use common::rpc::{BackendClient, plugin_preference_user_data_from_npb, plugin_preference_user_data_to_npb, RpcDownloadPluginRequest, RpcDownloadStatus, RpcDownloadStatusRequest, RpcEntrypointTypeSettings, RpcNoProtoBufPluginPreferenceUserData, RpcPluginPreference, RpcPluginPreferenceValueType, RpcPluginsRequest, RpcSetEntrypointStateRequest, RpcSetPluginStateRequest, RpcSetPreferenceValueRequest};
use common::rpc::rpc_backend_client::RpcBackendClient;
use common::rpc::rpc_ui_property_value::Value;

pub fn run() {
    ManagementAppModel::run(Settings {
        id: None,
        window: window::Settings {
            size: Size::new(900.0, 600.0),
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
    preference_user_data: HashMap<(PluginId, Option<EntrypointId>, String), PluginPreferenceUserData>,
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
    UpdatePreferenceValue {
        plugin_id: PluginId,
        entrypoint_id: Option<EntrypointId>,
        name: String,
        user_data: PluginPreferenceUserData
    },
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
    plugin_description: String,
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
    entrypoint_description: String,
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
    CommandGenerator,
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
        new_value: String
    },
    ListOfNumbers {
        value: Option<Vec<f64>>,
        new_value: f64
    },
    ListOfEnums {
        value: Option<Vec<String>>,
        new_value: Option<SelectItem>
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
                preference_user_data: HashMap::new(),
                selected_item: SelectedItem::None,
                header: scrollable::Id::unique(),
                body: scrollable::Id::unique(),
                running_downloads: HashSet::new(),
            },
            Command::batch([
                font::load(icons::BOOTSTRAP_FONT_BYTES).map(ManagementAppMsg::FontLoaded),
                Command::perform(
                    async {
                        reload_plugins(backend_client).await
                    },
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
                self.preference_user_data = plugins.iter()
                    .map(|(plugin_id, plugin)| {
                        let mut result = vec![];

                        for (name, user_data) in &plugin.preferences_user_data {
                            result.push(((plugin_id.clone(), None, name.clone()), user_data.clone()))
                        }

                        for (entrypoint_id, entrypoint) in &plugin.entrypoints {
                            for (name, user_data) in &entrypoint.preferences_user_data {
                                result.push(((plugin_id.clone(), Some(entrypoint_id.clone()), name.clone()), user_data.clone()))
                            }
                        }

                        result
                    })
                    .flatten()
                    .collect();

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
                    async {
                        reload_plugins(backend_client).await
                    },
                    ManagementAppMsg::PluginsReloaded,
                )
            }
            ManagementAppMsg::CheckDownloadStatus => {
                if self.running_downloads.is_empty() {
                    Command::none()
                } else {
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
            }
            ManagementAppMsg::Noop => {
                Command::none()
            }
            ManagementAppMsg::UpdatePreferenceValue { plugin_id, entrypoint_id, name, user_data } => {
                self.preference_user_data
                    .insert((plugin_id.clone(), entrypoint_id.clone(), name.clone()), user_data.clone());

                let request = RpcSetPreferenceValueRequest {
                    plugin_id: plugin_id.to_string(),
                    entrypoint_id: entrypoint_id.map(|id| id.to_string()).unwrap_or_default(),
                    preference_name: name,
                    preference_value: Some(plugin_preference_user_data_from_npb(plugin_preference_user_data_to_grpc(user_data))),
                };

                let mut backend_client = self.backend_client.clone();

                Command::perform(
                    async move {
                        backend_client.set_preference_value(Request::new(request))
                            .await
                            .unwrap();
                    },
                    |_| ManagementAppMsg::Noop,
                )
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

                let description_label: Element<_> = text("Description")
                    .font(Font {
                        weight: Weight::Bold,
                        ..Font::DEFAULT
                    })
                    .into();

                let description_label = container(description_label)
                    .padding(Padding::from([0.0, 0.0, 0.0, 10.0]))
                    .into();

                let description = container(text(&plugin.plugin_description))
                    .padding(Padding::new(10.0))
                    .into();

                let mut column_content = vec![
                    name,
                    description_label,
                    description,
                ];

                for element in preferences_ui(plugin_id.clone(), None, &plugin.preferences, &self.preference_user_data) {
                    column_content.push(element)
                }

                let column: Element<_> = column(column_content)
                    .padding(Padding::from([0.0, 5.0, 0.0, 0.0]))
                    .into();

                let column: Element<_> = scrollable(column)
                    .width(Length::Fill)
                    .into();

                container(column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([5.0, 0.0]))
                    .into()
            }
            SelectedItem::Entrypoint { plugin_id, entrypoint_id } => {
                let plugins = self.plugins.borrow();
                let plugin = plugins.get(&plugin_id).unwrap();
                let entrypoint = plugin.entrypoints.get(entrypoint_id).unwrap();

                let name = container(text(&entrypoint.entrypoint_name))
                    .padding(Padding::new(10.0))
                    .into();

                let description_label: Element<_> = text("Description")
                    .font(Font {
                        weight: Weight::Bold,
                        ..Font::DEFAULT
                    })
                    .into();

                let description_label = container(description_label)
                    .padding(Padding::from([0.0, 0.0, 0.0, 10.0]))
                    .into();

                let description = container(text(&entrypoint.entrypoint_description))
                    .padding(Padding::new(10.0))
                    .into();

                let mut column_content = vec![
                    name,
                    description_label,
                    description,
                ];

                for element in preferences_ui(plugin_id.clone(), Some(entrypoint_id.clone()), &entrypoint.preferences, &self.preference_user_data) {
                    column_content.push(element)
                }

                let column: Element<_> = column(column_content)
                    .padding(Padding::from([0.0, 5.0, 0.0, 0.0]))
                    .into();

                let column: Element<_> = scrollable(column)
                    .width(Length::Fill)
                    .into();

                container(column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([5.0, 0.0]))
                    .into()
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
                            EntrypointType::InlineView => "Inline View",
                            EntrypointType::CommandGenerator => "Command Generator"
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
                let (enabled, show_checkbox, plugin_id, entrypoint_id) = match &row_entry {
                    Row::Plugin { plugins, plugin_id } => {
                        let plugins = plugins.borrow();
                        let plugin = plugins.get(&plugin_id).unwrap();

                        (
                            plugin.enabled,
                            true,
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
                            plugin.enabled,
                            plugin.plugin_id.clone(),
                            Some(entrypoint.entrypoint_id.clone())
                        )
                    }
                };

                let on_toggle = if show_checkbox {
                    Some(move |enabled| {
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
                } else {
                    None
                };

                let checkbox: Element<_> = checkbox("", enabled)
                    .on_toggle_maybe(on_toggle)
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

fn preferences_ui<'a>(
    plugin_id: PluginId,
    entrypoint_id: Option<EntrypointId>,
    preferences: &HashMap<String, PluginPreference>,
    preference_user_data: &HashMap<(PluginId, Option<EntrypointId>, String), PluginPreferenceUserData>
) -> Vec<Element<'a, ManagementAppMsg>> {
    let mut column_content = vec![];

    let mut preferences: Vec<_> = preferences.iter()
        .map(|entry| entry)
        .collect();

    preferences.sort_by_key(|(&ref key, _)| key);

    for (preference_name, preference) in preferences {
        let plugin_id = plugin_id.clone();
        let entrypoint_id = entrypoint_id.clone();

        let user_data = preference_user_data.get(&(plugin_id.clone(), entrypoint_id.clone(), preference_name.clone()));

        let preference_name = preference_name.to_owned();

        let preference_label: Element<_> = text(&preference_name)
            .font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            })
            .into();

        let preference_label = container(preference_label)
            .padding(Padding::from([0.0, 0.0, 0.0, 10.0]))
            .into();

        column_content.push(preference_label);

        let description = match preference {
            PluginPreference::Number { description, .. } => description,
            PluginPreference::String { description, .. } => description,
            PluginPreference::Enum { description, .. } => description,
            PluginPreference::Bool { description, .. } => description,
            PluginPreference::ListOfStrings { description, .. } => description,
            PluginPreference::ListOfNumbers { description, .. } => description,
            PluginPreference::ListOfEnums { description, .. } => description,
        };

        if !description.trim().is_empty() {
            let description = container(text(description))
                .padding(Padding::new(10.0))
                .into();

            column_content.push(description);
        }

        let input_fields: Vec<Element<_>> = match preference {
            PluginPreference::Number { default, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserData::Number { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let value = value.or(default.to_owned()).unwrap_or_default();

                let input_field: Element<_> = number_input(value, f64::MAX, std::convert::identity)
                    .bounds((f64::MIN, f64::MAX))
                    .width(Length::Fill)
                    .into();

                let input_field = input_field.map(Box::new(move |value| {
                    ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::Number {
                            value: Some(value),
                        },
                    }
                }));

                let input_field = container(input_field)
                    .width(Length::Fill)
                    .padding(Padding::new(10.0))
                    .into();

                vec![input_field]
            }
            PluginPreference::String { default, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserData::String { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let default = default.to_owned().unwrap_or_default();

                let input_field: Element<_> = text_input(&default, &value.unwrap_or_default())
                    .on_input(Box::new(move |value| {
                        ManagementAppMsg::UpdatePreferenceValue {
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                            name: preference_name.to_owned(),
                            user_data: PluginPreferenceUserData::String {
                                value: Some(value),
                            },
                        }
                    }))
                    .into();

                let input_field = container(input_field)
                    .padding(Padding::new(10.0))
                    .into();

                vec![input_field]
            }
            PluginPreference::Enum { default, enum_values, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserData::Enum { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let enum_values: Vec<_> = enum_values.iter()
                    .map(|enum_item| SelectItem { label: enum_item.label.to_owned(), value: enum_item.value.to_owned() })
                    .collect();

                let value = value.or(default.to_owned())
                    .map(|value| enum_values.iter().find(|item| item.value == value))
                    .flatten()
                    .map(|value| value.clone());

                let input_field: Element<_> = pick_list(
                    enum_values,
                    value,
                    Box::new(move |item: SelectItem| ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::Enum {
                            value: Some(item.value),
                        },
                    })
                )
                    .width(Length::Fill)
                    .into();

                let input_field = container(input_field)
                    .padding(Padding::new(10.0))
                    .width(Length::Fill)
                    .into();

                vec![input_field]
            }
            PluginPreference::Bool { default, .. } => {
                let value = match user_data {
                    None => None,
                    Some(PluginPreferenceUserData::Bool { value }) => value.to_owned(),
                    Some(_) => unreachable!()
                };

                let input_field: Element<_> = checkbox(preference_name.clone(), value.or(default.to_owned()).unwrap_or(false))
                    .on_toggle(Box::new(move |value| {
                        ManagementAppMsg::UpdatePreferenceValue {
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                            name: preference_name.to_owned(),
                            user_data: PluginPreferenceUserData::Bool {
                                value: Some(value),
                            },
                        }
                    }))
                    .into();

                let input_field = container(input_field)
                    .padding(Padding::new(10.0))
                    .into();

                vec![input_field]
            }
            PluginPreference::ListOfStrings { .. } => {
                let (value, new_value) = match user_data {
                    None => (None, "".to_owned()),
                    Some(PluginPreferenceUserData::ListOfStrings { value, new_value }) => (value.to_owned(), new_value.to_owned()),
                    Some(_) => unreachable!()
                };

                let mut items: Vec<_> = value.clone()
                    .unwrap_or(vec![])
                    .iter()
                    .enumerate()
                    .map(|(index, value_item)| {

                        let mut value = value.clone();
                        if let Some(value) = &mut value {
                            value.remove(index);
                        }

                        let item_text: Element<_> = text(value_item)
                            .into();

                        let item_text: Element<_> = container(item_text)
                            .padding(Padding::new(4.0))
                            .into();

                        let item_text = container(item_text)
                            .height(Length::Fixed(30.0))
                            .width(Length::Fill)
                            .style(theme::Container::Box)
                            .into();

                        let remove_icon = text(icons::BootstrapIcon::Dash)
                            .font(icons::BOOTSTRAP_FONT);

                        let remove_button: Element<_> = button(remove_icon)
                            .style(theme::Button::Primary)
                            .on_press(ManagementAppMsg::UpdatePreferenceValue {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                                name: preference_name.to_owned(),
                                user_data: PluginPreferenceUserData::ListOfStrings {
                                    value,
                                    new_value: new_value.clone(),
                                },
                            })
                            .into();

                        let item: Element<_> = row([item_text, remove_button])
                            .into();

                        let item = container(item)
                            .padding(Padding::from([5.0, 10.0]))
                            .into();

                        item
                    })
                    .collect();


                let save_value = match &value {
                    None => vec![new_value.clone()],
                    Some(value) => {
                        let mut save_value = value.clone();
                        save_value.push(new_value.clone());
                        save_value
                    }
                };

                let add_msg = if new_value.is_empty() {
                    None
                } else {
                    Some(ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::ListOfStrings {
                            value: Some(save_value),
                            new_value: "".to_owned(),
                        },
                    })
                };

                let add_icon: Element<_> = text(icons::BootstrapIcon::Plus)
                    .font(icons::BOOTSTRAP_FONT)
                    .into();

                let add_button: Element<_> = button(add_icon)
                    .style(theme::Button::Primary)
                    .on_press_maybe(add_msg)
                    .into();

                let add_text_input: Element<_> = text_input("", &new_value)
                    .on_input(move |new_value| ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::ListOfStrings {
                            value: value.clone(),
                            new_value,
                        },
                    })
                    .into();

                let add_item: Element<_> = row([add_text_input, add_button])
                    .into();

                let add_item: Element<_> = container(add_item)
                    .padding(Padding::new(10.0))
                    .into();

                items.push(add_item);

                items
            }
            PluginPreference::ListOfNumbers { .. } => {
                let (value, new_value) = match user_data {
                    None => (None, 0.0),
                    Some(PluginPreferenceUserData::ListOfNumbers { value, new_value }) => (value.to_owned(), new_value.to_owned()),
                    Some(_) => unreachable!()
                };


                let mut items: Vec<_> = value.clone()
                    .unwrap_or(vec![])
                    .iter()
                    .enumerate()
                    .map(|(index, value_item)| {

                        let mut value = value.clone();
                        if let Some(value) = &mut value {
                            value.remove(index);
                        }

                        let item_text: Element<_> = text(value_item)
                            .into();

                        let item_text: Element<_> = container(item_text)
                            .padding(Padding::new(4.0))
                            .into();

                        let item_text = container(item_text)
                            .height(Length::Fixed(30.0))
                            .width(Length::Fill)
                            .style(theme::Container::Box)
                            .into();

                        let remove_icon = text(icons::BootstrapIcon::Dash)
                            .font(icons::BOOTSTRAP_FONT);

                        let remove_button: Element<_> = button(remove_icon)
                            .style(theme::Button::Primary)
                            .on_press(ManagementAppMsg::UpdatePreferenceValue {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                                name: preference_name.to_owned(),
                                user_data: PluginPreferenceUserData::ListOfNumbers {
                                    value,
                                    new_value: new_value.clone(),
                                },
                            })
                            .into();

                        let item: Element<_> = row([item_text, remove_button])
                            .into();

                        let item = container(item)
                            .padding(Padding::from([5.0, 10.0]))
                            .into();

                        item
                    })
                    .collect();


                let save_value = match &value {
                    None => vec![new_value.clone()],
                    Some(value) => {
                        let mut save_value = value.clone();
                        save_value.push(new_value.clone());
                        save_value
                    }
                };

                let add_icon: Element<_> = text(icons::BootstrapIcon::Plus)
                    .font(icons::BOOTSTRAP_FONT)
                    .into();

                let add_button: Element<_> = button(add_icon)
                    .style(theme::Button::Primary)
                    .on_press(ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::ListOfNumbers {
                            value: Some(save_value),
                            new_value: 0.0,
                        },
                    })
                    .into();


                let add_number_input: Element<_> = number_input(new_value, f64::MAX, std::convert::identity)
                    .bounds((f64::MIN, f64::MAX))
                    .width(Length::Fill)
                    .into();

                let add_number_input = add_number_input.map(Box::new(move |new_value| {
                    ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::ListOfNumbers {
                            value: value.clone(),
                            new_value,
                        },
                    }
                }));

                let add_item: Element<_> = row([add_number_input, add_button])
                    .into();

                let add_item: Element<_> = container(add_item)
                    .padding(Padding::new(10.0))
                    .into();

                items.push(add_item);

                items
            }
            PluginPreference::ListOfEnums { enum_values, .. } => {
                let (value, new_value) = match user_data {
                    None => (None, None),
                    Some(PluginPreferenceUserData::ListOfEnums { value, new_value }) => (value.to_owned(), new_value.to_owned()),
                    Some(_) => unreachable!()
                };

                let mut items: Vec<_> = value.clone()
                    .unwrap_or(vec![])
                    .iter()
                    .enumerate()
                    .map(|(index, value_item)| {

                        let mut value = value.clone();
                        if let Some(value) = &mut value {
                            value.remove(index);
                        }

                        let item_text: Element<_> = text(value_item)
                            .into();

                        let item_text: Element<_> = container(item_text)
                            .padding(Padding::new(4.0))
                            .into();

                        let item_text = container(item_text)
                            .height(Length::Fixed(30.0))
                            .width(Length::Fill)
                            .style(theme::Container::Box)
                            .into();

                        let remove_icon = text(icons::BootstrapIcon::Dash)
                            .font(icons::BOOTSTRAP_FONT);

                        let remove_button: Element<_> = button(remove_icon)
                            .style(theme::Button::Primary)
                            .on_press(ManagementAppMsg::UpdatePreferenceValue {
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                                name: preference_name.to_owned(),
                                user_data: PluginPreferenceUserData::ListOfEnums {
                                    value,
                                    new_value: new_value.clone(),
                                },
                            })
                            .into();

                        let item: Element<_> = row([item_text, remove_button])
                            .into();

                        let item = container(item)
                            .padding(Padding::from([5.0, 10.0]))
                            .into();

                        item
                    })
                    .collect();


                let add_msg = match &new_value {
                    None => None,
                    Some(new_value) => {
                        let save_value = match &value {
                            None => vec![new_value.value.clone()],
                            Some(value) => {
                                let mut save_value = value.clone();
                                save_value.push(new_value.value.clone());
                                save_value
                            }
                        };

                        Some(ManagementAppMsg::UpdatePreferenceValue {
                            plugin_id: plugin_id.clone(),
                            entrypoint_id: entrypoint_id.clone(),
                            name: preference_name.to_owned(),
                            user_data: PluginPreferenceUserData::ListOfEnums {
                                value: Some(save_value),
                                new_value: None,
                            },
                        })
                    }
                };


                let add_icon: Element<_> = text(icons::BootstrapIcon::Plus)
                    .font(icons::BOOTSTRAP_FONT)
                    .into();

                let add_button: Element<_> = button(add_icon)
                    .style(theme::Button::Primary)
                    .on_press_maybe(add_msg)
                    .into();

                let enum_values: Vec<_> = enum_values.iter()
                    .map(|enum_item| SelectItem { label: enum_item.label.to_owned(), value: enum_item.value.to_owned() })
                    .collect();

                let add_enum_input: Element<_> = pick_list(
                    enum_values,
                    new_value,
                    Box::new(move |new_value: SelectItem| ManagementAppMsg::UpdatePreferenceValue {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                        name: preference_name.to_owned(),
                        user_data: PluginPreferenceUserData::ListOfEnums {
                            value: value.clone(),
                            new_value: Some(new_value),
                        },
                    }),
                )
                    .width(Length::Fill)
                    .into();


                let add_item: Element<_> = row([add_enum_input, add_button])
                    .into();

                let add_item: Element<_> = container(add_item)
                    .padding(Padding::new(10.0))
                    .into();

                items.push(add_item);

                items
            }
        };

        for input_field in input_fields {
            column_content.push(input_field);
        }
    }

    column_content
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
                    let id = EntrypointId::from_string(entrypoint.entrypoint_id);
                    let entrypoint_type: RpcEntrypointTypeSettings = entrypoint.entrypoint_type.try_into()
                        .expect("download status failed");

                    let entrypoint_type = match entrypoint_type {
                        RpcEntrypointTypeSettings::SCommand => EntrypointType::Command,
                        RpcEntrypointTypeSettings::SView => EntrypointType::View,
                        RpcEntrypointTypeSettings::SInlineView => EntrypointType::InlineView,
                        RpcEntrypointTypeSettings::SCommandGenerator => EntrypointType::CommandGenerator
                    };

                    let entrypoint = Entrypoint {
                        enabled: entrypoint.enabled,
                        entrypoint_id: id.clone(),
                        entrypoint_name: entrypoint.entrypoint_name.clone(),
                        entrypoint_description: entrypoint.entrypoint_description,
                        entrypoint_type,
                        preferences: entrypoint.preferences.into_iter()
                            .map(|(key, value)| (key, plugin_preference_from_grpc(value)))
                            .collect(),
                        preferences_user_data: entrypoint.preferences_user_data.into_iter()
                            .map(|(key, value)| (key, plugin_preference_user_data_from_grpc(plugin_preference_user_data_to_npb(value))))
                            .collect(),
                    };
                    (id, entrypoint)
                })
                .collect();

            let id = PluginId::from_string(plugin.plugin_id);
            let plugin = Plugin {
                plugin_id: id.clone(),
                plugin_name: plugin.plugin_name,
                plugin_description: plugin.plugin_description,
                show_entrypoints: true,
                enabled: plugin.enabled,
                entrypoints,
                preferences: plugin.preferences.into_iter()
                    .map(|(key, value)| (key, plugin_preference_from_grpc(value)))
                    .collect(),
                preferences_user_data: plugin.preferences_user_data.into_iter()
                    .map(|(key, value)| (key, plugin_preference_user_data_from_grpc(plugin_preference_user_data_to_npb(value))))
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

fn plugin_preference_user_data_from_grpc(value: RpcNoProtoBufPluginPreferenceUserData) -> PluginPreferenceUserData {
    match value {
        RpcNoProtoBufPluginPreferenceUserData::Number { value } => PluginPreferenceUserData::Number { value },
        RpcNoProtoBufPluginPreferenceUserData::String { value } => PluginPreferenceUserData::String { value },
        RpcNoProtoBufPluginPreferenceUserData::Enum { value } => PluginPreferenceUserData::Enum { value },
        RpcNoProtoBufPluginPreferenceUserData::Bool { value } => PluginPreferenceUserData::Bool { value },
        RpcNoProtoBufPluginPreferenceUserData::ListOfStrings { value } => PluginPreferenceUserData::ListOfStrings {
            value,
            new_value: "".to_owned()
        },
        RpcNoProtoBufPluginPreferenceUserData::ListOfNumbers { value } => PluginPreferenceUserData::ListOfNumbers {
            value,
            new_value: 0.0
        },
        RpcNoProtoBufPluginPreferenceUserData::ListOfEnums { value } => PluginPreferenceUserData::ListOfEnums {
            value,
            new_value: None
        },
    }
}

fn plugin_preference_user_data_to_grpc(value: PluginPreferenceUserData) -> RpcNoProtoBufPluginPreferenceUserData {
    match value {
        PluginPreferenceUserData::Number { value } => RpcNoProtoBufPluginPreferenceUserData::Number { value },
        PluginPreferenceUserData::String { value } => RpcNoProtoBufPluginPreferenceUserData::String { value },
        PluginPreferenceUserData::Enum { value } => RpcNoProtoBufPluginPreferenceUserData::Enum { value },
        PluginPreferenceUserData::Bool { value } => RpcNoProtoBufPluginPreferenceUserData::Bool { value },
        PluginPreferenceUserData::ListOfStrings { value, .. } => RpcNoProtoBufPluginPreferenceUserData::ListOfStrings { value },
        PluginPreferenceUserData::ListOfNumbers { value, .. } => RpcNoProtoBufPluginPreferenceUserData::ListOfNumbers { value },
        PluginPreferenceUserData::ListOfEnums { value, .. } => RpcNoProtoBufPluginPreferenceUserData::ListOfEnums { value },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectItem {
    value: String,
    label: String
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}


