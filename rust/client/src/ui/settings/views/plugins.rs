use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::PluginPreferenceUserData;
use gauntlet_common::model::SettingsEntrypointType;
use gauntlet_common::model::SettingsPlugin;
use gauntlet_server::global_hotkey::GlobalHotKeyManager;
use gauntlet_server::plugins::ApplicationManager;
use gauntlet_utils::channel::RequestResult;
use iced::Alignment;
use iced::Length;
use iced::Padding;
use iced::Task;
use iced::padding;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::text::Shaping;
use iced::widget::text_input;
use iced::widget::vertical_rule;
use iced_fonts::bootstrap::plus;

use crate::ui::settings::theme::Element;
use crate::ui::settings::theme::button::ButtonStyle;
use crate::ui::settings::theme::text::TextStyle;
use crate::ui::settings::ui::SettingsMsg;
use crate::ui::settings::views::plugins::preferences::PluginPreferencesMsg;
use crate::ui::settings::views::plugins::preferences::SelectItem;
use crate::ui::settings::views::plugins::preferences::preferences_ui;
use crate::ui::settings::views::plugins::table::PluginTableMsgIn;
use crate::ui::settings::views::plugins::table::PluginTableMsgOut;
use crate::ui::settings::views::plugins::table::PluginTableState;

mod preferences;
mod table;

#[derive(Debug, Clone)]
pub enum SettingsPluginMsgIn {
    InitSetting {
        global_entrypoint_shortcuts: HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>,
        show_global_shortcuts: bool,
    },
    PluginTableMsg(PluginTableMsgIn),
    PluginPreferenceMsg(PluginPreferencesMsg),
    FetchPlugins,
    PluginsReloaded(
        HashMap<PluginId, SettingsPlugin>,
        HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>,
        HashMap<(PluginId, EntrypointId), String>,
    ),
    RemovePlugin {
        plugin_id: PluginId,
    },
    ToggleShowEntrypoint {
        plugin_id: PluginId,
    },
    ToggleShowGeneratedEntrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    DownloadPlugin {
        plugin_id: PluginId,
    },
    SelectItem(SelectedItem),
}

pub enum SettingsPluginMsgOut {
    Inner(SettingsPluginMsgIn),
    Outer(SettingsMsg),
}

pub struct SettingsPluginsState {
    application_manager: Arc<ApplicationManager>,
    table_state: PluginTableState,
    plugin_data: Rc<RefCell<PluginDataContainer>>,
    preference_user_data: HashMap<(PluginId, Option<EntrypointId>, String), PluginPreferenceUserDataState>,
    selected_item: SelectedItem,
    global_entrypoint_shortcuts: HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>,
    entrypoint_search_aliases: HashMap<(PluginId, EntrypointId), String>,
}

impl SettingsPluginsState {
    pub fn new(application_manager: Arc<ApplicationManager>) -> Self {
        Self {
            application_manager: application_manager.clone(),
            plugin_data: Rc::new(RefCell::new(PluginDataContainer::new())),
            preference_user_data: HashMap::new(),
            selected_item: SelectedItem::None,
            table_state: PluginTableState::new(),
            global_entrypoint_shortcuts: HashMap::new(),
            entrypoint_search_aliases: HashMap::new(),
        }
    }

    pub fn update(
        &mut self,
        global_hotkey_manager: &Option<GlobalHotKeyManager>,
        message: SettingsPluginMsgIn,
    ) -> Task<SettingsPluginMsgOut> {
        let application_manager = self.application_manager.clone();
        match message {
            SettingsPluginMsgIn::InitSetting {
                global_entrypoint_shortcuts,
                show_global_shortcuts,
            } => {
                self.global_entrypoint_shortcuts = global_entrypoint_shortcuts;
                self.table_state.show_global_shortcuts = show_global_shortcuts;

                Task::none()
            }
            SettingsPluginMsgIn::PluginTableMsg(message) => {
                let application_manager = application_manager.clone();
                match self.table_state.update(message) {
                    PluginTableMsgOut::SetPluginState { enabled, plugin_id } => {
                        let application_manager = application_manager.clone();

                        Task::perform(
                            async move {
                                application_manager.set_plugin_state(plugin_id, enabled)?;

                                let plugins = application_manager.plugins()?;
                                let global_entrypoint_shortcuts =
                                    application_manager.get_global_entrypoint_shortcuts()?;
                                let entrypoint_aliases = application_manager.get_entrypoint_search_aliases()?;

                                Ok((plugins, global_entrypoint_shortcuts, entrypoint_aliases))
                            },
                            |result| {
                                handle_backend_error(
                                    result,
                                    |(plugins, global_entrypoint_shortcuts, entrypoint_aliases)| {
                                        SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::PluginsReloaded(
                                            plugins,
                                            global_entrypoint_shortcuts,
                                            entrypoint_aliases,
                                        ))
                                    },
                                )
                            },
                        )
                    }
                    PluginTableMsgOut::SetEntrypointState {
                        enabled,
                        plugin_id,
                        entrypoint_id,
                    } => {
                        let application_manager = application_manager.clone();

                        Task::perform(
                            async move {
                                application_manager.set_entrypoint_state(plugin_id, entrypoint_id, enabled)?;

                                let plugins = application_manager.plugins()?;
                                let global_entrypoint_shortcuts =
                                    application_manager.get_global_entrypoint_shortcuts()?;
                                let entrypoint_aliases = application_manager.get_entrypoint_search_aliases()?;

                                Ok((plugins, global_entrypoint_shortcuts, entrypoint_aliases))
                            },
                            |result| {
                                handle_backend_error(
                                    result,
                                    |(plugins, global_entrypoint_shortcuts, entrypoint_aliases)| {
                                        SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::PluginsReloaded(
                                            plugins,
                                            global_entrypoint_shortcuts,
                                            entrypoint_aliases,
                                        ))
                                    },
                                )
                            },
                        )
                    }
                    PluginTableMsgOut::SelectItem(selected_item) => {
                        Task::done(SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::SelectItem(
                            selected_item,
                        )))
                    }
                    PluginTableMsgOut::ToggleShowEntrypoints { plugin_id } => {
                        Task::done(SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::ToggleShowEntrypoint {
                            plugin_id,
                        }))
                    }
                    PluginTableMsgOut::ToggleShowGeneratedEntrypoints {
                        plugin_id,
                        entrypoint_id,
                    } => {
                        Task::done(SettingsPluginMsgOut::Inner(
                            SettingsPluginMsgIn::ToggleShowGeneratedEntrypoint {
                                plugin_id,
                                entrypoint_id,
                            },
                        ))
                    }
                    PluginTableMsgOut::ShortcutCaptured(plugin_id, entrypoint_id, shortcut) => {
                        let Some(global_hotkey_manager) = &global_hotkey_manager else {
                            return Task::none();
                        };

                        fn run(
                            application_manager: &ApplicationManager,
                            global_hotkey_manager: &GlobalHotKeyManager,
                            plugin_id: PluginId,
                            entrypoint_id: EntrypointId,
                            shortcut: Option<PhysicalShortcut>,
                        ) -> anyhow::Result<SettingsPluginMsgOut> {
                            application_manager.set_global_entrypoint_shortcut(
                                global_hotkey_manager,
                                plugin_id,
                                entrypoint_id,
                                shortcut,
                            )?;

                            let plugins = application_manager.plugins()?;
                            let global_entrypoint_shortcuts = application_manager.get_global_entrypoint_shortcuts()?;
                            let entrypoint_aliases = application_manager.get_entrypoint_search_aliases()?;

                            Ok(SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::PluginsReloaded(
                                plugins,
                                global_entrypoint_shortcuts,
                                entrypoint_aliases,
                            )))
                        }

                        let msg_out = run(
                            &application_manager,
                            global_hotkey_manager,
                            plugin_id,
                            entrypoint_id,
                            shortcut,
                        )
                        .unwrap_or_else(|err| SettingsPluginMsgOut::Outer(SettingsMsg::HandleBackendError(err.into())));

                        Task::done(msg_out)
                    }
                    PluginTableMsgOut::AliasChanged(plugin_id, entrypoint_id, shortcut) => {
                        let application_manager = application_manager.clone();

                        Task::perform(
                            async move {
                                application_manager.set_entrypoint_search_alias(plugin_id, entrypoint_id, shortcut)?;

                                let plugins = application_manager.plugins()?;
                                let global_entrypoint_shortcuts =
                                    application_manager.get_global_entrypoint_shortcuts()?;
                                let entrypoint_aliases = application_manager.get_entrypoint_search_aliases()?;

                                Ok((plugins, global_entrypoint_shortcuts, entrypoint_aliases))
                            },
                            |result| {
                                handle_backend_error(
                                    result,
                                    |(plugins, global_entrypoint_shortcuts, entrypoint_aliases)| {
                                        SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::PluginsReloaded(
                                            plugins,
                                            global_entrypoint_shortcuts,
                                            entrypoint_aliases,
                                        ))
                                    },
                                )
                            },
                        )
                    }
                }
            }
            SettingsPluginMsgIn::ToggleShowEntrypoint { plugin_id } => {
                let plugins = {
                    let mut plugin_data = self.plugin_data.borrow_mut();
                    let settings_plugin_data = plugin_data.plugins_state.get_mut(&plugin_id).unwrap();
                    settings_plugin_data.show_entrypoints = !settings_plugin_data.show_entrypoints;

                    plugin_data.plugins.clone()
                };

                self.apply_plugin_fetch(
                    plugins,
                    self.global_entrypoint_shortcuts.clone(),
                    self.entrypoint_search_aliases.clone(),
                );

                Task::none()
            }
            SettingsPluginMsgIn::ToggleShowGeneratedEntrypoint {
                plugin_id,
                entrypoint_id,
            } => {
                let plugins = {
                    let mut plugin_data = self.plugin_data.borrow_mut();
                    let settings_plugin_data = plugin_data.plugins_state.get_mut(&plugin_id).unwrap();
                    let settings_entrypoint_data = settings_plugin_data
                        .generator_entrypoint_state
                        .get_mut(&entrypoint_id)
                        .unwrap();

                    settings_entrypoint_data.show_entrypoints = !settings_entrypoint_data.show_entrypoints;

                    plugin_data.plugins.clone()
                };

                self.apply_plugin_fetch(
                    plugins,
                    self.global_entrypoint_shortcuts.clone(),
                    self.entrypoint_search_aliases.clone(),
                );

                Task::none()
            }
            SettingsPluginMsgIn::PluginPreferenceMsg(msg) => {
                match msg {
                    PluginPreferencesMsg::UpdatePreferenceValue {
                        plugin_id,
                        entrypoint_id,
                        id,
                        user_data,
                    } => {
                        self.preference_user_data.insert(
                            (plugin_id.clone(), entrypoint_id.clone(), id.clone()),
                            user_data.clone(),
                        );

                        let application_manager = application_manager.clone();

                        Task::perform(
                            async move {
                                application_manager.set_preference_value(
                                    plugin_id,
                                    entrypoint_id,
                                    id,
                                    user_data.to_user_data(),
                                )?;

                                Ok(())
                            },
                            |result| handle_backend_error(result, |()| SettingsPluginMsgOut::Outer(SettingsMsg::Noop)),
                        )
                    }
                }
            }
            SettingsPluginMsgIn::FetchPlugins => {
                let application_manager = self.application_manager.clone();

                Task::perform(
                    async move {
                        let plugins = application_manager.plugins()?;
                        let global_entrypoint_shortcuts = application_manager.get_global_entrypoint_shortcuts()?;
                        let entrypoint_aliases = application_manager.get_entrypoint_search_aliases()?;

                        Ok((plugins, global_entrypoint_shortcuts, entrypoint_aliases))
                    },
                    |result| {
                        handle_backend_error(result, |(plugins, global_entrypoint_shortcuts, entrypoint_aliases)| {
                            SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::PluginsReloaded(
                                plugins,
                                global_entrypoint_shortcuts,
                                entrypoint_aliases,
                            ))
                        })
                    },
                )
            }
            SettingsPluginMsgIn::PluginsReloaded(plugins, shortcuts, entrypoint_aliases) => {
                self.apply_plugin_fetch(plugins, shortcuts, entrypoint_aliases);

                Task::none()
            }
            SettingsPluginMsgIn::RemovePlugin { plugin_id } => {
                self.selected_item = SelectedItem::None;

                let application_manager = application_manager.clone();

                Task::perform(
                    async move {
                        application_manager.remove_plugin(plugin_id)?;

                        let plugins = application_manager.plugins()?;
                        let global_entrypoint_shortcuts = application_manager.get_global_entrypoint_shortcuts()?;
                        let entrypoint_aliases = application_manager.get_entrypoint_search_aliases()?;

                        Ok((plugins, global_entrypoint_shortcuts, entrypoint_aliases))
                    },
                    |result| {
                        handle_backend_error(result, |(plugins, global_entrypoint_shortcuts, entrypoint_aliases)| {
                            SettingsPluginMsgOut::Inner(SettingsPluginMsgIn::PluginsReloaded(
                                plugins,
                                global_entrypoint_shortcuts,
                                entrypoint_aliases,
                            ))
                        })
                    },
                )
            }
            SettingsPluginMsgIn::DownloadPlugin { plugin_id } => {
                Task::done(SettingsPluginMsgOut::Outer(SettingsMsg::DownloadPlugin { plugin_id }))
            }
            SettingsPluginMsgIn::SelectItem(selected_item) => {
                self.selected_item = selected_item;

                Task::none()
            }
        }
    }

    fn apply_plugin_fetch(
        &mut self,
        plugins: HashMap<PluginId, SettingsPlugin>,
        global_entrypoint_shortcuts: HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>,
        entrypoint_search_aliases: HashMap<(PluginId, EntrypointId), String>,
    ) {
        self.global_entrypoint_shortcuts = global_entrypoint_shortcuts.clone();

        self.preference_user_data = plugins
            .iter()
            .map(|(plugin_id, plugin)| {
                let mut result = vec![];

                for (id, user_data) in &plugin.preferences_user_data {
                    result.push((
                        (plugin_id.clone(), None, id.clone()),
                        PluginPreferenceUserDataState::from_user_data(user_data.clone()),
                    ))
                }

                for (entrypoint_id, entrypoint) in &plugin.entrypoints {
                    for (id, user_data) in &entrypoint.preferences_user_data {
                        result.push((
                            (plugin_id.clone(), Some(entrypoint_id.clone()), id.clone()),
                            PluginPreferenceUserDataState::from_user_data(user_data.clone()),
                        ))
                    }
                }

                result
            })
            .flatten()
            .collect();

        let mut plugin_data = self.plugin_data.borrow_mut();

        plugin_data.plugins_state = plugins
            .iter()
            .map(|(id, plugin)| {
                let plugin_data = plugin_data.plugins_state.get(&id);

                let show_entrypoints = plugin_data.map(|data| data.show_entrypoints).unwrap_or(true);

                let mut generator_entrypoint_state_old = plugin_data
                    .map(|data| data.generator_entrypoint_state.clone())
                    .unwrap_or_default();

                let generator_entrypoint_state = plugin
                    .entrypoints
                    .iter()
                    .filter(|(_, entrypoint)| {
                        matches!(entrypoint.entrypoint_type, SettingsEntrypointType::EntrypointGenerator)
                    })
                    .map(|(_, entrypoint)| {
                        let generator_data = generator_entrypoint_state_old
                            .remove(&entrypoint.entrypoint_id)
                            .unwrap_or(SettingsGeneratorData { show_entrypoints: true });

                        (entrypoint.entrypoint_id.clone(), generator_data)
                    })
                    .collect();

                (
                    id.clone(),
                    SettingsPluginData {
                        show_entrypoints,
                        generator_entrypoint_state,
                    },
                )
            })
            .collect();

        plugin_data.plugins = plugins;

        let mut plugin_refs: Vec<_> = plugin_data
            .plugins
            .iter()
            .map(|(_, plugin)| (plugin, plugin_data.plugins_state.get(&plugin.plugin_id).unwrap()))
            .collect();

        plugin_refs.sort_by_key(|(plugin, _)| &plugin.plugin_name);

        self.table_state.apply_plugin_reload(
            self.plugin_data.clone(),
            plugin_refs,
            global_entrypoint_shortcuts,
            entrypoint_search_aliases,
        )
    }

    pub fn view(&self) -> Element<SettingsPluginMsgIn> {
        let table: Element<_> = self
            .table_state
            .view()
            .map(|msg| SettingsPluginMsgIn::PluginTableMsg(msg));

        let table: Element<_> = container(table).padding(Padding::new(8.0)).into();

        let sidebar_content: Element<_> = match &self.selected_item {
            SelectedItem::None => {
                let text1: Element<_> = text("Select item from the list on the left").into();
                let text2: Element<_> = text("or").into();
                let text3: Element<_> = text("Click '+' to add new plugin").into();

                let text_column = column(vec![text1, text2, text3]).align_x(Alignment::Center);

                container(text_column)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
            SelectedItem::Plugin { plugin_id } => {
                let plugin_data = self.plugin_data.borrow();

                let plugin = plugin_data.plugins.get(&plugin_id);

                match plugin {
                    None => {
                        let loading_text: Element<_> = text("Loading...").into();

                        container(loading_text)
                            .align_y(Alignment::Center)
                            .align_x(Alignment::Center)
                            .height(Length::Fill)
                            .width(Length::Fill)
                            .into()
                    }
                    Some(plugin) => {
                        let name = text(plugin.plugin_name.to_string()).shaping(Shaping::Advanced);

                        let name = container(name).padding(Padding::new(8.0)).into();

                        let id: Element<_> = text(plugin.plugin_id.to_string())
                            .shaping(Shaping::Advanced)
                            .class(TextStyle::Subtitle)
                            .into();

                        let id = container(id).padding(padding::all(8.0).top(0)).into();

                        let mut column_content = vec![name, id];

                        if !plugin.plugin_description.is_empty() {
                            let description_label: Element<_> =
                                text("Description").size(14).class(TextStyle::Subtitle).into();

                            let description_label =
                                container(description_label).padding(padding::all(8.0).top(0)).into();

                            let description = text(plugin.plugin_description.to_string()).shaping(Shaping::Advanced);

                            let description = container(description).padding(Padding::new(8.0)).into();

                            let content: Element<_> = column(vec![description_label, description]).into();

                            column_content.push(content);
                        }

                        column_content.push(
                            preferences_ui(plugin_id.clone(), None, &plugin.preferences, &self.preference_user_data)
                                .map(|msg| SettingsPluginMsgIn::PluginPreferenceMsg(msg)),
                        );

                        let content: Element<_> = column(column_content).spacing(12).into();

                        let content: Element<_> = scrollable(content).height(Length::Fill).width(Length::Fill).into();

                        let mut column_content = vec![content];

                        if !plugin.plugin_id.to_string().starts_with("bundled://") {
                            let check_for_updates_text: Element<_> = text("Check for updates").into();

                            let check_for_updates_text_container: Element<_> = container(check_for_updates_text)
                                .width(Length::Fill)
                                .align_y(Alignment::Center)
                                .align_x(Alignment::Center)
                                .into();

                            let check_for_updates_button: Element<_> = button(check_for_updates_text_container)
                                .width(Length::Fill)
                                .class(ButtonStyle::Primary)
                                .on_press(SettingsPluginMsgIn::DownloadPlugin {
                                    plugin_id: plugin.plugin_id.clone(),
                                })
                                .into();

                            column_content.push(check_for_updates_button);

                            let remove_text: Element<_> = text("Remove plugin").into();

                            let remove_button_text_container: Element<_> = container(remove_text)
                                .width(Length::Fill)
                                .align_y(Alignment::Center)
                                .align_x(Alignment::Center)
                                .into();

                            let remove_button: Element<_> = button(remove_button_text_container)
                                .width(Length::Fill)
                                .class(ButtonStyle::Destructive)
                                .on_press(SettingsPluginMsgIn::RemovePlugin {
                                    plugin_id: plugin.plugin_id.clone(),
                                })
                                .into();

                            column_content.push(remove_button);
                        }

                        let content: Element<_> = column(column_content).spacing(8.0).into();

                        container(content).width(Length::Fill).height(Length::Fill).into()
                    }
                }
            }
            SelectedItem::Entrypoint {
                plugin_id,
                entrypoint_id,
            } => {
                let plugin_data = self.plugin_data.borrow();

                let entrypoint = plugin_data
                    .plugins
                    .get(&plugin_id)
                    .map(|plugin| plugin.entrypoints.get(entrypoint_id))
                    .flatten();

                match entrypoint {
                    None => {
                        let loading_text: Element<_> = text("Loading...").into();

                        container(loading_text)
                            .align_y(Alignment::Center)
                            .align_x(Alignment::Center)
                            .height(Length::Fill)
                            .width(Length::Fill)
                            .into()
                    }
                    Some(entrypoint) => {
                        let name = text(entrypoint.entrypoint_name.to_string()).shaping(Shaping::Advanced);

                        let name = container(name).padding(Padding::new(8.0)).into();

                        let mut column_content = vec![name];

                        if !entrypoint.entrypoint_description.is_empty() {
                            let description_label: Element<_> =
                                text("Description").size(14).class(TextStyle::Subtitle).into();

                            let description_label =
                                container(description_label).padding(padding::all(8.0).top(0)).into();

                            let description = container(text(entrypoint.entrypoint_description.to_string()))
                                .padding(Padding::new(8.0))
                                .into();

                            let content: Element<_> = column(vec![description_label, description]).into();

                            column_content.push(content);
                        }

                        column_content.push(
                            preferences_ui(
                                plugin_id.clone(),
                                Some(entrypoint_id.clone()),
                                &entrypoint.preferences,
                                &self.preference_user_data,
                            )
                            .map(|msg| SettingsPluginMsgIn::PluginPreferenceMsg(msg)),
                        );

                        let column: Element<_> = column(column_content).spacing(12).into();

                        let column: Element<_> = scrollable(column).width(Length::Fill).into();

                        container(column)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(Padding::from([4.0, 0.0]))
                            .into()
                    }
                }
            }
            SelectedItem::NewPlugin { repository_url } => {
                let url_input: Element<_> = text_input("Enter Git Repository URL", &repository_url)
                    .on_input(|value| {
                        SettingsPluginMsgIn::SelectItem(SelectedItem::NewPlugin { repository_url: value })
                    })
                    .on_submit(SettingsPluginMsgIn::DownloadPlugin {
                        plugin_id: PluginId::from_string(repository_url),
                    })
                    .into();

                let content: Element<_> = column(vec![
                    url_input,
                    text("Supported protocols:").into(),
                    text("http(s), ssh, git").into(),
                ])
                .into();

                container(content)
                    .padding(Padding::new(8.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .into()
            }
            SelectedItem::GeneratedEntrypoint {
                plugin_id,
                generated_entrypoint_id,
                generator_entrypoint_id,
            } => {
                let plugin_data = self.plugin_data.borrow();

                let entrypoint = plugin_data
                    .plugins
                    .get(&plugin_id)
                    .map(|plugin| plugin.entrypoints.get(generator_entrypoint_id))
                    .flatten()
                    .map(|entrypoint| entrypoint.generated_entrypoints.get(generated_entrypoint_id))
                    .flatten();

                match entrypoint {
                    None => {
                        let loading_text: Element<_> = text("Loading...").into();

                        container(loading_text)
                            .align_y(Alignment::Center)
                            .align_x(Alignment::Center)
                            .height(Length::Fill)
                            .width(Length::Fill)
                            .into()
                    }
                    Some(entrypoint) => {
                        let name: Element<_> = text(entrypoint.entrypoint_name.to_string())
                            .shaping(Shaping::Advanced)
                            .into();

                        let name: Element<_> = container(name).padding(padding::all(8.0)).into();

                        container(name)
                            .padding(Padding::from([4.0, 0.0]))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    }
                }
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
            plus()
        };

        let top_button_text_container: Element<_> = container(top_button_text)
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .into();

        let top_button_action = match plugin_url {
            Some(plugin_url) => {
                SettingsPluginMsgIn::DownloadPlugin {
                    plugin_id: PluginId::from_string(plugin_url),
                }
            }
            None => {
                SettingsPluginMsgIn::SelectItem(SelectedItem::NewPlugin {
                    repository_url: Default::default(),
                })
            }
        };

        let top_button = button(top_button_text_container)
            .width(Length::Fill)
            .on_press(top_button_action)
            .into();

        let sidebar: Element<_> = column(vec![top_button, sidebar_content])
            .padding(Padding::new(8.0))
            .into();

        let separator: Element<_> = vertical_rule(1).into();

        let table = container(table).width(Length::FillPortion(7)).into();
        let sidebar = container(sidebar).width(Length::FillPortion(3)).into();

        let content: Element<_> = row(vec![table, separator, sidebar]).into();

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
    plugins_state: HashMap<PluginId, SettingsPluginData>,
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
        repository_url: String,
    },
    Plugin {
        plugin_id: PluginId,
    },
    Entrypoint {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    GeneratedEntrypoint {
        plugin_id: PluginId,
        generator_entrypoint_id: EntrypointId,
        generated_entrypoint_id: EntrypointId,
    },
}

#[derive(Debug, Clone)]
struct SettingsPluginData {
    show_entrypoints: bool,
    generator_entrypoint_state: HashMap<EntrypointId, SettingsGeneratorData>,
}

#[derive(Debug, Clone)]
struct SettingsGeneratorData {
    show_entrypoints: bool,
}

#[derive(Debug, Clone)]
pub enum PluginPreferenceUserDataState {
    Number {
        value: Option<f64>,
        new_value: Option<String>,
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
        new_value: String,
    },
    ListOfNumbers {
        value: Option<Vec<f64>>,
        new_value: Option<String>,
    },
    ListOfEnums {
        value: Option<Vec<String>>,
        new_value: Option<SelectItem>,
    },
}

impl PluginPreferenceUserDataState {
    pub fn from_user_data(value: PluginPreferenceUserData) -> Self {
        match value {
            PluginPreferenceUserData::Number { value } => {
                PluginPreferenceUserDataState::Number { value, new_value: None }
            }
            PluginPreferenceUserData::String { value } => PluginPreferenceUserDataState::String { value },
            PluginPreferenceUserData::Enum { value } => PluginPreferenceUserDataState::Enum { value },
            PluginPreferenceUserData::Bool { value } => PluginPreferenceUserDataState::Bool { value },
            PluginPreferenceUserData::ListOfStrings { value } => {
                PluginPreferenceUserDataState::ListOfStrings {
                    value,
                    new_value: "".to_owned(),
                }
            }
            PluginPreferenceUserData::ListOfNumbers { value } => {
                PluginPreferenceUserDataState::ListOfNumbers { value, new_value: None }
            }
            PluginPreferenceUserData::ListOfEnums { value } => {
                PluginPreferenceUserDataState::ListOfEnums { value, new_value: None }
            }
        }
    }

    pub fn to_user_data(self) -> PluginPreferenceUserData {
        match self {
            PluginPreferenceUserDataState::Number { value, .. } => PluginPreferenceUserData::Number { value },
            PluginPreferenceUserDataState::String { value } => PluginPreferenceUserData::String { value },
            PluginPreferenceUserDataState::Enum { value } => PluginPreferenceUserData::Enum { value },
            PluginPreferenceUserDataState::Bool { value } => PluginPreferenceUserData::Bool { value },
            PluginPreferenceUserDataState::ListOfStrings { value, .. } => {
                PluginPreferenceUserData::ListOfStrings { value }
            }
            PluginPreferenceUserDataState::ListOfNumbers { value, .. } => {
                PluginPreferenceUserData::ListOfNumbers { value }
            }
            PluginPreferenceUserDataState::ListOfEnums { value, .. } => PluginPreferenceUserData::ListOfEnums { value },
        }
    }
}

pub fn handle_backend_error<T>(
    result: RequestResult<T>,
    convert: impl FnOnce(T) -> SettingsPluginMsgOut,
) -> SettingsPluginMsgOut {
    match result {
        Ok(val) => convert(val),
        Err(err) => SettingsPluginMsgOut::Outer(SettingsMsg::HandleBackendError(err)),
    }
}
