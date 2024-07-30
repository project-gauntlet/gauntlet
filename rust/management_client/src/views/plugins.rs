use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use iced::{Alignment, Command, Length, Padding};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, text_input, vertical_rule};
use iced_aw::core::icons;

use common::{settings_env_data_from_string, SettingsEnvData};
use common::model::{EntrypointId, PluginId, PluginPreferenceUserData, SettingsPlugin};
use common::rpc::backend_api::{BackendApi, BackendApiError};

use crate::theme::button::ButtonStyle;
use crate::theme::Element;
use crate::theme::text::TextStyle;
use crate::views::plugins::preferences::{PluginPreferencesMsg, preferences_ui, SelectItem};
use crate::views::plugins::table::{PluginTableMsgIn, PluginTableMsgOut, PluginTableState, PluginTableUpdateResult};

mod preferences;
mod table;

#[derive(Debug, Clone)]
pub enum ManagementAppPluginMsgIn {
    PluginTableMsg(PluginTableMsgIn),
    PluginPreferenceMsg(PluginPreferencesMsg),
    RequestPluginReload,
    PluginsReloaded(HashMap<PluginId, SettingsPlugin>),
    AddPlugin {
        plugin_id: PluginId,
    },
    RemovePlugin {
        plugin_id: PluginId
    },
    CheckDownloadStatus,
    DownloadStatus {
        plugins: Vec<PluginId>,
    },
    SelectItem(SelectedItem),
    Noop
}

pub enum ManagementAppPluginMsgOut {
    PluginsReloaded(HashMap<PluginId, SettingsPlugin>),
    DownloadStatus {
        plugins: Vec<PluginId>,
    },
    SelectedItem(SelectedItem),
    HandleBackendError(BackendApiError),
    Noop
}

pub struct ManagementAppPluginsState {
    backend_api: Option<BackendApi>,
    table_state: PluginTableState,
    plugin_data: Rc<RefCell<PluginDataContainer>>,
    preference_user_data: HashMap<(PluginId, Option<EntrypointId>, String), PluginPreferenceUserDataState>,
    selected_item: SelectedItem,
    running_downloads: HashSet<PluginId>,
}

const SETTINGS_ENV: &'static str = "GAUNTLET_INTERNAL_SETTINGS";

impl ManagementAppPluginsState {
    pub fn new(backend_api: Option<BackendApi>) -> Self {
        let settings_env_data = std::env::var(SETTINGS_ENV)
            .ok()
            .filter(|value| !value.is_empty())
            .map(|val| settings_env_data_from_string(val));

        let select_item = match settings_env_data {
            None => SelectedItem::None,
            Some(SettingsEnvData::OpenEntrypointPreferences { plugin_id, entrypoint_id }) => SelectedItem::Entrypoint {
                plugin_id: PluginId::from_string(plugin_id),
                entrypoint_id: EntrypointId::from_string(entrypoint_id),
            },
            Some(SettingsEnvData::OpenPluginPreferences { plugin_id }) => SelectedItem::Plugin {
                plugin_id: PluginId::from_string(plugin_id),
            },
        };

        Self {
            backend_api,
            plugin_data: Rc::new(RefCell::new(PluginDataContainer::new())),
            preference_user_data: HashMap::new(),
            selected_item: select_item,
            table_state: PluginTableState::new(),
            running_downloads: HashSet::new(),
        }
    }

    pub fn update(&mut self, message: ManagementAppPluginMsgIn) -> Command<ManagementAppPluginMsgOut> {
        let backend_api = match &self.backend_api {
            Some(backend_api) => backend_api.clone(),
            None => {
                return Command::none()
            }
        };

        match message {
            ManagementAppPluginMsgIn::PluginTableMsg(message) => {
                match self.table_state.update(message) {
                    PluginTableUpdateResult::Command(command) => command.map(|_| ManagementAppPluginMsgOut::Noop),
                    PluginTableUpdateResult::Value(msg) => {
                        match msg {
                            PluginTableMsgOut::SetPluginState { enabled, plugin_id } => {
                                let mut backend_client = backend_api.clone();

                                Command::perform(
                                    async move {
                                        backend_client.set_plugin_state(plugin_id, enabled)
                                            .await?;

                                        let plugins = backend_client.plugins()
                                            .await?;

                                        Ok(plugins)
                                    },
                                    |result| handle_backend_error(result, |plugins| ManagementAppPluginMsgOut::PluginsReloaded(plugins))
                                )
                            }
                            PluginTableMsgOut::SetEntrypointState { enabled, plugin_id, entrypoint_id } => {
                                let mut backend_client = backend_api.clone();

                                Command::perform(
                                    async move {
                                        backend_client.set_entrypoint_state(plugin_id, entrypoint_id, enabled)
                                            .await?;

                                        let plugins = backend_client.plugins()
                                            .await?;

                                        Ok(plugins)
                                    },
                                    |result| handle_backend_error(result, |plugins| ManagementAppPluginMsgOut::PluginsReloaded(plugins))
                                )
                            }
                            PluginTableMsgOut::SelectItem(selected_item) => {
                                Command::perform(async move { selected_item }, ManagementAppPluginMsgOut::SelectedItem)
                            }
                            PluginTableMsgOut::ToggleShowEntrypoints { plugin_id } => {
                                let plugins = {
                                    let mut plugin_data = self.plugin_data.borrow_mut();
                                    let settings_plugin_data = plugin_data.plugins_state.get_mut(&plugin_id).unwrap();
                                    settings_plugin_data.show_entrypoints = !settings_plugin_data.show_entrypoints;

                                    plugin_data.plugins.clone()
                                };

                                self.apply_plugin_reload(plugins);

                                Command::none()
                            }
                        }
                    }
                }
            }
            ManagementAppPluginMsgIn::PluginPreferenceMsg(msg) => {
                match msg {
                    PluginPreferencesMsg::UpdatePreferenceValue { plugin_id, entrypoint_id, name, user_data } => {
                        self.preference_user_data
                            .insert((plugin_id.clone(), entrypoint_id.clone(), name.clone()), user_data.clone());

                        let mut backend_api = backend_api.clone();

                        Command::perform(
                            async move {
                                backend_api.set_preference_value(plugin_id, entrypoint_id, name, user_data.to_user_data())
                                    .await?;

                                Ok(())
                            },
                            |result| handle_backend_error(result, |()| ManagementAppPluginMsgOut::Noop)
                        )
                    }
                }
            }
            ManagementAppPluginMsgIn::RequestPluginReload => {
                let mut backend_api = backend_api.clone();

                Command::perform(
                    async move {
                        let plugins = backend_api.plugins()
                            .await?;

                        Ok(plugins)
                    },
                    |result| handle_backend_error(result, |plugins| ManagementAppPluginMsgOut::PluginsReloaded(plugins))
                )
            }
            ManagementAppPluginMsgIn::PluginsReloaded(plugins) => {
                self.apply_plugin_reload(plugins);

                Command::none()
            }
            ManagementAppPluginMsgIn::AddPlugin { plugin_id } => {
                let mut backend_client = backend_api.clone();

                let exists = self.running_downloads.insert(plugin_id.clone());
                if !exists {
                    panic!("already downloading this plugins") // TODO proper error handling
                }

                Command::perform(
                    async move {
                        backend_client.download_plugin(plugin_id)
                            .await?;

                        Ok(())
                    },
                    |result| handle_backend_error(result, |()| ManagementAppPluginMsgOut::Noop)
                )
            }
            ManagementAppPluginMsgIn::RemovePlugin { plugin_id } => {
                self.selected_item = SelectedItem::None;

                let mut backend_client = backend_api.clone();

                Command::perform(
                    async move {
                        backend_client.remove_plugin(plugin_id)
                            .await?;

                        let plugins = backend_client.plugins()
                            .await?;

                        Ok(plugins)
                    },
                    |result| handle_backend_error(result, |plugins| ManagementAppPluginMsgOut::PluginsReloaded(plugins))
                )
            }
            ManagementAppPluginMsgIn::CheckDownloadStatus => {
                if self.running_downloads.is_empty() {
                    Command::none()
                } else {
                    let mut backend_client = backend_api.clone();

                    Command::perform(
                        async move {
                            let plugins = backend_client.download_status()
                                .await?;

                            Ok(plugins)
                        },
                        |result| handle_backend_error(result, |plugins| ManagementAppPluginMsgOut::DownloadStatus { plugins }),
                    )
                }
            }
            ManagementAppPluginMsgIn::DownloadStatus { plugins } => {
                for plugin in plugins {
                    self.running_downloads.remove(&plugin);
                }

                let mut backend_api = backend_api.clone();

                Command::perform(
                    async move {
                        let plugins = backend_api.plugins()
                            .await?;

                        Ok(plugins)
                    },
                    |result| handle_backend_error(result, |plugins| ManagementAppPluginMsgOut::PluginsReloaded(plugins))
                )
            }
            ManagementAppPluginMsgIn::SelectItem(selected_item) => {
                self.selected_item = selected_item;

                Command::none()
            }
            ManagementAppPluginMsgIn::Noop => {
                Command::none()
            }
        }
    }

    fn apply_plugin_reload(&mut self, plugins: HashMap<PluginId, SettingsPlugin>) {
        self.preference_user_data = plugins.iter()
            .map(|(plugin_id, plugin)| {
                let mut result = vec![];

                for (name, user_data) in &plugin.preferences_user_data {
                    result.push(((plugin_id.clone(), None, name.clone()), PluginPreferenceUserDataState::from_user_data(user_data.clone())))
                }

                for (entrypoint_id, entrypoint) in &plugin.entrypoints {
                    for (name, user_data) in &entrypoint.preferences_user_data {
                        result.push(((plugin_id.clone(), Some(entrypoint_id.clone()), name.clone()), PluginPreferenceUserDataState::from_user_data(user_data.clone())))
                    }
                }

                result
            })
            .flatten()
            .collect();

        let mut plugin_data = self.plugin_data.borrow_mut();

        plugin_data.plugins_state = plugins.iter()
            .map(|(id, plugin)| {
                let show_entrypoints = plugin_data.plugins_state
                    .get(&id)
                    .map(|data| data.show_entrypoints)
                    .unwrap_or(true);

                (id.clone(), SettingsPluginData { show_entrypoints })
            })
            .collect();

        plugin_data.plugins = plugins;

        let mut plugin_refs: Vec<_> = plugin_data.plugins
            .iter()
            .map(|(_, plugin)| (plugin, plugin_data.plugins_state.get(&plugin.plugin_id).unwrap()))
            .collect();

        plugin_refs.sort_by_key(|(plugin, _)| &plugin.plugin_name);
        
        self.table_state.apply_plugin_reload(self.plugin_data.clone(), plugin_refs)
    }

    pub fn view(&self) -> Element<ManagementAppPluginMsgIn> {
        let table: Element<_> = self.table_state.view()
            .map(|msg| ManagementAppPluginMsgIn::PluginTableMsg(msg));

        let table: Element<_> = container(table)
            .padding(Padding::new(8.0))
            .into();

        let sidebar_content: Element<_> = match &self.selected_item {
            SelectedItem::None => {
                let text1: Element<_> = text("Select item from the list on the left").into();
                let text2: Element<_> = text("or").into();
                let text3: Element<_> = text("Click '+' to add new plugin").into();

                let text_column = column(vec![text1, text2, text3])
                    .align_items(Alignment::Center);

                container(text_column)
                    .center_y()
                    .center_x()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
            SelectedItem::Plugin { plugin_id } => {
                let plugin_data = self.plugin_data.borrow();
                let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

                let name = container(text(&plugin.plugin_name))
                    .padding(Padding::new(8.0))
                    .into();

                let mut column_content = vec![
                    name,
                ];

                if !plugin.plugin_description.is_empty() {
                    let description_label: Element<_> = text("Description")
                        .size(14)
                        .style(TextStyle::Subtitle)
                        .into();

                    let description_label = container(description_label)
                        .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                        .into();

                    let description = container(text(&plugin.plugin_description))
                        .padding(Padding::new(8.0))
                        .into();

                    column_content.push(description_label);
                    column_content.push(description);
                }

                column_content.push(
                    preferences_ui(plugin_id.clone(), None, &plugin.preferences, &self.preference_user_data)
                        .map(|msg| ManagementAppPluginMsgIn::PluginPreferenceMsg(msg))
                );

                if !plugin.plugin_id.to_string().starts_with("builtin://") {
                    let remove_text: Element<_> = text("Remove plugin")
                        .into();

                    let remove_button_text_container: Element<_> = container(remove_text)
                        .width(Length::Fill)
                        .center_y()
                        .center_x()
                        .into();

                    let remove_button: Element<_> = button(remove_button_text_container)
                        .width(Length::Fill)
                        .style(ButtonStyle::Destructive)
                        .on_press(ManagementAppPluginMsgIn::RemovePlugin { plugin_id: plugin.plugin_id.clone() })
                        .into();

                    column_content.push(remove_button);
                }

                let column: Element<_> = column(column_content)
                    .padding(Padding::from([0.0, 4.0, 0.0, 0.0]))
                    .into();

                let column: Element<_> = scrollable(column)
                    .width(Length::Fill)
                    .into();

                container(column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([4.0, 0.0]))
                    .into()
            }
            SelectedItem::Entrypoint { plugin_id, entrypoint_id } => {
                let plugin_data = self.plugin_data.borrow();
                let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
                let entrypoint = plugin.entrypoints.get(entrypoint_id).unwrap();

                let name = container(text(&entrypoint.entrypoint_name))
                    .padding(Padding::new(8.0))
                    .into();

                let mut column_content = vec![
                    name,
                ];

                if !entrypoint.entrypoint_description.is_empty() {
                    let description_label: Element<_> = text("Description")
                        .size(14)
                        .style(TextStyle::Subtitle)
                        .into();

                    let description_label = container(description_label)
                        .padding(Padding::from([0.0, 0.0, 0.0, 8.0]))
                        .into();

                    let description = container(text(&entrypoint.entrypoint_description))
                        .padding(Padding::new(8.0))
                        .into();

                    column_content.push(description_label);
                    column_content.push(description);
                }

                column_content.push(
                    preferences_ui(plugin_id.clone(), Some(entrypoint_id.clone()), &entrypoint.preferences, &self.preference_user_data)
                        .map(|msg| ManagementAppPluginMsgIn::PluginPreferenceMsg(msg))
                );

                let column: Element<_> = column(column_content)
                    .padding(Padding::from([0.0, 4.0, 0.0, 0.0]))
                    .into();

                let column: Element<_> = scrollable(column)
                    .width(Length::Fill)
                    .into();

                container(column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(Padding::from([4.0, 0.0]))
                    .into()
            }
            SelectedItem::NewPlugin { repository_url } => {
                let url_input: Element<_> = text_input("Enter Git Repository URL", &repository_url)
                    .on_input(|value| ManagementAppPluginMsgIn::SelectItem(SelectedItem::NewPlugin { repository_url: value }))
                    .on_submit(ManagementAppPluginMsgIn::AddPlugin { plugin_id: PluginId::from_string(repository_url) })
                    .into();

                let content: Element<_> = column(vec![
                    url_input,
                    text("Supported protocols:").into(),
                    text("file, http(s), ssh").into(),
                ]).into();

                container(content)
                    .padding(Padding::new(8.0))
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
            text(icons::Bootstrap::Plus)
                .font(icons::BOOTSTRAP_FONT)
        };

        let top_button_text_container: Element<_> = container(top_button_text)
            .width(Length::Fill)
            .center_y()
            .center_x()
            .into();

        let top_button_action = match plugin_url {
            Some(plugin_url) => ManagementAppPluginMsgIn::AddPlugin { plugin_id: PluginId::from_string(plugin_url) },
            None => ManagementAppPluginMsgIn::SelectItem(SelectedItem::NewPlugin { repository_url: Default::default() })
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
            .padding(Padding::new(8.0))
            .into();

        let separator: Element<_> = vertical_rule(1)
            .into();

        let content: Element<_> = row(vec![table, separator, sidebar])
            .into();

        let content = container(content)
            .padding(Padding::new(4.0))
            .height(Length::Fill)
            .width(Length::Fill)
            .into();
        
        content
    }
}

#[derive(Debug, Clone)]
struct PluginDataContainer {
    plugins: HashMap<PluginId, SettingsPlugin>,
    plugins_state: HashMap<PluginId, SettingsPluginData>
}

impl PluginDataContainer {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_state: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SelectedItem {
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
struct SettingsPluginData {
    show_entrypoints: bool,
}


#[derive(Debug, Clone)]
pub enum PluginPreferenceUserDataState {
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

impl PluginPreferenceUserDataState {
    pub fn from_user_data(value: PluginPreferenceUserData) -> Self {
        match value {
            PluginPreferenceUserData::Number { value } => PluginPreferenceUserDataState::Number { value },
            PluginPreferenceUserData::String { value } => PluginPreferenceUserDataState::String { value },
            PluginPreferenceUserData::Enum { value } => PluginPreferenceUserDataState::Enum { value },
            PluginPreferenceUserData::Bool { value } => PluginPreferenceUserDataState::Bool { value },
            PluginPreferenceUserData::ListOfStrings { value } => PluginPreferenceUserDataState::ListOfStrings {
                value,
                new_value: "".to_owned()
            },
            PluginPreferenceUserData::ListOfNumbers { value } => PluginPreferenceUserDataState::ListOfNumbers {
                value,
                new_value: 0.0
            },
            PluginPreferenceUserData::ListOfEnums { value } => PluginPreferenceUserDataState::ListOfEnums {
                value,
                new_value: None
            },
        }
    }

    pub fn to_user_data(self) -> PluginPreferenceUserData {
        match self {
            PluginPreferenceUserDataState::Number { value } => PluginPreferenceUserData::Number { value },
            PluginPreferenceUserDataState::String { value } => PluginPreferenceUserData::String { value },
            PluginPreferenceUserDataState::Enum { value } => PluginPreferenceUserData::Enum { value },
            PluginPreferenceUserDataState::Bool { value } => PluginPreferenceUserData::Bool { value },
            PluginPreferenceUserDataState::ListOfStrings { value, .. } => PluginPreferenceUserData::ListOfStrings { value },
            PluginPreferenceUserDataState::ListOfNumbers { value, .. } => PluginPreferenceUserData::ListOfNumbers { value },
            PluginPreferenceUserDataState::ListOfEnums { value, .. } => PluginPreferenceUserData::ListOfEnums { value },
        }
    }
}

pub fn handle_backend_error<T>(result: Result<T, BackendApiError>, convert: impl FnOnce(T) -> ManagementAppPluginMsgOut) -> ManagementAppPluginMsgOut {
    match result {
        Ok(val) => convert(val),
        Err(err) => ManagementAppPluginMsgOut::HandleBackendError(err)
    }
}
