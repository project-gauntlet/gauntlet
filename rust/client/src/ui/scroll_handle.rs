use std::marker::PhantomData;
use iced::Command;
use iced::widget::scrollable;
use iced::widget::scrollable::{scroll_to, AbsoluteOffset};
use crate::ui::AppMsg;

pub const ESTIMATED_MAIN_LIST_ITEM_HEIGHT: f32 = 38.8;
pub const ESTIMATED_ACTION_ITEM_HEIGHT: f32 = 38.8; // TODO
pub const ESTIMATED_GRID_ITEM_HEIGHT: f32 = 190.0; // TODO

#[derive(Clone, Debug)]
pub struct ScrollHandle<T> {
    phantom: PhantomData<T>,
    pub scrollable_id: scrollable::Id,
    pub index: Option<usize>,
    offset: usize,
    item_height: f32,
}

impl<T> ScrollHandle<T> {
    pub fn new(first_focused: bool, item_height: f32) -> ScrollHandle<T> {
        ScrollHandle {
            phantom: PhantomData,
            scrollable_id: scrollable::Id::unique(),
            index: if first_focused { Some(0) } else { None },
            offset: 0,
            item_height,
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

    pub fn focus_next(&mut self, total_item_amount: usize) -> Option<Command<AppMsg>> {
        self.focus_next_in(total_item_amount, 1)
    }

    pub fn focus_next_in(&mut self, total_item_amount: usize, amount: usize) -> Option<Command<AppMsg>> {
        self.offset = if self.offset < 7 {
            self.offset + 1
        } else {
            7
        };

        match self.index.as_mut() {
            None => {
                // focus first
                if total_item_amount > 0 {
                    self.index = Some(0);

                    Some(self.scroll_to(0))
                } else {
                    None
                }
            }
            Some(index) => {
                // focus next if there is an item
                let new_index = *index + amount;
                if new_index < total_item_amount {
                    *index = new_index;

                    let index = *index;

                    Some(self.scroll_to(index))
                } else {
                    None
                }
            }
        }
    }

    pub fn focus_previous(&mut self) -> Option<Command<AppMsg>> {
        self.focus_previous_in(1)
    }

    pub fn focus_previous_in(&mut self, amount: usize) -> Option<Command<AppMsg>> {
        self.offset = if self.offset > 1 {
            self.offset - 1
        } else {
            1
        };

        match self.index.as_mut() {
            None => None,
            Some(index) => {
                match index.checked_sub(amount) { // basically a check if result is >= 0
                    Some(new_index) => {
                        *index = new_index;

                        let index = *index;

                        Some(self.scroll_to(index))
                    }
                    None => None
                }
            }
        }
    }
    pub fn scroll_to<Message: 'static>(&self, row_index: usize) -> Command<Message> {
        self.scroll_to_offset(row_index, true)
    }

    pub fn scroll_to_offset<Message: 'static>(&self, row_index: usize, no_offset: bool) -> Command<Message> {
        let mut pos_y = row_index as f32 * self.item_height;

        if !no_offset {
            pos_y = pos_y - (self.offset as f32 * self.item_height);
        }

        scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: pos_y })
    }
}