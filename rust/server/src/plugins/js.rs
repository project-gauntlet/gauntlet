use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;

use anyhow::{anyhow, Context};
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
use component_model::{Children, Component, create_component_model, PropertyType};

use crate::dbus::{DbusClientProxyProxy, ViewCreatedSignal, ViewEventSignal};
use crate::model::{IntermediatePropertyValue, IntermediateUiWidget, JsUiEvent, JsUiRequestData, JsUiResponseData, JsUiWidget};
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
                        widget_id: signal.event.widget_id,
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

const MODULES: [(&str, &str); 8] = [
    ("gauntlet:renderer:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react_renderer/dist/prod/renderer.js"))),
    ("gauntlet:renderer:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react_renderer/dist/dev/renderer.js"))),
    ("gauntlet:react:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/prod/react.production.min.js"))),
    ("gauntlet:react:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/dev/react.development.js"))),
    ("gauntlet:react-jsx-runtime:prod", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/prod/react-jsx-runtime.production.min.js"))),
    ("gauntlet:react-jsx-runtime:dev", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/react/dist/dev/react-jsx-runtime.development.js"))),
    ("gauntlet:core", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../js/core/dist/init.js"))),
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
        static PATH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\./(?<js_module>[a-zA-Z0-9_-]+)\.js$").expect("invalid regex"));

        if PLUGIN_VIEW_PATTERN.is_match(specifier) {
            return Ok(specifier.parse()?);
        }

        if PLUGIN_VIEW_PATTERN.is_match(referrer) || PLUGIN_MODULE_PATTERN.is_match(referrer) {
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
        op_react_replace_container_children,
    ],
    options = {
        event_receiver: EventReceiver,
        plugin_data: PluginData,
        dbus_client: DbusClient,
        component_model: ComponentModel,
    },
    state = |state, options| {
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

#[op(v8)]
fn op_react_replace_container_children(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    container: JsUiWidget,
    new_children: Vec<JsUiWidget>,
) -> anyhow::Result<()> {
    tracing::trace!(target = "renderer_rs_persistence", "Calling op_react_replace_container_children...");

    for new_child in &new_children {
        validate_child(&state, &container.widget_type, &new_child.widget_type)?
    }

    let new_children = new_children.into_iter()
        .map(|child| from_js_to_intermediate_widget(scope, child))
        .collect::<anyhow::Result<Vec<IntermediateUiWidget>>>()?;

    let data = JsUiRequestData::ReplaceContainerChildren {
        container: from_js_to_intermediate_widget(scope, container)?,
        new_children,
    };

    match make_request(&state, data).context("ReplaceContainerChildren frontend response")? {
        JsUiResponseData::Nothing => {
            tracing::trace!(target = "renderer_rs_persistence", "Calling op_react_replace_container_children returned");
            Ok(())
        },
        value @ _ => panic!("unsupported response type {:?}", value),
    }
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

fn validate_properties(state: &Rc<RefCell<OpState>>, internal_name: &str, properties: &HashMap<String, IntermediatePropertyValue>) -> anyhow::Result<()> {
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
                            IntermediatePropertyValue::Function(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Function) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a function", comp_prop.name, name))?
                                }
                            }
                            IntermediatePropertyValue::String(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::String) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a string", comp_prop.name, name))?
                                }
                            }
                            IntermediatePropertyValue::Number(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Number) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a number", comp_prop.name, name))?
                                }
                            }
                            IntermediatePropertyValue::Bool(_) => {
                                if !matches!(comp_prop.property_type, PropertyType::Boolean) {
                                    Err(anyhow::anyhow!("property {} on {} component has to be a boolean", comp_prop.name, name))?
                                }
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
                                .map(|member| (&member.component_internal_name, member))
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
                                .map(|member| (&member.component_internal_name, member))
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

async fn make_request_async(plugin_id: PluginId, dbus_client: DbusClientProxyProxy<'_>, data: JsUiRequestData) -> anyhow::Result<JsUiResponseData> {
    match data {
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
) -> anyhow::Result<HashMap<String, IntermediatePropertyValue>> {
    let vec = v8_properties.into_iter()
        .filter(|(name, _)| name.as_str() != "children")
        .map(|(name, value)| {
            let val = value.v8_value;
            if val.is_function() {
                let fn_value: v8::Local<v8::Function> = val.try_into()?;
                let global_fn = v8::Global::new(scope, fn_value);

                Ok((name, IntermediatePropertyValue::Function(global_fn)))
            } else if val.is_string() {
                Ok((name, IntermediatePropertyValue::String(val.to_rust_string_lossy(scope))))
            } else if val.is_number() {
                Ok((name, IntermediatePropertyValue::Number(val.number_value(scope).expect("expected number"))))
            } else if val.is_boolean() {
                Ok((name, IntermediatePropertyValue::Bool(val.boolean_value(scope))))
            } else {
                Err(anyhow!("invalid type for property '{:?}' - {:?}", name, val.type_of(scope).to_rust_string_lossy(scope)))
            }
        })
        .collect::<anyhow::Result<Vec<(_, _)>>>()?;

    Ok(vec.into_iter().collect())
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
        Self {
            components: components.into_iter()
                .filter_map(|component| {
                    match &component {
                        Component::Standard { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                        Component::Root { internal_name, .. } => Some((format!("gauntlet:{}", internal_name), component)),
                        Component::TextPart { internal_name } => Some((format!("gauntlet:{}", internal_name), component)),
                    }
                })
                .collect()
        }
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
