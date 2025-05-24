use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use deno_core::OpState;
use deno_core::op2;
use gauntlet_common_plugin_runtime::model::JsEvent;
use tokio::sync::mpsc::Receiver;

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
    let event_stream = { state.borrow().borrow::<EventReceiver>().event_stream.clone() };

    let mut event_stream = event_stream.borrow_mut();
    let event = event_stream
        .recv()
        .await
        .ok_or_else(|| anyhow!("event stream was suddenly closed"))?;

    tracing::trace!("Received plugin event {:?}", event);

    Ok(event)
}
