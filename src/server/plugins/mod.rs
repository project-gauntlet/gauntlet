use serde::Deserialize;

use crate::common::dbus::{DBusEntrypoint, DBusPlugin};
use crate::common::model::{EntrypointId, PluginId};
use crate::server::dirs::Dirs;
use crate::server::plugins::config_reader::ConfigReader;
use crate::server::plugins::data_db_repository::DataDbRepository;
use crate::server::plugins::downloader::PluginDownloader;
use crate::server::plugins::js::{PluginCode, PluginCommand, PluginCommandData, PluginRuntimeData, start_plugin_runtime};
use crate::server::search::{SearchIndex, SearchItem};

pub mod js;
mod data_db_repository;
mod config_reader;
mod downloader;

pub struct ApplicationManager {
    config_reader: ConfigReader,
    search_index: SearchIndex,
    command_broadcaster: tokio::sync::broadcast::Sender<PluginCommand>,
    db_repository: DataDbRepository,
    plugin_downloader: PluginDownloader,
}

impl ApplicationManager {
    pub async fn create(search_index: SearchIndex) -> anyhow::Result<Self> {
        let dirs = Dirs::new();
        let db_repository = DataDbRepository::new(dirs.clone()).await?;
        let plugin_downloader = PluginDownloader::new(db_repository.clone());
        let config_reader = ConfigReader::new(dirs);

        let (command_broadcaster, _) = tokio::sync::broadcast::channel::<PluginCommand>(100);

        Ok(Self {
            config_reader,
            search_index,
            command_broadcaster,
            db_repository,
            plugin_downloader,
        })
    }

    pub fn start_plugin_download(&mut self, repository_url: &str) -> String {
        unimplemented!()
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

    pub async fn set_plugin_state(&mut self, plugin_id: PluginId, enabled: bool) {
        let x = self.is_plugin_enabled(&plugin_id).await;
        println!("set_plugin_state {:?} {:?}", x, enabled);
        match (x, enabled) {
            (false, true) => {
                self.start_plugin(plugin_id).await;
            }
            (true, false) => {
                self.stop_plugin(plugin_id).await;
            }
            _ => {}
        }
    }

    pub async fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) {
        self.db_repository.set_plugin_entrypoint_enabled(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled)
            .await
            .unwrap();

        self.reload_search_index().await;
    }

    pub async fn reload_all_plugins(&mut self) {
        self.reload_search_index().await;

        self.db_repository.list_plugins()
            .await
            .unwrap()
            .into_iter()
            .filter(|plugin| plugin.enabled)
            .for_each(|plugin| {
                let receiver = self.command_broadcaster.subscribe();

                let data = PluginRuntimeData {
                    id: PluginId::from_string(plugin.id),
                    code: PluginCode { js: plugin.code.js },
                    command_receiver: receiver,
                };

                self.start_plugin_runtime(data)
            });
    }

    async fn is_plugin_enabled(&self, plugin_id: &PluginId) -> bool {
        self.db_repository.is_plugin_enabled(&plugin_id.to_string())
            .await
            .unwrap()
    }

    async fn start_plugin(&mut self, plugin_id: PluginId) {
        println!("plugin_id {:?}", plugin_id);

        self.db_repository.set_plugin_enabled(&plugin_id.to_string(), true)
            .await
            .unwrap();

        let plugin = self.db_repository.get_plugin_by_id(&plugin_id.to_string())
            .await
            .unwrap();

        let receiver = self.command_broadcaster.subscribe();
        let data = PluginRuntimeData {
            id: plugin_id,
            code: PluginCode { js: plugin.code.0.js },
            command_receiver: receiver,
        };

        self.reload_search_index().await;
        self.start_plugin_runtime(data)
    }

    async fn stop_plugin(&mut self, plugin_id: PluginId) {
        println!("stop_plugin {:?}", plugin_id);

        self.db_repository.set_plugin_enabled(&plugin_id.to_string(), false)
            .await
            .unwrap();

        let data = PluginCommand {
            id: plugin_id,
            data: PluginCommandData::Stop,
        };

        self.reload_search_index().await;
        self.send_command(data)
    }

    async fn reload_search_index(&mut self) {
        println!("reload_search_index");

        let search_items: Vec<_> = self.db_repository.list_plugins()
            .await
            .unwrap()
            .into_iter()
            .filter(|plugin| plugin.enabled)
            .flat_map(|plugin| {
                plugin.entrypoints
                    .into_iter()
                    .filter(|entrypoint| entrypoint.enabled)
                    .map(|entrypoint| {
                        SearchItem {
                            entrypoint_name: entrypoint.name.to_owned(),
                            entrypoint_id: entrypoint.id.to_string(),
                            plugin_name: plugin.name.to_owned(),
                            plugin_id: plugin.id.to_string(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        self.search_index.reload(search_items).unwrap();
    }

    fn start_plugin_runtime(&self, data: PluginRuntimeData) {
        let handle = move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local_set = tokio::task::LocalSet::new();
            local_set.block_on(&runtime, tokio::task::unconstrained(async move {
                start_plugin_runtime(data).await
            }))
        };

        std::thread::Builder::new()
            .name("plugin-js-thread".into())
            .spawn(handle)
            .expect("failed to spawn plugin js thread");
    }

    fn send_command(&self, command: PluginCommand) {
        self.command_broadcaster.send(command).unwrap();
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    readonly_ui: Option<bool>,
    // TODO 3 modes: no changes, changes saved to config, changes saved to data
    plugins: Option<Vec<PluginConfig>>,
}

#[derive(Debug, Deserialize)]
struct PluginConfig {
    id: String,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    plugin: PackageJsonPlugin,
}

#[derive(Debug, Deserialize)]
struct PackageJsonPlugin {
    entrypoints: Vec<PackageJsonPluginEntrypoint>,
    metadata: PackageJsonPluginMetadata,
}

#[derive(Debug, Deserialize)]
struct PackageJsonPluginEntrypoint {
    id: String,
    name: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct PackageJsonPluginMetadata {
    name: String,
}
