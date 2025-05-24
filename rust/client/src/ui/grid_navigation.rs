use std::ops::Div;
use std::ops::Rem;

#[derive(Debug)]
pub struct GridSectionData {
    pub start_index: usize,
    pub start_row_index: usize,
    pub amount_in_section: usize,
    pub width: usize,
}

struct GridRowData {
    row_index: usize,
    amount_in_row: usize,
    max_amount_in_row: usize,
}

struct GridCurrentRowData {
    row_index: usize,
    column_index: usize,
    amount_in_row: usize,
    max_amount_in_row: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct GridItemOffset {
    pub row_index: usize,
    pub offset: usize,
}

fn grid_row_data(
    amount_per_section_total: Vec<GridSectionData>,
    current_index: usize,
) -> (Option<GridRowData>, GridCurrentRowData, Option<GridRowData>) {
    let mut previous_section_index: Option<usize> = None;
    let mut current_section_index: usize = 0;

    for (
        index,
        GridSectionData {
            start_index,
            amount_in_section,
            ..
        },
    ) in amount_per_section_total.iter().enumerate()
    {
        if start_index + amount_in_section >= (current_index + 1) {
            current_section_index = index;
            break;
        }
        previous_section_index = Some(index);
    }

    let prev_section = previous_section_index.and_then(|index| amount_per_section_total.get(index));
    let current_section = amount_per_section_total
        .get(current_section_index)
        .expect("guarantied to exist");
    let next_section = amount_per_section_total.get(current_section_index + 1);

    let item_index_in_section = current_index - current_section.start_index;
    let row_index_in_section = usize::div(item_index_in_section, current_section.width);
    let row_index = current_section.start_row_index + row_index_in_section;

    let amount_of_rows = usize::div_ceil(current_section.amount_in_section, current_section.width);
    let last_row = row_index_in_section + 1 == amount_of_rows;

    let current_row = {
        if last_row {
            let reminder = usize::rem(current_section.amount_in_section, current_section.width);
            GridCurrentRowData {
                row_index,
                column_index: usize::rem(item_index_in_section, current_section.width),
                amount_in_row: if reminder == 0 { current_section.width } else { reminder },
                max_amount_in_row: current_section.width,
            }
        } else {
            GridCurrentRowData {
                row_index,
                column_index: usize::rem(item_index_in_section, current_section.width),
                amount_in_row: current_section.width,
                max_amount_in_row: current_section.width,
            }
        }
    };

    let prev_row = {
        if row_index_in_section == 0 {
            match prev_section {
                None => None,
                Some(prev_section) => {
                    let reminder = usize::rem(prev_section.amount_in_section, prev_section.width);
                    Some(GridRowData {
                        row_index: current_row.row_index - 1,
                        amount_in_row: if reminder == 0 { prev_section.width } else { reminder },
                        max_amount_in_row: prev_section.width,
                    })
                }
            }
        } else {
            Some(GridRowData {
                row_index: current_row.row_index - 1,
                amount_in_row: current_section.width,
                max_amount_in_row: current_section.width,
            })
        }
    };

    let next_row = {
        if last_row {
            match next_section {
                None => None,
                Some(next_section) => {
                    let reminder = usize::rem(next_section.amount_in_section, next_section.width);
                    Some(GridRowData {
                        row_index: current_row.row_index + 1,
                        amount_in_row: if reminder == 0 { next_section.width } else { reminder },
                        max_amount_in_row: next_section.width,
                    })
                }
            }
        } else {
            let amount_of_full_rows = usize::div(current_section.amount_in_section, current_section.width);
            if row_index_in_section + 1 == amount_of_full_rows {
                let reminder = usize::rem(current_section.amount_in_section, current_section.width);
                Some(GridRowData {
                    row_index: current_row.row_index + 1,
                    amount_in_row: if reminder == 0 { current_section.width } else { reminder },
                    max_amount_in_row: current_section.width,
                })
            } else {
                Some(GridRowData {
                    row_index: current_row.row_index + 1,
                    amount_in_row: current_section.width,
                    max_amount_in_row: current_section.width,
                })
            }
        }
    };

    (prev_row, current_row, next_row)
}

pub fn grid_up_offset(current_index: usize, amount_per_section_total: Vec<GridSectionData>) -> Option<GridItemOffset> {
    let (prev_row, current_row, _next_row) = grid_row_data(amount_per_section_total, current_index);

    match prev_row {
        None => None,
        Some(prev_row) => {
            if prev_row.amount_in_row < prev_row.max_amount_in_row {
                if prev_row.amount_in_row > (current_row.column_index + 1) {
                    Some(GridItemOffset {
                        row_index: prev_row.row_index,
                        offset: prev_row.max_amount_in_row - prev_row.amount_in_row + (current_row.column_index + 1),
                    })
                } else {
                    Some(GridItemOffset {
                        row_index: prev_row.row_index,
                        offset: current_row.column_index + 1,
                    })
                }
            } else {
                Some(GridItemOffset {
                    row_index: prev_row.row_index,
                    offset: current_row.max_amount_in_row,
                })
            }
        }
    }
}

pub fn grid_down_offset(
    current_index: usize,
    amount_per_section_total: Vec<GridSectionData>,
) -> Option<GridItemOffset> {
    let (_prev_row, current_row, next_row) = grid_row_data(amount_per_section_total, current_index);

    match next_row {
        None => None,
        Some(next_row) => {
            if (current_row.column_index + 1) > next_row.amount_in_row {
                Some(GridItemOffset {
                    row_index: next_row.row_index,
                    offset: (current_row.max_amount_in_row - (current_row.column_index + 1)) + next_row.amount_in_row,
                })
            } else {
                Some(GridItemOffset {
                    row_index: next_row.row_index,
                    offset: current_row.amount_in_row,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare_sections(data: Vec<Vec<Vec<usize>>>) -> Vec<GridSectionData> {
        let mut cumulative_item_index = 0;
        let mut cumulative_row_index = 0;

        data.iter()
            .map(|section| {
                let cumulative_item_index_at_start = cumulative_item_index;
                let cumulative_row_index_at_start = cumulative_row_index;

                let amount = section.iter().map(|row| row.iter().sum::<usize>()).sum();

                let width = section[0].len();

                assert!(section.iter().all(|row| row.len() == width));
                for row in section {
                    assert!(row.iter().all(|item| *item == 0 || *item == 1));
                }

                cumulative_item_index = cumulative_item_index_at_start + amount;
                cumulative_row_index = cumulative_row_index_at_start + (usize::div_ceil(amount, width));

                GridSectionData {
                    start_index: cumulative_item_index_at_start,
                    start_row_index: cumulative_row_index_at_start,
                    amount_in_section: amount,
                    width,
                }
            })
            .collect()
    }

    #[test]
    fn grid_down_last_row() {
        let sections_amount_width = prepare_sections(vec![vec![
            //fr     V
            vec![1, 1, 0],
        ]]);

        assert_eq!(grid_down_offset(1, sections_amount_width), None)
    }

    #[test]
    fn grid_down_inside_1() {
        let sections_amount_width = prepare_sections(vec![vec![
            //fr V
            vec![1, 1, 1],
            //to V
            vec![1, 1, 0],
        ]]);

        assert_eq!(
            grid_down_offset(0, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_down_inside_2() {
        let sections_amount_width = prepare_sections(vec![vec![
            //fr    V
            vec![1, 1, 1],
            //to    V
            vec![1, 1, 0],
        ]]);

        assert_eq!(
            grid_down_offset(1, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_down_inside_3() {
        let sections_amount_width = prepare_sections(vec![vec![
            //fr       V
            vec![1, 1, 1],
            //to    V
            vec![1, 1, 0],
        ]]);

        assert_eq!(
            grid_down_offset(2, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 2
            })
        )
    }

    #[test]
    fn grid_down_inside_4() {
        let sections_amount_width = prepare_sections(vec![vec![
            //fr       V
            vec![1, 1, 1],
            //to V
            vec![1, 0, 0],
        ]]);

        assert_eq!(
            grid_down_offset(2, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 1
            })
        )
    }

    #[test]
    fn grid_down_inside_5() {
        let sections_amount_width = prepare_sections(vec![vec![
            vec![1, 1, 1],
            //fr    V
            vec![1, 1, 1],
            //to    V
            vec![1, 1, 0],
        ]]);

        assert_eq!(
            grid_down_offset(4, sections_amount_width),
            Some(GridItemOffset {
                row_index: 2,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_down_outside_1() {
        let sections_amount_width = prepare_sections(vec![
            vec![
                //fr    V
                vec![1, 1, 1],
            ],
            vec![
                //to    V
                vec![1, 1, 1],
            ],
        ]);

        assert_eq!(
            grid_down_offset(1, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_down_outside_2() {
        let sections_amount_width = prepare_sections(vec![
            vec![
                //fr    V
                vec![1, 1, 0],
            ],
            vec![
                //to    V
                vec![1, 1, 1],
            ],
        ]);

        assert_eq!(
            grid_down_offset(1, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 2
            })
        )
    }

    #[test]
    fn grid_down_outside_3() {
        let sections_amount_width = prepare_sections(vec![
            vec![
                //fr       V
                vec![1, 1, 1],
            ],
            vec![
                //to    V
                vec![1, 1, 0],
            ],
            vec![vec![1, 1, 1]],
        ]);

        assert_eq!(
            grid_down_offset(2, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 2
            })
        )
    }

    #[test]
    fn grid_up_first_row() {
        let sections_amount_width = prepare_sections(vec![vec![
            //fr    V
            vec![1, 1, 0],
        ]]);

        assert_eq!(grid_up_offset(1, sections_amount_width), None)
    }

    #[test]
    fn grid_up_inside_1() {
        let sections_amount_width = prepare_sections(vec![vec![
            //to    V
            vec![1, 1, 1],
            //fr    V
            vec![1, 1, 0],
        ]]);

        assert_eq!(
            grid_up_offset(4, sections_amount_width),
            Some(GridItemOffset {
                row_index: 0,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_up_inside_2() {
        let sections_amount_width = prepare_sections(vec![vec![
            //to V
            vec![1, 1, 1],
            //fr V
            vec![1, 1, 0],
        ]]);

        assert_eq!(
            grid_up_offset(3, sections_amount_width),
            Some(GridItemOffset {
                row_index: 0,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_up_inside_3() {
        let sections_amount_width = prepare_sections(vec![vec![
            //to       V
            vec![1, 1, 1],
            //fr       V
            vec![1, 1, 1],
        ]]);

        assert_eq!(
            grid_up_offset(5, sections_amount_width),
            Some(GridItemOffset {
                row_index: 0,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_up_outside_1() {
        let sections_amount_width = prepare_sections(vec![
            vec![
                //to    V
                vec![1, 1, 1],
            ],
            vec![
                //fr    V
                vec![1, 1, 1],
            ],
        ]);

        assert_eq!(
            grid_up_offset(4, sections_amount_width),
            Some(GridItemOffset {
                row_index: 0,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_up_outside_2() {
        let sections_amount_width = prepare_sections(vec![
            vec![
                //to    V
                vec![1, 1, 0],
            ],
            vec![
                //fr       V
                vec![1, 1, 1],
            ],
        ]);

        assert_eq!(
            grid_up_offset(4, sections_amount_width),
            Some(GridItemOffset {
                row_index: 0,
                offset: 3
            })
        )
    }

    #[test]
    fn grid_up_outside_3() {
        let sections_amount_width = prepare_sections(vec![
            vec![
                //to V
                vec![1, 1, 0],
            ],
            vec![
                //fr V
                vec![1, 0, 0],
            ],
        ]);

        assert_eq!(
            grid_up_offset(2, sections_amount_width),
            Some(GridItemOffset {
                row_index: 0,
                offset: 2
            })
        )
    }

    #[test]
    fn grid_up_outside_4() {
        let sections_amount_width = prepare_sections(vec![
            vec![vec![1, 1, 1]],
            vec![
                //to    V
                vec![1, 1, 0],
            ],
            vec![
                //fr       V
                vec![1, 1, 1],
            ],
        ]);

        assert_eq!(
            grid_up_offset(7, sections_amount_width),
            Some(GridItemOffset {
                row_index: 1,
                offset: 3
            })
        )
    }
}
