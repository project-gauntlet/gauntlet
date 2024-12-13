use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use bytes::Bytes;
use deno_core::futures::executor::block_on;
use deno_core::futures::{FutureExt, Stream, StreamExt};
use deno_core::v8::{GetPropertyNamesArgs, KeyConversionMode, PropertyFilter};
use deno_core::{futures, op2, serde_v8, v8, FastString, ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode, ModuleSourceFuture, ModuleType, OpState, RequestedModuleType, ResolutionKind, StaticModuleLoader};
use deno_core::url::Url;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::deno_io::{Stdio, StdioPipe};
use deno_runtime::worker::{MainWorker, WorkerServiceOptions};
use deno_runtime::worker::WorkerOptions;
use deno_runtime::BootstrapOptions;
use deno_runtime::deno_fs::{FileSystem, FileSystemRc, RealFs};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_value::Value;
use tokio::net::TcpStream;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;

use common::dirs::Dirs;
use common::model::{EntrypointId, KeyboardEventOrigin, PhysicalKey, PluginId, RootWidget, SearchResultEntrypointType, UiPropertyValue, UiRenderLocation, UiWidgetId};
use common::rpc::frontend_api::FrontendApi;
use common_plugin_runtime::backend_for_plugin_runtime_api::BackendForPluginRuntimeApi;
use common_plugin_runtime::model::{AdditionalSearchItem, ClipboardData, PreferenceUserData};

use crate::model::{IntermediateUiEvent, JsKeyboardEventOrigin, JsUiEvent, JsUiPropertyValue, JsUiRenderLocation};
use crate::plugins::clipboard::Clipboard;
use crate::plugins::data_db_repository::{db_entrypoint_from_str, DataDbRepository, DbPluginClipboardPermissions, DbPluginEntrypointType, DbPluginPreference, DbPluginPreferenceUserData, DbReadPlugin, DbReadPluginEntrypoint};
use crate::plugins::icon_cache::IconCache;
use crate::plugins::js::assets::{asset_data, asset_data_blocking};
use crate::plugins::js::clipboard::{clipboard_clear, clipboard_read, clipboard_read_text, clipboard_write, clipboard_write_text};
use crate::plugins::js::command_generators::{get_command_generator_entrypoint_ids};
use crate::plugins::js::component_model::ComponentModel;
use crate::plugins::js::environment::{environment_gauntlet_version, environment_is_development, environment_plugin_cache_dir, environment_plugin_data_dir};
use crate::plugins::js::logs::{op_log_debug, op_log_error, op_log_info, op_log_trace, op_log_warn};
use crate::plugins::js::permissions::{permissions_to_deno, PluginPermissions, PluginPermissionsClipboard};
use crate::plugins::js::plugins::applications::current_os;
use crate::plugins::js::plugins::numbat::{run_numbat, NumbatContext};
use crate::plugins::js::plugins::settings::open_settings;
use crate::plugins::js::preferences::{entrypoint_preferences_required, get_entrypoint_preferences, get_plugin_preferences, plugin_preferences_required};
use crate::plugins::js::search::reload_search_index;
use crate::plugins::js::ui::{clear_inline_view, fetch_action_id_for_shortcut, op_component_model, op_inline_view_endpoint_id, op_react_replace_view, show_hud, show_plugin_error_view, show_preferences_required_view, update_loading_bar};
use crate::plugins::run_status::RunStatusGuard;
use crate::search::{SearchIndex, SearchIndexItem, SearchIndexItemAction};

mod ui;
mod plugins;
mod logs;
mod assets;
mod preferences;
mod search;
mod command_generators;
pub mod clipboard;
pub mod permissions;
mod environment;
mod component_model;

pub struct PluginRuntimeData {
    pub id: PluginId,
    pub uuid: String,
    pub name: String,
    pub entrypoint_names: HashMap<EntrypointId, String>,
    pub code: PluginCode,
    pub inline_view_entrypoint_id: Option<String>,
    pub permissions: PluginPermissions,
    pub command_receiver: tokio::sync::broadcast::Receiver<PluginCommand>,
    pub db_repository: DataDbRepository,
    pub search_index: SearchIndex,
    pub icon_cache: IconCache,
    pub frontend_api: FrontendApi,
    pub dirs: Dirs,
    pub clipboard: Clipboard,
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
        action_index: Option<usize>
    },
    HandleViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    HandleKeyboardEvent {
        entrypoint_id: EntrypointId,
        origin: KeyboardEventOrigin,
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
                            OnePluginCommandData::RunGeneratedCommand { entrypoint_id, action_index } => {
                                Some(IntermediateUiEvent::RunGeneratedCommand {
                                    entrypoint_id,
                                    action_index
                                })
                            }
                            OnePluginCommandData::HandleViewEvent { widget_id, event_name, event_arguments } => {
                                Some(IntermediateUiEvent::HandleViewEvent {
                                    widget_id,
                                    event_name,
                                    event_arguments,
                                })
                            }
                            OnePluginCommandData::HandleKeyboardEvent { entrypoint_id, origin, key, modifier_shift, modifier_control, modifier_alt, modifier_meta } => {
                                Some(IntermediateUiEvent::HandleKeyboardEvent {
                                    entrypoint_id,
                                    origin,
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
                                     data.name,
                                     data.entrypoint_names,
                                     data.code,
                                     data.permissions,
                                     data.inline_view_entrypoint_id,
                                     event_stream,
                                     data.frontend_api,
                                     data.db_repository,
                                     data.search_index,
                                     data.icon_cache,
                                     data.dirs,
                                     data.clipboard
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
    plugin_name: String,
    entrypoint_names: HashMap<EntrypointId, String>,
    code: PluginCode,
    permissions: PluginPermissions,
    inline_view_entrypoint_id: Option<String>,
    event_stream: Pin<Box<dyn Stream<Item=IntermediateUiEvent>>>,
    frontend_api: FrontendApi,
    repository: DataDbRepository,
    search_index: SearchIndex,
    icon_cache: IconCache,
    dirs: Dirs,
    clipboard: Clipboard,
) -> anyhow::Result<()> {
    let plugin_id_str = plugin_id.to_string();
    let dev_plugin = plugin_id_str.starts_with("file://");

    let (stdout, stderr) = if dev_plugin {
        let (out_log_file, err_log_file) = dirs.plugin_log_files(&plugin_uuid);

        std::fs::create_dir_all(out_log_file.parent().unwrap())?;
        std::fs::create_dir_all(err_log_file.parent().unwrap())?;

        let out_log_file = File::create(out_log_file)?;
        let err_log_file = File::create(err_log_file)?;

        (StdioPipe::file(out_log_file), StdioPipe::file(err_log_file))
    } else {
        (StdioPipe::inherit(), StdioPipe::inherit())
    };

    let home_dir = dirs.home_dir();
    let local_storage_dir = dirs.plugin_local_storage(&plugin_uuid);
    let plugin_cache_dir = dirs.plugin_cache(&plugin_uuid)?.to_str().expect("non-uft8 paths are not supported").to_string();
    let plugin_data_dir = dirs.plugin_data(&plugin_uuid)?.to_str().expect("non-uft8 paths are not supported").to_string();

    std::fs::create_dir_all(&plugin_cache_dir)
        .context("Unable to create plugin cache directory")?;

    std::fs::create_dir_all(&plugin_data_dir)
        .context("Unable to create plugin data directory")?;

    let init_url: ModuleSpecifier = "gauntlet:init".parse().expect("should be valid");

    let fs: Arc<dyn FileSystem> = Arc::new(RealFs);

    let permissions_container = permissions_to_deno(
        fs.clone(),
        &permissions,
        &home_dir,
        &PathBuf::from(&plugin_data_dir),
        &PathBuf::from(&plugin_cache_dir),
    )?;

    let runtime_permissions = PluginRuntimePermissions {
        clipboard: permissions.clipboard,
    };

    let gauntlet_esm = if cfg!(feature = "release") && !dev_plugin {
        prod::gauntlet_esm::init_ops_and_esm()
    } else {
        dev::gauntlet_esm::init_ops_and_esm()
    };

    let mut extensions = vec![
        gauntlet::init_ops(
            EventReceiver::new(event_stream),
            PluginData::new(
                plugin_id.clone(),
                plugin_uuid.clone(),
                plugin_cache_dir,
                plugin_data_dir,
                inline_view_entrypoint_id,
                home_dir
            ),
            ComponentModel::new(),
            BackendForPluginRuntimeApiImpl::new(
                icon_cache,
                repository,
                search_index,
                clipboard,
                frontend_api,
                plugin_uuid,
                plugin_id,
                plugin_name,
                entrypoint_names,
                runtime_permissions,
            ),
        ),
        gauntlet_esm,
    ];

    if plugin_id_str == "bundled://gauntlet" {
        extensions.push(gauntlet_internal_all::init_ops_and_esm(NumbatContext::new()));

        #[cfg(target_os = "macos")]
        extensions.push(gauntlet_internal_macos::init_ops_and_esm());

        #[cfg(target_os = "linux")]
        extensions.push(gauntlet_internal_linux::init_ops_and_esm());
    }

    let mut worker = MainWorker::bootstrap_from_options(
        init_url.clone(),
        WorkerServiceOptions {
            blob_store: Arc::new(Default::default()),
            broadcast_channel: Default::default(),
            feature_checker: Arc::new(Default::default()),
            fs,
            module_loader: Rc::new(CustomModuleLoader::new(code, dev_plugin)),
            node_services: None,
            npm_process_state_provider: None,
            permissions: permissions_container,
            root_cert_store_provider: None,
            fetch_dns_resolver: Default::default(),
            shared_array_buffer_store: None,
            compiled_wasm_module_store: None,
            v8_code_cache: None,
        },
        WorkerOptions {
            bootstrap: BootstrapOptions {
                is_stderr_tty: false,
                is_stdout_tty: false,
                ..Default::default()
            },
            extensions,
            maybe_inspector_server: None,
            should_wait_for_inspector_session: false,
            should_break_on_first_statement: false,
            origin_storage_dir: Some(local_storage_dir),
            stdio: Stdio {
                stdin: StdioPipe::inherit(),
                stdout,
                stderr,
            },
            ..Default::default()
        },
    );

    worker.execute_main_module(&init_url).await?;
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
    ("gauntlet:init", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/core/dist/init.js"))),
    ("gauntlet:bridge/components", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-components.js"))),
    ("gauntlet:bridge/hooks", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-hooks.js"))),
    ("gauntlet:bridge/helpers", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-helpers.js"))),
    ("gauntlet:bridge/core", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-core.js"))),
    ("gauntlet:bridge/react", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-react.js"))),
    ("gauntlet:bridge/react-jsx-runtime", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-react-jsx-runtime.js"))),
    ("gauntlet:bridge/internal-all", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-internal-all.js"))),
    ("gauntlet:bridge/internal-linux", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-internal-linux.js"))),
    ("gauntlet:bridge/internal-macos", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-internal-macos.js"))),
];

impl ModuleLoader for CustomModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
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
            ("gauntlet:init", _) => "gauntlet:init",
            ("gauntlet:core", _) => "gauntlet:bridge/core",
            ("gauntlet:bridge/internal-all", _) => "gauntlet:bridge/internal-all",
            ("gauntlet:bridge/internal-linux", _) => "gauntlet:bridge/internal-linux",
            ("gauntlet:bridge/internal-macos", _) => "gauntlet:bridge/internal-macos",
            ("react", _) => "gauntlet:bridge/react",
            ("react/jsx-runtime", _) => "gauntlet:bridge/react-jsx-runtime",
            ("@project-gauntlet/api/components", _) => "gauntlet:bridge/components",
            ("@project-gauntlet/api/hooks", _) => "gauntlet:bridge/hooks",
            ("@project-gauntlet/api/helpers", _) => "gauntlet:bridge/helpers",
            _ => {
                return Err(anyhow!("Illegal import with specifier '{}' and referrer '{}'", specifier, referrer))
            }
        };

        Ok(Url::parse(specifier)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleSpecifier>,
        is_dyn_import: bool,
        requested_module_type: RequestedModuleType,
    ) -> ModuleLoadResponse {

        let mut specifier = module_specifier.clone();
        specifier.set_query(None);

        match specifier.as_str() {
            "gauntlet:init" => {
                self.static_loader.load(module_specifier, maybe_referrer, is_dyn_import, requested_module_type)
            }
            "gauntlet:entrypoint" | "gauntlet:module" => {
                match module_specifier.query() {
                    None => {
                        ModuleLoadResponse::Sync(Err(anyhow!("Module specifier doesn't have query part")))
                    },
                    Some(entrypoint_id) => {
                        let result = self.code.js
                            .get(entrypoint_id)
                            .ok_or(anyhow!("Cannot find JS code path: {:?}", entrypoint_id))
                            .map(|js| ModuleSourceCode::String(js.clone().into()))
                            .map(|js| ModuleSource::new(ModuleType::JavaScript, js, module_specifier, None));

                        ModuleLoadResponse::Sync(result)
                    }
                }
            }
            _ => {
                if specifier.as_str().starts_with("gauntlet:bridge/"){
                    self.static_loader.load(module_specifier, maybe_referrer, is_dyn_import, requested_module_type)
                } else {
                    ModuleLoadResponse::Sync(Err(anyhow!("Module not found: specifier '{}' and referrer '{:?}'", specifier, maybe_referrer.map(|url| url.as_str()))))
                }
            }
        }
    }
}

deno_core::extension!(
    gauntlet,
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
        show_hud,
        update_loading_bar,

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

        // plugin environment
        environment_gauntlet_version,
        environment_is_development,
        environment_plugin_data_dir,
        environment_plugin_cache_dir,
    ],
    options = {
        event_receiver: EventReceiver,
        plugin_data: PluginData,
        component_model: ComponentModel,
        backend_api: BackendForPluginRuntimeApiImpl,
    },
    state = |state, options| {
        state.put(options.event_receiver);
        state.put(options.plugin_data);
        state.put(options.component_model);
        state.put(options.backend_api);
    },
);

mod prod {
    deno_core::extension!(
        gauntlet_esm,
        esm_entry_point = "ext:gauntlet/bootstrap.js",
        esm = [
            "ext:gauntlet/bootstrap.js" =  "../../js/bridge_build/dist/bridge-bootstrap.js",
            "ext:gauntlet/core.js" =  "../../js/core/dist/core.js",
            "ext:gauntlet/api/components.js" =  "../../js/api/dist/gen/components.js",
            "ext:gauntlet/api/hooks.js" =  "../../js/api/dist/hooks.js",
            "ext:gauntlet/api/helpers.js" =  "../../js/api/dist/helpers.js",
            "ext:gauntlet/renderer.js" =  "../../js/react_renderer/dist/prod/renderer.js",
            "ext:gauntlet/react.js" =  "../../js/react/dist/prod/react.production.min.js",
            "ext:gauntlet/react-jsx-runtime.js" =  "../../js/react/dist/prod/react-jsx-runtime.production.min.js",
        ],
    );
}

#[allow(long_running_const_eval)] // dev renderer is 22K line file which triggers rust lint
mod dev {
    deno_core::extension!(
        gauntlet_esm,
        esm_entry_point = "ext:gauntlet/bootstrap.js",
        esm = [
            "ext:gauntlet/bootstrap.js" =  "../../js/bridge_build/dist/bridge-bootstrap.js",
            "ext:gauntlet/core.js" =  "../../js/core/dist/core.js",
            "ext:gauntlet/api/components.js" =  "../../js/api/dist/gen/components.js",
            "ext:gauntlet/api/hooks.js" =  "../../js/api/dist/hooks.js",
            "ext:gauntlet/api/helpers.js" =  "../../js/api/dist/helpers.js",
            "ext:gauntlet/renderer.js" =  "../../js/react_renderer/dist/dev/renderer.js",
            "ext:gauntlet/react.js" =  "../../js/react/dist/dev/react.development.js",
            "ext:gauntlet/react-jsx-runtime.js" =  "../../js/react/dist/dev/react-jsx-runtime.development.js",
        ],
    );
}

deno_core::extension!(
    gauntlet_internal_all,
    ops = [
        // plugins numbat
        run_numbat,

        // plugins applications
        current_os,

        // plugins settings
        open_settings,
    ],
    esm_entry_point = "ext:gauntlet/internal-all/bootstrap.js",
    esm = [
        "ext:gauntlet/internal-all/bootstrap.js" =  "../../js/bridge_build/dist/bridge-internal-all-bootstrap.js",
        "ext:gauntlet/internal-all.js" =  "../../js/core/dist/internal-all.js",
    ],
    options = {
        numbat_context: NumbatContext,
    },
    state = |state, options| {
        state.put(options.numbat_context);
    },
);

#[cfg(target_os = "linux")]
deno_core::extension!(
    gauntlet_internal_linux,
    ops = [
        // plugins applications linux
        crate::plugins::js::plugins::applications::linux_app_from_path,
        crate::plugins::js::plugins::applications::linux_application_dirs,
        crate::plugins::js::plugins::applications::linux_open_application,
    ],
    esm_entry_point = "ext:gauntlet/internal-linux/bootstrap.js",
    esm = [
        "ext:gauntlet/internal-linux/bootstrap.js" =  "../../js/bridge_build/dist/bridge-internal-linux-bootstrap.js",
        "ext:gauntlet/internal-linux.js" =  "../../js/core/dist/internal-linux.js",
    ]
);

#[cfg(target_os = "macos")]
deno_core::extension!(
    gauntlet_internal_macos,
    ops = [
        // plugins applications macos
        crate::plugins::js::plugins::applications::macos_major_version,
        crate::plugins::js::plugins::applications::macos_settings_pre_13,
        crate::plugins::js::plugins::applications::macos_settings_13_and_post,
        crate::plugins::js::plugins::applications::macos_open_setting_13_and_post,
        crate::plugins::js::plugins::applications::macos_open_setting_pre_13,
        crate::plugins::js::plugins::applications::macos_system_applications,
        crate::plugins::js::plugins::applications::macos_application_dirs,
        crate::plugins::js::plugins::applications::macos_app_from_arbitrary_path,
        crate::plugins::js::plugins::applications::macos_app_from_path,
        crate::plugins::js::plugins::applications::macos_open_application,
    ],
    esm_entry_point = "ext:gauntlet/internal-macos/bootstrap.js",
    esm = [
        "ext:gauntlet/internal-macos/bootstrap.js" =  "../../js/bridge_build/dist/bridge-internal-macos-bootstrap.js",
        "ext:gauntlet/internal-macos.js" =  "../../js/core/dist/internal-macos.js",
    ]
);

#[op2(async)]
#[serde]
pub async fn op_plugin_get_pending_event(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsUiEvent> {
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

fn from_intermediate_to_js_event(event: IntermediateUiEvent) -> JsUiEvent {
    match event {
        IntermediateUiEvent::OpenView { entrypoint_id } => JsUiEvent::OpenView {
            entrypoint_id: entrypoint_id.to_string(),
        },
        IntermediateUiEvent::CloseView => JsUiEvent::CloseView,
        IntermediateUiEvent::RunCommand { entrypoint_id } => JsUiEvent::RunCommand {
            entrypoint_id
        },
        IntermediateUiEvent::RunGeneratedCommand { entrypoint_id, action_index } => JsUiEvent::RunGeneratedCommand {
            entrypoint_id,
            action_index,
        },
        IntermediateUiEvent::HandleViewEvent { widget_id, event_name, event_arguments } => {
            let event_arguments = event_arguments.into_iter()
                .map(|arg| match arg {
                    UiPropertyValue::String(value) => JsUiPropertyValue::String { value },
                    UiPropertyValue::Number(value) => JsUiPropertyValue::Number { value },
                    UiPropertyValue::Bool(value) => JsUiPropertyValue::Bool { value },
                    UiPropertyValue::Undefined => JsUiPropertyValue::Undefined,
                    UiPropertyValue::Array(_) | UiPropertyValue::Bytes(_) | UiPropertyValue::Object(_)  => {
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
        IntermediateUiEvent::HandleKeyboardEvent { entrypoint_id, origin, key, modifier_shift, modifier_control, modifier_alt, modifier_meta } => {
            JsUiEvent::KeyboardEvent {
                entrypoint_id: entrypoint_id.to_string(),
                origin: match origin {
                    KeyboardEventOrigin::MainView => JsKeyboardEventOrigin::MainView,
                    KeyboardEventOrigin::PluginView => JsKeyboardEventOrigin::PluginView,
                },
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
    plugin_cache_dir: String,
    plugin_data_dir: String,
    inline_view_entrypoint_id: Option<String>,
    home_dir: PathBuf,
}

impl PluginData {
    fn new(
        plugin_id: PluginId,
        plugin_uuid: String,
        plugin_cache_dir: String,
        plugin_data_dir: String,
        inline_view_entrypoint_id: Option<String>,
        home_dir: PathBuf,
    ) -> Self {
        Self {
            plugin_id,
            plugin_uuid,
            plugin_cache_dir,
            plugin_data_dir,
            inline_view_entrypoint_id,
            home_dir
        }
    }

    fn plugin_id(&self) -> PluginId {
        self.plugin_id.clone()
    }

    fn plugin_uuid(&self) -> &str {
        &self.plugin_uuid
    }

    fn plugin_cache_dir(&self) -> &str {
        &self.plugin_cache_dir
    }

    fn plugin_data_dir(&self) -> &str {
        &self.plugin_data_dir
    }

    fn inline_view_entrypoint_id(&self) -> Option<String> {
        self.inline_view_entrypoint_id.clone()
    }

    fn home_dir(&self) -> PathBuf {
        self.home_dir.clone()
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

#[derive(Clone)]
pub struct BackendForPluginRuntimeApiImpl {
    icon_cache: IconCache,
    repository: DataDbRepository,
    search_index: SearchIndex,
    clipboard: Clipboard,
    frontend_api: FrontendApi,
    plugin_uuid: String,
    plugin_id: PluginId,
    plugin_name: String,
    entrypoint_names: HashMap<EntrypointId, String>,
    permissions: PluginRuntimePermissions
}

impl BackendForPluginRuntimeApiImpl {
    fn new(
        icon_cache: IconCache,
        repository: DataDbRepository,
        search_index: SearchIndex,
        clipboard: Clipboard,
        frontend_api: FrontendApi,
        plugin_uuid: String,
        plugin_id: PluginId,
        plugin_name: String,
        entrypoint_names: HashMap<EntrypointId, String>,
        permissions: PluginRuntimePermissions
    ) -> Self {
        Self {
            icon_cache,
            repository,
            search_index,
            clipboard,
            frontend_api,
            plugin_uuid,
            plugin_id,
            plugin_name,
            entrypoint_names,
            permissions
        }
    }
}

impl BackendForPluginRuntimeApi for BackendForPluginRuntimeApiImpl {
    async fn reload_search_index(&self, generated_commands: Vec<AdditionalSearchItem>, refresh_search_list: bool) -> anyhow::Result<()> {
        self.icon_cache.clear_plugin_icon_cache_dir(&self.plugin_uuid)
            .context("error when clearing up icon cache before recreating it")?;

        let DbReadPlugin { name, .. } = self.repository.get_plugin_by_id(&self.plugin_id.to_string())
            .await
            .context("error when getting plugin by id")?;

        let entrypoints = self.repository.get_entrypoints_by_plugin_id(&self.plugin_id.to_string())
            .await
            .context("error when getting entrypoints by plugin id")?;

        let frecency_map = self.repository.get_frecency_for_plugin(&self.plugin_id.to_string())
            .await
            .context("error when getting frecency for plugin")?;

        let mut shortcuts = HashMap::new();

        for DbReadPluginEntrypoint { id, .. } in &entrypoints {
            let entrypoint_shortcuts = self.repository.action_shortcuts(&self.plugin_id.to_string(), id).await?;
            shortcuts.insert(id.clone(), entrypoint_shortcuts);
        }

        let mut plugins_search_items = generated_commands.into_iter()
            .map(|item| {
                let entrypoint_icon_path = match item.entrypoint_icon {
                    None => None,
                    Some(data) => Some(self.icon_cache.save_entrypoint_icon_to_cache(&self.plugin_uuid, &item.entrypoint_uuid, &data)?),
                };

                let entrypoint_frecency = frecency_map.get(&item.entrypoint_id).cloned().unwrap_or(0.0);

                let shortcuts = shortcuts
                    .get(&item.generator_entrypoint_id);

                let entrypoint_actions = item.entrypoint_actions.iter()
                    .map(|action| {
                        let shortcut = match (shortcuts, &action.id) {
                            (Some(shortcuts), Some(id)) => {
                                shortcuts.get(id).cloned()
                            }
                            _ => None
                        };

                        SearchIndexItemAction {
                            label: action.label.clone(),
                            shortcut,
                        }
                    })
                    .collect();

                Ok(SearchIndexItem {
                    entrypoint_type: SearchResultEntrypointType::GeneratedCommand,
                    entrypoint_id: EntrypointId::from_string(item.entrypoint_id),
                    entrypoint_name: item.entrypoint_name,
                    entrypoint_icon_path,
                    entrypoint_frecency,
                    entrypoint_actions,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let mut icon_asset_data = HashMap::new();

        for entrypoint in &entrypoints {
            if let Some(path_to_asset) = &entrypoint.icon_path {
                let result = self.repository.get_asset_data(&self.plugin_id.to_string(), path_to_asset)
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

                let entrypoint_frecency = frecency_map.get(&entrypoint_id).cloned().unwrap_or(0.0);

                let entrypoint_icon_path = match entrypoint.icon_path {
                    None => None,
                    Some(path_to_asset) => {
                        match icon_asset_data.get(&(entrypoint.id, path_to_asset)) {
                            None => None,
                            Some(data) => Some(self.icon_cache.save_entrypoint_icon_to_cache(&self.plugin_uuid, &entrypoint.uuid, data)?)
                        }
                    },
                };

                let entrypoint_id = EntrypointId::from_string(entrypoint_id);

                match &entrypoint_type {
                    DbPluginEntrypointType::Command => {
                        Ok(Some(SearchIndexItem {
                            entrypoint_type: SearchResultEntrypointType::Command,
                            entrypoint_name: entrypoint.name,
                            entrypoint_id,
                            entrypoint_icon_path,
                            entrypoint_frecency,
                            entrypoint_actions: vec![],
                        }))
                    },
                    DbPluginEntrypointType::View => {
                        Ok(Some(SearchIndexItem {
                            entrypoint_type: SearchResultEntrypointType::View,
                            entrypoint_name: entrypoint.name,
                            entrypoint_id,
                            entrypoint_icon_path,
                            entrypoint_frecency,
                            entrypoint_actions: vec![],
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

        self.search_index.save_for_plugin(self.plugin_id.clone(), name, plugins_search_items, refresh_search_list)
            .context("error when updating search index")?;

        Ok(())
    }

    async fn get_asset_data(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let data = self.repository.get_asset_data(&self.plugin_id.to_string(), &path)
            .await?;

        Ok(data)
    }

    async fn get_command_generator_entrypoint_ids(&self) -> anyhow::Result<Vec<String>> {
        let result = self.repository.get_entrypoints_by_plugin_id(&self.plugin_id.to_string()).await?
            .into_iter()
            .filter(|entrypoint| entrypoint.enabled)
            .filter(|entrypoint| matches!(db_entrypoint_from_str(&entrypoint.entrypoint_type), DbPluginEntrypointType::CommandGenerator))
            .map(|entrypoint| entrypoint.id)
            .collect::<Vec<_>>();

        Ok(result)
    }

    async fn get_plugin_preferences(&self) -> anyhow::Result<HashMap<String, PreferenceUserData>> {
        let DbReadPlugin { preferences, preferences_user_data, .. } = self.repository
            .get_plugin_by_id(&self.plugin_id.to_string())
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    }

    async fn get_entrypoint_preferences(&self, entrypoint_id: EntrypointId) -> anyhow::Result<HashMap<String, PreferenceUserData>> {
        let DbReadPluginEntrypoint { preferences, preferences_user_data, .. } = self.repository
            .get_entrypoint_by_id(&self.plugin_id.to_string(), &entrypoint_id.to_string())
            .await?;

        Ok(preferences_to_js(preferences, preferences_user_data))
    }

    async fn plugin_preferences_required(&self) -> anyhow::Result<bool> {
        let DbReadPlugin { preferences, preferences_user_data, .. } = self.repository
            .get_plugin_by_id(&self.plugin_id.to_string()).await?;

        Ok(any_preferences_missing_value(preferences, preferences_user_data))
    }

    async fn entrypoint_preferences_required(&self, entrypoint_id: EntrypointId) -> anyhow::Result<bool> {
        let DbReadPluginEntrypoint { preferences, preferences_user_data, .. } = self.repository
            .get_entrypoint_by_id(&self.plugin_id.to_string(), &entrypoint_id.to_string()).await?;

        Ok(any_preferences_missing_value(preferences, preferences_user_data))
    }

    async fn clipboard_read(&self) -> anyhow::Result<ClipboardData> {
        let allow = self
            .permissions
            .clipboard
            .contains(&PluginPermissionsClipboard::Read);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'read' permission for clipboard"));
        }

        tracing::debug!("Reading from clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.read()
    }

    async fn clipboard_read_text(&self) -> anyhow::Result<Option<String>> {
        let allow = self
            .permissions
            .clipboard
            .contains(&PluginPermissionsClipboard::Read);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'read' permission for clipboard"));
        }

        tracing::debug!("Reading text from clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.read_text()
    }

    async fn clipboard_write(&self, data: ClipboardData) -> anyhow::Result<()> {
        let allow = self
            .permissions
            .clipboard
            .contains(&PluginPermissionsClipboard::Write);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'write' permission for clipboard"));
        }

        tracing::debug!("Writing to clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.write(data)
    }

    async fn clipboard_write_text(&self, data: String) -> anyhow::Result<()> {
        let allow = self
            .permissions
            .clipboard
            .contains(&PluginPermissionsClipboard::Write);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'write' permission for clipboard"));
        }

        tracing::debug!("Writing text to clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.write_text(data)
    }

    async fn clipboard_clear(&self) -> anyhow::Result<()> {
        let allow = self
            .permissions
            .clipboard
            .contains(&PluginPermissionsClipboard::Clear);

        if !allow {
            return Err(anyhow!("Plugin doesn't have 'clear' permission for clipboard"));
        }

        tracing::debug!("Clearing clipboard, plugin id: {:?}", self.plugin_id);

        self.clipboard.clear()
    }

    async fn ui_update_loading_bar(&self, entrypoint_id: EntrypointId, show: bool) -> anyhow::Result<()> {
        self.frontend_api.update_loading_bar(self.plugin_id.clone(), entrypoint_id, show).await?;

        Ok(())
    }

    async fn ui_show_hud(&self, display: String) -> anyhow::Result<()> {
        self.frontend_api.show_hud(display).await?;

        Ok(())
    }

    async fn ui_get_action_id_for_shortcut(
        &self,
        entrypoint_id: EntrypointId,
        key: String,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool
    ) -> anyhow::Result<Option<String>> {
        let result = self.repository.get_action_id_for_shortcut(
            &self.plugin_id.to_string(),
            &entrypoint_id.to_string(),
            PhysicalKey::from_value(key),
            modifier_shift,
            modifier_control,
            modifier_alt,
            modifier_meta
        ).await?;

        Ok(result)
    }

    async fn ui_render(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation,
        top_level_view: bool,
        container: RootWidget,
        #[cfg(feature = "scenario_runner")]
        container_value: serde_value::Value,
        images: HashMap<UiWidgetId, Bytes>
    ) -> anyhow::Result<()> {

        let entrypoint_name = self.entrypoint_names
            .get(&entrypoint_id)
            .expect("entrypoint name for id should always exist")
            .to_string();

        self.frontend_api.replace_view(
            self.plugin_id.clone(),
            self.plugin_name.clone(),
            entrypoint_id,
            entrypoint_name,
            render_location,
            top_level_view,
            container,
            #[cfg(feature = "scenario_runner")]
            container_value,
            images
        ).await?;

        Ok(())
    }

    async fn ui_show_plugin_error_view(
        &self,
        entrypoint_id: EntrypointId,
        render_location: UiRenderLocation
    ) -> anyhow::Result<()> {
        self.frontend_api.show_plugin_error_view(
            self.plugin_id.clone(),
            entrypoint_id,
            render_location
        ).await?;

        Ok(())
    }

    async fn ui_show_preferences_required_view(
        &self,
        entrypoint_id: EntrypointId,
        plugin_preferences_required: bool,
        entrypoint_preferences_required: bool
    ) -> anyhow::Result<()> {

        self.frontend_api.show_preference_required_view(
            self.plugin_id.clone(),
            entrypoint_id,
            plugin_preferences_required,
            entrypoint_preferences_required
        ).await?;

        Ok(())
    }

    async fn ui_clear_inline_view(&self) -> anyhow::Result<()> {
        self.frontend_api.clear_inline_view(self.plugin_id.clone()).await?;

        Ok(())
    }
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

fn any_preferences_missing_value(preferences: HashMap<String, DbPluginPreference>, preferences_user_data: HashMap<String, DbPluginPreferenceUserData>) -> bool {
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
