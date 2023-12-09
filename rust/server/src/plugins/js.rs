use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::{Pin};
use std::rc::Rc;

use anyhow::anyhow;
use deno_core::{FastString, futures, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleType, op, OpState, ResolutionKind, serde_v8, StaticModuleLoader, v8};
use deno_core::futures::{FutureExt, Stream, StreamExt};
use deno_core::futures::executor::block_on;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use futures_concurrency::stream::Merge;
use once_cell::sync::Lazy;
use regex::Regex;
use common::model::PluginId;
use component_model::{create_component_model, Component};

use crate::dbus::{DbusClientProxyProxy, ViewCreatedSignal, ViewEventSignal};
use crate::model::{JsUiEvent, JsUiEventName, JsUiPropertyValue, JsUiWidget, JsUiWidgetId, JsUiRequestData, JsUiResponseData, to_dbus};
use crate::plugins::run_status::RunStatusGuard;

pub struct PluginRuntimeData {
    pub id: PluginId,
    pub code: PluginCode,
    pub command_receiver: tokio::sync::broadcast::Receiver<PluginCommand>,
}

pub struct PluginCode {
    pub js: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct PluginCommand {
    pub id: PluginId,
    pub data: PluginCommandData,
}

#[derive(Clone, Debug)]
pub enum PluginCommandData {
    Stop
}

pub async fn start_plugin_runtime(data: PluginRuntimeData, run_status_guard: RunStatusGuard) -> anyhow::Result<()> {
    let conn = zbus::Connection::session().await?;
    let client_proxy = DbusClientProxyProxy::new(&conn).await?;

    let component_model = create_component_model();

    let plugin_id = data.id.clone();
    let view_created_signal = client_proxy.receive_view_created_signal()
        .await?
        .filter_map(move |signal: ViewCreatedSignal| {
            let plugin_id = plugin_id.clone();
            async move {
                let signal = signal.args().unwrap();

                // TODO add logging here that we received signal
                if PluginId::from_string(signal.plugin_id) != plugin_id {
                    None
                } else {
                    Some(JsUiEvent::ViewCreated {
                        reconciler_mode: signal.event.reconciler_mode,
                        view_name: signal.event.view_name,
                    })
                }
            }
        });

    let plugin_id = data.id.clone();
    let view_event_signal = client_proxy.receive_view_event_signal()
        .await?
        .filter_map(move |signal: ViewEventSignal| {
            let plugin_id = plugin_id.clone();
            async move {
                let signal = signal.args().unwrap();

                // TODO add logging here that we received signal
                if PluginId::from_string(signal.plugin_id) != plugin_id {
                    None
                } else {
                    Some(JsUiEvent::ViewEvent {
                        event_name: signal.event.event_name,
                        widget: signal.event.widget.into(),
                    })
                }
            }
        });

    let mut command_receiver = data.command_receiver;
    let command_stream = async_stream::stream! {
        loop {
            yield command_receiver.recv().await.unwrap();
        }
    };

    let plugin_id = data.id.clone();
    let command_stream = command_stream
        .filter_map(move |command: PluginCommand| {
            let plugin_id = plugin_id.clone();
            async move {
                let id = command.id;

                // TODO add logging here that we received signal
                if id != plugin_id {
                    None
                } else {
                    match command.data {
                        PluginCommandData::Stop => {
                            Some(JsUiEvent::PluginCommand {
                                command_type: "stop".to_string(),
                            })
                        }
                    }
                }
            }
        });

    let event_stream = (view_event_signal, view_created_signal, command_stream).merge();
    let event_stream = Box::pin(event_stream);

    let thread_fn = move || {
        let _run_status_guard = run_status_guard;

        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("unable to start tokio runtime for plugin")
            .block_on(tokio::task::unconstrained(async move {
                start_js_runtime(data.id, data.code, event_stream, client_proxy, component_model).await
            }));

        tracing::error!(target = "plugin", "runtime execution failed {:?}", result)
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
    event_stream: Pin<Box<dyn Stream<Item=JsUiEvent>>>,
    client_proxy: DbusClientProxyProxy<'static>,
    component_model: Vec<Component>
) -> anyhow::Result<()> {

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
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(CustomModuleLoader::new(code)),
            extensions: vec![plugin_ext::init_ops_and_esm(
                EventHandlers::new(),
                EventReceiver::new(event_stream),
                PluginData::new(plugin_id),
                DbusClient::new(client_proxy),
                ComponentModel::new(component_model),
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

const MODULES: [(&str, &str); 9] = [
    ("gauntlet:core:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/core/dist/prod/init.js"))),
    ("gauntlet:renderer:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react_renderer/dist/prod/renderer.js"))),
    ("gauntlet:react:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/prod/react.production.min.js"))),
    ("gauntlet:react-jsx-runtime:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/prod/react-jsx-runtime.production.min.js"))),
    ("gauntlet:core:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/core/dist/dev/init.js"))),
    ("gauntlet:renderer:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react_renderer/dist/dev/renderer.js"))),
    ("gauntlet:react:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/dev/react.development.js"))),
    ("gauntlet:react-jsx-runtime:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/dev/react-jsx-runtime.development.js"))),
    ("gauntlet:api-components", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/api/gendist/components.js"))),
];

impl ModuleLoader for CustomModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, anyhow::Error> {
        static PLUGIN_VIEW_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gauntlet:view\?(?<entrypoint_id>[a-zA-Z0-9_-]+)$").expect("invalid regex"));
        static PLUGIN_MODULE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^gauntlet:module\?(?<entrypoint_id>[a-zA-Z0-9_-]+)$").expect("invalid regex"));
        static PATH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\./(?<js_module>\w+)\.js$").expect("invalid regex"));

        if PLUGIN_VIEW_PATTERN.is_match(specifier) {
            return Ok(specifier.parse()?);
        }

        if PLUGIN_VIEW_PATTERN.is_match(referrer) || PLUGIN_MODULE_PATTERN.is_match(referrer) {
            if let Some(captures) = PATH_PATTERN.captures(specifier) {
                return Ok(format!("gauntlet:module?{}", &captures["js_module"]).parse()?);
            }
        }

        let specifier = match (specifier, referrer) {
            ("gauntlet:core", _) => "gauntlet:core:dev",
            ("gauntlet:renderer", _) => "gauntlet:renderer:dev",
            ("react", _) => "gauntlet:react:dev",
            ("react/jsx-runtime", _) => "gauntlet:react-jsx-runtime:dev",
            ("@project-gauntlet/api/components", _) => "gauntlet:api-components",
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

        if &specifier == &"gauntlet:view".parse().unwrap() || &specifier == &"gauntlet:module".parse().unwrap() {
            let module = get_js_code(module_specifier, &self.code.js);

            return futures::future::ready(module).boxed_local();
        }

        self.static_loader.load(module_specifier, maybe_referrer, is_dynamic)
    }
}

fn get_js_code(module_specifier: &ModuleSpecifier, js: &HashMap<String, String>) -> anyhow::Result<ModuleSource> {
    let view_name = module_specifier.query().expect("invalid specifier, should be validated earlier");

    let js = js.get(view_name).ok_or(anyhow!("no code provided for view: {:?}", view_name))?;

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
        op_plugin_get_pending_event,
        op_react_get_container,
        op_react_create_instance,
        op_react_create_text_instance,
        op_react_append_child,
        op_react_insert_before,
        op_react_remove_child,
        op_react_set_properties,
        op_react_set_text,
        op_react_call_event_listener,
        op_react_clone_instance,
        op_react_replace_container_children,
    ],
    options = {
        event_listeners: EventHandlers,
        event_receiver: EventReceiver,
        plugin_data: PluginData,
        dbus_client: DbusClient,
        component_model: ComponentModel,
    },
    state = |state, options| {
        state.put(options.event_listeners);
        state.put(options.event_receiver);
        state.put(options.plugin_data);
        state.put(options.dbus_client);
        state.put(options.component_model);
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
async fn op_plugin_get_pending_event<'a>(
    state: Rc<RefCell<OpState>>,
) -> anyhow::Result<JsUiEvent> {
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

    tracing::trace!(target = "renderer_rs_common", "Received plugin event {:?}", event);

    Ok(event)
}

#[op]
fn op_react_get_container(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsUiWidget> {
    tracing::trace!(target = "renderer_rs_common", "Calling op_react_get_container...");

    let container = match make_request(&state, JsUiRequestData::GetContainer)? {
        JsUiResponseData::GetContainer { container } => container,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    tracing::trace!(target = "renderer_rs_common", "Calling op_react_get_container returned {:?}", container);

    Ok(container.into())
}

#[op]
fn op_react_append_child(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_common", "Calling op_react_append_child...");

    let data = JsUiRequestData::AppendChild {
        parent,
        child,
    };

    match make_request(&state, data)? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_common", "Calling op_react_append_child returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn op_react_remove_child(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_remove_child...");

    let data = JsUiRequestData::RemoveChild {
        parent: parent.into(),
        child: child.into(),
    };

    match make_request(&state, data)? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_remove_child returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn op_react_insert_before(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
    before_child: JsUiWidget,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_insert_before...");

    let data = JsUiRequestData::InsertBefore {
        parent,
        child,
        before_child,
    };

    match make_request(&state, data)? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_insert_before returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op(v8)]
fn op_react_create_instance<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget_type: String,
    v8_properties: HashMap<String, serde_v8::Value<'a>>,
) -> anyhow::Result<JsUiWidget> {
    // TODO component model
    tracing::trace!(target = "renderer_rs_common", "Calling op_react_create_instance...");

    let properties = convert_properties(scope, v8_properties)?;

    let conversion_properties = properties.clone();

    let properties = properties.into_iter()
        .map(|(name, val)| (name, val.into()))
        .collect();

    let data = JsUiRequestData::CreateInstance {
        widget_type,
        properties,
    };

    let widget = match make_request(&state, data)? {
        JsUiResponseData::CreateInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    assign_event_listeners(&state, &widget, &conversion_properties);

    tracing::trace!(target = "renderer_rs_common", "Calling op_react_create_instance returned {:?}", widget);

    Ok(widget.into())
}

#[op]
fn op_react_create_text_instance(
    state: Rc<RefCell<OpState>>,
    text: String,
) -> anyhow::Result<JsUiWidget> {
    tracing::trace!(target = "renderer_rs_common", "Calling op_react_create_text_instance...");

    let data = JsUiRequestData::CreateTextInstance { text };

    let widget = match make_request(&state, data)? {
        JsUiResponseData::CreateTextInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    tracing::trace!(target = "renderer_rs_common", "Calling op_react_create_text_instance returned {:?}", widget);

    Ok(widget.into())
}

#[op(v8)]
fn op_react_set_properties<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    v8_properties: HashMap<String, serde_v8::Value<'a>>,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_set_properties...");

    let properties = convert_properties(scope, v8_properties)?;

    assign_event_listeners(&state, &widget, &properties);

    let properties = properties.into_iter()
        .map(|(name, val)| (name, val.into()))
        .collect();

    let data = JsUiRequestData::SetProperties {
        widget,
        properties,
    };

    match make_request(&state, data)? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_set_properties returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op(v8)]
fn op_react_call_event_listener(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    event_name: String,
) {
    tracing::trace!(target = "renderer_rs_common", "Calling op_react_call_event_listener...");

    let event_handlers = {
        state.borrow()
            .borrow::<EventHandlers>()
            .clone()
    };

    event_handlers.call_listener_handler(scope, &widget.widget_id, &event_name);

    tracing::trace!(target = "renderer_rs_common", "Calling op_react_call_event_listener returned");
}

#[op]
fn op_react_set_text(
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    text: String,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_set_text...");

    let data = JsUiRequestData::SetText {
        widget,
        text,
    };

    match make_request(&state, data)? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_mutation", "Calling op_react_set_text returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op]
fn op_react_replace_container_children(
    state: Rc<RefCell<OpState>>,
    container: JsUiWidget,
    new_children: Vec<JsUiWidget>,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_persistence", "Calling op_react_replace_container_children...");

    let data = JsUiRequestData::ReplaceContainerChildren {
        container,
        new_children,
    };

    match make_request(&state, data)? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_persistence", "Calling op_react_replace_container_children returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
}

#[op(v8)]
fn op_react_clone_instance<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    instance: JsUiWidget,
    update_payload: Vec<String>,
    widget_type: String,
    old_props: HashMap<String, serde_v8::Value<'a>>,
    new_props: HashMap<String, serde_v8::Value<'a>>,
    keep_children: bool,
) -> anyhow::Result<JsUiWidget> {
    tracing::trace!(target = "renderer_rs_persistence", "Calling op_react_clone_instance...");

    // TODO component model

    let old_props = convert_properties(scope, old_props)?;
    let new_props = convert_properties(scope, new_props)?;

    let new_props_clone = new_props.clone();

    let old_props = old_props.into_iter()
        .map(|(name, val)| (name, val.into()))
        .collect();
    let new_props = new_props.into_iter()
        .map(|(name, val)| (name, val.into()))
        .collect();

    let data = JsUiRequestData::CloneInstance {
        widget: instance,
        update_payload,
        widget_type,
        old_props,
        new_props,
        keep_children,
    };

    let widget = match make_request(&state, data)? {
        JsUiResponseData::CloneInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    assign_event_listeners(&state, &widget, &new_props_clone);

    tracing::trace!(target = "renderer_rs_persistence", "Calling op_react_clone_instance returned");

    Ok(widget.into())
}

fn make_request(state: &Rc<RefCell<OpState>>, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    let (plugin_id, dbus_client) = {
        let state = state.borrow();

        let plugin_id = state
            .borrow::<PluginData>()
            .plugin_id()
            .clone();

        let dbus_client = state
            .borrow::<DbusClient>()
            .client()
            .clone();

        (plugin_id, dbus_client)
    };

    block_on(async {
        make_request_async(plugin_id, dbus_client, data).await
    })
}

async fn make_request_async(plugin_id: PluginId, dbus_client: DbusClientProxyProxy<'_>, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    match data {
        JsUiRequestData::GetContainer => {
            let container = dbus_client.get_container(&plugin_id.to_string()) // TODO add timeout handling
                .await
                .map(|container| JsUiResponseData::GetContainer { container: container.into() })
                .map_err(|err| err.into());

            container
        }
        JsUiRequestData::CreateInstance { widget_type, properties } => {
            let widget = dbus_client.create_instance(&plugin_id.to_string(), &widget_type, to_dbus(properties))
                .await
                .map(|widget| JsUiResponseData::CreateInstance { widget: widget.into() })
                .map_err(|err| err.into());

            widget
        }
        JsUiRequestData::CreateTextInstance { text } => {
            let widget = dbus_client.create_text_instance(&plugin_id.to_string(), &text)
                .await
                .map(|widget| JsUiResponseData::CreateTextInstance { widget: widget.into() })
                .map_err(|err| err.into());

            widget
        }
        JsUiRequestData::AppendChild { parent, child } => {
            let nothing = dbus_client.append_child(&plugin_id.to_string(), parent.into(), child.into())
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
        JsUiRequestData::RemoveChild { parent, child } => {
            let nothing = dbus_client.remove_child(&plugin_id.to_string(), parent.into(), child.into())
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
        JsUiRequestData::InsertBefore { parent, child, before_child } => {
            let nothing = dbus_client.insert_before(&plugin_id.to_string(), parent.into(), child.into(), before_child.into())
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
        JsUiRequestData::SetProperties { widget, properties } => {
            let nothing = dbus_client.set_properties(&plugin_id.to_string(), widget.into(), to_dbus(properties))
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
        JsUiRequestData::SetText { widget, text } => {
            let nothing = dbus_client.set_text(&plugin_id.to_string(), widget.into(), &text)
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
        JsUiRequestData::CloneInstance {
            widget,
            update_payload,
            widget_type,
            old_props,
            new_props,
            keep_children
        } => {
            let widget = dbus_client.clone_instance(&plugin_id.to_string(), widget.into(), update_payload, &widget_type, to_dbus(old_props), to_dbus(new_props), keep_children)
                .await
                .map(|widget| JsUiResponseData::CloneInstance { widget: widget.into() })
                .map_err(|err| err.into());

            widget
        }
        JsUiRequestData::ReplaceContainerChildren { container, new_children } => {
            let new_children = new_children.into_iter()
                .map(|child| child.into())
                .collect();

            let nothing = dbus_client.replace_container_children(&plugin_id.to_string(), container.into(), new_children)
                .await
                .map(|_| JsUiResponseData::Nothing)
                .map_err(|err| err.into());

            nothing
        }
    }
}

#[derive(Clone)]
pub enum ConversionPropertyValue {
    Function(v8::Global<v8::Function>),
    String(String),
    Number(f64),
    Bool(bool),
}

impl From<ConversionPropertyValue> for JsUiPropertyValue {
    fn from(value: ConversionPropertyValue) -> Self {
        match value {
            ConversionPropertyValue::Function(_) => JsUiPropertyValue::Function,
            ConversionPropertyValue::String(value) => JsUiPropertyValue::String(value),
            ConversionPropertyValue::Number(value) => JsUiPropertyValue::Number(value),
            ConversionPropertyValue::Bool(value) => JsUiPropertyValue::Bool(value),
        }
    }
}


fn convert_properties(
    scope: &mut v8::HandleScope,
    v8_properties: HashMap<String, serde_v8::Value>,
) -> anyhow::Result<HashMap<String, ConversionPropertyValue>> {
    let vec = v8_properties.into_iter()
        .filter(|(name, _)| name.as_str() != "children")
        .map(|(name, value)| {
            let val = value.v8_value;
            if val.is_function() {
                let fn_value: v8::Local<v8::Function> = val.try_into()?;
                let global_fn = v8::Global::new(scope, fn_value);

                Ok((name, ConversionPropertyValue::Function(global_fn)))
            } else if val.is_string() {
                Ok((name, ConversionPropertyValue::String(val.to_rust_string_lossy(scope))))
            } else if val.is_number() {
                Ok((name, ConversionPropertyValue::Number(val.number_value(scope).expect("expected number"))))
            } else if val.is_boolean() {
                Ok((name, ConversionPropertyValue::Bool(val.boolean_value(scope))))
            } else {
                Err(anyhow!("{:?}: {:?}", name, val.type_of(scope).to_rust_string_lossy(scope)))
            }
        })
        .collect::<Result<Vec<(_, _)>, anyhow::Error>>()?;

    Ok(vec.into_iter().collect())
}

fn assign_event_listeners(
    state: &Rc<RefCell<OpState>>,
    widget: &JsUiWidget,
    properties: &HashMap<String, ConversionPropertyValue>
) {
    let mut state_ref = state.borrow_mut();
    let event_listeners = state_ref.borrow_mut::<EventHandlers>();

    for (name, value) in properties {
        match value {
            ConversionPropertyValue::Function(global_fn) => {
                event_listeners.add_listener(widget.widget_id, name.clone(), global_fn.clone());
            }
            _ => {}
        }
    }
}


pub struct PluginData {
    plugin_id: PluginId,
}

impl PluginData {
    fn new(plugin_id: PluginId) -> Self {
        Self { plugin_id }
    }

    fn plugin_id(&self) -> PluginId {
        self.plugin_id.clone()
    }
}

pub struct DbusClient {
    proxy: DbusClientProxyProxy<'static>,
}

impl DbusClient {
    fn new(proxy: DbusClientProxyProxy<'static>) -> Self {
        Self { proxy }
    }

    fn client(&self) -> &DbusClientProxyProxy<'static> {
        &self.proxy
    }
}

pub struct ComponentModel {
    components: HashMap<String, Component>,
}

impl ComponentModel {
    fn new(components: Vec<Component>) -> Self {
        Self { components: components.into_iter().map(|component| (component.internal_name().to_owned(), component)).collect() }
    }

    fn component(&self, internal_name: &str) -> Option<&Component> {
        self.components.get(internal_name)
    }
}

pub struct EventReceiver {
    event_stream: Rc<RefCell<Pin<Box<dyn Stream<Item=JsUiEvent>>>>>,
}

impl EventReceiver {
    fn new(event_stream: Pin<Box<dyn Stream<Item=JsUiEvent>>>) -> EventReceiver {
        Self {
            event_stream: Rc::new(RefCell::new(event_stream)),
        }
    }
}


#[derive(Clone)]
pub struct EventHandlers {
    inner: Rc<RefCell<HashMap<JsUiWidgetId, HashMap<JsUiEventName, v8::Global<v8::Function>>>>>,
}

impl EventHandlers {
    fn new() -> EventHandlers {
        Self {
            inner: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    fn add_listener(&mut self, widget: JsUiWidgetId, event_name: JsUiEventName, function: v8::Global<v8::Function>) {
        let mut inner = self.inner.borrow_mut();
        inner.entry(widget).or_default().insert(event_name, function);
    }

    fn call_listener_handler(&self, scope: &mut v8::HandleScope, widget: &JsUiWidgetId, event_name: &JsUiEventName) {
        let inner = self.inner.borrow();
        let option_func = inner.get(widget)
            .map(|handlers| handlers.get(event_name))
            .flatten();

        if let Some(func) = option_func {
            let local_fn = v8::Local::new(scope, func);
            scope.enqueue_microtask(local_fn); // TODO call straight away instead of enqueue?
        };
    }
}

