use std::mem;

use iced::Rectangle;
use iced::Task;
use iced::Vector;
use iced::advanced::widget;
use iced::advanced::widget::Operation;
use iced::advanced::widget::operate;
use iced::advanced::widget::operation::Outcome;
use iced::advanced::widget::operation::Scrollable;
use iced::widget::container;
use iced::widget::scrollable;
use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::scrollable::scroll_to;
use indexmap::IndexMap;
use itertools::Itertools;

use crate::ui::AppMsg;

pub struct ScrollContent<T> {
    items: IndexMap<container::Id, T>,
}

impl<T> ScrollContent<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut content = Self { items: IndexMap::new() };
        content.update(items);
        content
    }

    pub fn new_with_ids(items: IndexMap<container::Id, T>) -> Self {
        Self { items }
    }

    pub fn update(&mut self, new_items: Vec<T>) {
        let items = mem::replace(&mut self.items, IndexMap::new());

        let mut keys: Vec<_> = items.into_keys().collect();

        keys.resize_with(new_items.len(), || container::Id::unique());

        self.items = keys.into_iter().zip(new_items.into_iter()).collect();
    }

    pub fn items(&self) -> &IndexMap<container::Id, T> {
        &self.items
    }

    pub fn ids(&self) -> Vec<container::Id> {
        self.items.iter().map(|(id, _)| id.clone()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ScrollHandle {
    pub scrollable_id: scrollable::Id,
    pub current_item_id: Option<container::Id>,
}

impl ScrollHandle {
    pub fn new(current_item_id: Option<container::Id>) -> ScrollHandle {
        ScrollHandle {
            scrollable_id: scrollable::Id::unique(),
            current_item_id,
        }
    }

    pub fn from(scroll_handle: &ScrollHandle, current_item_id: Option<container::Id>) -> ScrollHandle {
        ScrollHandle {
            scrollable_id: scroll_handle.scrollable_id.clone(),
            current_item_id,
        }
    }

    pub fn get<'a, T>(&self, items: &'a ScrollContent<T>) -> Option<&'a T> {
        self.get_by_id(items, self.current_item_id.clone())
    }

    pub fn get_by_id<'a, T>(
        &self,
        items: &'a ScrollContent<T>,
        target_item_id: Option<container::Id>,
    ) -> Option<&'a T> {
        let Some(id) = &target_item_id else {
            return None;
        };

        items.items.get(id)
    }

    pub fn get_index<T>(&self, items: &ScrollContent<T>) -> Option<usize> {
        let Some(id) = &self.current_item_id else {
            return None;
        };

        items.items.get_index_of(id)
    }

    pub fn grid_focus_down(&mut self, grid: Vec<Vec<container::Id>>) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        self.grid_focus_target(DownScrollBehavior, grid)
    }

    pub fn grid_focus_up(&mut self, grid: Vec<Vec<container::Id>>) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        self.grid_focus_target(UpScrollBehavior, grid)
    }

    pub fn grid_focus_left(&mut self, grid: Vec<Vec<container::Id>>) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        self.grid_focus_target(LeftScrollBehavior, grid)
    }

    pub fn grid_focus_right(&mut self, grid: Vec<Vec<container::Id>>) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        self.grid_focus_target(RightScrollBehavior, grid)
    }

    pub fn list_focus_down(&mut self, list: Vec<container::Id>) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        let grid = list.into_iter().map(|item| vec![item]).collect();
        self.grid_focus_target(DownScrollBehavior, grid)
    }

    pub fn list_focus_up(&mut self, list: Vec<container::Id>) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        let grid = list.into_iter().map(|item| vec![item]).collect();
        self.grid_focus_target(UpScrollBehavior, grid)
    }

    fn grid_focus_target(
        &mut self,
        behavior: impl ScrollBehavior,
        grid: Vec<Vec<container::Id>>,
    ) -> (Option<container::Id>, Option<Task<AppMsg>>) {
        let Some(current_item_id) = &self.current_item_id else {
            let target_item_id = behavior.unfocused(grid);
            return (target_item_id.clone(), self.focus_target(target_item_id));
        };

        let Some((row_index, row)) = grid.iter().find_position(|row| row.contains(&current_item_id)) else {
            return (self.current_item_id.clone(), None);
        };

        let Some(column_index) = row.iter().position(|item| item == current_item_id) else {
            return (self.current_item_id.clone(), None);
        };

        let target_item_id = behavior.focused(grid, current_item_id.clone(), row_index, column_index);

        (target_item_id.clone(), self.focus_target(target_item_id))
    }

    pub fn focus_target(&mut self, target_item_id: Option<container::Id>) -> Option<Task<AppMsg>> {
        if self.current_item_id == target_item_id {
            return None;
        }

        let task = focus_target(self.scrollable_id.clone(), target_item_id.clone(), 40.)
            .chain(Task::done(AppMsg::SetCurrentFocusedItem(target_item_id)));

        Some(task)
    }

    pub fn set_current_focused_item(&mut self, target_item_id: Option<container::Id>) -> Task<AppMsg> {
        self.current_item_id = target_item_id.clone();
        Task::none()
    }
}

trait ScrollBehavior {
    fn unfocused(&self, grid: Vec<Vec<container::Id>>) -> Option<container::Id>;
    fn focused(
        &self,
        grid: Vec<Vec<container::Id>>,
        current_item_id: container::Id,
        row_index: usize,
        column_index: usize,
    ) -> Option<container::Id>;
}

struct DownScrollBehavior;
impl ScrollBehavior for DownScrollBehavior {
    fn unfocused(&self, grid: Vec<Vec<container::Id>>) -> Option<container::Id> {
        grid.first()
            .map(|row| row.first().map(|item| item.clone()))
            .unwrap_or_default()
    }

    fn focused(
        &self,
        grid: Vec<Vec<container::Id>>,
        current_item_id: container::Id,
        row_index: usize,
        column_index: usize,
    ) -> Option<container::Id> {
        let Some(next_row) = grid.get(row_index + 1) else {
            return Some(current_item_id);
        };

        let Some(next_item) = next_row.get(column_index) else {
            return next_row.iter().last().cloned();
        };

        Some(next_item.clone())
    }
}

struct UpScrollBehavior;
impl ScrollBehavior for UpScrollBehavior {
    fn unfocused(&self, _grid: Vec<Vec<container::Id>>) -> Option<container::Id> {
        None
    }

    fn focused(
        &self,
        grid: Vec<Vec<container::Id>>,
        current_item_id: container::Id,
        row_index: usize,
        column_index: usize,
    ) -> Option<container::Id> {
        let Some(next_row_index) = row_index.checked_sub(1) else {
            return None;
        };

        let Some(next_row) = grid.get(next_row_index) else {
            return Some(current_item_id);
        };

        let Some(next_item) = next_row.get(column_index) else {
            return next_row.iter().last().cloned();
        };

        Some(next_item.clone())
    }
}

struct LeftScrollBehavior;
impl ScrollBehavior for LeftScrollBehavior {
    fn unfocused(&self, _grid: Vec<Vec<container::Id>>) -> Option<container::Id> {
        None
    }

    fn focused(
        &self,
        grid: Vec<Vec<container::Id>>,
        current_item_id: container::Id,
        row_index: usize,
        column_index: usize,
    ) -> Option<container::Id> {
        let Some(next_row) = grid.get(row_index) else {
            return Some(current_item_id);
        };

        let Some(next_column_index) = column_index.checked_sub(1) else {
            return Some(current_item_id);
        };

        let Some(next_item) = next_row.get(next_column_index) else {
            return Some(current_item_id);
        };

        Some(next_item.clone())
    }
}

struct RightScrollBehavior;
impl ScrollBehavior for RightScrollBehavior {
    fn unfocused(&self, _grid: Vec<Vec<container::Id>>) -> Option<container::Id> {
        None
    }

    fn focused(
        &self,
        grid: Vec<Vec<container::Id>>,
        current_item_id: container::Id,
        row_index: usize,
        column_index: usize,
    ) -> Option<container::Id> {
        let Some(next_row) = grid.get(row_index) else {
            return Some(current_item_id);
        };

        let Some(next_item) = next_row.get(column_index + 1) else {
            return Some(current_item_id);
        };

        Some(next_item.clone())
    }
}

fn focus_target(scrollable_id: scrollable::Id, target_item_id: Option<container::Id>, padding: f32) -> Task<AppMsg> {
    let Some(target_item_id) = target_item_id else {
        return scroll_to(scrollable_id.clone(), AbsoluteOffset::default());
    };

    struct CalculateScrollToIdOffset {
        scrollable: widget::Id,
        target: widget::Id,
        viewport_rectangle: Option<Rectangle>,
        viewport_translation: Option<Vector>,
        target_rectangle: Option<Rectangle>,
        padding: f32,
    }

    impl Operation<AbsoluteOffset> for CalculateScrollToIdOffset {
        fn container(
            &mut self,
            id: Option<&widget::Id>,
            bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<AbsoluteOffset>),
        ) {
            if Some(&self.target) == id {
                self.target_rectangle = Some(bounds)
            } else {
                operate_on_children(self);
            }
        }

        fn scrollable(
            &mut self,
            id: Option<&widget::Id>,
            bounds: Rectangle,
            _content_bounds: Rectangle,
            translation: Vector,
            _state: &mut dyn Scrollable,
        ) {
            if Some(&self.scrollable) == id {
                self.viewport_rectangle = Some(bounds);
                self.viewport_translation = Some(translation);
            }
        }

        fn finish(&self) -> Outcome<AbsoluteOffset> {
            let Some(target_rectangle) = self.target_rectangle else {
                return Outcome::None;
            };

            let Some(viewport_rectangle) = self.viewport_rectangle else {
                return Outcome::None;
            };

            let Some(viewport_translation) = self.viewport_translation else {
                return Outcome::None;
            };

            let r_x = target_rectangle.x;
            let r_y = target_rectangle.y;
            let r_w = target_rectangle.width;
            let r_h = target_rectangle.height;
            let t_x = viewport_translation.x;
            let t_y = viewport_translation.y;
            let v_w = viewport_rectangle.width;
            let v_h = viewport_rectangle.height;
            let v_x = viewport_rectangle.x;
            let v_y = viewport_rectangle.y;

            let pad = self.padding;

            let offset_x = t_x.max(r_x + r_w - (v_x + v_w) + pad).min(r_x - v_x - pad);
            let offset_y = t_y.max(r_y + r_h - (v_y + v_h) + pad).min(r_y - v_y - pad);

            let offset = AbsoluteOffset {
                x: offset_x,
                y: offset_y,
            };

            Outcome::Some(offset)
        }
    }

    let operation = CalculateScrollToIdOffset {
        scrollable: scrollable_id.clone().into(),
        target: target_item_id.clone().into(),
        viewport_rectangle: None,
        viewport_translation: None,
        target_rectangle: None,
        padding,
    };

    let scrollable_id = scrollable_id.clone();
    operate(operation).then(move |offset| scroll_to(scrollable_id.clone(), offset))
}
