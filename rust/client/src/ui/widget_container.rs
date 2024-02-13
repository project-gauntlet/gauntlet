use zbus::SignalContext;
use common::model::PluginId;
use std::future::Future;
use crate::model::NativeUiWidget;
use crate::ui::theme::Element;
use crate::ui::widget::{ComponentRenderContext, ComponentWidgetEvent, ComponentWidgetWrapper};

pub struct PluginWidgetContainer {
    root_widget: ComponentWidgetWrapper,
}

impl PluginWidgetContainer {
    pub fn new() -> Self {
        Self {
            root_widget: ComponentWidgetWrapper::root(0),
        }
    }

    pub fn render_widget<'a>(&self, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
        self.root_widget.render_widget(context)
    }

    fn create_component_widget(&mut self, ui_widget: NativeUiWidget) -> ComponentWidgetWrapper {
        let children = ui_widget.widget_children
            .into_iter()
            .map(|ui_widget| self.create_component_widget(ui_widget))
            .collect();

        ComponentWidgetWrapper::widget(ui_widget.widget_id, &ui_widget.widget_type, ui_widget.widget_properties, children)
            .expect("unable to create widget")
    }

    pub fn replace_view(&mut self, container: NativeUiWidget) {
        tracing::trace!("replace_view is called. container: {:?}", container);

        let children = container.widget_children.into_iter()
            .map(|child| self.create_component_widget(child))
            .collect::<Vec<_>>();

        self.root_widget.find_child_with_id(container.widget_id)
            .unwrap_or_else(|| panic!("widget with id {:?} doesn't exist", container.widget_id))
            .set_children(children)
            .expect("unable to set children");
    }

    pub fn handle_event<'a, 'b>(&'a self, signal_context: &'b SignalContext<'_>, plugin_id: PluginId, event: ComponentWidgetEvent) -> impl Future<Output=()> + 'b + Send {
        let widget = self.root_widget
            .find_child_with_id(event.widget_id())
            .expect("created event for non existing widget?");

        event.handle(signal_context, plugin_id, widget)
    }
}
