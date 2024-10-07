use std::collections::HashMap;
use common::model::{EntrypointId, PhysicalShortcut, PluginId, UiRenderLocation, UiWidget, UiWidgetId};
use crate::model::UiViewEvent;

use crate::ui::widget::{ActionPanel, ComponentWidgetEvent};
use crate::ui::widget_container::PluginWidgetContainer;

pub struct ClientContext {
    inline_views: Vec<(PluginId, PluginWidgetContainer)>, // Vec to have stable ordering
    inline_view_shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>,
    view: PluginWidgetContainer,
}

impl ClientContext {
    pub fn new() -> Self {
        Self {
            inline_views: vec![],
            inline_view_shortcuts: HashMap::new(),
            view: PluginWidgetContainer::new(),
        }
    }

    pub fn get_all_inline_view_containers(&self) -> &Vec<(PluginId, PluginWidgetContainer)> {
        &self.inline_views
    }

    pub fn get_first_inline_view_container(&self) -> Option<&PluginWidgetContainer> {
        self.inline_views.first()
            .map(|(_, container)| container)
    }

    pub fn get_first_inline_view_action_panel(&self) -> Option<ActionPanel> {
        self.get_first_inline_view_container()
            .map(|container| {
                match self.inline_view_shortcuts.get(&container.get_plugin_id()) {
                    None => container.get_action_panel(&HashMap::new()),
                    Some(shortcuts) => container.get_action_panel(shortcuts),
                }
            })
            .flatten()
    }

    pub fn get_inline_view_container(&self, plugin_id: &PluginId) -> &PluginWidgetContainer {
        self.inline_views.iter()
            .find(|(id, _)| id == plugin_id)
            .map(|(_, container)| container)
            .expect("there should always be container for plugin at this point")
    }

    pub fn get_mut_inline_view_container(&mut self, plugin_id: &PluginId) -> &mut PluginWidgetContainer {
        if let Some(index) = self.inline_views.iter().position(|(id, _)| id == plugin_id) {
            let (_, container) = &mut self.inline_views[index];
            container
        } else {
            self.inline_views.push((plugin_id.clone(), PluginWidgetContainer::new()));
            let (_, container) = self.inline_views.last_mut().expect("getting just pushed item");
            container
        }
    }

    pub fn get_view_container(&self) -> &PluginWidgetContainer {
        &self.view
    }

    pub fn get_mut_view_container(&mut self) -> &mut PluginWidgetContainer {
        &mut self.view
    }

    pub fn get_view_plugin_id(&self) -> PluginId {
        self.view.get_plugin_id()
    }

    pub fn get_view_entrypoint_id(&self) -> EntrypointId {
        self.view.get_entrypoint_id()
    }

    pub fn replace_view(&mut self, render_location: UiRenderLocation, container: UiWidget, plugin_id: &PluginId, plugin_name: &str, entrypoint_id: &EntrypointId, entrypoint_name: &str) {
        match render_location {
            UiRenderLocation::InlineView => self.get_mut_inline_view_container(plugin_id).replace_view(container, plugin_id, plugin_name, entrypoint_id, entrypoint_name),
            UiRenderLocation::View => self.get_mut_view_container().replace_view(container, plugin_id, plugin_name, entrypoint_id, entrypoint_name)
        }
    }

    pub fn set_inline_view_shortcuts(&mut self, shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>) {
        self.inline_view_shortcuts = shortcuts;
    }

     pub fn clear_all_inline_views(&mut self) {
        self.inline_views.clear()
    }

    pub fn clear_inline_view(&mut self, plugin_id: &PluginId) {
        if let Some(index) = self.inline_views.iter().position(|(id, _)| id == plugin_id) {
            self.inline_views.remove(index);
        }
    }

    pub fn handle_event(&self, render_location: UiRenderLocation, plugin_id: &PluginId, event: ComponentWidgetEvent) -> Option<UiViewEvent> {
        match render_location {
            UiRenderLocation::InlineView => self.get_inline_view_container(&plugin_id).handle_event(plugin_id.clone(), event),
            UiRenderLocation::View => self.get_view_container().handle_event(plugin_id.clone(), event)
        }
    }

    pub fn toggle_action_panel(&self) {
        self.view.toggle_action_panel()
    }

    pub fn get_action_ids(&self) -> Vec<UiWidgetId> {
        self.view.get_action_ids()
    }
}
