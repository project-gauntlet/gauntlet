use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use relm4::gtk;

use crate::plugins::Plugin;
use crate::react_side::UiEvent;
use crate::channel::RequestSender;

pub mod gui;
pub mod gtk_side;


pub struct PluginUiData {
    pub plugin: Plugin,
}

#[derive(Clone)]
pub struct PluginContainerContainer { // creative name, isn't it?
    containers: Rc<RefCell<HashMap<String, gtk::Widget>>>,
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
    sender: RequestSender<(String, UiEvent), ()>,
}

impl PluginEventSenderContainer {
    pub fn new(sender: RequestSender<(String, UiEvent), ()>) -> Self {
        Self {
            sender,
        }
    }

    fn send_event(&self, plugin_uuid: &str, event: UiEvent) {
        self.sender.send((plugin_uuid.to_owned(), event)).unwrap();
    }
}
