use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use deno_core::{FastString, futures, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleType, op, OpState, ResolutionKind, serde_v8, StaticModuleLoader, v8};
use deno_core::futures::{FutureExt, Stream, StreamExt};
use deno_core::futures::executor::block_on;
use deno_core::v8::{GetPropertyNamesArgs, KeyConversionMode, PropertyFilter};
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::permissions::{Permissions, PermissionsContainer, PermissionsOptions};
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use indexmap::IndexMap;
use numbat::InterpreterResult;
use numbat::markup::Formatter;
use numbat::module_importer::BuiltinModuleImporter;
use numbat::pretty_print::PrettyPrint;
use numbat::value::Value;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use common::model::{EntrypointId, PluginId, UiRenderLocation, SearchIndexPluginEntrypointType, UiPropertyValue, UiWidget, UiWidgetId};
use common::rpc::frontend_api::FrontendApi;
use common::rpc::frontend_server::wait_for_frontend_server;
use component_model::{Children, Component, create_component_model, Property, PropertyType, SharedType};

use crate::model::{IntermediateUiEvent, JsUiPropertyValue, JsUiRenderLocation, JsUiEvent, JsUiRequestData, JsUiResponseData, JsUiWidget, PreferenceUserData};
use crate::plugins::applications::{DesktopEntry, get_apps};
use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_from_str, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::icon_cache::IconCache;
use crate::plugins::run_status::RunStatusGuard;
use crate::search::{SearchIndex, SearchIndexItem};

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
}

pub struct PluginCode {
    pub js: HashMap<String, String>,
}

pub struct PluginPermissions {
    pub environment: Vec<String>,
    pub high_resolution_time: bool,
    pub network: Vec<String>,
    pub ffi: Vec<PathBuf>,
    pub fs_read_access: Vec<PathBuf>,
    pub fs_write_access: Vec<PathBuf>,
    pub run_subprocess: Vec<String>,
    pub system: Vec<String>,
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
    Stop,
    RenderView {
        entrypoint_id: EntrypointId,
    },
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
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    },
    ReloadSearchIndex,
}

#[derive(Clone, Debug)]
pub enum AllPluginCommandData {
    OpenInlineView {
        text: String
    }
}

pub async fn start_plugin_runtime(data: PluginRuntimeData, run_status_guard: RunStatusGuard) -> anyhow::Result<()> {
    wait_for_frontend_server().await;

    let frontend_api = FrontendApi::new().await?;

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
                            OnePluginCommandData::Stop => {
                                Some(IntermediateUiEvent::StopPlugin)
                            }
                            OnePluginCommandData::RenderView { entrypoint_id } => {
                                Some(IntermediateUiEvent::OpenView {
                                    entrypoint_id,
                                })
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

    let thread_fn = move || {
        let _run_status_guard = run_status_guard;

        let result_plugin_id = data.id.clone();
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("unable to start tokio runtime for plugin")
            .block_on(tokio::task::unconstrained(async move {
                let result_plugin_id = data.id.clone();
                let result = start_js_runtime(
                    data.id,
                    data.uuid.clone(),
                    data.code,
                    data.permissions,
                    data.inline_view_entrypoint_id,
                    event_stream,
                    frontend_api,
                    component_model,
                    data.db_repository,
                    data.search_index,
                    data.icon_cache.clone(),
                ).await;

                if let Err(err) = data.icon_cache.clear_plugin_icon_cache_dir(&data.uuid) {
                    tracing::error!(target = "plugin", "plugin {:?} unable to cleanup icon cache {:?}", result_plugin_id, err)
                }

                result
            }));

        if let Err(err) = result {
            tracing::error!(target = "plugin", "Plugin runtime failed {:?} - {:?}", result_plugin_id, err)
        } else {
            tracing::info!(target = "plugin", "Plugin runtime stopped {:?}", result_plugin_id)
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
) -> anyhow::Result<()> {
    let permissions_container = PermissionsContainer::new(Permissions::from_options(&PermissionsOptions {
        allow_env: if permissions.environment.is_empty() { None } else { Some(permissions.environment) },
        deny_env: None,
        allow_hrtime: permissions.high_resolution_time,
        deny_hrtime: false,
        allow_net: if permissions.network.is_empty() { None } else { Some(permissions.network) },
        deny_net: None,
        allow_ffi: if permissions.ffi.is_empty() { None } else { Some(permissions.ffi) },
        deny_ffi: None,
        allow_read: if permissions.fs_read_access.is_empty() { None } else { Some(permissions.fs_read_access) },
        deny_read: None,
        allow_run: if permissions.run_subprocess.is_empty() { None } else { Some(permissions.run_subprocess) },
        deny_run: None,
        allow_sys: if permissions.system.is_empty() { None } else { Some(permissions.system) },
        deny_sys: None,
        allow_write: if permissions.fs_write_access.is_empty() { None } else { Some(permissions.fs_write_access) },
        deny_write: None,
        prompt: false,
    })?);

    // let _inspector_server = Arc::new(
    //     InspectorServer::new(
    //         "127.0.0.1:9229".parse::<SocketAddr>().unwrap(),
    //         "test",
    //     )
    // );

    let core_url = "gauntlet:core".parse().expect("should be valid");
    let unused_url = "gauntlet:unused".parse().expect("should be valid");

    let mut worker = MainWorker::bootstrap_from_options(
        unused_url,
        permissions_container,
        WorkerOptions {
            module_loader: Rc::new(CustomModuleLoader::new(code)),
            extensions: vec![plugin_ext::init_ops_and_esm(
                EventReceiver::new(event_stream),
                PluginData::new(plugin_id, plugin_uuid, inline_view_entrypoint_id),
                frontend_api,
                ComponentModel::new(component_model),
                repository,
                search_index,
                icon_cache,
            )],
            // maybe_inspector_server: Some(inspector_server.clone()),
            // should_wait_for_inspector_session: true,
            // should_break_on_first_statement: true,
            maybe_inspector_server: None,
            should_wait_for_inspector_session: false,
            should_break_on_first_statement: false,
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
}

impl CustomModuleLoader {
    fn new(code: PluginCode) -> Self {
        let module_map: HashMap<_, _> = MODULES.iter()
            .map(|(key, value)| (key.parse().expect("provided key is not valid url"), FastString::from_static(value)))
            .collect();
        Self {
            code,
            static_loader: StaticModuleLoader::new(module_map),
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

        let specifier = match (specifier, referrer) {
            ("gauntlet:core", _) => "gauntlet:core",
            ("gauntlet:renderer", _) => if cfg!(feature = "release") { "gauntlet:renderer:prod" } else { "gauntlet:renderer:dev" },
            ("react", _) => if cfg!(feature = "release") { "gauntlet:react:prod" } else { "gauntlet:react:dev" },
            ("react/jsx-runtime", _) => if cfg!(feature = "release") { "gauntlet:react-jsx-runtime:prod" } else { "gauntlet:react-jsx-runtime:dev" },
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
        op_log_trace,
        op_log_debug,
        op_log_info,
        op_log_warn,
        op_log_error,
        op_component_model,
        asset_data,
        asset_data_blocking,
        op_plugin_get_pending_event,
        op_react_replace_view,
        op_inline_view_endpoint_id,
        get_plugin_preferences,
        get_entrypoint_preferences,
        show_plugin_error_view,
        clear_inline_view,
        plugin_id,
        load_search_index,
        get_command_generator_entrypoint_ids,
        fetch_action_id_for_shortcut,
        plugin_preferences_required,
        entrypoint_preferences_required,
        show_preferences_required_view,
        run_numbat,
        open_settings,
        list_applications,
        open_application,
    ],
    options = {
        event_receiver: EventReceiver,
        plugin_data: PluginData,
        frontend_api: FrontendApi,
        component_model: ComponentModel,
        db_repository: DataDbRepository,
        search_index: SearchIndex,
        icon_cache: IconCache,
    },
    state = |state, options| {
        state.put(options.event_receiver);
        state.put(options.plugin_data);
        state.put(options.frontend_api);
        state.put(options.component_model);
        state.put(options.db_repository);
        state.put(options.search_index);
        state.put(options.icon_cache);
    },
);

#[op]
fn op_log_trace(target: String, message: String) {
    tracing::trace!(target = target, message)
}

#[op]
fn op_log_debug(target: String, message: String) {
    tracing::debug!(target = target, message)
}

#[op]
fn op_log_info(target: String, message: String) {
    tracing::info!(target = target, message)
}

#[op]
fn op_log_warn(target: String, message: String) {
    tracing::warn!(target = target, message)
}

#[op]
fn op_log_error(target: String, message: String) {
    tracing::error!(target = target, message)
}

#[op]
fn op_component_model(state: Rc<RefCell<OpState>>) -> HashMap<String, Component> {
    state.borrow()
        .borrow::<ComponentModel>()
        .components
        .clone()
}

#[op]
fn plugin_id(state: Rc<RefCell<OpState>>) -> String {
    state.borrow()
        .borrow::<PluginData>()
        .plugin_id
        .clone()
        .to_string()
}

#[op]
async fn asset_data(state: Rc<RefCell<OpState>>, path: String) -> anyhow::Result<Vec<u8>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data {:?}", path);

    repository.get_asset_data(&plugin_id.to_string(), &path).await
}

#[op]
fn asset_data_blocking(state: Rc<RefCell<OpState>>, path: String) -> anyhow::Result<Vec<u8>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    tracing::trace!(target = "renderer_rs", "Fetching asset data blocking {:?}", path);

    block_on(async {
        let data = repository.get_asset_data(&plugin_id.to_string(), &path).await?;

        Ok(data)
    })
}

#[op]
async fn plugin_preferences_required(state: Rc<RefCell<OpState>>) -> anyhow::Result<bool> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let DbReadPlugin { preferences, preferences_user_data, .. } = repository
        .get_plugin_by_id(&plugin_id.to_string()).await?;

    Ok(all_preferences_required(preferences, preferences_user_data))
}

#[op]
async fn entrypoint_preferences_required(state: Rc<RefCell<OpState>>, entrypoint_id: String) -> anyhow::Result<bool> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let DbReadPluginEntrypoint { preferences, preferences_user_data, .. } = repository
        .get_entrypoint_by_id(&plugin_id.to_string(), &entrypoint_id).await?;

    Ok(all_preferences_required(preferences, preferences_user_data))
}


#[op]
fn show_preferences_required_view(state: Rc<RefCell<OpState>>, entrypoint_id: String, plugin_preferences_required: bool, entrypoint_preferences_required: bool) -> anyhow::Result<()> {
    let data = JsUiRequestData::ShowPreferenceRequiredView {
        entrypoint_id: EntrypointId::from_string(entrypoint_id),
        plugin_preferences_required,
        entrypoint_preferences_required
    };

    match make_request(&state, data).context("ShowPreferenceRequiredView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling show_preferences_required_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}


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

#[op]
fn clear_inline_view(state: Rc<RefCell<OpState>>) -> anyhow::Result<()> {
    let data = JsUiRequestData::ClearInlineView;

    match make_request(&state, data).context("ClearInlineView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling clear_inline_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn op_inline_view_endpoint_id(state: Rc<RefCell<OpState>>) -> Option<String> {
    state.borrow()
        .borrow::<PluginData>()
        .inline_view_entrypoint_id()
        .clone()
}

#[op]
fn get_plugin_preferences(state: Rc<RefCell<OpState>>) -> anyhow::Result<HashMap<String, PreferenceUserData>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    block_on(async {
        let DbReadPlugin { preferences, preferences_user_data, .. } = repository
            .get_plugin_by_id(&plugin_id.to_string())
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    })
}

#[op]
fn get_entrypoint_preferences(state: Rc<RefCell<OpState>>, entrypoint_id: &str) -> anyhow::Result<HashMap<String, PreferenceUserData>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    block_on(async {
        let DbReadPluginEntrypoint { preferences, preferences_user_data, .. } = repository
            .get_entrypoint_by_id(&plugin_id.to_string(), entrypoint_id)
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    })
}

#[op]
async fn get_command_generator_entrypoint_ids(state: Rc<RefCell<OpState>>) -> anyhow::Result<Vec<String>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let result = repository.get_entrypoints_by_plugin_id(&plugin_id.to_string()).await?
        .into_iter()
        .filter(|entrypoint| entrypoint.enabled)
        .filter(|entrypoint| matches!(db_entrypoint_from_str(&entrypoint.entrypoint_type), DbPluginEntrypointType::CommandGenerator))
        .map(|entrypoint| entrypoint.id)
        .collect::<Vec<_>>();

    Ok(result)
}

#[op]
async fn fetch_action_id_for_shortcut(
    state: Rc<RefCell<OpState>>,
    entrypoint_id: String,
    key: String,
    modifier_shift: bool,
    modifier_control: bool,
    modifier_alt: bool,
    modifier_meta: bool
) -> anyhow::Result<Option<String>> {
    let (plugin_id, repository) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        (plugin_id, repository)
    };

    let result = repository.get_action_id_for_shortcut(
        &plugin_id.to_string(),
        &entrypoint_id,
        &key,
        modifier_shift,
        modifier_control,
        modifier_alt,
        modifier_meta
    ).await?;

    Ok(result)
}

#[op]
fn show_plugin_error_view(state: Rc<RefCell<OpState>>, entrypoint_id: String, render_location: JsUiRenderLocation) -> anyhow::Result<()> {
    let data = JsUiRequestData::ShowPluginErrorView {
        entrypoint_id: EntrypointId::from_string(entrypoint_id),
        render_location,
    };

    match make_request(&state, data).context("ClearInlineView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling show_plugin_error_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
async fn load_search_index(state: Rc<RefCell<OpState>>, generated_commands: Vec<AdditionalSearchItem>) -> anyhow::Result<()> {
    let (plugin_id, plugin_uuid, repository, search_index, icon_cache) = {
        let state = state.borrow();

        let plugin_data = state.borrow::<PluginData>();

        let plugin_id = plugin_data
            .plugin_id()
            .clone();

        let plugin_uuid = plugin_data
            .plugin_uuid()
            .to_owned();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        let search_index = state
            .borrow::<SearchIndex>()
            .clone();

        let icon_cache = state
            .borrow::<IconCache>()
            .clone();

        (plugin_id, plugin_uuid, repository, search_index, icon_cache)
    };

    icon_cache.clear_plugin_icon_cache_dir(&plugin_uuid)
        .context("error when clearing up icon cache before recreating it")?;

    let DbReadPlugin { name, .. } = repository.get_plugin_by_id(&plugin_id.to_string())
        .await
        .context("error when getting plugin by id")?;

    let entrypoints = repository.get_entrypoints_by_plugin_id(&plugin_id.to_string())
        .await
        .context("error when getting entrypoints by plugin id")?;

    let mut plugins_search_items = generated_commands.into_iter()
        .map(|item| {
            let entrypoint_icon_path = match item.entrypoint_icon {
                None => None,
                Some(data) => Some(icon_cache.save_entrypoint_icon_to_cache(&plugin_uuid, &item.entrypoint_uuid, data)?),
            };

            Ok(SearchIndexItem {
                entrypoint_type: SearchIndexPluginEntrypointType::GeneratedCommand,
                entrypoint_id: item.entrypoint_id,
                entrypoint_name: item.entrypoint_name,
                entrypoint_icon_path,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut icon_asset_data = HashMap::new();

    for entrypoint in &entrypoints {
        if let Some(path_to_asset) = &entrypoint.icon_path {
            let result = repository.get_asset_data(&plugin_id.to_string(), path_to_asset)
                .await;

            if let Ok(data) = result {
                icon_asset_data.insert((entrypoint.id.clone(), path_to_asset.clone()), data);
            }
        }
    }

    let mut builtin_search_items = entrypoints.into_iter()
        .filter(|entrypoint| entrypoint.enabled)
        .map(|entrypoint| {
            let entrypoint_type = db_entrypoint_from_str(&entrypoint.entrypoint_type);
            let entrypoint_id = entrypoint.id.to_string();

            let entrypoint_icon_path = match entrypoint.icon_path {
                None => None,
                Some(path_to_asset) => {
                    match icon_asset_data.remove(&(entrypoint.id, path_to_asset)) {
                        None => None,
                        Some(data) => Some(icon_cache.save_entrypoint_icon_to_cache(&plugin_uuid, &entrypoint.uuid, data)?)
                    }
                },
            };

            match &entrypoint_type {
                DbPluginEntrypointType::Command => {
                    Ok(Some(SearchIndexItem {
                        entrypoint_type: SearchIndexPluginEntrypointType::Command,
                        entrypoint_name: entrypoint.name,
                        entrypoint_id,
                        entrypoint_icon_path,
                    }))
                },
                DbPluginEntrypointType::View => {
                    Ok(Some(SearchIndexItem {
                        entrypoint_type: SearchIndexPluginEntrypointType::View,
                        entrypoint_name: entrypoint.name,
                        entrypoint_id,
                        entrypoint_icon_path,
                    }))
                },
                DbPluginEntrypointType::CommandGenerator | DbPluginEntrypointType::InlineView => {
                    Ok(None)
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|item| item)
        .collect::<Vec<_>>();

    plugins_search_items.append(&mut builtin_search_items);

    search_index.save_for_plugin(plugin_id, name, plugins_search_items)
        .context("error when updating search index")?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AdditionalSearchItem {
    entrypoint_name: String,
    entrypoint_id: String,
    entrypoint_uuid: String,
    entrypoint_icon: Option<Vec<u8>>,
}

fn preferences_to_js(
    preferences: HashMap<String, DbPluginPreference>,
    mut preferences_user_data: HashMap<String, DbPluginPreferenceUserData>
) -> HashMap<String, PreferenceUserData> {
    preferences.into_iter()
        .map(|(name, preference)| {
            let user_data = match preferences_user_data.remove(&name) {
                None => match preference {
                    DbPluginPreference::Number { default, .. } => PreferenceUserData::Number(default.expect("at this point preference should always have value")),
                    DbPluginPreference::String { default, .. } => PreferenceUserData::String(default.expect("at this point preference should always have value")),
                    DbPluginPreference::Enum { default, .. } => PreferenceUserData::String(default.expect("at this point preference should always have value")),
                    DbPluginPreference::Bool { default, .. } => PreferenceUserData::Bool(default.expect("at this point preference should always have value")),
                    DbPluginPreference::ListOfStrings { default, .. } => PreferenceUserData::ListOfStrings(default.expect("at this point preference should always have value")),
                    DbPluginPreference::ListOfNumbers { default, .. } => PreferenceUserData::ListOfNumbers(default.expect("at this point preference should always have value")),
                    DbPluginPreference::ListOfEnums { default, .. } => PreferenceUserData::ListOfStrings(default.expect("at this point preference should always have value")),
                }
                Some(user_data) => match user_data {
                    DbPluginPreferenceUserData::Number { value } => PreferenceUserData::Number(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::String { value } => PreferenceUserData::String(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::Enum { value } => PreferenceUserData::String(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::Bool { value } => PreferenceUserData::Bool(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::ListOfStrings { value } => PreferenceUserData::ListOfStrings(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::ListOfNumbers { value } => PreferenceUserData::ListOfNumbers(value.expect("at this point preference should always have value")),
                    DbPluginPreferenceUserData::ListOfEnums { value } => PreferenceUserData::ListOfStrings(value.expect("at this point preference should always have value")),
                }
            };

            (name, user_data)
        })
        .collect()
}

#[op(v8)]
fn op_react_replace_view(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    render_location: JsUiRenderLocation,
    top_level_view: bool,
    entrypoint_id: &str,
    container: JsUiWidget,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view...");

    let comp_state = state.borrow();
    let component_model = comp_state.borrow::<ComponentModel>();

    // TODO fix validation
    // for new_child in &container.widget_children {
    //     validate_child(&state, &container.widget_type, &new_child.widget_type)?
    // }

    let Component::Root { shared_types, .. } = component_model.components.get("gauntlet:root").unwrap() else {
        unreachable!()
    };

    let data = JsUiRequestData::ReplaceView {
        entrypoint_id: EntrypointId::from_string(entrypoint_id),
        render_location,
        top_level_view,
        container: from_js_to_intermediate_widget(scope, container, component_model, shared_types)?,
    };

    match make_request(&state, data).context("ReplaceView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[derive(Debug, Serialize)]
struct NumbatResult {
    left: String,
    right: String,
}

#[op]
fn run_numbat(input: String) -> anyhow::Result<NumbatResult> {
    // TODO add check for plugin id

    let mut context = numbat::Context::new(BuiltinModuleImporter::default());
    let _ = context.interpret("use prelude", numbat::resolver::CodeSource::Internal);

    let (statements, result) = context.interpret(&input, numbat::resolver::CodeSource::Text)?;

    let formatter = numbat::markup::PlainTextFormatter;

    let expression = statements
        .iter()
        .map(|s| formatter.format(&s.pretty_print(), false))
        .collect::<Vec<_>>()
        .join(" ")
        .replace('➞', "to");

    let value = match result {
        InterpreterResult::Value(value) => value,
        InterpreterResult::Continue => Err(anyhow!("numbat returned Continue"))?
    };

    let value = match value {
        Value::Quantity(value) => value.to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::String(value) => value,
    };

    Ok(NumbatResult {
        left: expression,
        right: value
    })
}

#[op]
fn open_settings() -> anyhow::Result<()> {
    std::process::Command::new(std::env::current_exe()?)
        .args(["management"])
        .spawn()?;

    Ok(())
}

#[op]
fn list_applications() -> Vec<DesktopEntry> {
    get_apps()
}

#[op]
fn open_application(command: Vec<String>) -> anyhow::Result<()> {
    let path = &command[0];
    let args = &command[1..];

    std::process::Command::new(Path::new(path))
        .args(args)
        .spawn()?;

    Ok(())
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

fn validate_properties(state: &Rc<RefCell<OpState>>, internal_name: &str, properties: &HashMap<String, UiPropertyValue>) -> anyhow::Result<()> {
    let state = state.borrow();
    let component_model = state.borrow::<ComponentModel>();

    let component = component_model.components.get(internal_name).ok_or(anyhow::anyhow!("invalid component internal name: {}", internal_name))?;

    match component {
        Component::Standard { name, props, .. } => {
            for comp_prop in props {
                match properties.get(&comp_prop.name) {
                    None => {
                        if !comp_prop.optional {
                            Err(anyhow::anyhow!("property {} is required on {} component", comp_prop.name, name))?
                        }
                    }
                    Some(prop_value) => {
                        match prop_value {
                            UiPropertyValue::String(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::String) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a string", comp_prop.name, name))?
                                }
                            }
                            UiPropertyValue::Number(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Number) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a number", comp_prop.name, name))?
                                }
                            }
                            UiPropertyValue::Bool(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Boolean) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a boolean", comp_prop.name, name))?
                                }
                            }
                            UiPropertyValue::Undefined => {
                                if !comp_prop.optional {
                                    Err(anyhow::anyhow!("property {} on {} component has to be optional", comp_prop.name, name))?
                                }
                            }
                            UiPropertyValue::Bytes(_) => {
                                todo!()
                            }
                            UiPropertyValue::Object(_) => {
                                todo!()
                            }
                        }
                    }
                }
            }
        }
        Component::Root { .. } => Err(anyhow::anyhow!("properties of root cannot be validated, likely a bug"))?,
        Component::TextPart { .. } => Err(anyhow::anyhow!("properties of text_part cannot be validated, likely a bug"))?,
    }

    Ok(())
}

fn validate_child(state: &Rc<RefCell<OpState>>, parent_internal_name: &str, child_internal_name: &str) -> anyhow::Result<()> {
    let state = state.borrow();
    let component_model = state.borrow::<ComponentModel>();

    let components = &component_model.components;
    let parent_component = components.get(parent_internal_name).ok_or(anyhow::anyhow!("invalid parent component internal name: {}", parent_internal_name))?;
    let child_component = components.get(child_internal_name).ok_or(anyhow::anyhow!("invalid component internal name: {}", child_internal_name))?;

    match parent_component {
        Component::Standard { name: parent_name, children: parent_children, .. } => {
            match parent_children {
                Children::StringOrMembers { members, .. } => {
                    match child_component {
                        Component::Standard { internal_name, name, .. } => {
                            let allowed_members: HashMap<_, _> = members.iter()
                                .map(|(_, member)| (&member.component_internal_name, member))
                                .collect();

                            match allowed_members.get(internal_name) {
                                None => Err(anyhow::anyhow!("{} component not be a child of {}", name, parent_name))?,
                                Some(_) => (),
                            }
                        }
                        Component::Root { .. } => Err(anyhow::anyhow!("root component not be a child"))?,
                        Component::TextPart { .. } => ()
                    }
                }
                Children::Members { members } => {
                    match child_component {
                        Component::Standard { internal_name, name, .. } => {
                            let allowed_members: HashMap<_, _> = members.iter()
                                .map(|(_, member)| (&member.component_internal_name, member))
                                .collect();

                            match allowed_members.get(internal_name) {
                                None => Err(anyhow::anyhow!("{} component not be a child of {}", name, parent_name))?,
                                Some(_) => (),
                            }
                        }
                        Component::Root { .. } => Err(anyhow::anyhow!("root component not be a child"))?,
                        Component::TextPart { .. } => Err(anyhow::anyhow!("{} component can not have text child", parent_name))?
                    }
                }
                Children::String { .. } => {
                    match child_component {
                        Component::TextPart { .. } => (),
                        _ => Err(anyhow::anyhow!("{} component can only have text child", parent_name))?
                    }
                }
                Children::None => {
                    Err(anyhow::anyhow!("{} component cannot have children", parent_name))?
                }
            }
        }
        Component::Root { children, .. } => {
            let allowed_children: HashMap<_, _> = children.iter()
                .map(|member| (&member.component_internal_name, member))
                .collect();

            match child_component {
                Component::Standard { internal_name, name, .. } => {
                    match allowed_children.get(internal_name) {
                        None => Err(anyhow::anyhow!("{} component not be a child of root", name))?,
                        Some(..) => (),
                    }
                }
                Component::Root { .. } => Err(anyhow::anyhow!("root component not be a child"))?,
                Component::TextPart { .. } => Err(anyhow::anyhow!("root component can not have text child"))?
            }
        }
        Component::TextPart { .. } => Err(anyhow::anyhow!("text part component can not have children"))?
    }

    Ok(())
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
                key,
                modifier_shift,
                modifier_control,
                modifier_alt,
                modifier_meta
            }
        }
        IntermediateUiEvent::StopPlugin => JsUiEvent::StopPlugin,
        IntermediateUiEvent::OpenInlineView { text } => JsUiEvent::OpenInlineView { text },
        IntermediateUiEvent::ReloadSearchIndex => JsUiEvent::ReloadSearchIndex,
    }
}

fn from_js_to_intermediate_widget(scope: &mut v8::HandleScope, ui_widget: JsUiWidget, component_model: &ComponentModel, shared_types: &IndexMap<String, SharedType>) -> anyhow::Result<UiWidget> {
    let children = ui_widget.widget_children.into_iter()
        .map(|child| from_js_to_intermediate_widget(scope, child, component_model, shared_types))
        .collect::<anyhow::Result<Vec<UiWidget>>>()?;

    let component = component_model.components
        .get(&ui_widget.widget_type)
        .expect(&format!("component with type {} doesn't exist", &ui_widget.widget_type));

    let empty = vec![];
    let text_part = vec![Property { name: "value".to_owned(), optional: false, property_type: PropertyType::String }];
    let props = match component {
        Component::Standard { props, .. } => props,
        Component::Root { .. } => &empty,
        Component::TextPart { .. } => &text_part,
    };

    let props = props.into_iter()
        .map(|prop| (&prop.name, &prop.property_type))
        .collect::<HashMap<_, _>>();

    let properties = from_js_to_intermediate_properties(scope, ui_widget.widget_properties, &props, shared_types);

    Ok(UiWidget {
        widget_id: ui_widget.widget_id,
        widget_type: ui_widget.widget_type,
        widget_properties: properties?,
        widget_children: children,
    })
}

fn from_js_to_intermediate_properties(
    scope: &mut v8::HandleScope,
    v8_properties: HashMap<String, serde_v8::Value>,
    component_props: &HashMap<&String, &PropertyType>,
    shared_types: &IndexMap<String, SharedType>
) -> anyhow::Result<HashMap<String, UiPropertyValue>> {
    let vec = v8_properties.into_iter()
        .filter(|(name, _)| name.as_str() != "children")
        .filter(|(_, value)| !value.v8_value.is_function())
        .map(|(name, value)| {
            let val = value.v8_value;

            let Some(property_type) = component_props.get(&name) else {
                return Err(anyhow!("unknown property encountered {:?}", name))
            };

            convert(scope, property_type, name, val, shared_types)
        })
        .collect::<anyhow::Result<Vec<(_, _)>>>()?;

    Ok(vec.into_iter().collect())
}

fn convert(
    scope: &mut v8::HandleScope,
    property_type: &PropertyType,
    name: String,
    value: v8::Local<v8::Value>,
    shared_types: &IndexMap<String, SharedType>
) -> anyhow::Result<(String, UiPropertyValue)> {
    match property_type {
        PropertyType::String | PropertyType::Enum { .. } => {
            if value.is_string() {
                convert_string(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Number => {
            if value.is_number() {
                convert_num(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Boolean => {
            if value.is_boolean() {
                convert_boolean(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Component { .. } => {
            panic!("components should not be present here")
        }
        PropertyType::Function { .. } => {
            panic!("functions are filtered out")
        }
        PropertyType::ImageData => {
            if value.is_array() { // TODO arraybuffer? fix when migrating to deno's op2
                convert_bytes(scope, name, value)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Object { name: object_name } => {
            if value.is_object() {
                convert_object(scope, name, value, object_name, shared_types)
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
        PropertyType::Union { items } => {
            if value.is_string() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::String | PropertyType::Enum { .. })) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_string(scope, name, value)
                }
            } else if value.is_number() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::Number)) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_num(scope, name, value)
                }
            } else if value.is_boolean() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::Boolean)) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_boolean(scope, name, value)
                }
            } else if value.is_array() { // TODO arraybuffer? fix when migrating to deno's op2
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::ImageData)) {
                    None => invalid_type_err(name, value, property_type),
                    Some(_) => convert_bytes(scope, name, value)
                }
            } else if value.is_object() {
                match items.iter().find(|prop_type| matches!(prop_type, PropertyType::Object { .. })) {
                    None => invalid_type_err(name, value, property_type),
                    Some(PropertyType::Object { name: object_name }) => {
                        convert_object(scope, name, value, object_name, shared_types)
                    },
                    _ => unreachable!()
                }
            } else {
                invalid_type_err(name, value, property_type)
            }
        }
    }
}

fn convert_num(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::Number(value.number_value(scope).expect("expected number"))))
}

fn convert_string(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::String(value.to_rust_string_lossy(scope))))
}

fn convert_boolean(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::Bool(value.boolean_value(scope))))
}

fn convert_bytes(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>) -> anyhow::Result<(String, UiPropertyValue)> {
    Ok((name, UiPropertyValue::Bytes(serde_v8::from_v8(scope, value)?)))
}

fn convert_object(scope: &mut v8::HandleScope, name: String, value: v8::Local<v8::Value>, object_name: &str, shared_types: &IndexMap<String, SharedType>) -> anyhow::Result<(String, UiPropertyValue)> {
    let object: v8::Local<v8::Object> = value.try_into().context(format!("error while reading property {}", name))?;

    let props = object
        .get_own_property_names(scope, GetPropertyNamesArgs {
            property_filter: PropertyFilter::ONLY_ENUMERABLE | PropertyFilter::SKIP_SYMBOLS,
            key_conversion: KeyConversionMode::NoNumbers,
            ..Default::default()
        })
        .context("error getting get_own_property_names".to_string())?;

    let mut result_obj: HashMap<String, UiPropertyValue> = HashMap::new();

    for index in 0..props.length() {
        let key = props.get_index(scope, index).unwrap();
        let value = object.get(scope, key).unwrap();
        let key = key.to_string(scope).unwrap().to_rust_string_lossy(scope);

        let property_type = match shared_types.get(object_name).unwrap() {
            SharedType::Enum { .. } => unreachable!(),
            SharedType::Object { items } => items.get(&key).unwrap()
        };

        let (key, value) = convert(scope, property_type, key, value, shared_types)?;

        result_obj.insert(key, value);
    }

    Ok((name, UiPropertyValue::Object(result_obj)))
}

fn invalid_type_err<T>(name: String, value: v8::Local<v8::Value>, property_type: &PropertyType) -> anyhow::Result<T> {
    Err(anyhow!("invalid type for property {:?}, found: {:?}, expected: {}", name, value.type_repr(), expected_type(property_type)))
}

fn expected_type(prop_type: &PropertyType) -> String {
    match prop_type {
        PropertyType::String => "string".to_owned(),
        PropertyType::Number => "number".to_owned(),
        PropertyType::Boolean => "boolean".to_owned(),
        PropertyType::Component { .. } => {
            panic!("components should not be present here")
        }
        PropertyType::Function { .. } => {
            panic!("functions are filtered out")
        }
        PropertyType::ImageData => "bytearray".to_owned(),
        PropertyType::Enum { .. } => "enum".to_owned(),
        PropertyType::Union { items } => {
            items.into_iter()
                .map(|prop_type| expected_type(prop_type))
                .collect::<Vec<_>>()
                .join(", ")
        },
        PropertyType::Object { .. } => "object".to_owned(),
    }
}

fn object_to_json(
    scope: &mut v8::HandleScope,
    val: v8::Local<v8::Value>
) -> anyhow::Result<String> {
    let local = scope.get_current_context();
    let global = local.global(scope);
    let json_string = v8::String::new(scope, "JSON").ok_or(anyhow!("Unable to create JSON string"))?;
    let json_object = global.get(scope, json_string.into()).ok_or(anyhow!("Global JSON object not found"))?;
    let json_object: v8::Local<v8::Object> = json_object.try_into()?;
    let stringify_string = v8::String::new(scope, "stringify").ok_or(anyhow!("Unable to create stringify string"))?;
    let stringify_object = json_object.get(scope, stringify_string.into()).ok_or(anyhow!("Unable to get stringify on global JSON object"))?;
    let stringify_fn: v8::Local<v8::Function> = stringify_object.try_into()?;
    let undefined = v8::undefined(scope).into();

    let json_object = stringify_fn.call(scope, undefined, &[val]).ok_or(anyhow!("Unable to get serialize prop"))?;
    let json_string: v8::Local<v8::String> = json_object.try_into()?;

    let result = json_string.to_rust_string_lossy(scope);

    Ok(result)
}

fn all_preferences_required(preferences: HashMap<String, DbPluginPreference>, preferences_user_data: HashMap<String, DbPluginPreferenceUserData>) -> bool {
    for (name, preference) in preferences {
        match preferences_user_data.get(&name) {
            None => {
                let no_default = match preference {
                    DbPluginPreference::Number { default, .. } => default.is_none(),
                    DbPluginPreference::String { default, .. } => default.is_none(),
                    DbPluginPreference::Enum { default, .. } => default.is_none(),
                    DbPluginPreference::Bool { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfStrings { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfNumbers { default, .. } => default.is_none(),
                    DbPluginPreference::ListOfEnums { default, .. } => default.is_none(),
                };

                if no_default {
                    return true
                }
            }
            Some(preference) => {
                let no_value = match preference {
                    DbPluginPreferenceUserData::Number { value } => value.is_none(),
                    DbPluginPreferenceUserData::String { value } => value.is_none(),
                    DbPluginPreferenceUserData::Enum { value } => value.is_none(),
                    DbPluginPreferenceUserData::Bool { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfStrings { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfNumbers { value } => value.is_none(),
                    DbPluginPreferenceUserData::ListOfEnums { value } => value.is_none(),
                };

                if no_value {
                    return true
                }
            }
        }
    }

    false
}

pub struct PluginData {
    plugin_id: PluginId,
    plugin_uuid: String,
    inline_view_entrypoint_id: Option<String>
}

impl PluginData {
    fn new(plugin_id: PluginId, plugin_uuid: String, inline_view_entrypoint_id: Option<String>) -> Self {
        Self {
            plugin_id,
            plugin_uuid,
            inline_view_entrypoint_id
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
