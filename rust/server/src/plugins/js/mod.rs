use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, Context};
use deno_core::futures::executor::block_on;
use deno_core::futures::{FutureExt, Stream, StreamExt};
use deno_core::v8::{GetPropertyNamesArgs, KeyConversionMode, PropertyFilter};
use deno_core::{futures, op, serde_v8, v8, FastString, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleType, OpState, ResolutionKind, StaticModuleLoader};
use deno_runtime::BootstrapOptions;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::deno_io::{Stdio, StdioPipe};
use deno_runtime::permissions::{Descriptor, EnvDescriptor, NetDescriptor, Permissions, PermissionsContainer, PermissionsOptions, ReadDescriptor, UnaryPermission, WriteDescriptor};
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;

use common::dirs::Dirs;
use common::model::{EntrypointId, PhysicalKey, PluginId, SearchResultEntrypointType, UiPropertyValue, UiRenderLocation, UiWidget, UiWidgetId};
use common::rpc::frontend_api::FrontendApi;
use component_model::{create_component_model, Children, Component, Property, PropertyType, SharedType};

use crate::model::{IntermediateUiEvent, JsUiEvent, JsUiPropertyValue, JsUiRenderLocation, JsUiRequestData, JsUiResponseData, JsUiWidget, PreferenceUserData};
use crate::plugins::applications::{get_apps, DesktopEntry};
use crate::plugins::data_db_repository::{db_entrypoint_from_str, DataDbRepository, DbPluginClipboardPermissions, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::icon_cache::IconCache;
use crate::plugins::js::assets::{asset_data, asset_data_blocking};
use crate::plugins::js::clipboard::{clipboard_clear, clipboard_read, clipboard_read_text, clipboard_write, clipboard_write_text};
use crate::plugins::js::command_generators::get_command_generator_entrypoint_ids;
use crate::plugins::js::logs::{op_log_debug, op_log_error, op_log_info, op_log_trace, op_log_warn};
use crate::plugins::js::permissions::{permissions_to_deno, PluginPermissions, PluginPermissionsClipboard};
use crate::plugins::js::plugins::applications::{list_applications, open_application};
use crate::plugins::js::plugins::numbat::{run_numbat, NumbatContext};
use crate::plugins::js::plugins::settings::open_settings;
use crate::plugins::js::preferences::{entrypoint_preferences_required, get_entrypoint_preferences, get_plugin_preferences, plugin_preferences_required};
use crate::plugins::js::search::reload_search_index;
use crate::plugins::js::ui::{clear_inline_view, fetch_action_id_for_shortcut, op_component_model, op_inline_view_endpoint_id, op_react_replace_view, show_plugin_error_view, show_preferences_required_view};
use crate::plugins::run_status::RunStatusGuard;
use crate::search::{SearchIndex, SearchIndexItem};

mod ui;
mod plugins;
mod logs;
mod assets;
mod preferences;
mod search;
mod command_generators;
mod clipboard;
pub mod permissions;

pub struct PluginRuntimeData {
    pub id: PluginId,
    pub uuid: String,
    pub code: PluginCode,
    pub inline_view_entrypoint_id: Option<String>,
    pub permissions: PluginPermissions,
    pub command_receiver: tokio::sync::broadcast::Receiver<PluginCommand>,
    pub db_repository: DataDbRepository,
    pub search_index: SearchIndex,
    pub icon_cache: IconCache,
    pub frontend_api: FrontendApi,
    pub dirs: Dirs,
}

pub struct PluginCode {
    pub js: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct PluginRuntimePermissions {
    pub clipboard: Vec<PluginPermissionsClipboard>,
}

#[derive(Clone, Debug)]
pub enum PluginCommand {
    One {
        id: PluginId,
        data: OnePluginCommandData,
    },
    All {
        data: AllPluginCommandData,
    }
}

#[derive(Clone, Debug)]
pub enum OnePluginCommandData {
    RenderView {
        entrypoint_id: EntrypointId,
    },
    CloseView,
    RunCommand {
        entrypoint_id: String,
    },
    RunGeneratedCommand {
        entrypoint_id: String,
    },
    HandleViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    HandleKeyboardEvent {
        entrypoint_id: EntrypointId,
        key: PhysicalKey,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    },
    ReloadSearchIndex,
    RefreshSearchIndex,
}

#[derive(Clone, Debug)]
pub enum AllPluginCommandData {
    OpenInlineView {
        text: String
    }
}

pub async fn start_plugin_runtime(data: PluginRuntimeData, run_status_guard: RunStatusGuard) -> anyhow::Result<()> {
    let component_model = create_component_model();

    let mut command_receiver = data.command_receiver;
    let command_stream = async_stream::stream! {
        loop {
            yield command_receiver.recv().await.unwrap();
        }
    };

    let plugin_id = data.id.clone();
    let event_stream = command_stream
        .filter_map(move |command: PluginCommand| {
            let plugin_id = plugin_id.clone();

            let event = match command {
                PluginCommand::One { id, data } => {
                    if id != plugin_id {
                        None
                    } else {
                        match data {
                            OnePluginCommandData::RenderView { entrypoint_id } => {
                                Some(IntermediateUiEvent::OpenView {
                                    entrypoint_id,
                                })
                            }
                            OnePluginCommandData::CloseView => {
                                Some(IntermediateUiEvent::CloseView)
                            }
                            OnePluginCommandData::RunCommand { entrypoint_id } => {
                                Some(IntermediateUiEvent::RunCommand {
                                    entrypoint_id,
                                })
                            }
                            OnePluginCommandData::RunGeneratedCommand { entrypoint_id } => {
                                Some(IntermediateUiEvent::RunGeneratedCommand {
                                    entrypoint_id,
                                })
                            }
                            OnePluginCommandData::HandleViewEvent { widget_id, event_name, event_arguments } => {
                                Some(IntermediateUiEvent::HandleViewEvent {
                                    widget_id,
                                    event_name,
                                    event_arguments,
                                })
                            }
                            OnePluginCommandData::HandleKeyboardEvent { entrypoint_id, key, modifier_shift, modifier_control, modifier_alt, modifier_meta } => {
                                Some(IntermediateUiEvent::HandleKeyboardEvent {
                                    entrypoint_id,
                                    key,
                                    modifier_shift,
                                    modifier_control,
                                    modifier_alt,
                                    modifier_meta
                                })
                            }
                            OnePluginCommandData::ReloadSearchIndex => {
                                Some(IntermediateUiEvent::ReloadSearchIndex)
                            }
                            OnePluginCommandData::RefreshSearchIndex => {
                                Some(IntermediateUiEvent::RefreshSearchIndex)
                            }
                        }
                    }
                }
                PluginCommand::All { data } => {
                    match data {
                        AllPluginCommandData::OpenInlineView { text } => {
                            Some(IntermediateUiEvent::OpenInlineView { text })
                        }
                    }
                }
            };

            async move {
                event
            }
        });

    let event_stream = Box::pin(event_stream);

    let cache = data.icon_cache.clone();
    let plugin_uuid = data.uuid.clone();
    let plugin_id = data.id.clone();

    let thread_fn = move || {
        let plugin_id = data.id.clone();

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("unable to start tokio runtime for plugin")
            .block_on({
                let plugin_id = data.id.clone();

                async move {
                    tokio::select! {
                        _ = run_status_guard.stopped() => {
                            tracing::info!(target = "plugin", "Plugin runtime has been stopped {:?}", plugin_id)
                        }
                        result @ _ = {
                            tokio::task::unconstrained(async move {
                                 start_js_runtime(
                                     data.id.clone(),
                                     data.uuid,
                                     data.code,
                                     data.permissions,
                                     data.inline_view_entrypoint_id,
                                     event_stream,
                                     data.frontend_api,
                                     component_model,
                                     data.db_repository,
                                     data.search_index,
                                     data.icon_cache,
                                     data.dirs
                                 ).await
                            })
                        } => {
                            if let Err(err) = result {
                                tracing::error!(target = "plugin", "Plugin runtime has failed {:?} - {:?}", plugin_id, err)
                            } else {
                                tracing::error!(target = "plugin", "Plugin runtime has stopped unexpectedly {:?}", plugin_id)
                            }
                        }
                    }
                }
            });

        if let Err(err) = cache.clear_plugin_icon_cache_dir(&plugin_uuid) {
            tracing::error!(target = "plugin", "plugin {:?} unable to cleanup icon cache {:?}", plugin_id, err)
        }
    };

    std::thread::Builder::new()
        .name("plugin-js-thread".into())
        .spawn(thread_fn)
        .expect("failed to spawn plugin js thread");

    Ok(())
}

async fn start_js_runtime(
    plugin_id: PluginId,
    plugin_uuid: String,
    code: PluginCode,
    permissions: PluginPermissions,
    inline_view_entrypoint_id: Option<String>,
    event_stream: Pin<Box<dyn Stream<Item=IntermediateUiEvent>>>,
    frontend_api: FrontendApi,
    component_model: Vec<Component>,
    repository: DataDbRepository,
    search_index: SearchIndex,
    icon_cache: IconCache,
    dirs: Dirs,
) -> anyhow::Result<()> {

    let dev_plugin = plugin_id.to_string().starts_with("file://");

    let (stdout, stderr) = if dev_plugin {
        let (out_log_file, err_log_file) = dirs.plugin_log_files(&plugin_uuid);

        std::fs::create_dir_all(out_log_file.parent().unwrap())?;
        std::fs::create_dir_all(err_log_file.parent().unwrap())?;

        let out_log_file = File::create(out_log_file)?;
        let err_log_file = File::create(err_log_file)?;

        (StdioPipe::File(out_log_file), StdioPipe::File(err_log_file))
    } else {
        (StdioPipe::Inherit, StdioPipe::Inherit)
    };

    let local_storage_dir = dirs.plugin_local_storage(&plugin_uuid);

    // let _inspector_server = Arc::new(
    //     InspectorServer::new(
    //         "127.0.0.1:9229".parse::<SocketAddr>().unwrap(),
    //         "test",
    //     )
    // );

    let core_url = "gauntlet:core".parse().expect("should be valid");
    let unused_url = "gauntlet:unused".parse().expect("should be valid");

    let numbat_context = if plugin_id.to_string() == "builtin://calculator" {
        Some(NumbatContext::new())
    } else {
        None
    };

    let permissions_container = permissions_to_deno(&permissions, &dirs, &plugin_uuid)?;

    let runtime_permissions = PluginRuntimePermissions {
        clipboard: permissions.clipboard,
    };

    let mut worker = MainWorker::bootstrap_from_options(
        unused_url,
        permissions_container,
        WorkerOptions {
            bootstrap: BootstrapOptions {
                is_tty: false,
                ..Default::default()
            },
            module_loader: Rc::new(CustomModuleLoader::new(code, dev_plugin)),
            extensions: vec![plugin_ext::init_ops_and_esm(
                EventReceiver::new(event_stream),
                PluginData::new(plugin_id, plugin_uuid, inline_view_entrypoint_id, runtime_permissions),
                frontend_api,
                ComponentModel::new(component_model),
                repository,
                search_index,
                icon_cache,
                numbat_context
            )],
            // maybe_inspector_server: Some(inspector_server.clone()),
            // should_wait_for_inspector_session: true,
            // should_break_on_first_statement: true,
            maybe_inspector_server: None,
            should_wait_for_inspector_session: false,
            should_break_on_first_statement: false,
            origin_storage_dir: Some(local_storage_dir),
            stdio: Stdio {
                stdin: StdioPipe::Inherit,
                stdout,
                stderr,
            },
            ..Default::default()
        },
    );

    worker.execute_side_module(&core_url).await?;
    worker.run_event_loop(false).await?;

    Ok(())
}

pub struct CustomModuleLoader {
    code: PluginCode,
    static_loader: StaticModuleLoader,
    dev_plugin: bool,
}

impl CustomModuleLoader {
    fn new(code: PluginCode, dev_plugin: bool) -> Self {
        let module_map: HashMap<_, _> = MODULES.iter()
            .map(|(key, value)| (key.parse().expect("provided key is not valid url"), FastString::from_static(value)))
            .collect();
        Self {
            code,
            static_loader: StaticModuleLoader::new(module_map),
            dev_plugin
        }
    }
}

const MODULES: [(&str, &str); 10] = [
    ("gauntlet:renderer:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react_renderer/dist/prod/renderer.js"))),
    ("gauntlet:renderer:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react_renderer/dist/dev/renderer.js"))),
    ("gauntlet:react:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/prod/react.production.min.js"))),
    ("gauntlet:react:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/dev/react.development.js"))),
    ("gauntlet:react-jsx-runtime:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/prod/react-jsx-runtime.production.min.js"))),
    ("gauntlet:react-jsx-runtime:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/dev/react-jsx-runtime.development.js"))),
    ("gauntlet:core", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/core/dist/init.js"))),
    ("gauntlet:api-components", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/api/dist/gen/components.js"))),
    ("gauntlet:api-hooks", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/api/dist/hooks.js"))),
    ("gauntlet:api-helpers", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/api/dist/helpers.js"))),
];

impl ModuleLoader for CustomModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, anyhow::Error> {
        static PLUGIN_ENTRYPOINT_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gauntlet:entrypoint\?(?<entrypoint_id>[a-zA-Z0-9_-]+)$").expect("invalid regex"));
        static PLUGIN_MODULE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gauntlet:module\?(?<entrypoint_id>[a-zA-Z0-9_-]+)$").expect("invalid regex"));
        static PATH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\./(?<js_module>[a-zA-Z0-9_-]+)\.js$").expect("invalid regex"));

        if PLUGIN_ENTRYPOINT_PATTERN.is_match(specifier) {
            return Ok(specifier.parse()?);
        }

        if PLUGIN_ENTRYPOINT_PATTERN.is_match(referrer) || PLUGIN_MODULE_PATTERN.is_match(referrer) {
            if let Some(captures) = PATH_PATTERN.captures(specifier) {
                return Ok(format!("gauntlet:module?{}", &captures["js_module"]).parse()?);
            }
        }

        let prod_react = cfg!(feature = "release") && !self.dev_plugin;

        let specifier = match (specifier, referrer) {
            ("gauntlet:core", _) => "gauntlet:core",
            ("gauntlet:renderer", _) => if prod_react { "gauntlet:renderer:prod" } else { "gauntlet:renderer:dev" },
            ("react", _) => if prod_react { "gauntlet:react:prod" } else { "gauntlet:react:dev" },
            ("react/jsx-runtime", _) => if prod_react { "gauntlet:react-jsx-runtime:prod" } else { "gauntlet:react-jsx-runtime:dev" },
            ("@project-gauntlet/api/components", _) => "gauntlet:api-components",
            ("@project-gauntlet/api/hooks", _) => "gauntlet:api-hooks",
            ("@project-gauntlet/api/helpers", _) => "gauntlet:api-helpers",
            _ => {
                return Err(anyhow!("Could not resolve module with specifier: {} and referrer: {}", specifier, referrer));
            }
        };

        self.static_loader.resolve(specifier, referrer, kind)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleSpecifier>,
        is_dynamic: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let mut specifier = module_specifier.clone();
        specifier.set_query(None);

        if &specifier == &"gauntlet:entrypoint".parse().unwrap() || &specifier == &"gauntlet:module".parse().unwrap() {
            let module = get_js_code(module_specifier, &self.code.js);

            return futures::future::ready(module).boxed_local();
        }

        self.static_loader.load(module_specifier, maybe_referrer, is_dynamic)
    }
}

fn get_js_code(module_specifier: &ModuleSpecifier, js: &HashMap<String, String>) -> anyhow::Result<ModuleSource> {
    let entrypoint_id = module_specifier.query().expect("invalid specifier, should be validated earlier");

    let js = js.get(entrypoint_id).ok_or(anyhow!("no code provided for view: {:?}", entrypoint_id))?;

    let module = ModuleSource::new(ModuleType::JavaScript, js.clone().into(), module_specifier);

    Ok(module)
}


deno_core::extension!(
    plugin_ext,
    ops = [
        // core
        op_plugin_get_pending_event,

        // logs
        op_log_trace,
        op_log_debug,
        op_log_info,
        op_log_warn,
        op_log_error,

        // command generators
        get_command_generator_entrypoint_ids,

        // assets
        asset_data,
        asset_data_blocking,

        // ui
        op_react_replace_view,
        op_inline_view_endpoint_id,
        show_plugin_error_view,
        clear_inline_view,
        show_preferences_required_view,
        op_component_model,
        fetch_action_id_for_shortcut,

        // preferences
        get_plugin_preferences,
        get_entrypoint_preferences,
        plugin_preferences_required,
        entrypoint_preferences_required,

        // search
        reload_search_index,

        // clipboard
        clipboard_read_text,
        clipboard_read,
        clipboard_write,
        clipboard_write_text,
        clipboard_clear,

        // plugins numbat
        run_numbat,

        // plugins applications
        list_applications,
        open_application,

        // plugins settings
        open_settings,
    ],
    options = {
        event_receiver: EventReceiver,
        plugin_data: PluginData,
        frontend_api: FrontendApi,
        component_model: ComponentModel,
        db_repository: DataDbRepository,
        search_index: SearchIndex,
        icon_cache: IconCache,
        numbat_context: Option<NumbatContext>,
    },
    state = |state, options| {
        state.put(options.event_receiver);
        state.put(options.plugin_data);
        state.put(options.frontend_api);
        state.put(options.component_model);
        state.put(options.db_repository);
        state.put(options.search_index);
        state.put(options.icon_cache);
        state.put(options.numbat_context);
    },
);

#[op]
async fn op_plugin_get_pending_event(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsUiEvent> {
    let event_stream = {
        state.borrow()
            .borrow::<EventReceiver>()
            .event_stream
            .clone()
    };

    let mut event_stream = event_stream.borrow_mut();
    let event = event_stream.next()
        .await
        .ok_or_else(|| anyhow!("event stream was suddenly closed"))?;

    tracing::trace!(target = "renderer_rs", "Received plugin event {:?}", event);

    Ok(from_intermediate_to_js_event(event))
}

fn make_request(state: &Rc<RefCell<OpState>>, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    let (plugin_id, mut frontend_api) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let frontend_api = state
            .borrow::<FrontendApi>()
            .clone();

        (plugin_id, frontend_api)
    };

    block_on(async {
        make_request_async(plugin_id, &mut frontend_api, data).await
    })
}

async fn make_request_async(plugin_id: PluginId, frontend_api: &mut FrontendApi, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    match data {
        JsUiRequestData::ReplaceView { render_location, top_level_view, container, entrypoint_id } => {
            let render_location = match render_location { // TODO into?
                JsUiRenderLocation::InlineView => UiRenderLocation::InlineView,
                JsUiRenderLocation::View => UiRenderLocation::View,
            };

            frontend_api.replace_view(plugin_id, entrypoint_id, render_location, top_level_view, container).await?;

            Ok(JsUiResponseData::Nothing)
        }
        JsUiRequestData::ClearInlineView => {

            frontend_api.clear_inline_view(plugin_id).await?;

            Ok(JsUiResponseData::Nothing)
        }
        JsUiRequestData::ShowPreferenceRequiredView { plugin_preferences_required, entrypoint_preferences_required, entrypoint_id } => {

            frontend_api.show_preference_required_view(plugin_id, entrypoint_id, plugin_preferences_required, entrypoint_preferences_required).await?;

            Ok(JsUiResponseData::Nothing)
        }
        JsUiRequestData::ShowPluginErrorView { entrypoint_id, render_location } => {
            let render_location = match render_location { // TODO into?
                JsUiRenderLocation::InlineView => UiRenderLocation::InlineView,
                JsUiRenderLocation::View => UiRenderLocation::View,
            };

            frontend_api.show_plugin_error_view(plugin_id, entrypoint_id, render_location).await?;

            Ok(JsUiResponseData::Nothing)
        }
    }
}

fn from_intermediate_to_js_event(event: IntermediateUiEvent) -> JsUiEvent {
    match event {
        IntermediateUiEvent::OpenView { entrypoint_id } => JsUiEvent::OpenView {
            entrypoint_id: entrypoint_id.to_string(),
        },
        IntermediateUiEvent::CloseView => JsUiEvent::CloseView,
        IntermediateUiEvent::RunCommand { entrypoint_id } => JsUiEvent::RunCommand {
            entrypoint_id
        },
        IntermediateUiEvent::RunGeneratedCommand { entrypoint_id } => JsUiEvent::RunGeneratedCommand {
            entrypoint_id
        },
        IntermediateUiEvent::HandleViewEvent { widget_id, event_name, event_arguments } => {
            let event_arguments = event_arguments.into_iter()
                .map(|arg| match arg {
                    UiPropertyValue::String(value) => JsUiPropertyValue::String { value },
                    UiPropertyValue::Number(value) => JsUiPropertyValue::Number { value },
                    UiPropertyValue::Bool(value) => JsUiPropertyValue::Bool { value },
                    UiPropertyValue::Undefined => JsUiPropertyValue::Undefined,
                    UiPropertyValue::Bytes(_) | UiPropertyValue::Object(_)  => {
                        todo!()
                    }
                })
                .collect();

            JsUiEvent::ViewEvent {
                widget_id,
                event_name,
                event_arguments,
            }
        }
        IntermediateUiEvent::HandleKeyboardEvent { entrypoint_id, key, modifier_shift, modifier_control, modifier_alt, modifier_meta } => {
            JsUiEvent::KeyboardEvent {
                entrypoint_id: entrypoint_id.to_string(),
                key: key.to_value(),
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta
            }
        }
        IntermediateUiEvent::OpenInlineView { text } => JsUiEvent::OpenInlineView { text },
        IntermediateUiEvent::ReloadSearchIndex => JsUiEvent::ReloadSearchIndex,
        IntermediateUiEvent::RefreshSearchIndex => JsUiEvent::RefreshSearchIndex,
    }
}

pub struct PluginData {
    plugin_id: PluginId,
    plugin_uuid: String,
    inline_view_entrypoint_id: Option<String>,
    permissions: PluginRuntimePermissions
}

impl PluginData {
    fn new(plugin_id: PluginId, plugin_uuid: String, inline_view_entrypoint_id: Option<String>, permissions: PluginRuntimePermissions) -> Self {
        Self {
            plugin_id,
            plugin_uuid,
            inline_view_entrypoint_id,
            permissions
        }
    }

    fn plugin_id(&self) -> PluginId {
        self.plugin_id.clone()
    }

    fn plugin_uuid(&self) -> &str {
        &self.plugin_uuid
    }

    fn inline_view_entrypoint_id(&self) -> Option<String> {
        self.inline_view_entrypoint_id.clone()
    }

    fn permissions(&self) -> &PluginRuntimePermissions {
        &self.permissions
    }
}

pub struct ComponentModel {
    components: HashMap<String, Component>,
}

impl ComponentModel {
    fn new(components: Vec<Component>) -> Self {
        Self {
            components: components.into_iter()
                .filter_map(|component| {
                    match &component {
                        Component::Standard { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                        Component::Root { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                        Component::TextPart { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                    }
                })
                .collect()
        }
    }
}

pub struct EventReceiver {
    event_stream: Rc<RefCell<Pin<Box<dyn Stream<Item=IntermediateUiEvent>>>>>,
}

impl EventReceiver {
    fn new(event_stream: Pin<Box<dyn Stream<Item=IntermediateUiEvent>>>) -> EventReceiver {
        Self {
            event_stream: Rc::new(RefCell::new(event_stream)),
        }
    }
}
