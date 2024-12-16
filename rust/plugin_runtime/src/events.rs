use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use anyhow::anyhow;
use bincode::{Decode, Encode};
use deno_core::{op2, OpState};
use deno_core::futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;
use common::model::UiWidgetId;

#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(tag = "type")]
pub enum JsEvent {
    OpenView {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    CloseView,
    RunCommand {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String
    },
    RunGeneratedCommand {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
        #[serde(rename = "actionIndex")]
        action_index: Option<usize>
    },
    ViewEvent {
        #[serde(rename = "widgetId")]
        widget_id: UiWidgetId,
        #[serde(rename = "eventName")]
        event_name: String,
        #[serde(rename = "eventArguments")]
        event_arguments: Vec<JsUiPropertyValue>,
    },
    KeyboardEvent {
        #[serde(rename = "entrypointId")]
        entrypoint_id: String,
        origin: JsKeyboardEventOrigin,
        key: String,
        #[serde(rename = "modifierShift")]
        modifier_shift: bool,
        #[serde(rename = "modifierControl")]
        modifier_control: bool,
        #[serde(rename = "modifierAlt")]
        modifier_alt: bool,
        #[serde(rename = "modifierMeta")]
        modifier_meta: bool
    },
    OpenInlineView {
        #[serde(rename = "text")]
        text: String,
    },
    ReloadSearchIndex,
    RefreshSearchIndex,
}

#[derive(Clone, Debug, Deserialize, Serialize, Encode, Decode)]
pub enum JsKeyboardEventOrigin {
    MainView,
    PluginView,
}

// FIXME this could have been serde_v8::AnyValue but it doesn't support undefined, make a pr?
#[derive(Debug, Deserialize, Serialize, Encode, Decode)]
#[serde(tag = "type")]
pub enum JsUiPropertyValue {
    String {
        value: String
    },
    Number {
        value: f64
    },
    Bool {
        value: bool
    },
    Undefined,
}

pub struct EventReceiver {
    event_stream: Rc<RefCell<Receiver<JsEvent>>>,
}

impl EventReceiver {
    pub fn new(event_stream: Receiver<JsEvent>) -> EventReceiver {
        Self {
            event_stream: Rc::new(RefCell::new(event_stream)),
        }
    }
}

#[op2(async)]
#[serde]
pub async fn op_plugin_get_pending_event(state: Rc<RefCell<OpState>>) -> anyhow::Result<JsEvent> {
    let event_stream = {
        state.borrow()
            .borrow::<EventReceiver>()
            .event_stream
            .clone()
    };

    let mut event_stream = event_stream.borrow_mut();
    let event = event_stream.recv()
        .await
        .ok_or_else(|| anyhow!("event stream was suddenly closed"))?;

    tracing::trace!("Received plugin event {:?}", event);

    Ok(event)
}

