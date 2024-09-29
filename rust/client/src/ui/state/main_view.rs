use iced::Command;
use common::model::SearchResultEntrypointAction;
use crate::ui::{AppModel, AppMsg};
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::Focus;

pub enum MainViewState {
    None,
    ActionPanel {
        // ephemeral state
        entrypoint_actions_size: usize,
        focused_action_item: ScrollHandle<SearchResultEntrypointAction>,
    }
}

impl MainViewState {
    pub fn new() -> Self {
        MainViewState::None
    }

    pub fn initial(prev_state: &mut MainViewState) {
        *prev_state = Self::None
    }

    pub fn action_panel(prev_state: &mut MainViewState, entrypoint_actions: &[SearchResultEntrypointAction]) {
        *prev_state = Self::ActionPanel {
            entrypoint_actions_size: entrypoint_actions.len(),
            focused_action_item: ScrollHandle::new(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, MainViewState::None)
    }

    pub fn is_action_panel(&self) -> bool {
        matches!(self, MainViewState::None)
    }
}

impl Focus for MainViewState {
    fn enter(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn escape(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn tab(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn shift_tab(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn arrow_up(&mut self) -> Command<AppMsg> {
        match self {
            MainViewState::None => Command::none(),
            MainViewState::ActionPanel { focused_action_item, .. } => {
                focused_action_item.focus_previous()
            }
        }
    }

    fn arrow_down(&mut self) -> Command<AppMsg> {
        match self {
            MainViewState::None => Command::none(),
            MainViewState::ActionPanel { entrypoint_actions_size, focused_action_item } => {
                if *entrypoint_actions_size != 0 {
                    focused_action_item.focus_next(*entrypoint_actions_size + 1)
                } else {
                    Command::none()
                }
            }
        }
    }

    fn arrow_left(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn arrow_right(&mut self) -> Command<AppMsg> {
        todo!()
    }
}
