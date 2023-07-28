use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use deno_core::futures::task::AtomicWaker;
use crate::react_side::{PluginReactContext, UiEvent, UiRequest};
use crate::PluginUiContext;

#[derive(Clone)]
pub struct PluginManager {
    inner: Rc<RefCell<PluginManagerInner>>,
}

pub struct PluginManagerInner {
    plugins: Vec<Plugin>,
    ui_contexts: HashMap<String, PluginUiContext>
}

impl PluginManager {

    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(PluginManagerInner {
                plugins: vec![Plugin::new("x"), Plugin::new("y"), Plugin::new("z")],
                ui_contexts: HashMap::new(),
            }))
        }
    }

    pub fn get_ui_context(&mut self, plugin_id: &str) -> Option<PluginUiContext> {
        self.inner
            .borrow_mut()
            .ui_contexts
            .get_mut(plugin_id)
            .map(|context| context.clone())
    }

    pub fn create_all_contexts(&mut self) -> (Vec<PluginReactContext>, Vec<PluginUiContext>) {
        let (react_contexts, ui_contexts): (Vec<_>, Vec<_>) = self.inner
            .borrow()
            .plugins
            .iter()
            .map(|plugin| self.create_contexts_for_plugin(plugin.clone()))
            .unzip();

        self.inner.borrow_mut().ui_contexts = ui_contexts.iter()
            .map(|context| (context.plugin.id().clone(), context.clone()))
            .collect::<HashMap<_, _>>();

        (react_contexts, ui_contexts)
    }

    fn create_contexts_for_plugin(&self, plugin: Plugin) -> (PluginReactContext, PluginUiContext) {
        let (react_request_sender, react_request_receiver) = tokio::sync::mpsc::unbounded_channel::<UiRequest>();
        let react_request_receiver = Rc::new(RefCell::new(react_request_receiver));

        let (react_event_sender, react_event_receiver) = std::sync::mpsc::channel::<UiEvent>();
        let event_waker = Arc::new(AtomicWaker::new());

        let ui_context = PluginUiContext::new(plugin.clone(), react_request_receiver, react_event_sender, event_waker.clone());
        let react_context = PluginReactContext::new(plugin.clone(), react_event_receiver, event_waker, react_request_sender);

        (react_context, ui_context)
    }

}


#[derive(Clone)]
pub struct Plugin {
    id: String
}

impl Plugin {
    fn new(id: &str) -> Self {
        Self {
            id: id.into()
        }
    }

    fn id(&self) -> &String {
        &self.id
    }
}