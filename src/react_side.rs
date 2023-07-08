use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::{Future, poll_fn};
use std::net::SocketAddr;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::task::Poll;

use deno_core::{op, OpState, serde_v8, v8};
use deno_core::futures::task::AtomicWaker;
use deno_runtime::deno_core::FsModuleLoader;
use deno_runtime::deno_core::ModuleSpecifier;
use deno_runtime::inspector_server::InspectorServer;
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

pub async fn run_react(react_context: ReactContext) {
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("react_renderer/dist/main.js");
    let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();

    let inspector_server = Arc::new(
        InspectorServer::new(
            "127.0.0.1:9229".parse::<SocketAddr>().unwrap(),
            "test",
        )
    );

    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(FsModuleLoader),
            extensions: vec![gtk_ext::init_ops(
                EventHandlers::new(),
                react_context,
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

    worker.execute_main_module(&main_module).await.unwrap();
    worker.run_event_loop(false).await.unwrap();
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
        op_get_next_pending_gui_event,
        op_call_event_listener,
    ],
    options = {
        event_listeners: EventHandlers,
        react_context: ReactContext,
    },
    state = |state, options| {
        state.put(options.event_listeners);
        state.put(options.react_context);
    },
    customizer = |ext: &mut deno_core::ExtensionBuilder| {
        ext.force_op_registration();
    },
);



#[op]
pub async fn op_gtk_get_container(state: Rc<RefCell<OpState>>) -> GuiWidget {
    println!("op_gtk_get_container");

    let container = match make_request(&state, UiRequestData::GetContainer).await {
        UiResponseData::GetContainer { container: container_pointer } => container_pointer,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    println!("op_gtk_get_container end");

    container
}

#[op]
pub async fn op_gtk_append_child(
    state: Rc<RefCell<OpState>>,
    parent: GuiWidget,
    child: GuiWidget,
) {
    println!("op_gtk_append_child");

    let data = UiRequestData::AppendChild { parent, child };

    let _ = make_request(&state, data).await;
}

#[op]
pub async fn op_gtk_remove_child(
    state: Rc<RefCell<OpState>>,
    parent: GuiWidget,
    child: GuiWidget,
) {
    println!("op_gtk_remove_child");

    let data = UiRequestData::RemoveChild { parent, child };

    let _ = make_request(&state, data).await;
}

#[op]
pub async fn op_gtk_insert_before(
    state: Rc<RefCell<OpState>>,
    parent: GuiWidget,
    child: GuiWidget,
    before_child: GuiWidget,
) {
    println!("op_gtk_insert_before");

    let data = UiRequestData::InsertBefore {
        parent,
        child,
        before_child,
    };

    let _ = make_request(&state, data);
}

#[op]
pub async fn op_gtk_create_instance(
    state: Rc<RefCell<OpState>>,
    jsx_type: String,
) -> GuiWidget {
    println!("op_gtk_create_instance");

    let data = UiRequestData::CreateInstance {
        type_: jsx_type,
    };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateInstance { widget: widget_pointer } => widget_pointer,
        value @ _ => panic!("unsupported response type {:?}", value),
    };
    println!("op_gtk_create_instance end");

    widget
}

#[op]
pub async fn op_gtk_create_text_instance(
    state: Rc<RefCell<OpState>>,
    text: String,
) -> GuiWidget {
    println!("op_gtk_create_text_instance");

    let data = UiRequestData::CreateTextInstance { text };

    let widget = match make_request(&state, data).await {
        UiResponseData::CreateTextInstance { widget: widget_pointer } => widget_pointer,
        value @ _ => panic!("unsupported response type {:?}", value),
    };

    return widget;
}

#[op(v8)]
pub fn op_gtk_set_properties<'a>(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: GuiWidget,
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
        widget,
        properties,
    };

    drop(state_ref);

    Ok(async move {
        let _ = make_request(&state, data).await;

        Ok(())
    })
}

#[op]
pub async fn op_get_next_pending_gui_event<'a>(
    state: Rc<RefCell<OpState>>,
) -> GuiEvent2 {
    let react_context = {
        state.borrow()
            .borrow::<ReactContext>()
            .clone()
    };

    poll_fn(|cx| {
        let receiver = &react_context.event_receiver;
        receiver.waker.register(cx.waker());
        let receiver = receiver.inner.borrow();

        match receiver.try_recv() {
            Ok(value) => {
                println!("Poll::Ready {:?}", value);
                let event = GuiEvent2 {
                    widget_id: GuiWidget {
                        widget_id: value.widget_id
                    },
                    event_name: value.event_name,
                };
                Poll::Ready(event)
            }
            Err(TryRecvError::Disconnected) => panic!("disconnected"),
            Err(TryRecvError::Empty) => Poll::Pending
        }
    }).await
}

#[op(v8)]
pub fn op_call_event_listener(
    scope: &mut v8::HandleScope,
    state: Rc<RefCell<OpState>>,
    widget: GuiWidget,
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
pub async fn op_gtk_set_text(
    state: Rc<RefCell<OpState>>,
    widget: GuiWidget,
    text: String,
) {
    println!("op_gtk_set_text");

    let data = UiRequestData::SetText { widget, text };

    let _ = make_request(&state, data).await;
}


#[must_use]
async fn make_request(state: &Rc<RefCell<OpState>>, data: UiRequestData) -> UiResponseData {
    let react_context = {
        state.borrow()
            .borrow::<ReactContext>()
            .clone()
    };

    let (tx, rx) = tokio::sync::oneshot::channel();

    react_context.request_sender.send(UiRequest { response_sender: tx, data }).unwrap();

    rx.await.unwrap()
}


#[derive(Clone)]
pub struct ReactContext {
    event_receiver: EventReceiver,
    request_sender: UnboundedSender<UiRequest>,
}

impl ReactContext {
    pub fn new(receiver: Receiver<UiEvent>, event_waker: Arc<AtomicWaker>, request_sender: UnboundedSender<UiRequest>) -> ReactContext {
        Self {
            event_receiver: EventReceiver::new(receiver, event_waker),
            request_sender,
        }
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
    listeners: HashMap<WidgetId, HashMap<GtkEventName, v8::Global<v8::Function>>>,
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

    fn add_listener(&mut self, widget: WidgetId, event_name: GtkEventName, function: v8::Global<v8::Function>) {
        let mut inner = self.inner.borrow_mut();
        inner.listeners.entry(widget).or_default().insert(event_name, function);
    }

    fn call_listener_handler(&self, scope: &mut v8::HandleScope, widget: &WidgetId, event_name: &GtkEventName) {
        let inner = self.inner.borrow();
        let option_func = inner.listeners.get(widget)
            .map(|handlers| handlers.get(event_name))
            .flatten();

        if let Some(func) = option_func {
            let local_fn = v8::Local::new(scope, func);
            scope.enqueue_microtask(local_fn);
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
        container: GuiWidget
    },
    CreateInstance {
        widget: GuiWidget
    },
    CreateTextInstance {
        widget: GuiWidget
    },
    Unit,
}

#[derive(Debug)]
pub enum UiRequestData {
    GetContainer,
    CreateInstance {
        type_: String,
    },
    CreateTextInstance {
        text: String,
    },
    AppendChild {
        parent: GuiWidget,
        child: GuiWidget,
    },
    RemoveChild {
        parent: GuiWidget,
        child: GuiWidget,
    },
    InsertBefore {
        parent: GuiWidget,
        child: GuiWidget,
        before_child: GuiWidget,
    },
    SetProperties {
        widget: GuiWidget,
        properties: HashMap<String, PropertyValue>,
    },
    SetText {
        widget: GuiWidget,
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


#[derive(Debug)]
pub struct UiEvent {
    pub widget_id: WidgetId,
    pub event_name: GtkEventName,
}

pub type WidgetId = u32;
pub type GtkEventName = String;


#[derive(Debug, Deserialize, Serialize)]
pub struct GuiWidget {
    widget_id: WidgetId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuiEvent2 {
    pub widget_id: GuiWidget,
    pub event_name: GtkEventName,
}

impl GuiWidget {
    pub fn new(widget_id: WidgetId) -> Self {
        Self { widget_id }
    }

    pub fn widget_id(&self) -> WidgetId {
        self.widget_id
    }
}
