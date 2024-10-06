use std::marker::PhantomData;
use iced::Command;
use iced::widget::scrollable;
use iced::widget::scrollable::{scroll_to, AbsoluteOffset};
use crate::ui::AppMsg;

// TODO this size of item for main view list, incorrect for actions,
//  but amount of actions is usually small so it is not that noticeable
const ESTIMATED_ITEM_SIZE: f32 = 38.8;

#[derive(Clone, Debug)]
pub struct ScrollHandle<T> {
    phantom: PhantomData<T>,
    pub scrollable_id: scrollable::Id,
    pub index: Option<usize>,
    offset: usize,
}

impl<T> ScrollHandle<T> {
    pub fn new(first_focused: bool) -> ScrollHandle<T> {
        ScrollHandle {
            phantom: PhantomData,
            scrollable_id: scrollable::Id::unique(),
            index: if first_focused { Some(0) } else { None },
            offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = None;
        self.offset = 0;
    }

    pub fn get<'a>(&self, search_results: &'a [T]) -> Option<&'a T> {
        match self.index {
            None => None,
            Some(index) => search_results.get(index)
        }
    }

    pub fn focus_next(&mut self, total_item_amount: usize) -> Command<AppMsg> {
        self.offset = if self.offset < 8 {
            self.offset + 1
        } else {
            8
        };

        match self.index.as_mut() {
            None => {
                if total_item_amount > 0 {
                    self.index = Some(0);

                    self.scroll_to(0)
                } else {
                    Command::none()
                }
            }
            Some(index) => {
                if *index < total_item_amount - 1 {
                    *index = *index + 1;

                    let index = *index;

                    self.scroll_to(index)
                } else {
                    Command::none()
                }
            }
        }
    }

    pub fn focus_previous(&mut self) -> Command<AppMsg> {
        self.offset = if self.offset > 1 {
            self.offset - 1
        } else {
            1
        };

        match self.index.as_mut() {
            None => Command::none(),
            Some(index) => {
                if *index > 0 {
                    *index = *index - 1;

                    let index = *index;

                    self.scroll_to(index)
                } else {
                    Command::none()
                }
            }
        }
    }

    fn scroll_to<Message: 'static>(&self, index: usize) -> Command<Message> {
        let pos_y = index as f32 * ESTIMATED_ITEM_SIZE - (self.offset as f32 * ESTIMATED_ITEM_SIZE);

        scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: pos_y })
    }
}