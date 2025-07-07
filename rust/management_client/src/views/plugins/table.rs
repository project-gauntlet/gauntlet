use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::PluginId;
use gauntlet_common::model::SettingsEntrypointType;
use gauntlet_common::model::SettingsPlugin;
use iced::Alignment;
use iced::Length;
use iced::Task;
use iced::advanced::text::Shaping;
use iced::padding;
use iced::widget::Space;
use iced::widget::button;
use iced::widget::checkbox;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::text_input;
use iced_fonts::bootstrap::caret_down;
use iced_fonts::bootstrap::caret_right;

use crate::components::shortcut_selector::ShortcutData;
use crate::components::shortcut_selector::shortcut_selector;
use crate::theme::Element;
use crate::theme::button::ButtonStyle;
use crate::theme::container::ContainerStyle;
use crate::theme::text_input::TextInputStyle;
use crate::views::plugins::PluginDataContainer;
use crate::views::plugins::SelectedItem;
use crate::views::plugins::SettingsPluginData;

#[derive(Debug, Clone)]
pub enum PluginTableMsgIn {
    SelectItem(SelectedItem),
    EnabledToggleItem(EnabledItem),
    ToggleShowEntrypoints {
        plugin_id: PluginId,
    },
    ToggleShowGeneratedEntrypoints {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    ShortcutCaptured(PluginId, EntrypointId, Option<PhysicalShortcut>),
    AliasChanged(PluginId, EntrypointId, String),
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
    },
    ToggleShowGeneratedEntrypoints {
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
    },
    ShortcutCaptured(PluginId, EntrypointId, Option<PhysicalShortcut>),
    AliasChanged(PluginId, EntrypointId, Option<String>),
}

pub struct PluginTableState {
    rows: Vec<Row>,
    pub show_global_shortcuts: bool,
}

impl PluginTableState {
    pub fn new() -> Self {
        Self {
            rows: vec![],
            show_global_shortcuts: false,
        }
    }

    pub fn update(&mut self, message: PluginTableMsgIn) -> Task<PluginTableMsgOut> {
        match message {
            PluginTableMsgIn::EnabledToggleItem(item) => {
                match item {
                    EnabledItem::Plugin { enabled, plugin_id } => {
                        Task::done(PluginTableMsgOut::SetPluginState { enabled, plugin_id })
                    }
                    EnabledItem::Entrypoint {
                        enabled,
                        plugin_id,
                        entrypoint_id,
                    } => {
                        Task::done(PluginTableMsgOut::SetEntrypointState {
                            enabled,
                            plugin_id,
                            entrypoint_id,
                        })
                    }
                }
            }
            PluginTableMsgIn::SelectItem(item) => Task::done(PluginTableMsgOut::SelectItem(item)),
            PluginTableMsgIn::ToggleShowEntrypoints { plugin_id } => {
                Task::done(PluginTableMsgOut::ToggleShowEntrypoints { plugin_id })
            }
            PluginTableMsgIn::ToggleShowGeneratedEntrypoints {
                plugin_id,
                entrypoint_id,
            } => {
                Task::done(PluginTableMsgOut::ToggleShowGeneratedEntrypoints {
                    plugin_id,
                    entrypoint_id,
                })
            }
            PluginTableMsgIn::ShortcutCaptured(plugin_id, entrypoint_id, shortcut) => {
                Task::done(PluginTableMsgOut::ShortcutCaptured(plugin_id, entrypoint_id, shortcut))
            }
            PluginTableMsgIn::AliasChanged(plugin_id, entrypoint_id, alias) => {
                let alias = alias.trim().to_owned();
                let alias = if alias.is_empty() { None } else { Some(alias) };

                Task::done(PluginTableMsgOut::AliasChanged(plugin_id, entrypoint_id, alias))
            }
        }
    }

    pub fn apply_plugin_reload(
        &mut self,
        plugin_data: Rc<RefCell<PluginDataContainer>>,
        plugin_refs: Vec<(&SettingsPlugin, &SettingsPluginData)>,
        global_entrypoint_shortcuts: HashMap<(PluginId, EntrypointId), (PhysicalShortcut, Option<String>)>,
        entrypoint_search_aliases: HashMap<(PluginId, EntrypointId), String>,
    ) {
        self.rows = plugin_refs
            .iter()
            .flat_map(|(plugin, plugin_state)| {
                let mut result = vec![];

                result.push(Row::Plugin {
                    plugin_data: plugin_data.clone(),
                    plugin_id: plugin.plugin_id.clone(),
                });

                if plugin_state.show_entrypoints {
                    let mut entrypoints: Vec<_> = plugin.entrypoints.iter().map(|(_, entrypoint)| entrypoint).collect();

                    entrypoints.sort_by_key(|entrypoint| &entrypoint.entrypoint_name);

                    for entrypoint in entrypoints {
                        let global_entrypoint_shortcut = global_entrypoint_shortcuts
                            .get(&(plugin.plugin_id.clone(), entrypoint.entrypoint_id.clone()));
                        let shortcut = global_entrypoint_shortcut.map(|(shortcut, _)| shortcut).cloned();
                        let error = global_entrypoint_shortcut.map(|(_, error)| error).cloned().flatten();

                        let search_alias = entrypoint_search_aliases
                            .get(&(plugin.plugin_id.clone(), entrypoint.entrypoint_id.clone()))
                            .cloned();

                        let entrypoint_row = Row::Entrypoint {
                            plugin_data: plugin_data.clone(),
                            plugin_id: plugin.plugin_id.clone(),
                            entrypoint_id: entrypoint.entrypoint_id.clone(),
                            shortcut_data: ShortcutData { shortcut, error },
                            search_alias,
                        };

                        result.push(entrypoint_row);

                        let show_generated_entrypoints = plugin_state
                            .generator_entrypoint_state
                            .get(&entrypoint.entrypoint_id)
                            .map(|data| data.show_entrypoints)
                            .unwrap_or(true);

                        if show_generated_entrypoints {
                            let mut generated_entrypoints: Vec<_> = entrypoint
                                .generated_entrypoints
                                .iter()
                                .map(|(_, entrypoint)| entrypoint)
                                .collect();

                            generated_entrypoints.sort_by_key(|entrypoint| &entrypoint.entrypoint_name);

                            for data in generated_entrypoints {
                                let global_entrypoint_shortcut = global_entrypoint_shortcuts
                                    .get(&(plugin.plugin_id.clone(), data.entrypoint_id.clone()));
                                let shortcut = global_entrypoint_shortcut.map(|(shortcut, _)| shortcut).cloned();
                                let error = global_entrypoint_shortcut.map(|(_, error)| error).cloned().flatten();

                                let search_alias = entrypoint_search_aliases
                                    .get(&(plugin.plugin_id.clone(), data.entrypoint_id.clone()))
                                    .cloned();

                                let generated_entrypoint_row = Row::GeneratedEntrypoint {
                                    plugin_data: plugin_data.clone(),
                                    plugin_id: plugin.plugin_id.clone(),
                                    generator_entrypoint_id: entrypoint.entrypoint_id.clone(),
                                    generated_entrypoint_id: data.entrypoint_id.clone(),
                                    shortcut_data: ShortcutData { shortcut, error },
                                    search_alias,
                                };

                                result.push(generated_entrypoint_row);
                            }
                        }
                    }
                }

                result
            })
            .collect();
    }

    pub fn view(&self) -> Element<PluginTableMsgIn> {
        let mut rows: Vec<Element<_>> = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row_item)| {
                let style = if index % 2 != 0 {
                    ContainerStyle::TableEvenRow
                } else {
                    ContainerStyle::Transparent
                };

                table_row(
                    style,
                    name_cell(row_item),
                    type_cell(row_item),
                    if self.show_global_shortcuts {
                        Some(shortcut_cell(row_item))
                    } else {
                        None
                    },
                    alias_cell(row_item),
                    enable_cell(row_item),
                )
            })
            .collect();

        let header = table_row(
            ContainerStyle::TableEvenRow,
            header_cell("Name", true),
            header_cell("Type", false),
            if self.show_global_shortcuts {
                Some(header_cell("Shortcut", false))
            } else {
                None
            },
            header_cell("Alias", false),
            header_cell("Enabled", false),
        );

        rows.insert(0, header);

        let table: Element<_> = column(rows).into();

        scrollable(table).into()
    }
}

#[derive(Debug, Clone)]
pub enum EnabledItem {
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
        plugin_id: PluginId,
    },
    Entrypoint {
        plugin_data: Rc<RefCell<PluginDataContainer>>,
        plugin_id: PluginId,
        entrypoint_id: EntrypointId,
        shortcut_data: ShortcutData,
        search_alias: Option<String>,
    },
    GeneratedEntrypoint {
        plugin_data: Rc<RefCell<PluginDataContainer>>,
        plugin_id: PluginId,
        generator_entrypoint_id: EntrypointId,
        generated_entrypoint_id: EntrypointId,
        shortcut_data: ShortcutData,
        search_alias: Option<String>,
    },
}

fn table_row<'a>(
    style: ContainerStyle,
    name_cell: Element<'a, PluginTableMsgIn>,
    type_cell: Element<'a, PluginTableMsgIn>,
    shortcut_cell: Option<Element<'a, PluginTableMsgIn>>,
    alias_cell: Element<'a, PluginTableMsgIn>,
    enable_cell: Element<'a, PluginTableMsgIn>,
) -> Element<'a, PluginTableMsgIn> {
    let content = row([])
        .push(container(name_cell).width(Length::FillPortion(38)))
        .push(container(type_cell).width(Length::FillPortion(12)))
        .push_maybe(shortcut_cell.map(|item| container(item).width(Length::FillPortion(24))))
        .push(container(alias_cell).width(Length::FillPortion(15)))
        .push(container(enable_cell).width(Length::FillPortion(10)));

    container(content).class(style).into()
}

fn header_cell<'a>(value: &str, first: bool) -> Element<'a, PluginTableMsgIn> {
    let mut element = container(text(value.to_string()))
        .height(Length::Fixed(30.0))
        .align_y(Alignment::Center);

    if first {
        element = element.padding(padding::left(8.0))
    }

    element.into()
}

fn name_cell<'a>(row_entry: &Row) -> Element<'a, PluginTableMsgIn> {
    let toggle = match row_entry {
        Row::Plugin { plugin_data, plugin_id } => {
            let plugin_data = plugin_data.borrow();
            let plugin_data = plugin_data.plugins_state.get(&plugin_id).unwrap();

            let icon: Element<_> = if plugin_data.show_entrypoints {
                caret_down().into()
            } else {
                caret_right().into()
            };

            button(icon)
                .on_press(PluginTableMsgIn::ToggleShowEntrypoints {
                    plugin_id: plugin_id.clone(),
                })
                .width(Length::Shrink)
                .height(Length::Fixed(40.0))
                .padding(8.0)
                .class(ButtonStyle::TableRow)
                .into()
        }
        Row::GeneratedEntrypoint { .. } => horizontal_space().width(Length::Shrink).into(),
        Row::Entrypoint {
            plugin_data,
            plugin_id,
            entrypoint_id,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let plugin_data = plugin_data.plugins_state.get(&plugin_id).unwrap();

            let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

            if matches!(entrypoint.entrypoint_type, SettingsEntrypointType::EntrypointGenerator) {
                let icon: Element<_> = if plugin_data
                    .generator_entrypoint_state
                    .get(&entrypoint_id)
                    .unwrap()
                    .show_entrypoints
                {
                    caret_down().into()
                } else {
                    caret_right().into()
                };

                let content = button(icon)
                    .on_press(PluginTableMsgIn::ToggleShowGeneratedEntrypoints {
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                    })
                    .width(Length::Shrink)
                    .height(Length::Fixed(40.0))
                    .padding(8.0)
                    .class(ButtonStyle::TableRow)
                    .into();

                let space: Element<_> = Space::with_width(Length::Fixed(10.0)).into();

                row(vec![space, content]).into()
            } else {
                horizontal_space().width(Length::Shrink).into()
            }
        }
    };

    let content: Element<_> = match row_entry {
        Row::Plugin { plugin_data, plugin_id } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

            let plugin_name = text(plugin.plugin_name.to_string()).shaping(Shaping::Advanced).size(14);

            container(plugin_name).align_y(Alignment::Center).into()
        }
        Row::Entrypoint {
            plugin_data,
            plugin_id,
            entrypoint_id,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

            let text: Element<_> = text(entrypoint.entrypoint_name.to_string())
                .shaping(Shaping::Advanced)
                .size(14)
                .into();

            let space: Element<_> = if let SettingsEntrypointType::EntrypointGenerator = entrypoint.entrypoint_type {
                Space::with_width(Length::Fixed(4.0)).into()
            } else {
                Space::with_width(Length::Fixed(45.0)).into()
            };

            let text: Element<_> = row(vec![space, text]).into();

            container(text).align_y(Alignment::Center).into()
        }
        Row::GeneratedEntrypoint {
            plugin_data,
            plugin_id,
            generator_entrypoint_id,
            generated_entrypoint_id,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let entrypoint = plugin.entrypoints.get(&generator_entrypoint_id).unwrap();
            let generated_entrypoint = entrypoint.generated_entrypoints.get(&generated_entrypoint_id).unwrap();

            let text: Element<_> = text(generated_entrypoint.entrypoint_name.to_string())
                .shaping(Shaping::Advanced)
                .size(14)
                .into();

            let space: Element<_> = Space::with_width(Length::Fixed(65.0)).into();

            let text: Element<_> = row(vec![space, text]).into();

            container(text).align_y(Alignment::Center).into()
        }
    };

    let msg = match row_entry {
        Row::Plugin { plugin_id, .. } => {
            SelectedItem::Plugin {
                plugin_id: plugin_id.clone(),
            }
        }
        Row::Entrypoint {
            entrypoint_id,
            plugin_id,
            ..
        } => {
            SelectedItem::Entrypoint {
                plugin_id: plugin_id.clone(),
                entrypoint_id: entrypoint_id.clone(),
            }
        }
        Row::GeneratedEntrypoint {
            plugin_id,
            generator_entrypoint_id,
            generated_entrypoint_id,
            ..
        } => {
            SelectedItem::GeneratedEntrypoint {
                plugin_id: plugin_id.clone(),
                generator_entrypoint_id: generator_entrypoint_id.clone(),
                generated_entrypoint_id: generated_entrypoint_id.clone(),
            }
        }
    };

    let content = button(content)
        .class(ButtonStyle::TableRow)
        .on_press(PluginTableMsgIn::SelectItem(msg))
        .width(Length::Fill)
        .height(Length::Fixed(40.0))
        .padding(padding::all(8).left(0))
        .into();

    row(vec![toggle, content]).into()
}

fn type_cell<'a>(row_entry: &Row) -> Element<'a, PluginTableMsgIn> {
    let content: Element<_> = match row_entry {
        Row::Plugin { .. } => container(text("Plugin").size(14)).align_y(Alignment::Center).into(),
        Row::Entrypoint {
            plugin_data,
            plugin_id,
            entrypoint_id,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

            let entrypoint_type = match entrypoint.entrypoint_type {
                SettingsEntrypointType::Command => "Command",
                SettingsEntrypointType::View => "View",
                SettingsEntrypointType::InlineView => "Inline",
                SettingsEntrypointType::EntrypointGenerator => "Generator",
            };

            container(text(entrypoint_type.to_string()).size(14))
                .align_y(Alignment::Center)
                .into()
        }
        Row::GeneratedEntrypoint { .. } => container(text("Generated").size(14)).align_y(Alignment::Center).into(),
    };

    let msg = match row_entry {
        Row::Plugin { plugin_id, .. } => {
            SelectedItem::Plugin {
                plugin_id: plugin_id.clone(),
            }
        }
        Row::Entrypoint {
            entrypoint_id,
            plugin_id,
            ..
        } => {
            SelectedItem::Entrypoint {
                plugin_id: plugin_id.clone(),
                entrypoint_id: entrypoint_id.clone(),
            }
        }
        Row::GeneratedEntrypoint {
            plugin_id,
            generated_entrypoint_id,
            generator_entrypoint_id,
            ..
        } => {
            SelectedItem::GeneratedEntrypoint {
                plugin_id: plugin_id.clone(),
                generator_entrypoint_id: generator_entrypoint_id.clone(),
                generated_entrypoint_id: generated_entrypoint_id.clone(),
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

fn shortcut_cell<'a>(row_entry: &Row) -> Element<'a, PluginTableMsgIn> {
    match row_entry {
        Row::Plugin { .. } => horizontal_space().into(),
        Row::Entrypoint {
            plugin_data,
            plugin_id,
            entrypoint_id,
            shortcut_data,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

            if let SettingsEntrypointType::View | SettingsEntrypointType::Command = entrypoint.entrypoint_type {
                let shortcut_selector = shortcut_selector(
                    shortcut_data,
                    {
                        let plugin_id = plugin_id.clone();
                        let entrypoint_id = entrypoint_id.clone();

                        move |shortcut| {
                            PluginTableMsgIn::ShortcutCaptured(plugin_id.clone(), entrypoint_id.clone(), shortcut)
                        }
                    },
                    ContainerStyle::Box,
                    true,
                );

                container(shortcut_selector)
                    .height(Length::Fixed(40.0))
                    .width(Length::Fill)
                    .into()
            } else {
                horizontal_space().into()
            }
        }
        Row::GeneratedEntrypoint {
            plugin_id,
            generated_entrypoint_id,
            shortcut_data,
            ..
        } => {
            let shortcut_selector = shortcut_selector(
                shortcut_data,
                {
                    let plugin_id = plugin_id.clone();
                    let generated_entrypoint_id = generated_entrypoint_id.clone();

                    move |shortcut| {
                        PluginTableMsgIn::ShortcutCaptured(plugin_id.clone(), generated_entrypoint_id.clone(), shortcut)
                    }
                },
                ContainerStyle::Box,
                true,
            );

            container(shortcut_selector)
                .height(Length::Fixed(40.0))
                .width(Length::Fill)
                .into()
        }
    }
}

fn alias_cell<'a>(row_entry: &Row) -> Element<'a, PluginTableMsgIn> {
    match &row_entry {
        Row::Plugin { .. } => horizontal_space().into(),
        Row::Entrypoint {
            plugin_data,
            plugin_id,
            entrypoint_id,
            search_alias,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

            if let SettingsEntrypointType::View | SettingsEntrypointType::Command = entrypoint.entrypoint_type {
                let input = text_input("Add Alias", search_alias.as_deref().unwrap_or(""))
                    .class(TextInputStyle::EntrypointAlias)
                    .padding(padding::all(12.0).left(7.0))
                    .size(14)
                    .on_input({
                        let plugin_id = plugin_id.clone();
                        let entrypoint_id = entrypoint_id.clone();

                        move |alias| PluginTableMsgIn::AliasChanged(plugin_id.clone(), entrypoint_id.clone(), alias)
                    });

                container(input).height(Length::Fixed(40.0)).width(Length::Fill).into()
            } else {
                horizontal_space().into()
            }
        }
        Row::GeneratedEntrypoint {
            plugin_id,
            generated_entrypoint_id,
            search_alias,
            ..
        } => {
            let input = text_input("Add Alias", search_alias.as_deref().unwrap_or(""))
                .class(TextInputStyle::EntrypointAlias)
                .padding(padding::all(12.0).left(7.0))
                .size(14)
                .on_input({
                    let plugin_id = plugin_id.clone();
                    let generated_entrypoint_id = generated_entrypoint_id.clone();

                    move |alias| {
                        PluginTableMsgIn::AliasChanged(plugin_id.clone(), generated_entrypoint_id.clone(), alias)
                    }
                });

            container(input).height(Length::Fixed(40.0)).width(Length::Fill).into()
        }
    }
}

fn enable_cell<'a>(row: &Row) -> Element<'a, PluginTableMsgIn> {
    let (enabled, show_checkbox, plugin_id, entrypoint_id) = match row {
        Row::Plugin { plugin_data, plugin_id } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();

            (plugin.enabled, true, plugin.plugin_id.clone(), None)
        }
        Row::Entrypoint {
            plugin_data,
            entrypoint_id,
            plugin_id,
            ..
        } => {
            let plugin_data = plugin_data.borrow();
            let plugin = plugin_data.plugins.get(&plugin_id).unwrap();
            let entrypoint = plugin.entrypoints.get(&entrypoint_id).unwrap();

            (
                entrypoint.enabled,
                plugin.enabled,
                plugin.plugin_id.clone(),
                Some(entrypoint.entrypoint_id.clone()),
            )
        }
        Row::GeneratedEntrypoint { .. } => return horizontal_space().into(),
    };

    let on_toggle = if show_checkbox {
        Some(move |enabled| {
            let enabled_item = match &entrypoint_id {
                None => {
                    EnabledItem::Plugin {
                        enabled,
                        plugin_id: plugin_id.clone(),
                    }
                }
                Some(entrypoint_id) => {
                    EnabledItem::Entrypoint {
                        enabled,
                        plugin_id: plugin_id.clone(),
                        entrypoint_id: entrypoint_id.clone(),
                    }
                }
            };
            PluginTableMsgIn::EnabledToggleItem(enabled_item)
        })
    } else {
        None
    };

    let checkbox: Element<_> = checkbox("", enabled).on_toggle_maybe(on_toggle).into();

    container(checkbox)
        .width(Length::Fill)
        .height(Length::Fixed(40.0))
        .align_y(Alignment::Center)
        .align_x(Alignment::Center)
        .into()
}
