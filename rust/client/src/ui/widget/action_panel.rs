use std::cell::Cell;
use std::collections::HashMap;

use gauntlet_common::model::ActionPanelSectionWidget;
use gauntlet_common::model::ActionPanelSectionWidgetOrderedMembers;
use gauntlet_common::model::ActionPanelWidget;
use gauntlet_common::model::ActionPanelWidgetOrderedMembers;
use gauntlet_common::model::ActionWidget;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::UiWidgetId;
use gauntlet_common_ui::shortcut_to_text;
use iced::Alignment;
use iced::Font;
use iced::Length;
use iced::advanced::text::Shaping;
use iced::font::Weight;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;

use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::rule::RuleStyle;
use crate::ui::theme::text::TextStyle;

#[derive(Debug)]
pub struct ActionPanel {
    pub title: Option<String>,
    pub items: Vec<ActionPanelItem>,
}

impl ActionPanel {
    pub fn action_count(&self) -> usize {
        self.items.iter().map(|item| item.action_count()).sum()
    }

    pub fn find_first(&self) -> Option<(String, UiWidgetId)> {
        ActionPanelItem::find_first(&self.items)
    }
}

#[derive(Debug)]
pub enum ActionPanelItem {
    Action {
        label: String,
        widget_id: UiWidgetId,
        physical_shortcut: Option<PhysicalShortcut>,
    },
    ActionSection {
        title: Option<String>,
        items: Vec<ActionPanelItem>,
    },
}

impl ActionPanelItem {
    fn action_count(&self) -> usize {
        match self {
            ActionPanelItem::Action { .. } => 1,
            ActionPanelItem::ActionSection { items, .. } => items.iter().map(|item| item.action_count()).sum(),
        }
    }

    fn find_first(items: &[ActionPanelItem]) -> Option<(String, UiWidgetId)> {
        for item in items {
            match item {
                ActionPanelItem::Action { label, widget_id, .. } => return Some((label.to_string(), *widget_id)),
                ActionPanelItem::ActionSection { items, .. } => {
                    if let Some(item) = Self::find_first(items) {
                        return Some(item);
                    }
                }
            }
        }

        None
    }
}

pub fn convert_action_panel(
    action_panel: &Option<ActionPanelWidget>,
    action_shortcuts: &HashMap<String, PhysicalShortcut>,
) -> Option<ActionPanel> {
    match action_panel {
        Some(ActionPanelWidget { content, title, .. }) => {
            fn action_widget_to_action(
                ActionWidget { __id__, id, label }: &ActionWidget,
                action_shortcuts: &HashMap<String, PhysicalShortcut>,
            ) -> ActionPanelItem {
                let physical_shortcut: Option<PhysicalShortcut> =
                    id.as_ref().map(|id| action_shortcuts.get(id)).flatten().cloned();

                ActionPanelItem::Action {
                    label: label.clone(),
                    widget_id: *__id__,
                    physical_shortcut,
                }
            }

            let items = content
                .ordered_members
                .iter()
                .map(|members| {
                    match members {
                        ActionPanelWidgetOrderedMembers::Action(widget) => {
                            action_widget_to_action(widget, action_shortcuts)
                        }
                        ActionPanelWidgetOrderedMembers::ActionPanelSection(ActionPanelSectionWidget {
                            content,
                            title,
                            ..
                        }) => {
                            let section_items = content
                                .ordered_members
                                .iter()
                                .map(|members| {
                                    match members {
                                        ActionPanelSectionWidgetOrderedMembers::Action(widget) => {
                                            action_widget_to_action(widget, action_shortcuts)
                                        }
                                    }
                                })
                                .collect();

                            ActionPanelItem::ActionSection {
                                title: title.clone(),
                                items: section_items,
                            }
                        }
                    }
                })
                .collect();

            Some(ActionPanel {
                title: title.clone(),
                items,
            })
        }
        _ => None,
    }
}

fn render_action_panel_items<'a, T: 'a + Clone>(
    root: bool,
    title: Option<String>,
    items: Vec<ActionPanelItem>,
    action_panel_focus_index: Option<usize>,
    on_action_click: &dyn Fn(UiWidgetId) -> T,
    index_counter: &Cell<usize>,
) -> Vec<Element<'a, T>> {
    let mut columns = vec![];

    if let Some(title) = title {
        let text: Element<_> = text(title)
            .shaping(Shaping::Advanced)
            .font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            })
            .themed(TextStyle::ActionSectionTitle);

        let text = container(text).themed(
            if root {
                ContainerStyle::ActionPanelTitle
            } else {
                ContainerStyle::ActionSectionTitle
            },
        );

        columns.push(text)
    } else {
        if !root {
            let separator: Element<_> = horizontal_rule(1).themed(RuleStyle::ActionPanel);

            columns.push(separator);
        }
    }

    for item in items {
        match item {
            ActionPanelItem::Action {
                label,
                widget_id,
                physical_shortcut,
            } => {
                let physical_shortcut = match index_counter.get() {
                    0 => {
                        Some(PhysicalShortcut {
                            // primary
                            physical_key: PhysicalKey::Enter,
                            modifier_shift: false,
                            modifier_control: false,
                            modifier_alt: false,
                            modifier_meta: false,
                        })
                    }
                    1 => {
                        Some(PhysicalShortcut {
                            // secondary
                            physical_key: PhysicalKey::Enter,
                            modifier_shift: true,
                            modifier_control: false,
                            modifier_alt: false,
                            modifier_meta: false,
                        })
                    }
                    _ => physical_shortcut,
                };

                let shortcut_element: Option<Element<_>> =
                    physical_shortcut.as_ref().map(|shortcut| render_shortcut(shortcut));

                let content: Element<_> = if let Some(shortcut_element) = shortcut_element {
                    let text: Element<_> = text(label).shaping(Shaping::Advanced).into();

                    let space: Element<_> = horizontal_space().into();

                    row([text, space, shortcut_element]).align_y(Alignment::Center).into()
                } else {
                    text(label).shaping(Shaping::Advanced).into()
                };

                let style = match action_panel_focus_index {
                    None => ButtonStyle::Action,
                    Some(focused_index) => {
                        if focused_index == index_counter.get() {
                            ButtonStyle::ActionFocused
                        } else {
                            ButtonStyle::Action
                        }
                    }
                };

                index_counter.set(index_counter.get() + 1);

                let content = button(content)
                    .on_press(on_action_click(widget_id))
                    .width(Length::Fill)
                    .themed(style);

                columns.push(content);
            }
            ActionPanelItem::ActionSection { title, items } => {
                let content = render_action_panel_items(
                    false,
                    title,
                    items,
                    action_panel_focus_index,
                    on_action_click,
                    index_counter,
                );

                for content in content {
                    columns.push(content);
                }
            }
        };
    }

    columns
}

pub fn render_action_panel<'a, T: 'a + Clone, F: Fn(UiWidgetId) -> T>(
    action_panel: ActionPanel,
    on_action_click: F,
    action_panel_scroll_handle: &ScrollHandle,
) -> Element<'a, T> {
    let columns = render_action_panel_items(
        true,
        action_panel.title,
        action_panel.items,
        action_panel_scroll_handle.index,
        &on_action_click,
        &Cell::new(0),
    );

    let actions: Element<_> = column(columns).into();

    let actions: Element<_> = scrollable(actions)
        .id(action_panel_scroll_handle.scrollable_id.clone())
        .width(Length::Fill)
        .into();

    container(actions).themed(ContainerStyle::ActionPanel)
}

pub fn render_shortcut<'a, T: 'a>(shortcut: &PhysicalShortcut) -> Element<'a, T> {
    let mut result = vec![];

    let (key_name, alt_modifier_text, meta_modifier_text, control_modifier_text, shift_modifier_text) =
        shortcut_to_text(shortcut);

    fn apply_modifier<'result, 'element, T: 'element>(
        result: &'result mut Vec<Element<'element, T>>,
        modifier: Option<Element<'element, T>>,
    ) {
        if let Some(modifier) = modifier {
            let modifier: Element<_> = container(modifier).themed(ContainerStyle::ActionShortcutModifier);

            let modifier: Element<_> = container(modifier).themed(ContainerStyle::ActionShortcutModifiersInit);

            result.push(modifier);
        }
    }

    apply_modifier(&mut result, meta_modifier_text);
    apply_modifier(&mut result, control_modifier_text);
    apply_modifier(&mut result, shift_modifier_text);
    apply_modifier(&mut result, alt_modifier_text);

    let key_name: Element<_> = container(key_name).themed(ContainerStyle::ActionShortcutModifier);

    result.push(key_name);

    row(result).themed(RowStyle::ActionShortcut)
}
