use common::model::{UiPropertyValue, UiWidgetId};

#[derive(Debug, Clone)]
pub enum UiViewEvent {
    View {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    Open {
        href: String
    },
}
