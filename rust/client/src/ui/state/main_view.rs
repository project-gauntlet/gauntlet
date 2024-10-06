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

    pub fn action_panel(prev_state: &mut MainViewState, focus_first: bool) {
        *prev_state = Self::ActionPanel {
            focused_action_item: ScrollHandle::new(focus_first),
        }
    }
}

impl Focus<SearchResultEntrypointAction> for MainViewState {
    fn primary(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        match self {
            MainViewState::None => {
                panic!("invalid state")
            }
            MainViewState::ActionPanel { focused_action_item } => {
                match focused_action_item.index {
                    None => Command::none(),
                    Some(widget_id) => {
                        Command::perform(async {}, move |_| AppMsg::OnEntrypointAction { widget_id, keyboard: true })
                    }
                }
            }
        }
    }

    fn secondary(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        // secondary action doesn't do anything when action panel is open
        panic!("invalid state")
    }

    fn back(&mut self) -> Command<AppMsg> {
        match self {
            MainViewState::None => {
                Command::perform(async {}, |_| AppMsg::HideWindow)
            }
            MainViewState::ActionPanel { .. } => {
                MainViewState::initial(self);
                Command::none()
            }
        }
    }

    fn next(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn previous(&mut self) -> Command<AppMsg> {
        todo!()
    }

    fn up(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        match self {
            MainViewState::None => Command::none(),
            MainViewState::ActionPanel { focused_action_item, .. } => {
                focused_action_item.focus_previous()
            }
        }
    }

    fn down(&mut self, focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
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

    fn left(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        todo!()
    }

    fn right(&mut self, _focus_list: &[SearchResultEntrypointAction]) -> Command<AppMsg> {
        todo!()
    }
}
