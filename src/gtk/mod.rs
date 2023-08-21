use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::futures::task::AtomicWaker;
use relm4::gtk;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::plugins::Plugin;
use crate::react_side::{UiEvent, UiRequest};

pub mod gui;
pub mod gtk_side;


pub struct PluginUiData {
    pub plugin: Plugin,
    pub request_receiver: UnboundedReceiver<UiRequest>,
    pub event_sender: UnboundedSender<UiEvent>,
    pub event_waker: Arc<AtomicWaker>,
}


#[derive(Clone)]
pub struct PluginUiContext {
    plugin: Plugin,
    request_receiver: Rc<RefCell<UnboundedReceiver<UiRequest>>>,
}

impl PluginUiContext {
    pub fn new(plugin: Plugin, request_receiver: UnboundedReceiver<UiRequest>) -> Self {
        Self {
            plugin,
            request_receiver: Rc::new(RefCell::new(request_receiver)),
        }
    }

    async fn request_recv(&self) -> Option<UiRequest> {
        self.request_receiver.borrow_mut().recv().await
    }

    pub fn plugin(&self) -> &Plugin {
        &self.plugin
    }
}

#[derive(Clone)]
pub struct PluginContainerContainer { // creative name, isn't it?
    containers: Rc<RefCell<HashMap<String, gtk::Widget>>>
}

impl PluginContainerContainer {
    pub(crate) fn new() -> Self {
        Self {
            containers: Rc::new(RefCell::new(HashMap::new()))
        }
    }

    fn current_container(&self, plugin_id: &str) -> Option<gtk::Widget> {
        self.containers.borrow().get(plugin_id).cloned()
    }

    fn set_current_container(&mut self, plugin_id: &str, container: gtk::Widget) {
        self.containers.borrow_mut().insert(plugin_id.to_owned(), container);
    }
}

#[derive(Clone)]
pub struct PluginEventSenderContainer {
    senders: Rc<RefCell<HashMap<String, (UnboundedSender<UiEvent>, Arc<AtomicWaker>)>>>
}

impl PluginEventSenderContainer {
    pub fn new(senders: HashMap<String, (UnboundedSender<UiEvent>, Arc<AtomicWaker>)>) -> Self {
        Self {
            senders: Rc::new(RefCell::new(senders)),
        }
    }

    fn send_event(&self, plugin_id: &str, event: UiEvent) {
        let senders = self.senders.borrow();
        let (sender, waker) = senders.get(plugin_id).unwrap();
        sender.send(event).unwrap();
        waker.wake();
    }
}
