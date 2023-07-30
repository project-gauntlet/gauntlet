use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::{Future, poll_fn};
use std::net::SocketAddr;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::task::Poll;

use deno_core::{futures, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleType, op, OpState, ResolutionKind, serde_v8, v8};
use deno_core::anyhow::anyhow;
use deno_core::futures::FutureExt;
use deno_core::futures::task::AtomicWaker;
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::inspector_server::InspectorServer;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

use crate::plugins::Plugin;

pub async fn run_react(react_context: PluginReactContext) {
    let _inspector_server = Arc::new(
        InspectorServer::new(
            "127.0.0.1:9229".parse::<SocketAddr>().unwrap(),
            "test",
        )
    );

    let worker = MainWorker::bootstrap_from_options(
        "plugin:unused".parse().unwrap(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(CustomModuleLoader::new(react_context.plugin)),
            extensions: vec![gtk_ext::init_ops_and_esm(
                EventHandlers::new(),
                EventReceiver::new(react_context.event_receiver, react_context.event_receiver_waker),
                RequestSender::new(react_context.request_sender),
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

    let worker = Rc::new(RefCell::new(worker));

    task::spawn_local(async move {
        let mut worker = worker.borrow_mut();
        worker.execute_side_module(&"plugin:core".parse().unwrap()).await.unwrap();
        worker.run_event_loop(false).await.unwrap();
    });
}

pub struct CustomModuleLoader {
    plugin: Plugin,
    inner: FsModuleLoader,
}

impl CustomModuleLoader {
    fn new(plugin: Plugin) -> Self {
        Self {
            plugin,
            inner: FsModuleLoader
        }
    }
}

impl ModuleLoader for CustomModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_core::anyhow::Error> {

        if specifier.starts_with("plugin:view?") {
            return Ok(specifier.parse()?)
        };

        if specifier == "./vendor.js" && referrer.starts_with("plugin:view?") {
            return Ok("plugin:view?vendor".parse()?)
        };

        let specifier = match (specifier, referrer) {
            ("plugin:core", _) => "ext:gtk_ext/core/dist/prod/init.js",
            ("plugin:renderer", _) => "ext:gtk_ext/react_renderer/dist/prod/renderer.js",
            ("react", _) => "ext:gtk_ext/react/dist/prod/react.production.min.js",
            ("react/jsx-runtime", _) => "ext:gtk_ext/react/dist/prod/react-jsx-runtime.production.min.js",
            _ => {
                return Err(anyhow!("Could not resolve module with specifier: {} and referrer: {}", specifier, referrer));
            }
        };

        return Ok(specifier.parse()?);
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleSpecifier>,
        is_dynamic: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {

        let mut specifier = module_specifier.clone();
        specifier.set_query(None);

        if &specifier == &"plugin:view".parse().unwrap() {
            let view_name = module_specifier.query().unwrap();

            let js = self.plugin.code().js();
            let js = js.get(view_name).unwrap();

            let module = ModuleSource::new(ModuleType::JavaScript, js.to_owned().into(), module_specifier);

            return futures::future::ready(Ok(module)).boxed_local()
        }


        self.inner.load(module_specifier, maybe_referrer, is_dynamic)
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
    esm_entry_point = "ext:gtk_ext/entry_point.js",
    esm = [
        dir "../js",
        "entry_point.js",
        "react_renderer/dist/prod/renderer.js",
        "react/dist/prod/react.production.min.js", // TODO dev https://github.com/rollup/plugins/issues/1546
        "react/dist/prod/react-jsx-runtime.production.min.js",
        "core/dist/prod/init.js",
    ],
    options = {
        event_listeners: EventHandlers,
        event_receiver: EventReceiver,
        request_sender: RequestSender,
    },
    state = |state, options| {
        state.put(options.event_listeners);
        state.put(options.event_receiver);
        state.put(options.request_sender);
    },
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
        ext.force_op_registration();
    },
);



#[op]
async fn op_gtk_get_container(state: Rc<RefCell<OpState>>) -> JsUiWidget {
    println!("op_gtk_get_container");

    let container = match make_request(&state, UiRequestData::GetContainer).await {
        UiResponseData::GetContainer { container } => container,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    println!("op_gtk_get_container end");

    container.into()
}

#[op]
async fn op_gtk_append_child(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
) {
    println!("op_gtk_append_child");

    let data = UiRequestData::AppendChild {
        parent: parent.into(),
        child: child.into(),
    };

    let _ = make_request(&state, data).await;
}

#[op]
async fn op_gtk_remove_child(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
) {
    println!("op_gtk_remove_child");

    let data = UiRequestData::RemoveChild {
        parent: parent.into(),
        child: child.into(),
    };

    let _ = make_request(&state, data).await;
}

#[op]
async fn op_gtk_insert_before(
    state: Rc<RefCell<OpState>>,
    parent: JsUiWidget,
    child: JsUiWidget,
    before_child: JsUiWidget,
) {
    println!("op_gtk_insert_before");

    let data = UiRequestData::InsertBefore {
        parent: parent.into(),
        child: child.into(),
        before_child: before_child.into(),
    };

    let _ = make_request(&state, data);
}

#[op]
async fn op_gtk_create_instance(
    state: Rc<RefCell<OpState>>,
    widget_type: String,
) -> JsUiWidget {
    println!("op_gtk_create_instance");

    let data = UiRequestData::CreateInstance {
        widget_type,
    };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };
    println!("op_gtk_create_instance end");

    widget.into()
}

#[op]
async fn op_gtk_create_text_instance(
    state: Rc<RefCell<OpState>>,
    text: String,
) -> JsUiWidget {
    println!("op_gtk_create_text_instance");

    let data = UiRequestData::CreateTextInstance { text };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateTextInstance { widget } => widget,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    return widget.into();
}

#[op(v8)]
fn op_gtk_set_properties<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    props: HashMap<String, serde_v8::Value<'a>>,
) -> Result<impl Future<Output=Result<(), deno_core::anyhow::Error>> + 'static, deno_core::anyhow::Error> {
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
                (name.clone(), PropertyValue::Function)
            } else if val.is_string() {
                (name.clone(), PropertyValue::String(val.to_rust_string_lossy(scope)))
            } else if val.is_number() {
                (name.clone(), PropertyValue::Number(val.number_value(scope).unwrap()))
            } else if val.is_boolean() {
                (name.clone(), PropertyValue::Bool(val.boolean_value(scope)))
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

    Ok(async move {
        let _ = make_request(&state, data).await;

        Ok(())
    })
}

#[op]
async fn op_get_next_pending_ui_event<'a>(
    state: Rc<RefCell<OpState>>,
) -> JsUiEvent {
    let event_receiver = {
        state.borrow()
            .borrow::<EventReceiver>()
            .clone()
    };

    poll_fn(|cx| {
        event_receiver.waker.register(cx.waker());
        let receiver = event_receiver.inner.borrow();

        match receiver.try_recv() {
            Ok(value) => {
                println!("Poll::Ready {:?}", value);
                Poll::Ready(value.into())
            }
            Err(TryRecvError::Disconnected) => panic!("disconnected"),
            Err(TryRecvError::Empty) => Poll::Pending
        }
    }).await
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

    event_handlers.call_listener_handler(scope, &widget.widget_id, &event_name)
}

#[op]
async fn op_gtk_set_text(
    state: Rc<RefCell<OpState>>,
    widget: JsUiWidget,
    text: String,
) {
    println!("op_gtk_set_text");

    let data = UiRequestData::SetText {
        widget: widget.into(),
        text,
    };

    let _ = make_request(&state, data).await;
}


#[must_use]
async fn make_request(state: &Rc<RefCell<OpState>>, data: UiRequestData) -> UiResponseData {
    let request_sender = {
        state.borrow()
            .borrow::<RequestSender>()
            .clone()
    };

    let (tx, rx) = tokio::sync::oneshot::channel();

    request_sender.inner.send(UiRequest { response_sender: tx, data }).unwrap();

    rx.await.unwrap()
}


pub struct PluginReactContext {
    plugin: Plugin,
    event_receiver: Receiver<UiEvent>,
    event_receiver_waker: Arc<AtomicWaker>,
    request_sender: UnboundedSender<UiRequest>,
}

impl PluginReactContext {
    pub fn new(plugin: Plugin, event_receiver: Receiver<UiEvent>, event_receiver_waker: Arc<AtomicWaker>, request_sender: UnboundedSender<UiRequest>) -> PluginReactContext {
        Self {
            plugin,
            event_receiver,
            event_receiver_waker,
            request_sender,
        }
    }
}

#[derive(Clone)]
pub struct RequestSender {
    inner: UnboundedSender<UiRequest>,
}

impl RequestSender {
    fn new(sender: UnboundedSender<UiRequest>) -> Self {
        Self { inner: sender }
    }
}

#[derive(Clone)]
pub struct EventReceiver {
    inner: Rc<RefCell<Receiver<UiEvent>>>,
    waker: Arc<AtomicWaker>,
}

impl EventReceiver {
    fn new(receiver: Receiver<UiEvent>, waker: Arc<AtomicWaker>) -> EventReceiver {
        Self {
            inner: Rc::new(RefCell::new(receiver)),
            waker,
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
pub struct UiRequest {
    pub response_sender: tokio::sync::oneshot::Sender<UiResponseData>,
    pub data: UiRequestData,
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
        properties: HashMap<String, PropertyValue>,
    },
    SetText {
        widget: UiWidget,
        text: String,
    },
}

#[derive(Debug)]
pub enum PropertyValue {
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
                }
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
