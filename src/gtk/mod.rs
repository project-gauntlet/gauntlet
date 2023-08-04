use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use deno_core::futures::task::AtomicWaker;
use relm4::gtk;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::plugins::Plugin;
use crate::react_side::{UiEvent, UiRequest};

pub mod gui;
pub mod gtk_side;



#[derive(Clone)]
pub struct PluginUiContext {
    plugin: Plugin,
    request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>,
    event_sender: Sender<UiEvent>,
    event_waker: Arc<AtomicWaker>,
    inner: Rc<RefCell<Option<gtk::Widget>>>, // FIXME bare option after cloning it seems to be set to none?
}

impl PluginUiContext {
    pub fn new(plugin: Plugin, request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>, event_sender: Sender<UiEvent>, event_waker: Arc<AtomicWaker>) -> PluginUiContext {
        Self {
            plugin,
            request_receiver,
            event_sender,
            event_waker,
            inner: Rc::new(RefCell::new(None))
        }
    }

    async fn request_recv(&self) -> Option<UiRequest> {
        self.request_receiver.borrow_mut().recv().await
    }

    fn send_event(&self, event: UiEvent) {
        self.event_sender.send(event).unwrap();
        self.event_waker.wake();
    }

    fn current_container(&self) -> Option<gtk::Widget> {
        self.inner.borrow().clone()
    }

    fn set_current_container(&mut self, container: gtk::Widget) {
        *self.inner.borrow_mut() = Some(container);
    }

    pub fn plugin(&self) -> &Plugin {
        &self.plugin
    }
}