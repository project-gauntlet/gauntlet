use std::collections::HashMap;
use std::sync::Arc;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::RootWidget;
use gauntlet_common::model::UiRenderLocation;
use gauntlet_common::model::UiWidgetId;
use iced::Task;
use iced::widget::container;

use crate::model::UiViewEvent;
use crate::ui::AppMsg;
use crate::ui::view_container::PluginViewContainer;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::events::ComponentWidgetEvent;

pub struct ClientContext {
    views: Vec<(PluginId, PluginViewContainer)>, // Vec to have stable ordering
    inline_view_shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>,
}

impl ClientContext {
    pub fn new() -> Self {
        Self {
            views: vec![],
            inline_view_shortcuts: HashMap::new(),
        }
    }

    pub fn get_first_inline_view_container(&self) -> Option<&PluginViewContainer> {
        self.get_inline_view_containers()
            .iter()
            .next()
            .map(|(_, container)| container)
    }

    pub fn get_first_inline_view_action_panel(&self) -> Option<ActionPanel> {
        self.get_first_inline_view_container()
            .map(|container| {
                match self.inline_view_shortcuts.get(&container.plugin_id()) {
                    None => container.get_action_panel(&HashMap::new()),
                    Some(shortcuts) => container.get_action_panel(shortcuts),
                }
            })
            .flatten()
    }

    pub fn get_inline_view_containers(&self) -> Vec<&(PluginId, PluginViewContainer)> {
        self.views
            .iter()
            .filter(|(_, container)| matches!(container.render_location(), UiRenderLocation::InlineView))
            .collect()
    }

    pub fn get_mut_or_create_any_view_container(
        &mut self,
        render_location: UiRenderLocation,
        plugin_id: &PluginId,
        entrypoint_id: &EntrypointId,
    ) -> &mut PluginViewContainer {
        if let Some(index) = self.views.iter().position(|(id, _)| id == plugin_id) {
            let (_, container) = &mut self.views[index];
            container
        } else {
            let container = PluginViewContainer::new(render_location, plugin_id.clone(), entrypoint_id.clone());
            self.views.push((plugin_id.clone(), container));
            let (_, container) = self.views.last_mut().expect("getting just pushed item");
            container
        }
    }

    pub fn get_view_container(&self, plugin_id: &PluginId) -> Option<&PluginViewContainer> {
        self.views
            .iter()
            .find(|(id, _)| id == plugin_id)
            .filter(|(_, container)| matches!(container.render_location(), UiRenderLocation::View))
            .map(|(_, container)| container)
    }

    pub fn get_mut_view_container(&mut self, plugin_id: &PluginId) -> Option<&mut PluginViewContainer> {
        self.get_mut_any_view_container(plugin_id)
            .filter(|container| matches!(container.render_location(), UiRenderLocation::View))
    }

    pub fn get_mut_any_view_container(&mut self, plugin_id: &PluginId) -> Option<&mut PluginViewContainer> {
        self.views
            .iter_mut()
            .find(|(id, _)| id == plugin_id)
            .map(|(_, container)| container)
    }

    pub fn render_ui(
        &mut self,
        render_location: UiRenderLocation,
        container: Arc<RootWidget>,
        data: HashMap<UiWidgetId, Vec<u8>>,
        plugin_id: &PluginId,
        plugin_name: &str,
        entrypoint_id: &EntrypointId,
        entrypoint_name: &str,
    ) -> AppMsg {
        self.get_mut_or_create_any_view_container(render_location, plugin_id, entrypoint_id)
            .replace_view(container, data, plugin_name, entrypoint_name)
    }

    pub fn handle_event(&mut self, plugin_id: &PluginId, event: ComponentWidgetEvent) -> Option<UiViewEvent> {
        self.get_mut_any_view_container(plugin_id)
            .and_then(|view| view.handle_event(plugin_id.clone(), event))
    }

    pub fn set_inline_view_shortcuts(&mut self, shortcuts: HashMap<PluginId, HashMap<String, PhysicalShortcut>>) {
        self.inline_view_shortcuts = shortcuts;
    }

    pub fn clear_all_views(&mut self) {
        self.views.clear()
    }

    pub fn clear_view(&mut self, plugin_id: &PluginId) {
        if let Some(index) = self.views.iter().position(|(id, _)| id == plugin_id) {
            self.views.remove(index);
        }
    }

    pub fn set_current_focused_item(&mut self, plugin_id: PluginId, target_id: Option<container::Id>) -> Task<AppMsg> {
        self.get_mut_any_view_container(&plugin_id)
            .map(|view| view.set_focused_item_id(target_id))
            .unwrap_or(Task::none())
    }
}
