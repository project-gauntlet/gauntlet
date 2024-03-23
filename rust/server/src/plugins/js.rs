use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use deno_core::{FastString, futures, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleType, op, OpState, ResolutionKind, serde_v8, StaticModuleLoader, v8};
use deno_core::futures::{FutureExt, Stream, StreamExt};
use deno_core::futures::executor::block_on;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::permissions::{Permissions, PermissionsContainer, PermissionsOptions};
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tonic::Request;
use tonic::transport::Channel;

use common::model::{EntrypointId, PluginId, PropertyValue, RenderLocation};
use common::rpc::{FrontendClient, RpcClearInlineViewRequest, RpcRenderLocation, RpcReplaceViewRequest, RpcUiPropertyValue, RpcUiWidgetId};
use common::rpc::rpc_frontend_client::RpcFrontendClient;
use common::rpc::rpc_frontend_server::RpcFrontend;
use component_model::{Children, Component, create_component_model, PropertyType};

use crate::model::{from_rpc_to_intermediate_value, IntermediateUiEvent, IntermediateUiWidget, JsPropertyValue, JsRenderLocation, JsUiEvent, JsUiRequestData, JsUiResponseData, JsUiWidget, PreferenceUserData, UiWidgetId};
use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_from_str, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::run_status::RunStatusGuard;
use crate::search::{SearchIndexItem, SearchIndex, SearchIndexPluginEntrypointType};

pub struct PluginRuntimeData {
    pub id: PluginId,
    pub code: PluginCode,
    pub inline_view_entrypoint_id: Option<String>,
    pub permissions: PluginPermissions,
    pub command_receiver: tokio::sync::broadcast::Receiver<PluginCommand>,
    pub db_repository: DataDbRepository,
    pub search_index: SearchIndex
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
        frontend: String,
        entrypoint_id: String,
    },
    RunCommand {
        entrypoint_id: String,
    },
    RunGeneratedCommand {
        entrypoint_id: String,
    },
    HandleEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<PropertyValue>,
    },
    ReloadSearchIndex,
}

#[derive(Clone, Debug)]
pub enum AllPluginCommandData {
    OpenInlineView {
        text: String
    }
}

async fn wait_for_port() {
    loop {
        let addr: SocketAddr = "127.0.0.1:42321".parse().unwrap();

        if TcpStream::connect(addr).await.is_ok() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}


pub async fn start_plugin_runtime(data: PluginRuntimeData, run_status_guard: RunStatusGuard) -> anyhow::Result<()> {
    wait_for_port().await;

    let frontend_client = RpcFrontendClient::connect("http://127.0.0.1:42321").await?;

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
                                Some(IntermediateUiEvent::PluginCommand {
                                    command_type: "stop".to_string(),
                                })
                            }
                            OnePluginCommandData::RenderView { frontend, entrypoint_id } => {
                                Some(IntermediateUiEvent::OpenView {
                                    frontend,
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
                            OnePluginCommandData::HandleEvent { widget_id, event_name, event_arguments } => {
                                Some(IntermediateUiEvent::ViewEvent {
                                    widget_id,
                                    event_name,
                                    event_arguments,
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
                start_js_runtime(
                    data.id,
                    data.code,
                    data.permissions,
                    data.inline_view_entrypoint_id,
                    event_stream,
                    frontend_client,
                    component_model,
                    data.db_repository,
                    data.search_index,
                ).await
            }));

        if let Err(err) = result {
            tracing::error!(target = "plugin", "plugin {:?} runtime failed {:?}", result_plugin_id, err)
        } else {
            tracing::info!(target = "plugin", "plugin {:?} runtime stopped", result_plugin_id)
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
    code: PluginCode,
    permissions: PluginPermissions,
    inline_view_entrypoint_id: Option<String>,
    event_stream: Pin<Box<dyn Stream<Item=IntermediateUiEvent>>>,
    frontend_client: FrontendClient,
    component_model: Vec<Component>,
    repository: DataDbRepository,
    search_index: SearchIndex,
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
                PluginData::new(plugin_id, inline_view_entrypoint_id),
                frontend_client,
                ComponentModel::new(component_model),
                repository,
                search_index,
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
            ("gauntlet:renderer", _) => "gauntlet:renderer:dev",
            ("react", _) => "gauntlet:react:dev",
            ("react/jsx-runtime", _) => "gauntlet:react-jsx-runtime:dev",
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
        clear_inline_view,
        plugin_id,
        load_search_index,
        get_command_generator_entrypoint_ids,
    ],
    options = {
        event_receiver: EventReceiver,
        plugin_data: PluginData,
        frontend_client: FrontendClient,
        component_model: ComponentModel,
        db_repository: DataDbRepository,
        search_index: SearchIndex,
    },
    state = |state, options| {
        state.put(options.event_receiver);
        state.put(options.plugin_data);
        state.put(options.frontend_client);
        state.put(options.component_model);
        state.put(options.db_repository);
        state.put(options.search_index);
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
        .filter(|entrypoint| matches!(db_entrypoint_from_str(&entrypoint.entrypoint_type), DbPluginEntrypointType::CommandGenerator))
        .map(|entrypoint| entrypoint.id)
        .collect::<Vec<_>>();

    Ok(result)
}

#[op]
async fn load_search_index(state: Rc<RefCell<OpState>>, generated_commands: Vec<AdditionalSearchItem>) -> anyhow::Result<()> {
    let (plugin_id, repository, search_index) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let repository = state
            .borrow::<DataDbRepository>()
            .clone();

        let search_index = state
            .borrow::<SearchIndex>()
            .clone();

        (plugin_id, repository, search_index)
    };

    let DbReadPlugin { name, .. } = repository.get_plugin_by_id(&plugin_id.to_string()).await?;

    let entrypoints = repository.get_entrypoints_by_plugin_id(&plugin_id.to_string()).await?;

    let mut search_items = generated_commands.into_iter()
        .map(|item| SearchIndexItem {
            entrypoint_type: SearchIndexPluginEntrypointType::GeneratedCommand,
            entrypoint_id: item.entrypoint_id,
            entrypoint_name: item.entrypoint_name,
        })
        .collect::<Vec<_>>();

    let mut other_search_items = entrypoints
        .into_iter()
        .filter(|entrypoint| entrypoint.enabled)
        .filter_map(|entrypoint| {
            let entrypoint_type = db_entrypoint_from_str(&entrypoint.entrypoint_type);

            let entrypoint_name = entrypoint.name.to_owned();
            let entrypoint_id = entrypoint.id.to_string();

            match &entrypoint_type {
                DbPluginEntrypointType::Command => {
                    Some(SearchIndexItem {
                        entrypoint_type: SearchIndexPluginEntrypointType::Command,
                        entrypoint_name,
                        entrypoint_id,
                    })
                },
                DbPluginEntrypointType::View => {
                    Some(SearchIndexItem {
                        entrypoint_type: SearchIndexPluginEntrypointType::View,
                        entrypoint_name,
                        entrypoint_id,
                    })
                },
                DbPluginEntrypointType::CommandGenerator | DbPluginEntrypointType::InlineView => {
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    search_items.append(&mut other_search_items);

    search_index.save_for_plugin(plugin_id, name, search_items)?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AdditionalSearchItem {
    entrypoint_name: String,
    entrypoint_id: String,
}

fn preferences_to_js(
    preferences: HashMap<String, DbPluginPreference>,
    mut preferences_user_data: HashMap<String, DbPluginPreferenceUserData>
) -> HashMap<String, PreferenceUserData> {
    preferences.into_iter()
        .map(|(name, preference)| {
            let user_data = match preferences_user_data.remove(&name) {
                None => {
                    match preference {
                        DbPluginPreference::Number { default, .. } => PreferenceUserData::Number(default),
                        DbPluginPreference::String { default, ..  } => PreferenceUserData::String(default),
                        DbPluginPreference::Enum { default, ..  } => PreferenceUserData::String(default),
                        DbPluginPreference::Bool { default, ..  } => PreferenceUserData::Bool(default),
                        DbPluginPreference::ListOfStrings { default, ..  } => PreferenceUserData::ListOfStrings(default.unwrap_or(vec![])),
                        DbPluginPreference::ListOfNumbers { default, ..  } => PreferenceUserData::ListOfNumbers(default.unwrap_or(vec![])),
                        DbPluginPreference::ListOfEnums { default, ..  } => PreferenceUserData::ListOfStrings(default.unwrap_or(vec![])),
                    }
                }
                Some(user_data) => match user_data {
                    DbPluginPreferenceUserData::Number { value } => PreferenceUserData::Number(value),
                    DbPluginPreferenceUserData::String { value } => PreferenceUserData::String(value),
                    DbPluginPreferenceUserData::Enum { value } => PreferenceUserData::String(value),
                    DbPluginPreferenceUserData::Bool { value } => PreferenceUserData::Bool(value),
                    DbPluginPreferenceUserData::ListOfStrings { value } => PreferenceUserData::ListOfStrings(value.unwrap_or(vec![])),
                    DbPluginPreferenceUserData::ListOfNumbers { value } => PreferenceUserData::ListOfNumbers(value.unwrap_or(vec![])),
                    DbPluginPreferenceUserData::ListOfEnums { value } => PreferenceUserData::ListOfStrings(value.unwrap_or(vec![])),
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
    render_location: JsRenderLocation,
    top_level_view: bool,
    container: JsUiWidget,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view...");

    // TODO fix validation
    // for new_child in &container.widget_children {
    //     validate_child(&state, &container.widget_type, &new_child.widget_type)?
    // }

    let data = JsUiRequestData::ReplaceView {
        render_location,
        top_level_view,
        container: from_js_to_intermediate_widget(scope, container)?,
    };

    match make_request(&state, data).context("ReplaceView frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs", "Calling op_react_replace_view returned");
            Ok(())
        }
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

fn make_request(state: &Rc<RefCell<OpState>>, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    let (plugin_id, mut frontend_client) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let frontend_client = state
            .borrow::<FrontendClient>()
            .clone();

        (plugin_id, frontend_client)
    };

    block_on(async {
        make_request_async(plugin_id, &mut frontend_client, data).await
    })
}

fn validate_properties(state: &Rc<RefCell<OpState>>, internal_name: &str, properties: &HashMap<String, PropertyValue>) -> anyhow::Result<()> {
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
                            PropertyValue::String(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::String) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a string", comp_prop.name, name))?
                                }
                            }
                            PropertyValue::Number(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Number) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a number", comp_prop.name, name))?
                                }
                            }
                            PropertyValue::Bool(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Boolean) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a boolean", comp_prop.name, name))?
                                }
                            }
                            PropertyValue::Undefined => {
                                if !comp_prop.optional {
                                    Err(anyhow::anyhow!("property {} on {} component has to be optional", comp_prop.name, name))?
                                }
                            }
                            PropertyValue::Bytes(_) => {
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

async fn make_request_async(plugin_id: PluginId, frontend_client: &mut FrontendClient, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    match data {
        JsUiRequestData::ReplaceView { render_location, top_level_view, container } => {
            let rpc_render_location = match render_location {
                JsRenderLocation::InlineView => RpcRenderLocation::InlineViewLocation,
                JsRenderLocation::View => RpcRenderLocation::ViewLocation,
            };

            let request = Request::new(RpcReplaceViewRequest {
                top_level_view,
                plugin_id: plugin_id.to_string(),
                render_location: rpc_render_location.into(),
                container: Some(container.into())
            });

            let nothing = frontend_client.replace_view(request)
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
        JsUiRequestData::ClearInlineView => {
            let request = Request::new(RpcClearInlineViewRequest {
                plugin_id: plugin_id.to_string()
            });

            let nothing = frontend_client.clear_inline_view(request)
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
    }
}

fn from_intermediate_to_js_event(event: IntermediateUiEvent) -> JsUiEvent {
    match event {
        IntermediateUiEvent::OpenView { frontend, entrypoint_id } => JsUiEvent::OpenView {
            frontend,
            entrypoint_id,
        },
        IntermediateUiEvent::RunCommand { entrypoint_id } => JsUiEvent::RunCommand {
            entrypoint_id
        },
        IntermediateUiEvent::RunGeneratedCommand { entrypoint_id } => JsUiEvent::RunGeneratedCommand {
            entrypoint_id
        },
        IntermediateUiEvent::ViewEvent { widget_id, event_name, event_arguments } => {
            let event_arguments = event_arguments.into_iter()
                .map(|arg| match arg {
                    PropertyValue::String(value) => JsPropertyValue::String { value },
                    PropertyValue::Number(value) => JsPropertyValue::Number { value },
                    PropertyValue::Bool(value) => JsPropertyValue::Bool { value },
                    PropertyValue::Undefined => JsPropertyValue::Undefined,
                    PropertyValue::Bytes(_) => {
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
        IntermediateUiEvent::PluginCommand { command_type } => JsUiEvent::PluginCommand {
            command_type
        },
        IntermediateUiEvent::OpenInlineView { text } => JsUiEvent::OpenInlineView { text },
        IntermediateUiEvent::ReloadSearchIndex => JsUiEvent::ReloadSearchIndex,
    }
}

fn from_js_to_intermediate_widget(scope: &mut v8::HandleScope, ui_widget: JsUiWidget) -> anyhow::Result<IntermediateUiWidget> {
    let children = ui_widget.widget_children.into_iter()
        .map(|child| from_js_to_intermediate_widget(scope, child))
        .collect::<anyhow::Result<Vec<IntermediateUiWidget>>>()?;

    Ok(IntermediateUiWidget {
        widget_id: ui_widget.widget_id,
        widget_type: ui_widget.widget_type,
        widget_properties: from_js_to_intermediate_properties(scope, ui_widget.widget_properties)?,
        widget_children: children,
    })
}

fn from_js_to_intermediate_properties(
    scope: &mut v8::HandleScope,
    v8_properties: HashMap<String, serde_v8::Value>,
) -> anyhow::Result<HashMap<String, PropertyValue>> {
    let vec = v8_properties.into_iter()
        .filter(|(name, _)| name.as_str() != "children")
        .filter(|(_, value)| !value.v8_value.is_function())
        .map(|(name, value)| {
            let val = value.v8_value;
            if val.is_string() {
                Ok((name, PropertyValue::String(val.to_rust_string_lossy(scope))))
            } else if val.is_number() {
                Ok((name, PropertyValue::Number(val.number_value(scope).expect("expected number"))))
            } else if val.is_boolean() {
                Ok((name, PropertyValue::Bool(val.boolean_value(scope))))
            } else if val.is_array() {
                Ok((name, PropertyValue::Bytes(serde_v8::from_v8(scope, val)?)))
            } else {
                Err(anyhow!("invalid type for property {:?} - {:?}", name, val.type_repr()))
            }
        })
        .collect::<anyhow::Result<Vec<(_, _)>>>()?;

    Ok(vec.into_iter().collect())
}

pub struct PluginData {
    plugin_id: PluginId,
    inline_view_entrypoint_id: Option<String>
}

impl PluginData {
    fn new(plugin_id: PluginId, inline_view_entrypoint_id: Option<String>) -> Self {
        Self {
            plugin_id,
            inline_view_entrypoint_id
        }
    }

    fn plugin_id(&self) -> PluginId {
        self.plugin_id.clone()
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
