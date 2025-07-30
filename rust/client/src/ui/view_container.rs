use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use std::sync::Arc;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::UiRenderLocation;
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

pub struct PluginViewContainer {
    root_widget: Option<Arc<RootWidget>>,
    state: HashMap<UiWidgetId, ComponentWidgetState>,
    data: HashMap<UiWidgetId, Vec<u8>>,
    render_location: UiRenderLocation,
    plugin_id: PluginId,
    plugin_name: Option<String>,
    entrypoint_id: EntrypointId,
    entrypoint_name: Option<String>,
}

impl PluginViewContainer {
    pub fn new(render_location: UiRenderLocation, plugin_id: PluginId, entrypoint_id: EntrypointId) -> Self {
        Self {
            root_widget: None,
            state: HashMap::new(),
            data: HashMap::new(),
            render_location,
            plugin_id,
            plugin_name: None,
            entrypoint_id,
            entrypoint_name: None,
        }
    }

    pub fn plugin_id(&self) -> PluginId {
        self.plugin_id.clone()
    }

    pub fn entrypoint_id(&self) -> EntrypointId {
        self.entrypoint_id.clone()
    }

    pub fn render_location(&self) -> UiRenderLocation {
        self.render_location.clone()
    }

    pub fn replace_view(
        &mut self,
        container: Arc<RootWidget>,
        data: HashMap<UiWidgetId, Vec<u8>>,
        plugin_name: &str,
        entrypoint_name: &str,
    ) -> AppMsg {
        tracing::trace!("replace_view is called. container: {:?}", container);

        self.plugin_name = Some(plugin_name.to_string());
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
            ComponentWidgets::new(&self.root_widget, &self.state, &self.data).first_open(self.plugin_id.clone())
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
            self.entrypoint_name.as_deref(),
            action_shortcuts,
        )
    }

    pub fn render_inline_root_widget<'a>(&self) -> Element<'a, ComponentWidgetEvent> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data)
            .render_root_inline_widget(self.plugin_name.as_deref(), self.entrypoint_name.as_deref())
    }

    pub fn append_text(&mut self, text: &str) -> Task<AppMsg> {
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).append_text(text)
    }

    pub fn backspace_text(&mut self) -> Task<AppMsg> {
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).backspace_text()
    }

    pub fn focus_search_bar(&self, widget_id: UiWidgetId) -> Task<AppMsg> {
        ComponentWidgets::new(&self.root_widget, &self.state, &self.data).focus_search_bar(widget_id)
    }

    pub fn toggle_action_panel(&mut self) {
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).toggle_action_panel()
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
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).focus_up()
    }

    pub fn focus_down(&mut self) -> Task<AppMsg> {
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).focus_down()
    }

    pub fn focus_left(&mut self) -> Task<AppMsg> {
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).focus_left()
    }

    pub fn focus_right(&mut self) -> Task<AppMsg> {
        ComponentWidgetsMut::new(&mut self.root_widget, &mut self.state, &self.plugin_id).focus_right()
    }
}
