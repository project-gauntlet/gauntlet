use common::model::{PluginId, RenderLocation};

use crate::model::{NativeUiViewEvent, NativeUiWidget};
use crate::ui::widget::ComponentWidgetEvent;
use crate::ui::widget_container::PluginWidgetContainer;

pub struct ClientContext {
    inline_views: Vec<(PluginId, PluginWidgetContainer)>,
    view: PluginWidgetContainer,
}

impl ClientContext {
    pub fn new() -> Self {
        Self {
            inline_views: vec![],
            view: PluginWidgetContainer::new(),
        }
    }

    pub fn get_all_inline_view_containers(&self) -> &Vec<(PluginId, PluginWidgetContainer)> {
        &self.inline_views
    }

    pub fn get_inline_view_container(&self, plugin_id: &PluginId) -> &PluginWidgetContainer {
        self.inline_views.iter()
            .find(|(id, _)| id == plugin_id)
            .map(|(_, container)| container)
            .unwrap()
    }

    pub fn get_mut_inline_view_container(&mut self, plugin_id: &PluginId) -> &mut PluginWidgetContainer {
        if let Some(index) = self.inline_views.iter().position(|(id, _)| id == plugin_id) {
            let (_, container) = &mut self.inline_views[index];
            container
        } else {
            self.inline_views.push((plugin_id.clone(), PluginWidgetContainer::new()));
            let (_, container) = self.inline_views.last_mut().unwrap();
            container
        }
    }

    pub fn get_view_container(&self) -> &PluginWidgetContainer {
        &self.view
    }

    pub fn get_mut_view_container(&mut self) -> &mut PluginWidgetContainer {
        &mut self.view
    }

    pub fn replace_view(&mut self, render_location: RenderLocation, container: NativeUiWidget, plugin_id: &PluginId) {
        match render_location {
            RenderLocation::InlineView => self.get_mut_inline_view_container(plugin_id).replace_view(container),
            RenderLocation::View => self.get_mut_view_container().replace_view(container)
        }
    }

    pub fn clear_inline_view(&mut self, plugin_id: &PluginId) {
        if let Some(index) = self.inline_views.iter().position(|(id, _)| id == plugin_id) {
            self.inline_views.remove(index);
        }
    }

    pub fn handle_event(&self, render_location: RenderLocation, plugin_id: &PluginId, event: ComponentWidgetEvent) -> Option<NativeUiViewEvent> {
        match render_location {
            RenderLocation::InlineView => self.get_inline_view_container(&plugin_id).handle_event(plugin_id.clone(), event),
            RenderLocation::View => self.get_view_container().handle_event(plugin_id.clone(), event)
        }
    }
}
