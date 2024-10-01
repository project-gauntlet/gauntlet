use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::Focus;
use crate::ui::AppMsg;
use common::model::UiWidgetId;
use iced::Command;

#[derive(Debug, Clone)]
pub enum PluginViewState {
    None,
    ActionPanel {
        // ephemeral state
        focused_action_item: ScrollHandle<UiWidgetId>,
    }
}

impl PluginViewState {
    pub fn new() -> Self {
        PluginViewState::None
    }

    pub fn initial(prev_state: &mut PluginViewState) {
        *prev_state = Self::None
    }

    pub fn action_panel(prev_state: &mut PluginViewState) {
        *prev_state = Self::ActionPanel {
            focused_action_item: ScrollHandle::new(),
        }
    }
}

impl Focus<UiWidgetId> for PluginViewState {
    fn enter(&mut self, focus_list: &[UiWidgetId]) -> Command<AppMsg> {
        match self {
            PluginViewState::None => Command::none(),
            PluginViewState::ActionPanel { focused_action_item, .. } => {
                if let Some(widget_id) = focused_action_item.get(focus_list) {
                    let widget_id = *widget_id;
                    Command::perform(async {}, move |_| AppMsg::OnEntrypointAction(widget_id))
                } else {
                    Command::none()
                }
            }
        }
    }

    fn escape(&mut self) -> Command<AppMsg> {
        match self {
            PluginViewState::None => {
                panic!("invalid state")
            }
            PluginViewState::ActionPanel { .. } => {
                Command::perform(async {}, |_| AppMsg::ToggleActionPanel)
            }
        }
    }

    fn tab(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn shift_tab(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn arrow_up(&mut self, _focus_list: &[UiWidgetId]) -> Command<AppMsg> {
        match self {
            PluginViewState::None => Command::none(),
            PluginViewState::ActionPanel { focused_action_item, .. } => {
                focused_action_item.focus_previous()
            }
        }
    }

    fn arrow_down(&mut self, focus_list: &[UiWidgetId]) -> Command<AppMsg> {
        match self {
            PluginViewState::None => Command::none(),
            PluginViewState::ActionPanel { focused_action_item } => {
                if focus_list.len() != 0 {
                    focused_action_item.focus_next(focus_list.len())
                } else {
                    Command::none()
                }
            }
        }
    }

    fn arrow_left(&mut self, _focus_list: &[UiWidgetId]) -> Command<AppMsg> {
        todo!()
    }

    fn arrow_right(&mut self, _focus_list: &[UiWidgetId]) -> Command<AppMsg> {
        todo!()
    }
}
