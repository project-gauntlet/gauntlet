use std::collections::hash_map::Entry;
use crate::model::UiViewEvent;
use crate::ui::state::PluginViewState;
use crate::ui::theme::Element;
use crate::ui::widget::{create_state, ActionPanel, ComponentWidgetEvent, ComponentWidgetState, ComponentWidgets};
use common::model::{EntrypointId, PhysicalShortcut, PluginId, RootWidget, UiWidgetId};
use std::collections::HashMap;
use std::mem;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use iced::Task;
use crate::ui::AppMsg;

pub struct PluginWidgetContainer {
    root_widget: Arc<Mutex<Option<RootWidget>>>,
    state: Arc<Mutex<HashMap<UiWidgetId, ComponentWidgetState>>>,
    images: HashMap<UiWidgetId, bytes::Bytes>,
    plugin_id: Option<PluginId>,
    plugin_name: Option<String>,
    entrypoint_id: Option<EntrypointId>,
    entrypoint_name: Option<String>
}

impl PluginWidgetContainer {
    pub fn new() -> Self {
        Self {
            root_widget: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(HashMap::new())),
            images: HashMap::new(),
            plugin_id: None,
            plugin_name: None,
            entrypoint_id: None,
            entrypoint_name: None,
        }
    }

    pub fn get_plugin_id(&self) -> PluginId {
        self.plugin_id.clone().expect("plugin id should always exist after render")
    }

    pub fn get_entrypoint_id(&self) -> EntrypointId {
        self.entrypoint_id.clone().expect("entrypoint id should always exist after render")
    }

    pub fn replace_view(
        &mut self,
        container: RootWidget,
        images: HashMap<UiWidgetId, bytes::Bytes>,
        plugin_id: &PluginId,
        plugin_name: &str,
        entrypoint_id: &EntrypointId,
        entrypoint_name: &str
    ) -> AppMsg {
        tracing::trace!("replace_view is called. container: {:?}", container);

        self.plugin_id = Some(plugin_id.clone());
        self.plugin_name = Some(plugin_name.to_string());
        self.entrypoint_id = Some(entrypoint_id.clone());
        self.entrypoint_name = Some(entrypoint_name.to_string());
        self.images = images;

        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        // use new state with values from old state but only widget ids which exists in new state
        // so we this way we use already existing values but remove state for removed widgets
        let old_state = mem::replace(state.deref_mut(), create_state(&container));

        for (key, value) in old_state.into_iter() {
            match state.entry(key) {
                Entry::Occupied(mut entry) => {
                    entry.insert(value);
                }
                Entry::Vacant(_) => {}
            }
        }

        let first_open = match root_widget.as_ref() {
            None => true,
            Some(root_widget) => root_widget.content.is_none()
        };

        *root_widget = Some(container);

        if first_open {
            ComponentWidgets::new(&mut root_widget, &mut state, &self.images)
                .first_open()
        } else {
            AppMsg::Noop
        }
    }

    pub fn handle_event(&self, plugin_id: PluginId, event: ComponentWidgetEvent) -> Option<UiViewEvent> {
        let mut state = self.state.lock().expect("lock is poisoned");

        let widget_id = event.widget_id();

        event.handle(plugin_id, state.get_mut(&widget_id))
    }

    pub fn render_root_widget<'a>(
        &self,
        plugin_view_state: &PluginViewState,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images)
            .render_root_widget(plugin_view_state, self.entrypoint_name.as_ref(), action_shortcuts)
    }

    pub fn render_inline_root_widget<'a>(&self) -> Element<'a, ComponentWidgetEvent> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images)
            .render_root_inline_widget(self.plugin_name.as_ref(), self.entrypoint_name.as_ref())
    }

    pub fn append_text(&self, text: &str) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).append_text(text)
    }

    pub fn backspace_text(&self) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).backspace_text()
    }

    pub fn focus_search_bar(&self, widget_id: UiWidgetId) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).focus_search_bar(widget_id)
    }

    pub fn toggle_action_panel(&self) {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).toggle_action_panel()
    }

    pub fn get_action_ids(&self) -> Vec<UiWidgetId> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).get_action_ids()
    }

    pub fn get_action_panel(&self, action_shortcuts: &HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).get_action_panel(action_shortcuts)
    }

    pub fn focus_up(&self) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).focus_up()
    }

    pub fn focus_down(&self) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).focus_down()
    }

    pub fn focus_left(&self) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).focus_left()
    }

    pub fn focus_right(&self) -> Task<AppMsg> {
        let mut root_widget = self.root_widget.lock().expect("lock is poisoned");
        let mut state = self.state.lock().expect("lock is poisoned");

        ComponentWidgets::new(&mut root_widget, &mut state, &self.images).focus_right()
    }
}
