use gauntlet_common::model::UiPropertyValue;
use gauntlet_common::model::UiWidgetId;

use crate::ui::AppMsg;

#[derive(Debug, Clone)]
pub enum UiViewEvent {
    View {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    Open {
        href: String,
    },
    AppEvent {
        event: AppMsg,
    },
}
