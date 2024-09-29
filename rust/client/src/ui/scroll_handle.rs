use std::marker::PhantomData;
use iced::Command;
use iced::widget::scrollable;
use iced::widget::scrollable::{scroll_to, AbsoluteOffset};
use crate::ui::AppMsg;

const ESTIMATED_ITEM_SIZE: f32 = 38.8;

#[derive(Clone, Debug)]
pub struct ScrollHandle<T> {
    phantom: PhantomData<T>,
    pub scrollable_id: scrollable::Id,
    pub index: usize,
    pub offset: usize,
}

impl<T> ScrollHandle<T> {
    pub fn new() -> ScrollHandle<T> {
        ScrollHandle {
            phantom: PhantomData,
            scrollable_id: scrollable::Id::unique(),
            index: 0,
            offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.offset = 0;
    }

    pub fn get<'a>(&self, search_results: &'a [T]) -> Option<&'a T> {
        search_results.get(self.index)
    }

    pub fn focus_next(&mut self, item_amount: usize) -> Command<AppMsg> {
        self.offset = if self.offset < 8 {
            self.offset + 1
        } else {
            8
        };

        if self.index < item_amount - 1 {
            self.index = self.index + 1;

            let pos_y = self.index as f32 * ESTIMATED_ITEM_SIZE - (self.offset as f32 * ESTIMATED_ITEM_SIZE);

            scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: pos_y })
        } else {
            Command::none()
        }
    }

    pub fn focus_previous(&mut self) -> Command<AppMsg> {
        self.offset = if self.offset > 1 {
            self.offset - 1
        } else {
            1
        };

        if self.index > 0 {
            self.index = self.index - 1;

            let pos_y = self.index as f32 * ESTIMATED_ITEM_SIZE - (self.offset as f32 * ESTIMATED_ITEM_SIZE);

            scroll_to(self.scrollable_id.clone(), AbsoluteOffset { x: 0.0, y: pos_y })
        } else {
            Command::none()
        }
    }
}