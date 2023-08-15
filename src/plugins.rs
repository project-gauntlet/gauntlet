use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use deno_core::anyhow::Context;
use deno_core::futures::task::AtomicWaker;
use deno_core::serde_json;
use directories::ProjectDirs;
use serde::Deserialize;
use crate::gtk::PluginUiContext;
use crate::react_side::{PluginReactContext, UiEvent, UiRequest};

#[derive(Clone)]
pub struct PluginManager {
    inner: Rc<RefCell<PluginManagerInner>>,
}

pub struct PluginManagerInner {
    plugins: Vec<Plugin>,
    ui_contexts: HashMap<String, PluginUiContext>
}

impl PluginManager {

    pub fn create() -> Self {
        let plugins = PluginLoader.load_plugins();

        Self {
            inner: Rc::new(RefCell::new(PluginManagerInner {
                plugins,
                ui_contexts: HashMap::new(),
            }))
        }
    }

    pub fn plugins(&self) -> Ref<'_, Vec<Plugin>> {
        Ref::map(self.inner.borrow(), |inn| &inn.plugins)
    }

    pub fn ui_context(&mut self, plugin_id: &str) -> Option<PluginUiContext> {
        self.inner
            .borrow_mut()
            .ui_contexts
            .get_mut(plugin_id)
            .map(|context| context.clone())
    }

    pub fn create_all_contexts(&mut self) -> (Vec<PluginReactContext>, Vec<PluginUiContext>) {
        let (react_contexts, ui_contexts): (Vec<_>, Vec<_>) = self.inner
            .borrow()
            .plugins
            .iter()
            .map(|plugin| self.create_contexts_for_plugin(plugin.clone()))
            .unzip();

        self.inner.borrow_mut().ui_contexts = ui_contexts.iter()
            .map(|context| (context.plugin().id().to_owned(), context.clone()))
            .collect::<HashMap<_, _>>();

        (react_contexts, ui_contexts)
    }

    fn create_contexts_for_plugin(&self, plugin: Plugin) -> (PluginReactContext, PluginUiContext) {
        let (react_request_sender, react_request_receiver) = tokio::sync::mpsc::unbounded_channel::<UiRequest>();
        let react_request_receiver = Rc::new(RefCell::new(react_request_receiver));

        let (react_event_sender, react_event_receiver) = std::sync::mpsc::channel::<UiEvent>();
        let event_waker = Arc::new(AtomicWaker::new());

        let ui_context = PluginUiContext::new(plugin.clone(), react_request_receiver, react_event_sender, event_waker.clone());
        let react_context = PluginReactContext::new(plugin.clone(), react_event_receiver, event_waker, react_request_sender);

        (react_context, ui_context)
    }

}

#[derive(Debug, Deserialize)]
struct Config {
    readonly_ui: Option<bool>,
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

                (id, js_content)
            })
            .collect();

        let package_path = plugin_dir.join("package.json");
        let package_content = std::fs::read_to_string(package_path).unwrap();
        let package_json: PackageJson = serde_json::from_str(&package_content).unwrap();

        let entrypoints: Vec<_> = package_json.plugin
            .entrypoints
            .into_iter()
            .map(|entrypoint| PluginEntrypoint::new(entrypoint.id, entrypoint.name, entrypoint.path))
            .collect();

        Plugin::new(&plugin.id, &package_json.plugin.metadata.name, PluginCode::new(js, None), entrypoints)
    }

}

#[derive(Clone)]
pub struct Plugin {
    inner: Arc<PluginInner>
}

pub struct PluginInner {
    id: String,
    name: String,
    code: PluginCode,
    entrypoints: Vec<PluginEntrypoint>
}

impl Plugin {
    fn new(id: &str, name: &str, code: PluginCode, entrypoints: Vec<PluginEntrypoint>) -> Self {
        Self {
            inner: Arc::new(PluginInner {
                id: id.into(),
                name: name.into(),
                code,
                entrypoints,
            })
        }
    }

    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn code(&self) -> &PluginCode {
        &self.inner.code
    }

    pub fn entrypoints(&self) -> &Vec<PluginEntrypoint> {
        &self.inner.entrypoints
    }
}

#[derive(Clone)]
pub struct PluginEntrypoint {
    id: String,
    name: String,
    path: String,
}

impl PluginEntrypoint {
    fn new(id: String, name: String, path: String) -> Self {
        Self {
            id,
            name,
            path,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Clone)]
pub struct PluginCode {
    js: HashMap<String, String>,
    css: Option<String>,
}

impl PluginCode {
    fn new(js: HashMap<String, String>, css: Option<String>) -> Self {
        Self {
            js,
            css,
        }
    }

    pub fn js(&self) -> &HashMap<String, String> {
        &self.js
    }

    pub fn css(&self) -> &Option<String> {
        &self.css
    }
}