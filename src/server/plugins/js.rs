use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use anyhow::anyhow;
use deno_core::{FastString, futures, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleType, op, OpState, ResolutionKind, serde_v8, StaticModuleLoader, v8};
use deno_core::futures::{FutureExt, Stream, StreamExt};
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use futures_concurrency::stream::Merge;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

use crate::client::dbus::{DbusClientProxyProxy, ViewCreatedSignal, ViewEventSignal};
use crate::server::plugins::Plugin;
use crate::utils::channel::{channel, RequestSender};

pub async fn start_js_runtime(plugin: Plugin) -> anyhow::Result<()> {
    let conn = zbus::Connection::session().await?;
    let client_proxy = DbusClientProxyProxy::new(&conn).await?;

    let plugin_uuid = plugin.id().to_owned();
    let view_created_signal = client_proxy.receive_view_created_signal()
        .await?
        .filter_map(move |signal: ViewCreatedSignal| {
            let plugin_uuid = plugin_uuid.clone();
            async move {
                let signal = signal.args().unwrap();

                if signal.plugin_uuid != plugin_uuid {
                    None
                } else {
                    Some(UiEvent::ViewCreated {
                        view_name: signal.event.view_name
                    })
                }
            }
        });

    let plugin_uuid = plugin.id().to_owned();
    let view_event_signal = client_proxy.receive_view_event_signal()
        .await?
        .filter_map(move |signal: ViewEventSignal| {
            let plugin_uuid = plugin_uuid.clone();
            async move {
                let signal = signal.args().unwrap();

                if signal.plugin_uuid != plugin_uuid {
                    None
                } else {
                    Some(UiEvent::ViewEvent {
                        event_name: signal.event.event_name,
                        widget_id: signal.event.widget_id,
                    })
                }
            }
        });

    let event_stream = (view_event_signal, view_created_signal).merge();

    let (tx, mut rx) = channel::<UiRequestData, UiResponseData>();

    let plugin_uuid: String = plugin.id().to_owned();
    tokio::spawn(tokio::task::unconstrained(async move {
        println!("starting request handler loop");

        while let Ok((request_data, responder)) = rx.recv().await {
            match request_data {
                UiRequestData::GetContainer => {
                    let container = client_proxy.get_container(&plugin_uuid) // TODO add timeout handling
                        .await
                        .unwrap()
                        .into();
                    responder.respond(UiResponseData::GetContainer { container }).unwrap()
                }
                UiRequestData::CreateInstance { widget_type } => {
                    let widget = client_proxy.create_instance(&plugin_uuid, &widget_type)
                        .await
                        .unwrap()
                        .into();
                    responder.respond(UiResponseData::CreateInstance { widget }).unwrap()
                }
                UiRequestData::CreateTextInstance { text } => {
                    let widget = client_proxy.create_text_instance(&plugin_uuid, &text)
                        .await
                        .unwrap()
                        .into();

                    responder.respond(UiResponseData::CreateTextInstance { widget }).unwrap()
                }
                UiRequestData::AppendChild { parent, child } => {
                    client_proxy.append_child(&plugin_uuid, parent.into(), child.into())
                        .await
                        .unwrap();
                }
                UiRequestData::RemoveChild { parent, child } => {
                    client_proxy.remove_child(&plugin_uuid, parent.into(), child.into())
                        .await
                        .unwrap();
                }
                UiRequestData::InsertBefore { parent, child, before_child } => {
                    client_proxy.insert_before(&plugin_uuid, parent.into(), child.into(), before_child.into())
                        .await
                        .unwrap();
                }
                UiRequestData::SetProperties { widget, properties } => {
                    client_proxy.set_properties(&plugin_uuid, widget.into(), properties.into())
                        .await
                        .unwrap();
                }
                UiRequestData::SetText { widget, text } => {
                    client_proxy.set_text(&plugin_uuid, widget.into(), &text)
                        .await
                        .unwrap();
                }
            }
        }
        println!("stopped request handler loop");
    }));

    // let _inspector_server = Arc::new(
    //     InspectorServer::new(
    //         "127.0.0.1:9229".parse::<SocketAddr>().unwrap(),
    //         "test",
    //     )
    // );

    let mut worker = MainWorker::bootstrap_from_options(
        "plugin:unused".parse().unwrap(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(CustomModuleLoader::new(plugin)),
            extensions: vec![gtk_ext::init_ops_and_esm(
                EventHandlers::new(),
                EventReceiver::new(Box::pin(event_stream)),
                RequestSender1::new(tx),
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

    worker.execute_side_module(&"plugin:core".parse().unwrap()).await.unwrap();
    worker.run_event_loop(false).await.unwrap();

    Ok(())
}

pub struct CustomModuleLoader {
    plugin: Plugin,
    static_loader: StaticModuleLoader,
}

impl CustomModuleLoader {
    fn new(plugin: Plugin) -> Self {
        let module_map: HashMap<_, _> = MODULES.iter()
            .map(|(key, value)| (key.parse().unwrap(), FastString::from_static(value)))
            .collect();
        Self {
            plugin,
            static_loader: StaticModuleLoader::new(module_map),
        }
    }
}

const MODULES: [(&str, &str); 4] = [
    ("plugin:core", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/core/dist/prod/init.js"))),
    ("plugin:renderer", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/react_renderer/dist/prod/renderer.js"))),
    ("plugin:react", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/react/dist/prod/react.production.min.js"))), // TODO dev https://github.com/rollup/plugins/issues/1546
    ("plugin:react-jsx-runtime", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/react/dist/prod/react-jsx-runtime.production.min.js"))),
];

impl ModuleLoader for CustomModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, anyhow::Error> {
        static PLUGIN_VIEW_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^plugin:view\?(?<entrypoint_id>[a-zA-Z0-9_-]+)$").unwrap());
        static PLUGIN_MODULE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^plugin:module\?(?<entrypoint_id>[a-zA-Z0-9_-]+)$").unwrap());
        static PATH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\./(?<js_module>\w+)\.js$").unwrap());

        if PLUGIN_VIEW_PATTERN.is_match(specifier) {
            return Ok(specifier.parse()?);
        }

        if PLUGIN_VIEW_PATTERN.is_match(referrer) || PLUGIN_MODULE_PATTERN.is_match(referrer) {
            if let Some(captures) = PATH_PATTERN.captures(specifier) {
                return Ok(format!("plugin:module?{}", &captures["js_module"]).parse()?);
            }
        }

        let specifier = match (specifier, referrer) {
            ("plugin:core", _) => "plugin:core",
            ("plugin:renderer", _) => "plugin:renderer",
            ("react", _) => "plugin:react",
            ("react/jsx-runtime", _) => "plugin:react-jsx-runtime",
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

        if &specifier == &"plugin:view".parse().unwrap() || &specifier == &"plugin:module".parse().unwrap() {
            let view_name = module_specifier.query().unwrap();

            let js = self.plugin.code().js();
            let js = js.get(view_name).unwrap();

            let module = ModuleSource::new(ModuleType::JavaScript, js.to_owned().into(), module_specifier);

            return futures::future::ready(Ok(module)).boxed_local();
        }

        self.static_loader.load(module_specifier, maybe_referrer, is_dynamic)
    }
}


deno_core::extension!(
    gtk_ext,
    ops = [
        op_gtk_get_container,
        op_gtk_create_instance,
        op_gtk_create_text_instance,
        op_gtk_append_child,
        op_gtk_insert_before,
        op_gtk_remove_child,
        op_gtk_set_properties,
        op_gtk_set_text,
        op_get_next_pending_ui_event,
        op_call_event_listener,
    ],
    options = {
        event_listeners: EventHandlers,
        event_receiver: EventReceiver,
        request_sender: RequestSender1,
    },
    state = |state, options| {
        state.put(options.event_listeners);
        state.put(options.event_receiver);
        state.put(options.request_sender);
    },
);



#[op]
async fn op_gtk_get_container(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsUiWidget> {
    println!("op_gtk_get_container");

    let container = match make_request_receive(&state, UiRequestData::GetContainer).await? {
        UiResponseData::GetContainer { container } => container,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    println!("op_gtk_get_container end");

    Ok(container.into())
}

#[op]
async fn op_gtk_append_child(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
) -> anyhow::Result<()> {
    println!("op_gtk_append_child");

    let data = UiRequestData::AppendChild {
        parent: parent.into(),
        child: child.into(),
    };

    let _ = make_request(&state, data)?;

    println!("op_gtk_append_child end");

    Ok(())
}

#[op]
async fn op_gtk_remove_child(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
) -> anyhow::Result<()> {
    println!("op_gtk_remove_child");

    let data = UiRequestData::RemoveChild {
        parent: parent.into(),
        child: child.into(),
    };

    let _ = make_request(&state, data)?;

    println!("op_gtk_remove_child end");

    Ok(())
}

#[op]
async fn op_gtk_insert_before(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
    before_child: JsUiWidget,
) -> anyhow::Result<()> {
    println!("op_gtk_insert_before");

    let data = UiRequestData::InsertBefore {
        parent: parent.into(),
        child: child.into(),
        before_child: before_child.into(),
    };

    let _ = make_request(&state, data)?;

    println!("op_gtk_insert_before end");

    Ok(())
}

#[op]
async fn op_gtk_create_instance(
    state: Rc<RefCell<OpState>>,
    widget_type: String,
) -> anyhow::Result<JsUiWidget> {
    println!("op_gtk_create_instance");

    let data = UiRequestData::CreateInstance {
        widget_type,
    };

    let widget = match make_request_receive(&state, data).await? {
        UiResponseData::CreateInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };
    println!("op_gtk_create_instance end");

    Ok(widget.into())
}

#[op]
async fn op_gtk_create_text_instance(
    state: Rc<RefCell<OpState>>,
    text: String,
) -> anyhow::Result<JsUiWidget> {
    println!("op_gtk_create_text_instance");

    let data = UiRequestData::CreateTextInstance { text };

    let widget = match make_request_receive(&state, data).await? {
        UiResponseData::CreateTextInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    println!("op_gtk_create_text_instance end");

    Ok(widget.into())
}

#[op(v8)]
fn op_gtk_set_properties<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    props: HashMap<String, serde_v8::Value<'a>>,
) -> anyhow::Result<impl Future<Output=anyhow::Result<()>> + 'static> {
    println!("op_gtk_set_properties");

    let mut state_ref = state.borrow_mut();
    let event_listeners = state_ref.borrow_mut::<EventHandlers>();

    let properties = props.iter()
        .filter(|(name, _)| name.as_str() != "children")
        .map(|(name, value)| {
            let val = value.v8_value;
            if val.is_function() {
                let fn_value: v8::Local<v8::Function> = val.try_into().unwrap();
                let global_fn = v8::Global::new(scope, fn_value);
                event_listeners.add_listener(widget.widget_id, name.clone(), global_fn);
                (name.clone(), UiPropertyValue::Function)
            } else if val.is_string() {
                (name.clone(), UiPropertyValue::String(val.to_rust_string_lossy(scope)))
            } else if val.is_number() {
                (name.clone(), UiPropertyValue::Number(val.number_value(scope).unwrap()))
            } else if val.is_boolean() {
                (name.clone(), UiPropertyValue::Bool(val.boolean_value(scope)))
            } else {
                panic!("{:?}: {:?}", name, val.type_of(scope).to_rust_string_lossy(scope))
            }
        })
        .collect::<HashMap<_, _>>();

    let data = UiRequestData::SetProperties {
        widget: widget.into(),
        properties,
    };

    drop(state_ref);

    println!("op_gtk_set_properties end");

    Ok(async move {
        let _ = make_request(&state, data)?;

        Ok(())
    })
}

#[op]
async fn op_get_next_pending_ui_event<'a>(
    state: Rc<RefCell<OpState>>,
) -> anyhow::Result<JsUiEvent> {
    let event_stream = {
        state.borrow()
            .borrow::<EventReceiver>()
            .event_stream
            .clone()
    };

    println!("op_get_next_pending_ui_event");

    let mut event_stream = event_stream.borrow_mut();
    let event = event_stream.next()
        .await
        .ok_or_else(|| anyhow!("event stream was suddenly closed"))?
        .into();
    Ok(event)
}

#[op(v8)]
fn op_call_event_listener(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    event_name: String,
) {
    println!("op_call_event_listener");

    let event_handlers = {
        state.borrow()
            .borrow::<EventHandlers>()
            .clone()
    };

    event_handlers.call_listener_handler(scope, &widget.widget_id, &event_name);

    println!("op_call_event_listener end");
}

#[op]
async fn op_gtk_set_text(
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    text: String,
) -> anyhow::Result<()> {
    println!("op_gtk_set_text");

    let data = UiRequestData::SetText {
        widget: widget.into(),
        text,
    };

    println!("op_gtk_set_text end");

    let _ = make_request(&state, data)?;

    Ok(())
}

async fn make_request_receive(state: &Rc<RefCell<OpState>>, data: UiRequestData) -> anyhow::Result<UiResponseData> {
    let request_sender = {
        state.borrow()
            .borrow::<RequestSender1>()
            .clone()
    };

    let result = request_sender.channel.send_receive(data).await?;

    Ok(result)
}

fn make_request(state: &Rc<RefCell<OpState>>, data: UiRequestData) -> anyhow::Result<()> {
    let request_sender = {
        state.borrow()
            .borrow::<RequestSender1>()
            .clone()
    };

    let _ = request_sender.channel.send(data)?;

    Ok(())
}


#[derive(Clone)]
pub struct RequestSender1 {
    channel: RequestSender<UiRequestData, UiResponseData>,
}

impl RequestSender1 {
    fn new(channel: RequestSender<UiRequestData, UiResponseData>) -> Self {
        Self { channel }
    }
}

pub struct EventReceiver {
    event_stream: Rc<RefCell<Pin<Box<dyn Stream<Item=UiEvent>>>>>,
}

impl EventReceiver {
    fn new(event_stream: Pin<Box<dyn Stream<Item=UiEvent>>>) -> EventReceiver {
        Self {
            event_stream: Rc::new(RefCell::new(event_stream)),
        }
    }
}


#[derive(Clone)]
pub struct EventHandlers {
    inner: Rc<RefCell<EventHandlersInner>>,
}

pub struct EventHandlersInner {
    listeners: HashMap<UiWidgetId, HashMap<UiEventName, v8::Global<v8::Function>>>,
}

impl EventHandlers {
    fn new() -> EventHandlers {
        Self {
            inner: Rc::new(RefCell::new(
                EventHandlersInner {
                    listeners: HashMap::new()
                }
            ))
        }
    }

    fn add_listener(&mut self, widget: UiWidgetId, event_name: UiEventName, function: v8::Global<v8::Function>) {
        let mut inner = self.inner.borrow_mut();
        inner.listeners.entry(widget).or_default().insert(event_name, function);
    }

    fn call_listener_handler(&self, scope: &mut v8::HandleScope, widget: &UiWidgetId, event_name: &UiEventName) {
        let inner = self.inner.borrow();
        let option_func = inner.listeners.get(widget)
            .map(|handlers| handlers.get(event_name))
            .flatten();

        if let Some(func) = option_func {
            let local_fn = v8::Local::new(scope, func);
            scope.enqueue_microtask(local_fn); // TODO call straight away instead of enqueue?
        };
    }
}


#[derive(Debug)]
pub enum UiResponseData {
    GetContainer {
        container: UiWidget
    },
    CreateInstance {
        widget: UiWidget
    },
    CreateTextInstance {
        widget: UiWidget
    },
    Unit,
}

#[derive(Debug)]
pub enum UiRequestData {
    GetContainer,
    CreateInstance {
        widget_type: String,
    },
    CreateTextInstance {
        text: String,
    },
    AppendChild {
        parent: UiWidget,
        child: UiWidget,
    },
    RemoveChild {
        parent: UiWidget,
        child: UiWidget,
    },
    InsertBefore {
        parent: UiWidget,
        child: UiWidget,
        before_child: UiWidget,
    },
    SetProperties {
        widget: UiWidget,
        properties: HashMap<String, UiPropertyValue>,
    },
    SetText {
        widget: UiWidget,
        text: String,
    },
}

#[derive(Debug)]
pub enum UiPropertyValue {
    Function,
    String(String),
    Number(f64),
    Bool(bool),
}

pub type UiWidgetId = u32;
pub type UiEventName = String;

#[derive(Debug)]
pub enum UiEvent {
    ViewCreated {
        view_name: String
    },
    ViewDestroyed,
    ViewEvent {
        event_name: UiEventName,
        widget_id: UiWidgetId,
    },
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct UiEventViewCreated {
    pub view_name: String,
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct UiEventViewEvent {
    pub event_name: UiEventName,
    pub widget_id: UiWidgetId,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum JsUiEvent {
    ViewCreated {
        #[serde(rename = "viewName")]
        view_name: String
    },
    ViewDestroyed,
    ViewEvent {
        widget: JsUiWidget,
        #[serde(rename = "eventName")]
        event_name: UiEventName,
    },
}

impl From<UiEvent> for JsUiEvent {
    fn from(value: UiEvent) -> Self {
        match value {
            UiEvent::ViewCreated { view_name } => JsUiEvent::ViewCreated { view_name },
            UiEvent::ViewDestroyed => JsUiEvent::ViewDestroyed,
            UiEvent::ViewEvent {
                event_name,
                widget_id
            } => JsUiEvent::ViewEvent {
                event_name,
                widget: JsUiWidget {
                    widget_id
                },
            }
        }
    }
}

#[derive(Debug)]
pub struct UiWidget {
    pub widget_id: UiWidgetId,
}

impl From<UiWidget> for JsUiWidget {
    fn from(value: UiWidget) -> Self {
        Self {
            widget_id: value.widget_id
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct JsUiWidget {
    #[serde(rename = "widgetId")]
    widget_id: UiWidgetId,
}

impl From<JsUiWidget> for UiWidget {
    fn from(value: JsUiWidget) -> Self {
        Self {
            widget_id: value.widget_id
        }
    }
}
