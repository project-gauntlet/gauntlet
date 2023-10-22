use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::Context;
use deno_core::serde_json;
use serde::Deserialize;
use crate::common::dbus::{DBusEntrypoint, DBusPlugin};

use crate::common::model::{EntrypointId, PluginId};
use crate::server::plugins::js::{PluginCommand, PluginCommandData, PluginContextData, start_js_runtime};
use crate::server::search::{SearchIndex, SearchItem};

pub mod js;

#[derive(Clone)]
pub struct PluginManager {
    inner: Arc<RwLock<PluginManagerInner>>,
}

pub struct PluginManagerInner {
    plugins: HashMap<PluginId, Plugin>,
    search_index: SearchIndex,
    command_broadcaster: tokio::sync::broadcast::Sender<PluginCommand>,
}

impl PluginManager {
    pub fn create(search_index: SearchIndex) -> Self {
        let plugins = PluginLoader.load_plugins()
            .into_iter()
            .map(|plugin| (plugin.id.clone(), plugin))
            .collect();

        let (tx, _) = tokio::sync::broadcast::channel::<PluginCommand>(100);

        Self {
            inner: Arc::new(RwLock::new(PluginManagerInner {
                plugins,
                search_index,
                command_broadcaster: tx
            })),
        }
    }

    pub fn start_plugin_download(&mut self, repository_url: &str) -> String {
        unimplemented!()
    }

    pub fn plugins(&self) -> Vec<DBusPlugin> {
        let plugins = &self.inner.read().unwrap().plugins;

        plugins.iter()
            .map(|(_, plugin)| DBusPlugin {
                plugin_id: plugin.id().to_string(),
                plugin_name: plugin.name().to_owned(),
                enabled: plugin.enabled(),
                entrypoints: plugin.entrypoints()
                    .into_iter()
                    .map(|entrypoint| DBusEntrypoint {
                        enabled: entrypoint.enabled(),
                        entrypoint_id: entrypoint.id().to_string(),
                        entrypoint_name: entrypoint.name().to_owned()
                    })
                    .collect()
            })
            .collect()
    }

    pub fn set_plugin_state(&mut self, plugin_id: PluginId, enabled: bool) {
        let mut inner = self.inner.write().unwrap();
        inner.set_plugin_state(plugin_id, enabled);
    }

    pub fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) {
        let mut inner = self.inner.write().unwrap();
        inner.set_entrypoint_state(plugin_id, entrypoint_id, enabled);
    }

    pub fn reload_all_plugins(&mut self) {
        let mut inner = self.inner.write().unwrap();
        inner.reload_all_plugins();
    }
}

impl PluginManagerInner {
    fn set_plugin_state(&mut self, plugin_id: PluginId, enabled: bool) {
        let x = self.is_plugin_enabled(&plugin_id);
        println!("set_plugin_state {:?} {:?}", x, enabled );
        match (x, enabled) {
            (false, true) => {
                self.start_plugin(plugin_id);
            },
            (true, false) => {
                self.stop_plugin(plugin_id);
            }
            _ => {}
        }
    }

    fn set_entrypoint_state(&mut self, plugin_id: PluginId, entrypoint_id: EntrypointId, enabled: bool) {
        let entrypoint = self.plugins.get_mut(&plugin_id)
            .unwrap()
            .entrypoints_mut()
            .iter_mut()
            .find(|entrypoint| entrypoint.id() == entrypoint_id)
            .unwrap();

        entrypoint.enabled = enabled;

        self.reload_search_index();
    }

    fn reload_all_plugins(&mut self) {
        self.reload_search_index();

        self.plugins
            .iter()
            .filter(|(_, plugin)| plugin.enabled)
            .for_each(|(_, plugin)| {
                let receiver = self.command_broadcaster.subscribe();

                let data = PluginContextData {
                    id: plugin.id(),
                    code: plugin.code().clone(),
                    command_receiver: receiver,
                };

                self.start_plugin_context(data)
            });
    }

    fn is_plugin_enabled(&self, plugin_id: &PluginId) -> bool {
        let plugin = self.plugins.get(plugin_id).unwrap();

        plugin.enabled
    }

    fn start_plugin(&mut self, plugin_id: PluginId) {
        println!("plugin_id {:?}", plugin_id);
        let plugin = self.plugins.get_mut(&plugin_id).unwrap();

        plugin.enabled = true;

        let receiver = self.command_broadcaster.subscribe();
        let data = PluginContextData {
            id: plugin_id.clone(),
            code: plugin.code().clone(),
            command_receiver: receiver,
        };

        self.reload_search_index();
        self.start_plugin_context(data)
    }

    fn stop_plugin(&mut self, plugin_id: PluginId) {
        println!("stop_plugin {:?}", plugin_id);
        let plugin = self.plugins.get_mut(&plugin_id).unwrap();

        plugin.enabled = false;

        let data = PluginCommand {
            id: plugin.id(),
            data: PluginCommandData::Stop,
        };

        self.reload_search_index();
        self.send_command(data)
    }

    fn reload_search_index(&mut self) {
        println!("reload_search_index");

        let search_items: Vec<_> = self.plugins
            .iter()
            .filter(|(_, plugin)| plugin.enabled)
            .flat_map(|(_, plugin)| {
                plugin.entrypoints()
                    .iter()
                    .filter(|entrypoint| entrypoint.enabled)
                    .map(|entrypoint| {
                        SearchItem {
                            entrypoint_name: entrypoint.name().to_owned(),
                            entrypoint_id: entrypoint.id().to_string(),
                            plugin_name: plugin.name().to_owned(),
                            plugin_id: plugin.id().to_string(),
                        }
                    })
            })
            .collect();

        self.search_index.reload(search_items).unwrap();
    }

    fn start_plugin_context(&self, data: PluginContextData) {
        let handle = move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local_set = tokio::task::LocalSet::new();
            local_set.block_on(&runtime, tokio::task::unconstrained(async move {
                start_js_runtime(data).await
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
struct Config {
    readonly_ui: Option<bool>, // TODO 3 modes: no changes, changes saved to config, changes saved to data
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

pub struct PluginLoader;

impl PluginLoader {
    pub fn load_plugins(&self) -> Vec<Plugin> {
        // let project_dirs = ProjectDirs::from("org", "placeholdername", "placeholdername").unwrap();

        // let config_dir = project_dirs.config_dir();
        let config_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_data/xdg_config/placeholdername");

        std::fs::create_dir_all(&config_dir).unwrap();

        let config_file = config_dir.join("config.toml");
        let config_file_path = config_file.display().to_string();
        let config_content = std::fs::read_to_string(config_file).context(config_file_path).unwrap();
        let config: Config = toml::from_str(&config_content).unwrap();

        let plugins: Vec<_> = config.plugins.unwrap()
            .into_iter()
            .map(|plugin| self.fetch_plugin(plugin))
            .collect();

        plugins
    }

    fn fetch_plugin(&self, plugin: PluginConfig) -> Plugin {
        // TODO fetch from git repo
        let plugin_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/plugin");
        let dist_dir = plugin_dir.join("dist");

        let dist_paths = std::fs::read_dir(dist_dir).unwrap();

        let js: HashMap<_, _> = dist_paths.into_iter()
            .map(|dist_path| dist_path.unwrap().path())
            .filter(|dist_path| dist_path.extension() == Some(OsStr::from_bytes(b"js")))
            .map(|dist_path| {
                let js_content = std::fs::read_to_string(&dist_path).unwrap();
                let id = dist_path.file_stem().unwrap().to_str().unwrap().to_owned();

                (id, JsCode::new(js_content))
            })
            .collect();

        let package_path = plugin_dir.join("package.json");
        let package_content = std::fs::read_to_string(package_path).unwrap();
        let package_json: PackageJson = serde_json::from_str(&package_content).unwrap();

        let entrypoints: Vec<_> = package_json.plugin
            .entrypoints
            .into_iter()
            .map(|entrypoint| PluginEntrypoint::new(EntrypointId::new(entrypoint.id), entrypoint.name, entrypoint.path))
            .collect();

        Plugin::new(
            PluginId::new(plugin.id),
            &package_json.plugin.metadata.name,
            true,
            PluginCode::new(js),
            entrypoints
        )
    }
}

pub struct Plugin {
    id: PluginId,
    name: String,
    enabled: bool,
    code: PluginCode,
    entrypoints: Vec<PluginEntrypoint>,
}

impl Plugin {
    fn new(id: PluginId, name: &str, enabled: bool, code: PluginCode, entrypoints: Vec<PluginEntrypoint>) -> Self {
        Self {
            id,
            name: name.into(),
            enabled,
            code,
            entrypoints,
        }
    }

    pub fn id(&self) -> PluginId {
        self.id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn code(&self) -> &PluginCode {
        &self.code
    }

    pub fn entrypoints(&self) -> &Vec<PluginEntrypoint> {
        &self.entrypoints
    }

    pub fn entrypoints_mut(&mut self) -> &mut Vec<PluginEntrypoint> {
        &mut self.entrypoints
    }
}

#[derive(Clone)]
pub struct PluginEntrypoint {
    id: EntrypointId,
    name: String,
    enabled: bool,
    path: String,
}

impl PluginEntrypoint {
    fn new(id: EntrypointId, name: String, path: String) -> Self {
        Self {
            id,
            name,
            enabled: true, // TODO load from config
            path,
        }
    }

    pub fn id(&self) -> EntrypointId {
        self.id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Clone)]
pub struct PluginCode {
    js: HashMap<String, JsCode>,
}

impl PluginCode {
    fn new(js: HashMap<String, JsCode>) -> Self {
        Self {
            js,
        }
    }

    pub fn js(&self) -> &HashMap<String, JsCode> {
        &self.js
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsCode(Arc<str>);

impl JsCode {
    pub fn new(code: impl ToString) -> Self {
        JsCode(code.to_string().into())
    }
}

impl ToString for JsCode {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
