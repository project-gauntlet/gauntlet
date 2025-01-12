use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::{anyhow, Context};
use deno_core::{FastString, ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType, RequestedModuleType, ResolutionKind, StaticModuleLoader};
use deno_core::futures::Stream;
use deno_core::url::Url;
use deno_runtime::BootstrapOptions;
use deno_runtime::deno_fs::{FileSystem, RealFs};
use deno_runtime::deno_io::{Stdio, StdioPipe};
use deno_runtime::worker::{MainWorker, WorkerOptions, WorkerServiceOptions};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Receiver;
use gauntlet_common::model::PluginId;
use crate::api::BackendForPluginRuntimeApiProxy;
use crate::assets::{asset_data, asset_data_blocking};
use crate::clipboard::{clipboard_clear, clipboard_read, clipboard_read_text, clipboard_write, clipboard_write_text};
use crate::entrypoint_generators::get_entrypoint_generator_entrypoint_ids;
use crate::component_model::ComponentModel;
use crate::environment::{environment_gauntlet_version, environment_is_development, environment_plugin_cache_dir, environment_plugin_data_dir};
use crate::events::{op_plugin_get_pending_event, EventReceiver, JsEvent};
use crate::JsPluginCode;
use crate::logs::{op_log_debug, op_log_error, op_log_info, op_log_trace, op_log_warn};
use crate::model::JsInit;
use crate::permissions::{permissions_to_deno};
use crate::plugin_data::PluginData;
use crate::plugins::applications::{current_os, wayland, ApplicationContext};
use crate::plugins::numbat::{run_numbat, NumbatContext};
use crate::plugins::settings::open_settings;
use crate::preferences::{entrypoint_preferences_required, get_entrypoint_preferences, get_plugin_preferences, plugin_preferences_required};
use crate::search::reload_search_index;
use crate::ui::{clear_inline_view, fetch_action_id_for_shortcut, hide_window, op_component_model, op_entrypoint_names, op_inline_view_entrypoint_id, op_react_replace_view, show_hud, show_plugin_error_view, show_preferences_required_view, update_loading_bar};



pub struct CustomModuleLoader {
    code: JsPluginCode,
    static_loader: StaticModuleLoader,
    dev_plugin: bool,
}

impl CustomModuleLoader {
    fn new(code: JsPluginCode, dev_plugin: bool) -> Self {
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

const MODULES: [(&str, &str); 11] = [
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
    ("gauntlet:bridge/internal-windows", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/bridge_build/dist/bridge-internal-windows.js"))),
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
            ("gauntlet:bridge/internal-windows", _) => "gauntlet:bridge/internal-windows",
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

        // entrypoint generators
        get_entrypoint_generator_entrypoint_ids,

        // assets
        asset_data,
        asset_data_blocking,

        // ui
        op_react_replace_view,
        op_inline_view_entrypoint_id,
        op_entrypoint_names,
        show_plugin_error_view,
        clear_inline_view,
        show_preferences_required_view,
        op_component_model,
        fetch_action_id_for_shortcut,
        show_hud,
        hide_window,
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
        backend_api: BackendForPluginRuntimeApiProxy,
        outer_handle: Handle
    },
    state = |state, options| {
        state.put(options.event_receiver);
        state.put(options.plugin_data);
        state.put(options.component_model);
        state.put(options.backend_api);
        state.put(options.outer_handle);
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
        wayland,

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
        application_context: ApplicationContext,
    },
    state = |state, options| {
        state.put(options.numbat_context);
        state.put(options.application_context);
    },
);

#[cfg(target_os = "macos")]
deno_core::extension!(
    gauntlet_internal_macos,
    ops = [
        // plugins applications macos
        crate::plugins::applications::macos_major_version,
        crate::plugins::applications::macos_settings_pre_13,
        crate::plugins::applications::macos_settings_13_and_post,
        crate::plugins::applications::macos_open_setting_13_and_post,
        crate::plugins::applications::macos_open_setting_pre_13,
        crate::plugins::applications::macos_system_applications,
        crate::plugins::applications::macos_application_dirs,
        crate::plugins::applications::macos_app_from_arbitrary_path,
        crate::plugins::applications::macos_app_from_path,
        crate::plugins::applications::macos_open_application,
    ],
    esm_entry_point = "ext:gauntlet/internal-macos/bootstrap.js",
    esm = [
        "ext:gauntlet/internal-macos/bootstrap.js" =  "../../js/bridge_build/dist/bridge-internal-macos-bootstrap.js",
        "ext:gauntlet/internal-macos.js" =  "../../js/core/dist/internal-macos.js",
    ]
);


pub async fn start_js_runtime(
    outer_handle: Handle,
    init: JsInit,
    event_stream: Receiver<JsEvent>,
    api: BackendForPluginRuntimeApiProxy,
) -> anyhow::Result<()> {

    let stdout = if let Some(stdout_file) = init.stdout_file {
        let stdout_file = PathBuf::from(stdout_file);

        std::fs::create_dir_all(stdout_file.parent().unwrap())?;

        let out_log_file = File::create(stdout_file)?;

        StdioPipe::file(out_log_file)
    } else {
        StdioPipe::inherit()
    };

    let stderr = if let Some(stderr_file) = init.stderr_file {
        let stderr_file = PathBuf::from(stderr_file);

        std::fs::create_dir_all(stderr_file.parent().unwrap())?;

        let err_log_file = File::create(stderr_file)?;

        StdioPipe::file(err_log_file)
    } else {
        StdioPipe::inherit()
    };

    std::fs::create_dir_all(&init.plugin_cache_dir)
        .context("Unable to create plugin cache directory")?;

    std::fs::create_dir_all(&init.plugin_data_dir)
        .context("Unable to create plugin data directory")?;

    let init_url: ModuleSpecifier = "gauntlet:init".parse().expect("should be valid");

    let fs: Arc<dyn FileSystem> = Arc::new(RealFs);

    let home_dir = PathBuf::from(init.home_dir);

    let permissions_container = permissions_to_deno(
        fs.clone(),
        &init.permissions,
        &home_dir,
        Path::new(&init.plugin_data_dir),
        Path::new(&init.plugin_cache_dir),
    )?;

    let gauntlet_esm = if cfg!(feature = "release") && !init.dev_plugin {
        prod::gauntlet_esm::init_ops_and_esm()
    } else {
        dev::gauntlet_esm::init_ops_and_esm()
    };

    let mut extensions = vec![
        gauntlet::init_ops(
            EventReceiver::new(event_stream),
            PluginData::new(
                init.plugin_id.clone(),
                init.plugin_uuid.clone(),
                init.plugin_cache_dir,
                init.plugin_data_dir,
                init.inline_view_entrypoint_id,
                init.entrypoint_names,
                home_dir
            ),
            ComponentModel::new(),
            api,
            outer_handle
        ),
        gauntlet_esm,
    ];

    if init.plugin_id.to_string() == "bundled://gauntlet" {
        extensions.push(gauntlet_internal_all::init_ops_and_esm(
            NumbatContext::new(),
            ApplicationContext::new()?
        ));

        #[cfg(target_os = "macos")]
        extensions.push(gauntlet_internal_macos::init_ops_and_esm());

        #[cfg(target_os = "linux")]
        extensions.push(crate::plugins::applications::gauntlet_internal_linux::init_ops_and_esm());

        #[cfg(target_os = "windows")]
        extensions.push(crate::plugins::applications::gauntlet_internal_windows::init_ops_and_esm());
    }

    let mut worker = MainWorker::bootstrap_from_options(
        init_url.clone(),
        WorkerServiceOptions {
            blob_store: Arc::new(Default::default()),
            broadcast_channel: Default::default(),
            feature_checker: Arc::new(Default::default()),
            fs,
            module_loader: Rc::new(CustomModuleLoader::new(init.code, init.dev_plugin)),
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
            origin_storage_dir: Some(PathBuf::from(init.local_storage_dir)),
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

