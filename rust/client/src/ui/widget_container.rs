use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use std::sync::Arc;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::UiWidgetId;
use iced::Task;

use crate::model::UiViewEvent;
use crate::ui::AppMsg;
use crate::ui::state::PluginViewState;
use crate::ui::theme::Element;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::data_mut::ComponentWidgetsMut;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::state::ComponentWidgetState;
use crate::ui::widget::state::create_state;

pub struct PluginWidgetContainer {
    root_widget: Option<Arc<RootWidget>>,
    state: HashMap<UiWidgetId, ComponentWidgetState>,
    data: HashMap<UiWidgetId, Vec<u8>>,
    plugin_id: Option<PluginId>,
    plugin_name: Option<String>,
    entrypoint_id: Option<EntrypointId>,
    entrypoint_name: Option<String>,
}

impl PluginWidgetContainer {
    pub fn new() -> Self {
        Self {
            root_widget: None,
            state: HashMap::new(),
            data: HashMap::new(),
            plugin_id: None,
            plugin_name: None,
            entrypoint_id: None,
            entrypoint_name: None,
        }
    }

    pub fn get_plugin_id(&self) -> PluginId {
        self.plugin_id
            .clone()
            .expect("plugin id should always exist after render")
    }

    pub fn get_entrypoint_id(&self) -> EntrypointId {
        self.entrypoint_id
            .clone()
            .expect("entrypoint id should always exist after render")
    }

    pub fn replace_view(
        &mut self,
        container: Arc<RootWidget>,
        data: HashMap<UiWidgetId, Vec<u8>>,
        plugin_id: &PluginId,
        plugin_name: &str,
        entrypoint_id: &EntrypointId,
        entrypoint_name: &str,
    ) -> AppMsg {
        tracing::trace!("replace_view is called. container: {:?}", container);

        self.plugin_id = Some(plugin_id.clone());
        self.plugin_name = Some(plugin_name.to_string());
        self.entrypoint_id = Some(entrypoint_id.clone());
        self.entrypoint_name = Some(entrypoint_name.to_string());
        self.data = data;

        // use new state with values from old state but only widget ids which exists in new state
        // so this way we use already existing values but remove state for removed widgets
        let old_state = mem::replace(&mut self.state, create_state(&container));

        for (key, value) in old_state.into_iter() {
            match self.state.entry(key) {
                Entry::Occupied(mut entry) => {
                    // copy over old value, but only if type of the widget didn't change
                    // if it did change, the widget state is reset
                    if mem::discriminant(entry.get()) == mem::discriminant(&value) {
                        entry.insert(value);
                    }
                }
                Entry::Vacant(_) => {}
            }
        }

        let first_open = match self.root_widget.as_ref() {
            None => true,
            Some(root_widget) => root_widget.content.is_none(),
        };

        self.root_widget = Some(container);

        if first_open {
            ComponentWidgets::new(&mut self.root_widget, &mut self.state, &self.data).first_open()
        } else {
            AppMsg::Noop
        }
    }

    pub fn handle_event(&mut self, plugin_id: PluginId, event: ComponentWidgetEvent) -> Option<UiViewEvent> {
        let widget_id = event.widget_id();

        event.handle(plugin_id, self.state.get_mut(&widget_id))
    }

    pub fn render_root_widget<'a>(
        &self,
        plugin_view_state: &PluginViewState,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data).render_root_widget(
            plugin_view_state,
            self.entrypoint_name.as_ref(),
            action_shortcuts,
        )
    }

    pub fn render_inline_root_widget<'a>(&self) -> Element<'a, ComponentWidgetEvent> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data)
            .render_root_inline_widget(self.plugin_name.as_ref(), self.entrypoint_name.as_ref())
    }

    pub fn append_text(&mut self, text: &str) -> Task<AppMsg> {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).append_text(text)
    }

    pub fn backspace_text(&mut self) -> Task<AppMsg> {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).backspace_text()
    }

    pub fn focus_search_bar(&self, widget_id: UiWidgetId) -> Task<AppMsg> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data).focus_search_bar(widget_id)
    }

    pub fn toggle_action_panel(&mut self) {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).toggle_action_panel()
    }

    pub fn get_action_ids(&self) -> Vec<UiWidgetId> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data).get_action_ids()
    }

    pub fn get_focused_item_id(&self) -> Option<String> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data).get_focused_item_id()
    }

    pub fn get_action_panel(&self, action_shortcuts: &HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data).get_action_panel(action_shortcuts)
    }

    pub fn focus_up(&mut self) -> Task<AppMsg> {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).focus_up()
    }

    pub fn focus_down(&mut self) -> Task<AppMsg> {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).focus_down()
    }

    pub fn focus_left(&mut self) -> Task<AppMsg> {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).focus_left()
    }

    pub fn focus_right(&mut self) -> Task<AppMsg> {
        let plugin_id = self.get_plugin_id();
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, plugin_id).focus_right()
    }
}
