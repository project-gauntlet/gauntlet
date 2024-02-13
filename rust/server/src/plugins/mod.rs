use common::dbus::{DBusEntrypoint, DBusEntrypointType, DBusPlugin};
use common::model::{EntrypointId, PluginId};

use crate::dirs::Dirs;
use crate::model::{entrypoint_from_str, PluginEntrypointType};
use crate::plugins::config_reader::ConfigReader;
use crate::plugins::data_db_repository::DataDbRepository;
use crate::plugins::js::{PluginCode, PluginCommand, OnePluginCommandData, PluginPermissions, PluginRuntimeData, start_plugin_runtime, AllPluginCommandData};
use crate::plugins::loader::PluginLoader;
use crate::plugins::run_status::RunStatusHolder;
use crate::search::{SearchIndex, SearchItem};

pub mod js;
mod data_db_repository;
mod config_reader;
mod loader;
mod run_status;

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

    pub async fn download_and_save_plugin(
        &mut self,
        signal_context: zbus::SignalContext<'_>,
        plugin_id: PluginId
    ) -> anyhow::Result<()> {
        self.plugin_downloader.download_and_save_plugin(signal_context, plugin_id).await
    }

    pub async fn save_local_plugin(
        &mut self,
        path: &str,
    ) -> anyhow::Result<()> {
        tracing::info!(target = "plugin", "Saving local plugin at path: {:?}", path);

        let plugin_id = self.plugin_downloader.save_local_plugin(path, true).await?;

        self.reload_plugin(plugin_id).await?;

        Ok(())
    }

    pub async fn plugins(&self) -> anyhow::Result<Vec<DBusPlugin>> {
        let plugins = self.db_repository.list_plugins().await?;

        let result = plugins
            .into_iter()
            .map(|plugin| {
                let entrypoints = plugin.entrypoints
                    .into_iter()
                    .map(|entrypoint| DBusEntrypoint {
                        enabled: entrypoint.enabled,
                        entrypoint_id: entrypoint.id,
                        entrypoint_name: entrypoint.name,
                        entrypoint_type: match entrypoint_from_str(&entrypoint.entrypoint_type) {
                            PluginEntrypointType::Command => DBusEntrypointType::Command,
                            PluginEntrypointType::View => DBusEntrypointType::View,
                            PluginEntrypointType::InlineView => DBusEntrypointType::InlineView
                        }
                    })
                    .collect();

                DBusPlugin {
                    plugin_id: plugin.id,
                    plugin_name: plugin.name,
                    enabled: plugin.enabled,
                    entrypoints,
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn set_plugin_state(&mut self, plugin_id: PluginId, set_enabled: bool) -> anyhow::Result<()> {
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

                self.stop_plugin(plugin_id).await;
            }
            _ => {}
        }

        self.reload_search_index().await?;

        Ok(())
    }

    pub async fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        self.db_repository.set_plugin_entrypoint_enabled(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled)
            .await?;

        self.reload_search_index().await?;

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
                    self.stop_plugin(plugin_id).await;
                }
                _ => {}
            }
        }

        self.reload_search_index().await?;

        Ok(())
    }

    pub fn handle_inline_view(&self, text: &str) {
        self.send_command(PluginCommand::All {
            data: AllPluginCommandData::OpenInlineView {
                text: text.to_owned()
            }
        })
    }

    async fn reload_plugin(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        let running = self.run_status_holder.is_plugin_running(&plugin_id);
        if running {
            self.stop_plugin(plugin_id.clone()).await;
        }

        self.start_plugin(plugin_id).await?;

        self.reload_search_index().await?;

        Ok(())
    }

    async fn is_plugin_enabled(&self, plugin_id: &PluginId) -> anyhow::Result<bool> {
        self.db_repository.is_plugin_enabled(&plugin_id.to_string())
            .await
    }

    async fn start_plugin(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
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
        };

        self.start_plugin_runtime(data);

        Ok(())
    }

    async fn stop_plugin(&mut self, plugin_id: PluginId) {
        tracing::info!(target = "plugin", "Stopping plugin with id: {:?}", plugin_id);

        let data = PluginCommand::One {
            id: plugin_id,
            data: OnePluginCommandData::Stop,
        };

        self.send_command(data)
    }

    async fn reload_search_index(&mut self) -> anyhow::Result<()> {
        tracing::info!("Reloading search index");

        let search_items: Vec<_> = self.db_repository.list_plugins()
            .await?
            .into_iter()
            .filter(|plugin| plugin.enabled)
            .flat_map(|plugin| {
                plugin.entrypoints
                    .into_iter()
                    .filter(|entrypoint| entrypoint.enabled)
                    .map(|entrypoint| {
                        SearchItem {
                            entrypoint_type: entrypoint_from_str(&entrypoint.entrypoint_type),
                            entrypoint_name: entrypoint.name.to_owned(),
                            entrypoint_id: entrypoint.id.to_string(),
                            plugin_name: plugin.name.to_owned(),
                            plugin_id: plugin.id.to_string(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        self.search_index.reload(search_items)?;

        Ok(())
    }

    fn start_plugin_runtime(&mut self, data: PluginRuntimeData) {
        let run_status_guard = self.run_status_holder.start_block(data.id.clone());

        tokio::spawn(async {
            start_plugin_runtime(data, run_status_guard).await
        });
    }

    fn send_command(&self, command: PluginCommand) {
        self.command_broadcaster.send(command).expect("all respective receivers were closed");
    }
}
