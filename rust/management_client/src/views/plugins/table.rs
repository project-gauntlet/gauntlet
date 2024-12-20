use std::cell::RefCell;
use std::rc::Rc;

use iced::{Alignment, Length, Renderer, Task};
use iced::advanced::text::Shaping;
use iced::widget::{button, checkbox, container, horizontal_space, row, scrollable, Space, text, value};
use iced::widget::scrollable::Id;
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};
use iced_table::table;

use gauntlet_common::model::{EntrypointId, PluginId, SettingsEntrypointType, SettingsPlugin};

use crate::theme::{Element, GauntletSettingsTheme};
use crate::theme::button::ButtonStyle;
use crate::views::plugins::{PluginDataContainer, SelectedItem, SettingsPluginData};

#[derive(Debug, Clone)]
pub enum PluginTableMsgIn {
    TableSyncHeader(scrollable::AbsoluteOffset),
    SelectItem(SelectedItem),
    EnabledToggleItem(EnabledItem),
    ToggleShowEntrypoints {
        plugin_id: PluginId,
    },
}

pub enum PluginTableMsgOut {
    SetPluginState {
        enabled: bool,
        plugin_id: PluginId,
    },
    SetEntrypointState {
        enabled: bool,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    SelectItem(SelectedItem),
    ToggleShowEntrypoints {
        plugin_id: PluginId,
    }
}

pub struct PluginTableState {
    columns: Vec<Column>,
    rows: Vec<Row>,
    header: Id,
    body: Id,
}

pub enum PluginTableUpdateResult {
    Command(Task<()>),
    Value(PluginTableMsgOut)
}

impl PluginTableState {
    pub fn new() -> Self {
        Self {
            columns: vec![
                Column::new(ColumnKind::ShowEntrypointsToggle),
                Column::new(ColumnKind::Name),
                Column::new(ColumnKind::Type),
                Column::new(ColumnKind::EnableToggle),
            ],
            rows: vec![],
            header: Id::unique(),
            body: Id::unique(),
        }
    }

    pub fn update(&mut self, message: PluginTableMsgIn) -> PluginTableUpdateResult {
        match message {
            PluginTableMsgIn::TableSyncHeader(offset) => {
                PluginTableUpdateResult::Command(
                    scrollable::scroll_to(self.header.clone(), offset)
                )
            }
            PluginTableMsgIn::EnabledToggleItem(item) => {
                match item {
                    EnabledItem::Plugin { enabled, plugin_id } => {
                        PluginTableUpdateResult::Value(
                            PluginTableMsgOut::SetPluginState { enabled, plugin_id }
                        )
                    }
                    EnabledItem::Entrypoint { enabled, plugin_id, entrypoint_id } => {
                        PluginTableUpdateResult::Value(
                            PluginTableMsgOut::SetEntrypointState { enabled, plugin_id, entrypoint_id }
                        )
                    }
                }
            }
            PluginTableMsgIn::SelectItem(item) => {
                PluginTableUpdateResult::Value(
                    PluginTableMsgOut::SelectItem(item)
                )
            },
            PluginTableMsgIn::ToggleShowEntrypoints { plugin_id } => {
                PluginTableUpdateResult::Value(
                    PluginTableMsgOut::ToggleShowEntrypoints { plugin_id }
                )
            }
        }
    }

    pub fn apply_plugin_reload(&mut self, plugin_data: Rc<RefCell<PluginDataContainer>>, plugin_refs: Vec<(&SettingsPlugin, &SettingsPluginData)>) {
        self.rows = plugin_refs
            .iter()
            .flat_map(|(plugin, plugin_state)| {
                let mut result = vec![];

                result.push(Row::Plugin {
                    plugin_data: plugin_data.clone(),
                    plugin_id: plugin.plugin_id.clone()
                });

                if plugin_state.show_entrypoints {
                    let mut entrypoints: Vec<_> = plugin.entrypoints
                        .iter()
                        .map(|(_, entrypoint)| entrypoint)
                        .collect();

                    entrypoints.sort_by_key(|entrypoint| &entrypoint.entrypoint_name);

                    let mut entrypoints: Vec<_> = entrypoints
                        .iter()
                        .map(|entrypoint| {
                            Row::Entrypoint {
                                plugin_data: plugin_data.clone(),
                                plugin_id: plugin.plugin_id.clone(),
                                entrypoint_id: entrypoint.entrypoint_id.clone(),
                            }
                        })
                        .collect();

                    result.append(&mut entrypoints);
                }

                result
            })
            .collect();
    }

    pub fn view(&self) -> Element<PluginTableMsgIn> {
        table(self.header.clone(), self.body.clone(), &self.columns, &self.rows, PluginTableMsgIn::TableSyncHeader)
            .cell_padding(0.0)
            .into()
    }
}


#[derive(Debug, Clone)]
enum EnabledItem {
    Plugin {
        enabled: bool,
        plugin_id: PluginId,
    },
    Entrypoint {
        enabled: bool,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}

enum Row {
    Plugin {
        plugin_data: Rc<RefCell<PluginDataContainer>>,
        plugin_id: PluginId
    },
    Entrypoint {
        plugin_data: Rc<RefCell<PluginDataContainer>>,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
}

enum ColumnKind {
    ShowEntrypointsToggle,
    Name,
    Type,
    EnableToggle,
}

struct Column {
    kind: ColumnKind,
}

impl Column {
    fn new(kind: ColumnKind) -> Self {
        Self {
            kind
        }
    }
}

impl<'a> table::Column<'a, PluginTableMsgIn, GauntletSettingsTheme, Renderer> for Column {
    type Row = Row;

    fn header(&'a self, _col_index: usize) -> Element<'a, PluginTableMsgIn> {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => {
                horizontal_space()
                    .into()
            }
            ColumnKind::Name => {
                container(text("Name"))
                    .height(Length::Fixed(30.0))
                    .align_y(Alignment::Center)
                    .into()
            }
            ColumnKind::Type => {
                container(text("Type"))
                    .height(Length::Fixed(30.0))
                    .align_y(Alignment::Center)
                    .into()
            }
            ColumnKind::EnableToggle => {
                container(text("Enabled"))
                    .height(Length::Fixed(30.0))
                    .align_y(Alignment::Center)
                    .into()
            }
        }
    }

    fn cell(
        &'a self,
        _col_index: usize,
        _row_index: usize,
        row_entry: &'a Self::Row,
    ) -> Element<'a, PluginTableMsgIn> {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => {
                match row_entry {
                    Row::Plugin { plugin_data, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin_data = plugin_data.plugins_state.get(&plugin_id).unwrap();

                        let icon = if plugin_data.show_entrypoints { Bootstrap::CaretDown } else { Bootstrap::CaretRight };

                        let icon: Element<_> = value(icon)
                            .font(BOOTSTRAP_FONT)
                            .into();

                        button(icon)
                            .on_press(PluginTableMsgIn::ToggleShowEntrypoints { plugin_id: plugin_id.clone() })
                            .width(Length::Fill)
                            .height(Length::Fixed(40.0))
                            .padding(8.0)
                            .class(ButtonStyle::TableRow)
                            .into()
                    }
                    Row::Entrypoint { .. } => {
                        horizontal_space()
                            .into()
                    }
                }
            }
            ColumnKind::Name => {
                let content: Element<_> = match row_entry {
                    Row::Plugin { plugin_data, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

                        let plugin_name = text(plugin.plugin_name.to_string())
                            .shaping(Shaping::Advanced);

                        container(plugin_name)
                            .align_y(Alignment::Center)
                            .into()
                    }
                    Row::Entrypoint { plugin_data, plugin_id, entrypoint_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        let text: Element<_> = text(entrypoint.entrypoint_name.to_string())
                            .shaping(Shaping::Advanced)
                            .into();

                        let text: Element<_> = row(vec![
                            Space::with_width(Length::Fixed(30.0)).into(),
                            text,
                        ]).into();

                        container(text)
                            .align_y(Alignment::Center)
                            .into()
                    }
                };

                let msg = match &row_entry {
                    Row::Plugin { plugin_data, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

                        SelectedItem::Plugin {
                            plugin_id: plugin.plugin_id.clone()
                        }
                    },
                    Row::Entrypoint { plugin_data, entrypoint_id, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        SelectedItem::Entrypoint {
                            plugin_id: plugin.plugin_id.clone(),
                            entrypoint_id: entrypoint.entrypoint_id.clone(),
                        }
                    }
                };

                button(content)
                    .class(ButtonStyle::TableRow)
                    .on_press(PluginTableMsgIn::SelectItem(msg))
                    .width(Length::Fill)
                    .height(Length::Fixed(40.0))
                    .padding(8.0)
                    .into()
            }
            ColumnKind::Type => {
                let content: Element<_> = match row_entry {
                    Row::Plugin { .. } => {
                        horizontal_space()
                            .into()
                    }
                    Row::Entrypoint { plugin_data, plugin_id, entrypoint_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        let entrypoint_type = match entrypoint.entrypoint_type {
                            SettingsEntrypointType::Command => "Command",
                            SettingsEntrypointType::View => "View",
                            SettingsEntrypointType::InlineView => "Inline View",
                            SettingsEntrypointType::CommandGenerator => "Command Generator"
                        };

                        container(text(entrypoint_type.to_string()))
                            .align_y(Alignment::Center)
                            .into()
                    }
                };

                let msg = match &row_entry {
                    Row::Plugin { plugin_data, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

                        SelectedItem::Plugin {
                            plugin_id: plugin.plugin_id.clone()
                        }
                    },
                    Row::Entrypoint { plugin_data, entrypoint_id, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        SelectedItem::Entrypoint {
                            plugin_id: plugin.plugin_id.clone(),
                            entrypoint_id: entrypoint.entrypoint_id.clone(),
                        }
                    }
                };

                button(content)
                    .class(ButtonStyle::TableRow)
                    .on_press(PluginTableMsgIn::SelectItem(msg))
                    .width(Length::Fill)
                    .height(Length::Fixed(40.0))
                    .padding(8.0)
                    .into()
            }
            ColumnKind::EnableToggle => {
                let (enabled, show_checkbox, plugin_id, entrypoint_id) = match &row_entry {
                    Row::Plugin { plugin_data, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

                        (
                            plugin.enabled,
                            true,
                            plugin.plugin_id.clone(),
                            None
                        )
                    }
                    Row::Entrypoint { plugin_data, entrypoint_id, plugin_id } => {
                        let plugin_data = plugin_data.borrow();
                        let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
                        let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

                        (
                            entrypoint.enabled,
                            plugin.enabled,
                            plugin.plugin_id.clone(),
                            Some(entrypoint.entrypoint_id.clone())
                        )
                    }
                };

                let on_toggle = if show_checkbox {
                    Some(move |enabled| {
                        let enabled_item = match &entrypoint_id {
                            None => EnabledItem::Plugin {
                                enabled,
                                plugin_id: plugin_id.clone(),
                            },
                            Some(entrypoint_id) => EnabledItem::Entrypoint {
                                enabled,
                                plugin_id: plugin_id.clone(),
                                entrypoint_id: entrypoint_id.clone(),
                            }
                        };
                        PluginTableMsgIn::EnabledToggleItem(enabled_item)
                    })
                } else {
                    None
                };

                let checkbox: Element<_> = checkbox("", enabled)
                    .on_toggle_maybe(on_toggle)
                    .into();

                container(checkbox)
                    .width(Length::Fill)
                    .height(Length::Fixed(40.0))
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center)
                    .into()
            }
        }
    }

    fn width(&self) -> f32 {
        match self.kind {
            ColumnKind::ShowEntrypointsToggle => 35.0,
            ColumnKind::Name => 350.0,
            ColumnKind::Type => 200.0,
            ColumnKind::EnableToggle => 75.0
        }
    }

    fn resize_offset(&self) -> Option<f32> {
        None
    }
}
