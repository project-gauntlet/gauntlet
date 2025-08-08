use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::widget::action_panel::action_item_container_id;

#[derive(Debug, Clone)]
pub enum PluginViewState {
    None,
    ActionPanel { scroll_handle: ScrollHandle },
}

impl PluginViewState {
    pub fn new() -> Self {
        PluginViewState::None
    }

    pub fn initial(prev_state: &mut PluginViewState) {
        *prev_state = Self::None
    }

    pub fn action_panel(prev_state: &mut PluginViewState, focus_first: bool) {
        let first_action_item = if focus_first {
            Some(action_item_container_id(0))
        } else {
            None
        };

        *prev_state = Self::ActionPanel {
            scroll_handle: ScrollHandle::new(first_action_item),
        }
    }
}
