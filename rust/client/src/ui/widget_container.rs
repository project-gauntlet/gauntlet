use common::model::{EntrypointId, PluginId, UiViewEvent, UiWidget};

use crate::ui::theme::Element;
use crate::ui::widget::{ComponentRenderContext, ComponentWidgetEvent, ComponentWidgetWrapper};

pub struct PluginWidgetContainer {
    root_widget: ComponentWidgetWrapper,
    plugin_id: Option<PluginId>,
    entrypoint_id: Option<EntrypointId>
}

impl PluginWidgetContainer {
    pub fn new() -> Self {
        Self {
            root_widget: ComponentWidgetWrapper::root(0),
            plugin_id: None,
            entrypoint_id: None,
        }
    }

    pub fn get_plugin_id(&self) -> PluginId {
        self.plugin_id.clone().expect("plugin id should always exist after render")
    }

    pub fn get_entrypoint_id(&self) -> EntrypointId {
        self.entrypoint_id.clone().expect("entrypoint id should always exist after render")
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

    pub fn replace_view(&mut self, container: UiWidget, plugin_id: &PluginId, entrypoint_id: &EntrypointId) {
        tracing::trace!("replace_view is called. container: {:?}", container);

        self.plugin_id = Some(plugin_id.clone());
        self.entrypoint_id = Some(entrypoint_id.clone());

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
}
