mod ui;
mod plugins;
mod logs;
mod assets;
mod preferences;
mod search;
mod command_generators;
mod clipboard;

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

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use common::model::{EntrypointId, PluginId, UiRenderLocation, UiPropertyValue, UiWidget, UiWidgetId, SearchResultEntrypointType, PhysicalKey};
use common::rpc::frontend_api::FrontendApi;
use component_model::{Children, Component, create_component_model, Property, PropertyType, SharedType};

use crate::model::{IntermediateUiEvent, JsUiPropertyValue, JsUiRenderLocation, JsUiEvent, JsUiRequestData, JsUiResponseData, JsUiWidget, PreferenceUserData};
use crate::plugins::applications::{DesktopEntry, get_apps};
use crate::plugins::data_db_repository::{DataDbRepository, db_entrypoint_from_str, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::icon_cache::IconCache;
use crate::plugins::js::plugins::applications::{list_applications, open_application};
use crate::plugins::js::assets::{asset_data, asset_data_blocking};
use crate::plugins::js::clipboard::{clipboard_clear, clipboard_read, clipboard_read_text, clipboard_write, clipboard_write_text};
use crate::plugins::js::command_generators::get_command_generator_entrypoint_ids;
use crate::plugins::js::logs::{op_log_debug, op_log_error, op_log_info, op_log_trace, op_log_warn};
use crate::plugins::js::plugins::numbat::run_numbat;
use crate::plugins::js::plugins::settings::open_settings;
use crate::plugins::js::preferences::{entrypoint_preferences_required, get_entrypoint_preferences, get_plugin_preferences, plugin_preferences_required};
use crate::plugins::js::search::load_search_index;
use crate::plugins::js::ui::{clear_inline_view, fetch_action_id_for_shortcut, op_component_model, op_inline_view_endpoint_id, op_react_replace_view, show_plugin_error_view, show_preferences_required_view};
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
    pub frontend_api: FrontendApi,
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
                            OnePluginCommandData::Stop => {
                                Some(IntermediateUiEvent::StopPlugin)
                            }
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
                    data.frontend_api,
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
        load_search_index,

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
        IntermediateUiEvent::StopPlugin => JsUiEvent::StopPlugin,
        IntermediateUiEvent::OpenInlineView { text } => JsUiEvent::OpenInlineView { text },
        IntermediateUiEvent::ReloadSearchIndex => JsUiEvent::ReloadSearchIndex,
    }
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
