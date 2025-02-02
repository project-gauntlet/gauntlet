use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::scroll_handle::ESTIMATED_ACTION_ITEM_HEIGHT;

#[derive(Debug, Clone)]
pub enum PluginViewState {
    None,
    ActionPanel {
        // ephemeral state
        focused_action_item: ScrollHandle,
    },
}

impl PluginViewState {
    pub fn new() -> Self {
        PluginViewState::None
    }

    pub fn initial(prev_state: &mut PluginViewState) {
        *prev_state = Self::None
    }

    pub fn action_panel(prev_state: &mut PluginViewState, focus_first: bool) {
        *prev_state = Self::ActionPanel {
            focused_action_item: ScrollHandle::new(focus_first, ESTIMATED_ACTION_ITEM_HEIGHT, 7),
        }
    }
}
