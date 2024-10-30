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

    pub fn reset(&mut self, first_focused: bool) {
        self.index = if first_focused { Some(0) } else { None };
        self.offset = 0;
    }

    pub fn unfocus(&mut self) {
        self.index = None;
    }

    pub fn get<'a>(&self, search_results: &'a [T]) -> Option<&'a T> {
        match self.index {
            None => None,
            Some(index) => search_results.get(index)
        }
    }

    pub fn focus_next_row(&mut self, total_items: usize, total_columns: usize) -> Command<AppMsg> {
        self.offset = if self.offset < 7 {
            self.offset + 1
        } else {
            7
        };

        match self.index.as_mut() {
            None => {
                // focus first
                if total_items > 0 {
                    self.index = Some(0);

                    self.scroll_to(0)
                } else {
                    Command::none()
                }
            }
            Some(index) => {
                // focus next row only if there is an item on it
                let new_index = *index + total_columns;
                if new_index < total_items {
                    *index = new_index;

                    let index = *index;

                    self.scroll_to(index)
                } else {
                    Command::none()
                }
            }
        }
    }

    pub fn focus_previous_row(&mut self, total_columns: usize) -> Command<AppMsg> {
        self.offset = if self.offset > 1 {
            self.offset - 1
        } else {
            1
        };

        match self.index.as_mut() {
            None => Command::none(),
            Some(index) => {
                match index.checked_sub(total_columns) { // basically a check if result is >= 0
                    Some(new_index) => {
                        *index = new_index;

                        let index = *index;

                        self.scroll_to(index)
                    }
                    None => Command::none()
                }
            }
        }
    }

    pub fn focus_next_column(&mut self, total_items: usize, total_columns: usize) -> Command<AppMsg> {
        match self.index.as_mut() {
            None => Command::none(),
            Some(index) => {
                // put focus on next column only if it doesn't put it on next row
                // and if there is an item there
                let new_index = *index + 1;
                if *index % total_columns != 0 && new_index <= total_items {
                    *index = new_index;
                }

                Command::none()
            }
        }
    }

    pub fn focus_previous_column(&mut self, total_columns: usize) -> Command<AppMsg> {
        match self.index.as_mut() {
            None => Command::none(),
            Some(index) => {
                // put focus on previous column only if it doesn't put it on previous row
                if *index == 0 {
                    Command::none()
                } else {
                    let new_index = *index - 1;
                    if new_index % total_columns != 0 {
                        *index = new_index;
                    }

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