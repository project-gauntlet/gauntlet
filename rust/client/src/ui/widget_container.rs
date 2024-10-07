use std::collections::HashMap;
use common::model::{EntrypointId, PhysicalShortcut, PluginId, UiWidget, UiWidgetId};
use crate::model::UiViewEvent;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::theme::Element;
use crate::ui::widget::{ActionPanel, ComponentRenderContext, ComponentWidgetEvent, ComponentWidgetWrapper};

pub struct PluginWidgetContainer {
    root_widget: ComponentWidgetWrapper,
    plugin_id: Option<PluginId>,
    plugin_name: Option<String>,
    entrypoint_id: Option<EntrypointId>,
    entrypoint_name: Option<String>
}

impl PluginWidgetContainer {
    pub fn new() -> Self {
        Self {
            root_widget: ComponentWidgetWrapper::root(0),
            plugin_id: None,
            plugin_name: None,
            entrypoint_id: None,
            entrypoint_name: None,
        }
    }

    pub fn get_plugin_id(&self) -> PluginId {
        self.plugin_id.clone().expect("plugin id should always exist after render")
    }

    pub fn get_plugin_name(&self) -> String {
        self.plugin_name.clone().expect("plugin name should always exist after render")
    }

    pub fn get_entrypoint_id(&self) -> EntrypointId {
        self.entrypoint_id.clone().expect("entrypoint id should always exist after render")
    }

    pub fn get_entrypoint_name(&self) -> String {
        self.entrypoint_name.clone().expect("entrypoint id should always exist after render")
    }

    pub fn get_action_panel(&self, action_shortcuts: &HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
        self.root_widget.get_action_panel(action_shortcuts)
    }

    pub fn render_widget<'a>(&self, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
        self.root_widget.render_widget(context)
    }

    fn create_component_widget(&mut self, ui_widget: UiWidget) -> ComponentWidgetWrapper {
        let children = ui_widget.widget_children
            .into_iter()
            .map(|ui_widget| self.create_component_widget(ui_widget))
            .collect();

        ComponentWidgetWrapper::widget(ui_widget.widget_id, &ui_widget.widget_type, ui_widget.widget_properties, children)
            .expect("unable to create widget")
    }

    pub fn replace_view(&mut self, container: UiWidget, plugin_id: &PluginId, plugin_name: &str, entrypoint_id: &EntrypointId, entrypoint_name: &str) {
        tracing::trace!("replace_view is called. container: {:?}", container);

        self.plugin_id = Some(plugin_id.clone());
        self.plugin_name = Some(plugin_name.to_string());
        self.entrypoint_id = Some(entrypoint_id.clone());
        self.entrypoint_name = Some(entrypoint_name.to_string());

        let children = container.widget_children.into_iter()
            .map(|child| self.create_component_widget(child))
            .collect::<Vec<_>>();

        self.root_widget.find_child_with_id(container.widget_id)
            .unwrap_or_else(|| panic!("widget with id {:?} doesn't exist", container.widget_id))
            .set_children(children)
            .expect("unable to set children");
    }

    pub fn handle_event(&self, plugin_id: PluginId, event: ComponentWidgetEvent) -> Option<UiViewEvent> {
        let widget = self.root_widget
            .find_child_with_id(event.widget_id())
            .expect("created event for non existing widget?");

        event.handle(plugin_id, widget)
    }

    pub fn toggle_action_panel(&self) {
        self.root_widget.toggle_action_panel()
    }

    pub fn get_action_ids(&self) -> Vec<UiWidgetId> {
        self.root_widget.get_action_ids()
    }
}
