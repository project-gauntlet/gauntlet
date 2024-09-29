use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::Focus;
use crate::ui::AppMsg;
use common::model::SearchResultEntrypointAction;
use iced::Command;

pub enum MainViewState {
    None,
    ActionPanel {
        // ephemeral state
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

    pub fn action_panel(prev_state: &mut MainViewState) {
        *prev_state = Self::ActionPanel {
            focused_action_item: ScrollHandle::new(),
        }
    }
}

impl Focus<SearchResultEntrypointAction> for MainViewState {
    fn enter(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
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

    fn arrow_up(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        match self {
            MainViewState::None => Command::none(),
            MainViewState::ActionPanel { focused_action_item, .. } => {
                focused_action_item.focus_previous()
            }
        }
    }

    fn arrow_down(&mut self, focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        match self {
            MainViewState::None => Command::none(),
            MainViewState::ActionPanel { focused_action_item } => {
                if focus_list.len() != 0 {
                    focused_action_item.focus_next(focus_list.len() + 1)
                } else {
                    Command::none()
                }
            }
        }
    }

    fn arrow_left(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        todo!()
    }

    fn arrow_right(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        todo!()
    }
}
