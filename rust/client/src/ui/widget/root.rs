use std::collections::HashMap;

use gauntlet_common::model::ActionPanelWidget;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common::model::RootWidgetMembers;
use gauntlet_common::model::SearchBarWidget;
use gauntlet_common::model::UiWidgetId;
use iced::Alignment;
use iced::Length;
use iced::advanced::text::Shaping;
use iced::widget::Space;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::horizontal_rule;
use iced::widget::horizontal_space;
use iced::widget::mouse_area;
use iced::widget::row;
use iced::widget::stack;
use iced::widget::text;
use iced::widget::value;
use iced::widget::vertical_rule;
use iced_fonts::BOOTSTRAP_FONT;
use iced_fonts::Bootstrap;

use crate::ui::custom_widgets::loading_bar::LoadingBar;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::PluginViewState;
use crate::ui::theme::Element;
use crate::ui::theme::ThemableWidget;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::rule::RuleStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::widget::action_panel::ActionPanel;
use crate::ui::widget::action_panel::convert_action_panel;
use crate::ui::widget::action_panel::render_action_panel;
use crate::ui::widget::action_panel::render_shortcut;
use crate::ui::widget::data::ComponentWidgets;
use crate::ui::widget::events::ComponentWidgetEvent;
use crate::ui::widget::state::RootState;

impl<'b> ComponentWidgets<'b> {
    pub fn render_root_widget<'a>(
        &self,
        plugin_view_state: &PluginViewState,
        entrypoint_name: Option<&String>,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        match &self.root_widget {
            None => horizontal_space().into(),
            Some(root) => {
                match &root.content {
                    None => horizontal_space().into(),
                    Some(content) => {
                        let entrypoint_name =
                            entrypoint_name.expect("entrypoint name should always exist after render");

                        match content {
                            RootWidgetMembers::Detail(widget) => {
                                let RootState { show_action_panel, .. } = self.root_state(widget.__id__);

                                let content = self.render_detail_widget(widget, false);

                                self.render_plugin_root(
                                    *show_action_panel,
                                    widget.__id__,
                                    None,
                                    &None,
                                    &widget.content.actions,
                                    content,
                                    widget.is_loading.unwrap_or(false),
                                    plugin_view_state,
                                    entrypoint_name,
                                    action_shortcuts,
                                )
                            }
                            RootWidgetMembers::Form(widget) => {
                                self.render_form_widget(widget, plugin_view_state, entrypoint_name, action_shortcuts)
                            }
                            RootWidgetMembers::List(widget) => {
                                self.render_list_widget(widget, plugin_view_state, entrypoint_name, action_shortcuts)
                            }
                            RootWidgetMembers::Grid(widget) => {
                                self.render_grid_widget(widget, plugin_view_state, entrypoint_name, action_shortcuts)
                            }
                            _ => {
                                panic!("used inline widget in non-inline place")
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn render_root_inline_widget<'a>(
        &self,
        plugin_name: Option<&String>,
        entrypoint_name: Option<&String>,
    ) -> Element<'a, ComponentWidgetEvent> {
        match &self.root_widget {
            None => horizontal_space().into(),
            Some(root) => {
                match &root.content {
                    None => horizontal_space().into(),
                    Some(content) => {
                        match content {
                            RootWidgetMembers::Inline(widget) => {
                                let entrypoint_name =
                                    entrypoint_name.expect("entrypoint name should always exist after render");
                                let plugin_name =
                                    plugin_name.expect("entrypoint name should always exist after render");

                                self.render_inline_widget(widget, plugin_name, entrypoint_name)
                            }
                            _ => {
                                panic!("used non-inline widget in inline place")
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_top_panel<'a>(&self, search_bar: &Option<SearchBarWidget>) -> Element<'a, ComponentWidgetEvent> {
        let icon = value(Bootstrap::ArrowLeft).font(BOOTSTRAP_FONT);

        let back_button: Element<_> = button(icon)
            .on_press(ComponentWidgetEvent::PreviousView)
            .themed(ButtonStyle::RootTopPanelBackButton);

        let search_bar_element = search_bar
            .as_ref()
            .map(|widget| self.render_search_bar_widget(widget))
            .unwrap_or_else(|| Space::with_width(Length::FillPortion(3)).into());

        let top_panel: Element<_> = row(vec![back_button, search_bar_element])
            .align_y(Alignment::Center)
            .themed(RowStyle::RootTopPanel);

        let top_panel: Element<_> = container(top_panel)
            .width(Length::Fill)
            .themed(ContainerStyle::RootTopPanel);

        top_panel
    }

    pub fn render_plugin_root<'a>(
        &self,
        show_action_panel: bool,
        root_widget_id: UiWidgetId,
        focused_item_id: Option<String>,
        search_bar: &Option<SearchBarWidget>,
        action_panel: &Option<ActionPanelWidget>,
        content: Element<'a, ComponentWidgetEvent>,
        is_loading: bool,
        plugin_view_state: &PluginViewState,
        entrypoint_name: &str,
        action_shortcuts: &HashMap<String, PhysicalShortcut>,
    ) -> Element<'a, ComponentWidgetEvent> {
        let top_panel = self.render_top_panel(search_bar);

        let top_separator = if is_loading {
            LoadingBar::new().into()
        } else {
            horizontal_rule(1).into()
        };

        let mut action_panel = convert_action_panel(action_panel, &action_shortcuts);

        let primary_action =
            action_panel
                .as_mut()
                .map(|panel| panel.find_first())
                .flatten()
                .map(|(label, widget_id)| {
                    let shortcut = PhysicalShortcut {
                        physical_key: PhysicalKey::Enter,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: false,
                    };

                    (label.to_string(), widget_id, shortcut)
                });

        match plugin_view_state {
            PluginViewState::None => {
                render_root(
                    show_action_panel,
                    top_panel,
                    top_separator,
                    None,
                    content,
                    primary_action,
                    action_panel,
                    None::<&ScrollHandle>,
                    entrypoint_name,
                    || {
                        ComponentWidgetEvent::ToggleActionPanel {
                            widget_id: root_widget_id,
                        }
                    },
                    |widget_id| {
                        ComponentWidgetEvent::RunPrimaryAction {
                            widget_id,
                            id: focused_item_id.clone(),
                        }
                    },
                    |widget_id| {
                        ComponentWidgetEvent::ActionClick {
                            widget_id,
                            id: focused_item_id.clone(),
                        }
                    },
                    || ComponentWidgetEvent::Noop,
                )
            }
            PluginViewState::ActionPanel { focused_action_item } => {
                render_root(
                    show_action_panel,
                    top_panel,
                    top_separator,
                    None,
                    content,
                    primary_action,
                    action_panel,
                    Some(&focused_action_item),
                    entrypoint_name,
                    || {
                        ComponentWidgetEvent::ToggleActionPanel {
                            widget_id: root_widget_id,
                        }
                    },
                    |widget_id| {
                        ComponentWidgetEvent::RunPrimaryAction {
                            widget_id,
                            id: focused_item_id.clone(),
                        }
                    },
                    |widget_id| {
                        ComponentWidgetEvent::ActionClick {
                            widget_id,
                            id: focused_item_id.clone(),
                        }
                    },
                    || ComponentWidgetEvent::Noop,
                )
            }
        }
    }
}

pub fn render_root<'a, T: 'a + Clone>(
    show_action_panel: bool,
    top_panel: Element<'a, T>,
    top_separator: Element<'a, T>,
    toast_text: Option<&str>,
    content: Element<'a, T>,
    primary_action: Option<(String, UiWidgetId, PhysicalShortcut)>,
    action_panel: Option<ActionPanel>,
    action_panel_scroll_handle: Option<&ScrollHandle>,
    entrypoint_name: &str,
    on_panel_toggle_click: impl Fn() -> T,
    on_panel_primary_click: impl Fn(UiWidgetId) -> T,
    on_action_click: impl Fn(UiWidgetId) -> T,
    noop_msg: impl Fn() -> T,
) -> Element<'a, T> {
    let entrypoint_name: Element<_> = text(entrypoint_name.to_string()).shaping(Shaping::Advanced).into();

    let panel_height = 16 + 8 + 2; // TODO get value from theme

    let primary_action = match primary_action {
        Some((label, widget_id, shortcut)) => {
            let label: Element<_> = text(label)
                .shaping(Shaping::Advanced)
                .themed(TextStyle::RootBottomPanelPrimaryActionText);

            let label: Element<_> = container(label).themed(ContainerStyle::RootBottomPanelPrimaryActionText);

            let shortcut = render_shortcut(&shortcut);

            let content: Element<_> = row(vec![label, shortcut]).into();

            let content: Element<_> = button(content)
                .on_press(on_panel_primary_click(widget_id))
                .themed(ButtonStyle::RootBottomPanelPrimaryActionButton);

            let content: Element<_> = container(content).themed(ContainerStyle::RootBottomPanelPrimaryActionButton);

            Some(content)
        }
        None => None,
    };

    let (hide_action_panel, action_panel, bottom_panel) = match action_panel {
        Some(action_panel) => {
            let actions_text: Element<_> = text("Actions").themed(TextStyle::RootBottomPanelActionToggleText);

            let actions_text: Element<_> =
                container(actions_text).themed(ContainerStyle::RootBottomPanelActionToggleText);

            let shortcut = render_shortcut(&PhysicalShortcut {
                physical_key: PhysicalKey::KeyK,
                modifier_shift: false,
                modifier_control: false,
                modifier_alt: true,
                modifier_meta: false,
            });

            let mut bottom_panel_content = vec![entrypoint_name];

            if let Some(toast_text) = toast_text {
                let toast_text = text(toast_text.to_string()).into();

                bottom_panel_content.push(toast_text);
            }

            let space = horizontal_space().into();

            bottom_panel_content.push(space);

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);

                let rule: Element<_> = vertical_rule(1).class(RuleStyle::PrimaryActionSeparator).into();

                let rule: Element<_> = container(rule)
                    .width(Length::Shrink)
                    .height(panel_height)
                    .max_height(panel_height)
                    .into();

                bottom_panel_content.push(rule);
            }

            let action_panel_toggle_content: Element<_> = row(vec![actions_text, shortcut]).into();

            let action_panel_toggle: Element<_> = button(action_panel_toggle_content)
                .on_press(on_panel_toggle_click())
                .themed(ButtonStyle::RootBottomPanelActionToggleButton);

            bottom_panel_content.push(action_panel_toggle);

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_y(Alignment::Center)
                .themed(RowStyle::RootBottomPanel);

            (!show_action_panel, Some(action_panel), bottom_panel)
        }
        None => {
            let space: Element<_> = Space::new(Length::Fill, panel_height).into();

            let mut bottom_panel_content = vec![];

            if let Some(toast_text) = toast_text {
                let toast_text = text(toast_text.to_string()).into();

                bottom_panel_content.push(toast_text);
            } else {
                bottom_panel_content.push(entrypoint_name);
            }

            bottom_panel_content.push(space);

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);
            }

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_y(Alignment::Center)
                .themed(RowStyle::RootBottomPanel);

            (true, None, bottom_panel)
        }
    };

    let bottom_panel: Element<_> = container(bottom_panel)
        .width(Length::Fill)
        .themed(ContainerStyle::RootBottomPanel);

    let content: Element<_> = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .themed(ContainerStyle::RootInner);

    let content: Element<_> = column(vec![top_panel, top_separator, content, bottom_panel]).into();

    let content: Element<_> = mouse_area(content)
        .on_press(
            if hide_action_panel {
                noop_msg()
            } else {
                on_panel_toggle_click()
            },
        )
        .into();

    let mut content = vec![content];

    if let (Some(action_panel), Some(action_panel_scroll_handle)) = (action_panel, action_panel_scroll_handle) {
        if !hide_action_panel {
            let action_panel = render_action_panel(action_panel, on_action_click, action_panel_scroll_handle);

            let action_panel: Element<_> = container(action_panel)
                .padding(gauntlet_common_ui::padding(0.0, 8.0, 48.0, 0.0))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill)
                .into();

            content.push(action_panel);
        }
    };

    stack(content).into()
}
