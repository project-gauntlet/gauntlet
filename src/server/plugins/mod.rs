use crate::common::dbus::{DBusEntrypoint, DBusPlugin};
use crate::common::model::{EntrypointId, PluginId};
use crate::server::dirs::Dirs;
use crate::server::plugins::config_reader::ConfigReader;
use crate::server::plugins::data_db_repository::DataDbRepository;
use crate::server::plugins::js::{PluginCode, PluginCommand, PluginCommandData, PluginRuntimeData, start_plugin_runtime};
use crate::server::plugins::loader::PluginLoader;
use crate::server::plugins::run_status::RunStatusHolder;
use crate::server::search::{SearchIndex, SearchItem};

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

    pub async fn new_remote_plugin(
        &mut self,
        signal_context: zbus::SignalContext<'_>,
        plugin_id: PluginId
    ) -> anyhow::Result<()> {
        self.plugin_downloader.add_remote_plugin(signal_context, plugin_id).await
    }

    pub async fn new_local_plugin(
        &mut self,
        plugin_id: PluginId,
    ) -> anyhow::Result<()> {
        self.plugin_downloader.add_local_plugin(plugin_id, false).await
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

    pub async fn set_plugin_state(&mut self, plugin_id: PluginId, set_enabled: bool) -> anyhow::Result<()> {
        let currently_running = self.run_status_holder.is_plugin_running(&plugin_id);
        let currently_enabled = self.is_plugin_enabled(&plugin_id).await;
        println!("set_plugin_state {:?} {:?}", currently_enabled, set_enabled);
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

        self.reload_search_index().await;

        Ok(())
    }

    pub async fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) -> anyhow::Result<()> {
        self.db_repository.set_plugin_entrypoint_enabled(&plugin_id.to_string(), &entrypoint_id.to_string(), enabled)
            .await?;

        self.reload_search_index().await;

        Ok(())
    }

    pub async fn reload_config(&self) -> anyhow::Result<()> {
        self.config_reader.reload_config().await?;

        Ok(())
    }

    pub async fn reload_all_plugins(&mut self) -> anyhow::Result<()> {

        if cfg!(feature = "dev") {
            let plugin_id = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/test_data/plugin/dist").to_owned();

            // ignore any error
            let _ = self.plugin_downloader.add_local_plugin(PluginId::from_string(plugin_id), true)
                .await;
        }

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

        self.reload_search_index().await;

        Ok(())
    }

    async fn is_plugin_enabled(&self, plugin_id: &PluginId) -> bool {
        self.db_repository.is_plugin_enabled(&plugin_id.to_string())
            .await
            .unwrap()
    }

    async fn start_plugin(&mut self, plugin_id: PluginId) -> anyhow::Result<()> {
        println!("plugin_id {:?}", plugin_id);

        let plugin = self.db_repository.get_plugin_by_id(&plugin_id.to_string())
            .await?;

        let receiver = self.command_broadcaster.subscribe();
        let data = PluginRuntimeData {
            id: plugin_id,
            code: PluginCode { js: plugin.code.js },
            command_receiver: receiver,
        };

        self.start_plugin_runtime(data);

        Ok(())
    }

    async fn stop_plugin(&mut self, plugin_id: PluginId) {
        println!("stop_plugin {:?}", plugin_id);

        let data = PluginCommand {
            id: plugin_id,
            data: PluginCommandData::Stop,
        };

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

    fn start_plugin_runtime(&mut self, data: PluginRuntimeData) {
        let run_status_guard = self.run_status_holder.start_block(data.id.clone());

        let handle = move || {
            let _run_status_guard = run_status_guard;
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
