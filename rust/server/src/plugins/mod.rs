use std::collections::HashMap;
use include_dir::{Dir, include_dir};

use common::model::{DownloadStatus, EntrypointId, PluginId, PropertyValue};
use common::rpc::{plugin_preference_user_data_from_npb, plugin_preference_user_data_to_npb, rpc_ui_property_value, RpcEntrypoint, RpcEntrypointTypeSettings, RpcEnumValue, RpcNoProtoBufPluginPreferenceUserData, RpcPlugin, RpcPluginPreference, RpcPluginPreferenceUserData, RpcPluginPreferenceValueType, RpcUiPropertyValue};

use crate::dirs::Dirs;
use crate::model::{from_rpc_to_intermediate_value, UiWidgetId};
use crate::plugins::config_reader::ConfigReader;
use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_from_str, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData};
use crate::plugins::js::{AllPluginCommandData, OnePluginCommandData, PluginCode, PluginCommand, PluginPermissions, PluginRuntimeData, start_plugin_runtime};
use crate::plugins::loader::PluginLoader;
use crate::plugins::run_status::RunStatusHolder;
use crate::search::{SearchIndex, SearchIndexPluginEntrypointType, SearchResultItem};

pub mod js;
mod data_db_repository;
mod config_reader;
mod loader;
mod run_status;
mod download_status;


static BUILTIN_PLUGINS: [(&str, Dir); 2] = [
    ("applications", include_dir!("$CARGO_MANIFEST_DIR/../../bundled_plugins/applications/dist")),
    ("calculator", include_dir!("$CARGO_MANIFEST_DIR/../../bundled_plugins/calculator/dist")),
];

pub struct ApplicationManager {
    config_reader: ConfigReader,
    search_index: SearchIndex,
    command_broadcaster: tokio::sync::broadcast::Sender<PluginCommand>,
    db_repository: DataDbRepository,
    plugin_downloader: PluginLoader,
    run_status_holder: RunStatusHolder,
}

impl ApplicationManager {
    pub async fn create(search_index: SearchIndex) -> anyhow::Result<Self> {
        let dirs = Dirs::new();
        let db_repository = DataDbRepository::new(dirs.clone()).await?;
        let plugin_downloader = PluginLoader::new(db_repository.clone());
        let config_reader = ConfigReader::new(dirs, db_repository.clone());
        let run_status_holder = RunStatusHolder::new();

        let (command_broadcaster, _) = tokio::sync::broadcast::channel::<PluginCommand>(100);

        Ok(Self {
            config_reader,
            search_index,
            command_broadcaster,
            db_repository,
            plugin_downloader,
            run_status_holder
        })
    }

    pub async fn download_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        self.plugin_downloader.download_plugin(plugin_id).await
    }

    pub fn download_status(&self) -> HashMap<String, DownloadStatus> {
        self.plugin_downloader.download_status()
    }

    pub async fn save_local_plugin(
        &self,
        path: &str,
    ) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Saving local plugin at path: {:?}", path);

        let plugin_id = self.plugin_downloader.save_local_plugin(path, true).await?;

        self.reload_plugin(plugin_id).await?;

        Ok(())
    }

    pub async fn load_builtin_plugins(&self) -> anyhow::Result<()> {
        for (id, dir) in &BUILTIN_PLUGINS {
            tracing::info!(target = "plugin", "Saving builtin plugin with id: {:?}", id);

            let plugin_id = self.plugin_downloader.save_builtin_plugin(id, dir).await?;

            self.reload_plugin(plugin_id).await?;
        }

        Ok(())
    }

    pub async fn plugins(&self) -> anyhow::Result<Vec<RpcPlugin>> {
        let result = self.db_repository
            .list_plugins_and_entrypoints()
            .await?
            .into_iter()
            .map(|(plugin, entrypoints)| {
                let entrypoints = entrypoints
                    .into_iter()
                    .map(|entrypoint| RpcEntrypoint {
                        enabled: entrypoint.enabled,
                        entrypoint_id: entrypoint.id,
                        entrypoint_name: entrypoint.name,
                        entrypoint_description: entrypoint.description,
                        entrypoint_type: match db_entrypoint_from_str(&entrypoint.entrypoint_type) {
                            DbPluginEntrypointType::Command => RpcEntrypointTypeSettings::SCommand,
                            DbPluginEntrypointType::View => RpcEntrypointTypeSettings::SView,
                            DbPluginEntrypointType::InlineView => RpcEntrypointTypeSettings::SInlineView,
                            DbPluginEntrypointType::CommandGenerator => RpcEntrypointTypeSettings::SCommandGenerator,
                        }.into(),
                        preferences: entrypoint.preferences.into_iter()
                            .map(|(key, value)| (key, plugin_preference_to_grpc(value)))
                            .collect(),
                        preferences_user_data: entrypoint.preferences_user_data.into_iter()
                            .map(|(key, value)| (key, plugin_preference_user_data_from_npb(plugin_preference_user_data_to_grpc(value))))
                            .collect(),
                    })
                    .collect();

                RpcPlugin {
                    plugin_id: plugin.id,
                    plugin_name: plugin.name,
                    plugin_description: plugin.description,
                    enabled: plugin.enabled,
                    entrypoints,
                    preferences: plugin.preferences.into_iter()
                        .map(|(key, value)| (key, plugin_preference_to_grpc(value)))
                        .collect(),
                    preferences_user_data: plugin.preferences_user_data.into_iter()
                        .map(|(key, value)| (key, plugin_preference_user_data_from_npb(plugin_preference_user_data_to_grpc(value))))
                        .collect(),
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn set_plugin_state(&self, plugin_id: PluginId, set_enabled: bool) -> anyhow::Result<()> {
        let currently_running = self.run_status_holder.is_plugin_running(&plugin_id);
        let currently_enabled = self.is_plugin_enabled(&plugin_id).await?;
        match (currently_running, currently_enabled, set_enabled) {
            (false, false, true) => {
                self.db_repository.set_plugin_enabled(&plugin_id.to_string(), true)
                    .await?;

                self.start_plugin(plugin_id).await?;
            }
            (false, true, true) => {
                self.start_plugin(plugin_id).await?;
            }
            (true, true, false) => {
                self.db_repository.set_plugin_enabled(&plugin_id.to_string(), false)
                    .await?;

                self.stop_plugin(plugin_id.clone()).await;
                self.search_index.remove_for_plugin(plugin_id)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn set_entrypoint_state(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        self.db_repository.set_plugin_entrypoint_enabled(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled)
            .await?;

        self.request_search_index_reload(plugin_id);

        Ok(())
    }

    pub async fn set_preference_value(&self, plugin_id: PluginId, entrypoint_id: Option<EntrypointId>, preference_name: String, preference_value: RpcPluginPreferenceUserData) -> anyhow::Result<()> {
        let user_data = plugin_preference_user_data_from_grpc(plugin_preference_user_data_to_npb(preference_value));

        self.db_repository.set_preference_value(plugin_id.to_string(), entrypoint_id.map(|id| id.to_string()), preference_name, user_data)
            .await?;

        Ok(())
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        self.config_reader.reload_config().await?;

        Ok(())
    }

    pub async fn reload_all_plugins(&mut self) -> anyhow::Result<()> {

        self.reload_config().await?;

        for plugin in self.db_repository.list_plugins().await? {
            let plugin_id = PluginId::from_string(plugin.id);
            let running = self.run_status_holder.is_plugin_running(&plugin_id);
            match (running, plugin.enabled) {
                (false, true) => {
                    self.start_plugin(plugin_id).await?;
                }
                (true, false) => {
                    self.stop_plugin(plugin_id.clone()).await;
                    self.search_index.remove_for_plugin(plugin_id)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn handle_inline_view(&self, text: &str) {
        self.send_command(PluginCommand::All {
            data: AllPluginCommandData::OpenInlineView {
                text: text.to_owned()
            }
        })
    }

    pub fn handle_run_command(&self, plugin_id: PluginId, entrypoint_id: String) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::RunCommand {
                entrypoint_id,
            }
        })
    }

    pub fn handle_run_generated_command(&self, plugin_id: PluginId, entrypoint_id: String) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::RunGeneratedCommand {
                entrypoint_id,
            }
        })
    }

    pub fn handle_render_view(&self, plugin_id: PluginId, frontend: String, entrypoint_id: String) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::RenderView {
                frontend,
                entrypoint_id,
            }
        })
    }

    pub fn handle_view_event(&self, plugin_id: PluginId, widget_id: UiWidgetId, event_name: String, event_arguments: Vec<PropertyValue>) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::HandleViewEvent {
                widget_id,
                event_name,
                event_arguments
            }
        })
    }

    pub fn handle_keyboard_event(&self, plugin_id: PluginId, entrypoint_id: EntrypointId, key: String, modifier_shift: bool, modifier_control: bool, modifier_alt: bool, modifier_meta: bool) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::HandleKeyboardEvent {
                entrypoint_id,
                key,
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta,
            }
        })
    }

    pub fn request_search_index_reload(&self, plugin_id: PluginId) {
        self.send_command(PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::ReloadSearchIndex
        })
    }

    async fn reload_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        let running = self.run_status_holder.is_plugin_running(&plugin_id);
        if running {
            self.stop_plugin(plugin_id.clone()).await;
        }

        self.start_plugin(plugin_id).await?;

        Ok(())
    }

    async fn is_plugin_enabled(&self, plugin_id: &PluginId) -> anyhow::Result<bool> {
        self.db_repository.is_plugin_enabled(&plugin_id.to_string())
            .await
    }

    async fn start_plugin(&self, plugin_id: PluginId) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Starting plugin with id: {:?}", plugin_id);

        let plugin_id_str = plugin_id.to_string();

        let plugin = self.db_repository.get_plugin_by_id(&plugin_id_str)
            .await?;

        let inline_view_entrypoint_id = self.db_repository.get_inline_view_entrypoint_id_for_plugin(&plugin_id_str)
            .await?;

        let receiver = self.command_broadcaster.subscribe();
        let data = PluginRuntimeData {
            id: plugin_id,
            code: PluginCode { js: plugin.code.js },
            inline_view_entrypoint_id,
            permissions: PluginPermissions {
                environment: plugin.permissions.environment,
                high_resolution_time: plugin.permissions.high_resolution_time,
                network: plugin.permissions.network,
                ffi: plugin.permissions.ffi,
                fs_read_access: plugin.permissions.fs_read_access,
                fs_write_access: plugin.permissions.fs_write_access,
                run_subprocess: plugin.permissions.run_subprocess,
                system: plugin.permissions.system
            },
            command_receiver: receiver,
            db_repository: self.db_repository.clone(),
            search_index: self.search_index.clone()
        };

        self.start_plugin_runtime(data);

        Ok(())
    }

    async fn stop_plugin(&self, plugin_id: PluginId) {
        tracing::info!(target = "plugin", "Stopping plugin with id: {:?}", plugin_id);

        let data = PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::Stop,
        };

        self.send_command(data)
    }

    fn start_plugin_runtime(&self, data: PluginRuntimeData) {
        let run_status_guard = self.run_status_holder.start_block(data.id.clone());

        tokio::spawn(async {
            start_plugin_runtime(data, run_status_guard)
                .await
                .expect("failed to start plugin runtime")
        });
    }

    fn send_command(&self, command: PluginCommand) {
        self.command_broadcaster.send(command).expect("all respective receivers were closed");
    }
}

fn plugin_preference_to_grpc(value: DbPluginPreference) -> RpcPluginPreference {
    match value {
        DbPluginPreference::Number { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Number.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::Number(value)) }),
                description,
                ..RpcPluginPreference::default()
            }
        }
        DbPluginPreference::String { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::String.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }),
                description,
                ..RpcPluginPreference::default()
            }
        }
        DbPluginPreference::Enum { default, description, enum_values } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Enum.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }),
                description,
                enum_values: enum_values.into_iter()
                    .map(|value| RpcEnumValue { label: value.label, value: value.value })
                    .collect(),
                ..RpcPluginPreference::default()
            }
        }
        DbPluginPreference::Bool { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::Bool.into(),
                default: default.map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::Bool(value)) }),
                description,
                ..RpcPluginPreference::default()
            }
        }
        DbPluginPreference::ListOfStrings { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfStrings.into(),
                default_list: default.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }).collect()).unwrap_or(vec![]),
                description,
                ..RpcPluginPreference::default()
            }
        }
        DbPluginPreference::ListOfNumbers { default, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfNumbers.into(),
                default_list: default.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::Number(value)) }).collect()).unwrap_or(vec![]),
                description,
                ..RpcPluginPreference::default()
            }
        }
        DbPluginPreference::ListOfEnums { default, enum_values, description } => {
            RpcPluginPreference {
                r#type: RpcPluginPreferenceValueType::ListOfEnums.into(),
                default_list: default.map(|value| value.into_iter().map(|value| RpcUiPropertyValue { value: Some(rpc_ui_property_value::Value::String(value)) }).collect()).unwrap_or(vec![]),
                description,
                enum_values: enum_values.into_iter()
                    .map(|value| RpcEnumValue { label: value.label, value: value.value })
                    .collect(),
                ..RpcPluginPreference::default()
            }
        }
    }
}

fn plugin_preference_user_data_from_grpc(value: RpcNoProtoBufPluginPreferenceUserData) -> DbPluginPreferenceUserData {
    match value {
        RpcNoProtoBufPluginPreferenceUserData::Number { value } => DbPluginPreferenceUserData::Number { value },
        RpcNoProtoBufPluginPreferenceUserData::String { value } => DbPluginPreferenceUserData::String { value },
        RpcNoProtoBufPluginPreferenceUserData::Enum { value } => DbPluginPreferenceUserData::Enum { value },
        RpcNoProtoBufPluginPreferenceUserData::Bool { value } => DbPluginPreferenceUserData::Bool { value },
        RpcNoProtoBufPluginPreferenceUserData::ListOfStrings { value } => DbPluginPreferenceUserData::ListOfStrings { value },
        RpcNoProtoBufPluginPreferenceUserData::ListOfNumbers { value } => DbPluginPreferenceUserData::ListOfNumbers { value },
        RpcNoProtoBufPluginPreferenceUserData::ListOfEnums { value } => DbPluginPreferenceUserData::ListOfEnums { value },
    }
}

fn plugin_preference_user_data_to_grpc(value: DbPluginPreferenceUserData) -> RpcNoProtoBufPluginPreferenceUserData {
    match value {
        DbPluginPreferenceUserData::Number { value } => RpcNoProtoBufPluginPreferenceUserData::Number { value },
        DbPluginPreferenceUserData::String { value } => RpcNoProtoBufPluginPreferenceUserData::String { value },
        DbPluginPreferenceUserData::Enum { value } => RpcNoProtoBufPluginPreferenceUserData::Enum { value },
        DbPluginPreferenceUserData::Bool { value } => RpcNoProtoBufPluginPreferenceUserData::Bool { value },
        DbPluginPreferenceUserData::ListOfStrings { value, .. } => RpcNoProtoBufPluginPreferenceUserData::ListOfStrings { value },
        DbPluginPreferenceUserData::ListOfNumbers { value, .. } => RpcNoProtoBufPluginPreferenceUserData::ListOfNumbers { value },
        DbPluginPreferenceUserData::ListOfEnums { value, .. } => RpcNoProtoBufPluginPreferenceUserData::ListOfEnums { value },
    }
}

