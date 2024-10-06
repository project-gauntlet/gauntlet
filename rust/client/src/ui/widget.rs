use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use anyhow::anyhow;
use common::model::{PhysicalKey, PhysicalShortcut, PluginId, UiPropertyValue, UiPropertyValueToEnum, UiPropertyValueToStruct, UiWidgetId};
use common_ui::shortcut_to_text;
use iced::alignment::{Horizontal, Vertical};
use iced::font::Weight;
use iced::widget::image::Handle;
use iced::widget::tooltip::Position;
use iced::widget::{button, checkbox, column, container, horizontal_rule, horizontal_space, image, mouse_area, pick_list, row, scrollable, text, text_input, tooltip, vertical_rule, Space};
use iced::{Alignment, Font, Length};
use iced_aw::core::icons;
use iced_aw::date_picker::Date;
use iced_aw::floating_element::Offset;
use iced_aw::helpers::{date_picker, grid, grid_row, wrap_horizontal};
use iced_aw::{floating_element, GridRow};
use itertools::Itertools;

use crate::model::UiViewEvent;
use crate::ui::AppMsg;
use crate::ui::custom_widgets::loading_bar::LoadingBar;
use crate::ui::scroll_handle::ScrollHandle;
use crate::ui::state::PluginViewState;
use crate::ui::theme::button::ButtonStyle;
use crate::ui::theme::container::ContainerStyle;
use crate::ui::theme::date_picker::DatePickerStyle;
use crate::ui::theme::grid::GridStyle;
use crate::ui::theme::image::ImageStyle;
use crate::ui::theme::pick_list::PickListStyle;
use crate::ui::theme::row::RowStyle;
use crate::ui::theme::rule::RuleStyle;
use crate::ui::theme::text::TextStyle;
use crate::ui::theme::text_input::TextInputStyle;
use crate::ui::theme::tooltip::TooltipStyle;
use crate::ui::theme::{Element, ThemableWidget};

#[derive(Clone, Debug)]
pub struct ComponentWidgetWrapper {
    id: UiWidgetId,
    inner: Arc<RwLock<(ComponentWidget, ComponentWidgetState)>>,
}

include!(concat!(env!("OUT_DIR"), "/components.rs"));

#[derive(Clone, Debug)]
pub enum ComponentWidgetState {
    TextField {
        state_value: String
    },
    PasswordField {
        state_value: String
    },
    Checkbox {
        state_value: bool
    },
    DatePicker {
        show_picker: bool,
        state_value: Date,
    },
    Select {
        state_value: Option<String>
    },
    Detail {
        show_action_panel: bool,
    },
    Form {
        show_action_panel: bool,
    },
    List {
        show_action_panel: bool,
    },
    Grid {
        show_action_panel: bool,
    },
    None
}

impl ComponentWidgetState {
    fn create(component_widget: &ComponentWidget) -> Self {
        match component_widget {
            ComponentWidget::TextField { value, .. } => ComponentWidgetState::TextField {
                state_value: value.to_owned().unwrap_or("".to_owned())
            },
            ComponentWidget::PasswordField { value, .. } => ComponentWidgetState::PasswordField {
                state_value: value.to_owned().unwrap_or("".to_owned())
            },
            ComponentWidget::Checkbox { value, .. } => ComponentWidgetState::Checkbox {
                state_value: value.to_owned().unwrap_or(false)
            },
            ComponentWidget::DatePicker { value, .. } => {
                let value = value
                    .to_owned()
                    .map(|value| parse_date(&value))
                    .flatten()
                    .map(|(year, month, day)| Date::from_ymd(year, month, day))
                    .unwrap_or(Date::today());

                ComponentWidgetState::DatePicker {
                    state_value: value,
                    show_picker: false,
                }
            },
            ComponentWidget::Select { value, .. } => ComponentWidgetState::Select {
                state_value: value.to_owned()
            },
            ComponentWidget::Detail { .. } => ComponentWidgetState::Detail {
                show_action_panel: false,
            },
            ComponentWidget::Form { .. } => ComponentWidgetState::Form {
                show_action_panel: false,
            },
            ComponentWidget::List { .. } => ComponentWidgetState::List {
                show_action_panel: false,
            },
            ComponentWidget::Grid { .. } => ComponentWidgetState::Grid {
                show_action_panel: false,
            },
            _ => ComponentWidgetState::None
        }
    }
}

#[derive(Debug, Clone)]
pub enum ComponentRenderContext {
    None,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    InlineRoot {
        plugin_name: String,
        entrypoint_name: String,
    },
    Inline,
    GridItem,
    List {
        widget_id: UiWidgetId
    },
    Grid {
        widget_id: UiWidgetId
    },
    Root {
        plugin_view_state: PluginViewState,
        entrypoint_name: String,
        action_shortcuts: HashMap<String, PhysicalShortcut>,
    },
}

impl ComponentRenderContext {
    fn is_content_centered(&self) -> bool {
        matches!(self, ComponentRenderContext::Inline | ComponentRenderContext::GridItem)
    }
}

impl ComponentWidgetWrapper {
    pub fn widget(
        id: UiWidgetId,
        widget_type: impl Into<String>,
        properties: HashMap<String, UiPropertyValue>,
        children: Vec<ComponentWidgetWrapper>
    ) -> anyhow::Result<Self> {
        let widget_type = widget_type.into();
        let widget = create_component_widget(&widget_type, properties, children)?;
        let widget_state = ComponentWidgetState::create(&widget);
        let widget = ComponentWidgetWrapper::new(id, widget, widget_state);

        Ok(widget)
    }

    pub fn root(id: UiWidgetId) -> Self {
        ComponentWidgetWrapper::new(id, ComponentWidget::Root { children: vec![] }, ComponentWidgetState::None)
    }

    fn new(id: UiWidgetId, widget: ComponentWidget, state: ComponentWidgetState) -> Self {
        Self {
            id,
            inner: Arc::new(RwLock::new((widget, state))),
        }
    }

    pub fn find_child_with_id(&self, widget_id: UiWidgetId) -> Option<ComponentWidgetWrapper> {
        if self.id == widget_id {
            return Some(self.clone())
        }

        self.get_children()
            .unwrap_or(vec![])
            .into_iter()
            .find_map(|child| child.find_child_with_id(widget_id))
    }

    pub fn find_child_by_type(&self, predicate: &dyn Fn(&ComponentWidget) -> bool) -> Option<ComponentWidgetWrapper> {
        self.get_children()
            .unwrap_or(vec![])
            .into_iter()
            .find_map(|child| {
                let (widget, _) = &*child.get();
                if predicate(widget) {
                    Some(child.clone())
                } else {
                    child.find_child_by_type(&predicate)
                }
            })
    }

    pub fn toggle_action_panel(&self) {
        {
            let (_, ref mut state) = &mut *self.get_mut();

            match state {
                ComponentWidgetState::Detail { show_action_panel, .. } => {
                    *show_action_panel = !*show_action_panel;
                },
                ComponentWidgetState::Form { show_action_panel, .. } => {
                    *show_action_panel = !*show_action_panel;
                },
                ComponentWidgetState::List { show_action_panel, .. } => {
                    *show_action_panel = !*show_action_panel;
                },
                ComponentWidgetState::Grid { show_action_panel, .. } => {
                    *show_action_panel = !*show_action_panel;
                },
                _ => {}
            };
        }

        self.get_children()
            .unwrap_or(vec![])
            .iter()
            .for_each(|child| child.toggle_action_panel());
    }

    pub fn get_all_widgets(&self) -> Vec<ComponentWidgetWrapper> {
        let mut result: Vec<_> = self.get_children()
            .unwrap_or(vec![])
            .iter()
            .flat_map(|component| component.get_all_widgets())
            .collect();

        result.push(self.clone());

        result
    }

    pub fn get_action_ids(&self) -> Vec<UiWidgetId> {
        self.get_all_widgets()
            .into_iter()
            .filter(|component| {
                let (widget, _) = &*component.get();

                matches!(widget, ComponentWidget::Action { .. })
            })
            .map(|component| component.id)
            .collect()
    }

    fn get(&self) -> RwLockReadGuard<'_, (ComponentWidget, ComponentWidgetState)> {
        self.inner.read().expect("lock is poisoned")
    }

    fn get_mut(&self) -> RwLockWriteGuard<'_, (ComponentWidget, ComponentWidgetState)> {
        self.inner.write().expect("lock is poisoned")
    }

    pub fn get_action_panel(&self, action_shortcuts: HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
        self.find_child_by_type(&|widget| matches!(widget, ComponentWidget::ActionPanel { .. }))
            .map(|widget| convert_action_panel(&[widget], action_shortcuts))
            .flatten()
    }

    pub fn render_widget<'a>(&self, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = self.id;
        let (widget, state) = &*self.get();
        match widget {
            ComponentWidget::TextPart { value } => render_text_part(value, context),
            ComponentWidget::ActionPanel { .. } | ComponentWidget::Action { .. } | ComponentWidget::ActionPanelSection { .. } => {
                unreachable!()
            }
            ComponentWidget::MetadataTagItem { children } => {
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let tag: Element<_> = button(content)
                    .on_press(ComponentWidgetEvent::TagClick { widget_id })
                    .themed(ButtonStyle::MetadataTagItem);

                container(tag)
                    .themed(ContainerStyle::MetadataTagItem)
            }
            ComponentWidget::MetadataTagList { label,  children } => {
                let value = wrap_horizontal(render_children(children, ComponentRenderContext::None))
                    .into();

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataLink { label, children, href } => {
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let icon: Element<_> = text(icons::Bootstrap::BoxArrowUpRight)
                    .font(icons::BOOTSTRAP_FONT)
                    .size(16)
                    .into();

                let icon = container(icon)
                    .themed(ContainerStyle::MetadataLinkIcon);

                let content: Element<_> = row([content, icon])
                    .align_items(Alignment::Center)
                    .into();

                let link: Element<_> = button(content)
                    .on_press(ComponentWidgetEvent::LinkClick { widget_id, href: href.to_owned() })
                    .themed(ButtonStyle::MetadataLink);

                let content: Element<_> = if href.is_empty() {
                    link
                } else {
                    let href: Element<_> = text(href)
                        .into();

                    tooltip(link, href, Position::Top)
                        .themed(TooltipStyle::Tooltip)
                };

                render_metadata_item(label, content)
                    .into()
            }
            ComponentWidget::MetadataValue { label, children} => {
                let value: Element<_> = render_children_string(children, ComponentRenderContext::None);

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataIcon { label, icon} => {
                let value = text(icon_to_bootstrap(icon))
                    .font(icons::BOOTSTRAP_FONT)
                    .size(26)
                    .into();

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataSeparator => {
                let separator: Element<_> = horizontal_rule(1)
                    .into();

                container(separator)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MetadataSeparator)
            }
            ComponentWidget::Metadata { children } => {
                let metadata: Element<_> = column(render_children(children, ComponentRenderContext::None))
                    .into();

                let metadata = container(metadata)
                    .width(Length::Fill)
                    .themed(ContainerStyle::MetadataInner);

                scrollable(metadata)
                    .width(Length::Fill)
                    .into()
            }
            ComponentWidget::Paragraph { children } => {
                let centered = context.is_content_centered();

                let paragraph: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let mut content = container(paragraph)
                    .width(Length::Fill);

                if centered {
                    content = content.center_x()
                }

                content.themed(ContainerStyle::ContentParagraph)
            }
            ComponentWidget::Image { source } => {
                let centered = context.is_content_centered();

                let content: Element<_> = match source {
                    ImageSource::_0(bytes) => {
                        image(Handle::from_memory(bytes.clone()))
                            .into()
                    }
                    ImageSource::_1(icon) => {
                        text(icon_to_bootstrap(icon))
                            .font(icons::BOOTSTRAP_FONT) // TODO size, height and width
                            .into()
                    }
                };

                let mut content = container(content)
                    .width(Length::Fill);

                if centered {
                    content = content.center_x()
                }

                content.themed(ContainerStyle::ContentImage)
            }
            ComponentWidget::H1 { children } => {
                render_children_string(children, ComponentRenderContext::H1)
            }
            ComponentWidget::H2 { children } => {
                render_children_string(children, ComponentRenderContext::H2)
            }
            ComponentWidget::H3 { children } => {
                render_children_string(children, ComponentRenderContext::H3)
            }
            ComponentWidget::H4 { children } => {
                render_children_string(children, ComponentRenderContext::H4)
            }
            ComponentWidget::H5 { children } => {
                render_children_string(children, ComponentRenderContext::H5)
            }
            ComponentWidget::H6 { children } => {
                render_children_string(children, ComponentRenderContext::H6)
            }
            ComponentWidget::HorizontalBreak => {
                let separator: Element<_> = horizontal_rule(1).into();

                container(separator)
                    .width(Length::Fill)
                    .themed(ContainerStyle::ContentHorizontalBreak)
            }
            ComponentWidget::CodeBlock { children } => {
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let content = container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::ContentCodeBlockText);

                container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::ContentCodeBlock)
            }
            ComponentWidget::Content { children } => {
                let centered = context.is_content_centered();

                let content: Element<_> = column(render_children(children, context))
                    .into();

                if centered {
                    container(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                        .into()
                } else {
                    content
                }
            }
            ComponentWidget::Detail { children, isLoading: is_loading } => {
                let ComponentWidgetState::Detail { show_action_panel } = *state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let is_in_list = matches!(context, ComponentRenderContext::List { .. });

                let metadata_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Metadata { .. }), ComponentRenderContext::None)
                    .map(|metadata_element| {
                        container(metadata_element)
                            .width(if is_in_list { Length::Fill } else { Length::FillPortion(2) })
                            .height(if is_in_list { Length::FillPortion(3) } else { Length::Fill })
                            .themed(ContainerStyle::DetailMetadata)
                    })
                    .ok();

                let content_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::None)
                    .map(|content_element| {
                        let content_element: Element<_> = container(content_element)
                            .width(Length::Fill)
                            .themed(ContainerStyle::DetailContentInner);

                        let content_element: Element<_> = scrollable(content_element)
                            .width(Length::Fill)
                            .into();

                        let content_element: Element<_> = container(content_element)
                            .width(if is_in_list { Length::Fill } else { Length::FillPortion(3) })
                            .height(if is_in_list { Length::FillPortion(5) } else { Length::Fill })
                            .themed(ContainerStyle::DetailContent);

                        content_element
                    })
                    .ok();

                let separator = if is_in_list {
                    horizontal_rule(1)
                        .into()
                } else {
                    vertical_rule(1)
                        .into()
                };

                let list_fn = |vec| {
                    if is_in_list {
                        column(vec)
                            .into()
                    } else {
                        row(vec)
                            .into()
                    }
                };

                let content: Element<_> = match (content_element, metadata_element) {
                    (Some(content_element), Some(metadata_element)) => {
                        list_fn(vec![content_element, separator, metadata_element])
                    }
                    (Some(content_element), None) => {
                        list_fn(vec![content_element])
                    }
                    (None, Some(metadata_element)) => {
                        list_fn(vec![metadata_element])
                    }
                    (None, None) => {
                        list_fn(vec![])
                    }
                };

                if is_in_list {
                    content
                } else {
                    render_plugin_root(show_action_panel, widget_id, children, content, context, is_loading.unwrap_or(false))
                }
            }
            ComponentWidget::Root { children } => {
                row(render_children(children, context))
                    .into()
            }
            ComponentWidget::TextField { .. } => {
                let ComponentWidgetState::TextField { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                text_input("", state_value)
                    .on_input(move |value| ComponentWidgetEvent::OnChangeTextField { widget_id, value })
                    .themed(TextInputStyle::FormInput)
            }
            ComponentWidget::PasswordField { .. } => {
                let ComponentWidgetState::PasswordField { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                text_input("", state_value)
                    .secure(true)
                    .on_input(move |value| ComponentWidgetEvent::OnChangePasswordField { widget_id, value })
                    .themed(TextInputStyle::FormInput)
            }
            ComponentWidget::Checkbox { title, .. } => {
                let ComponentWidgetState::Checkbox { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                checkbox(title.clone().unwrap_or_default(), state_value.to_owned())
                    .on_toggle(move |value| ComponentWidgetEvent::ToggleCheckbox { widget_id, value })
                    .into()
            }
            ComponentWidget::DatePicker { .. } => {
                let ComponentWidgetState::DatePicker { state_value, show_picker } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let button = button(text(state_value.to_string()))
                    .on_press(ComponentWidgetEvent::ToggleDatePicker { widget_id });

                // TODO unable to customize buttons here, split to separate button styles
                //     DatePickerUnderlay,
                //     DatePickerOverlay,

                date_picker(
                    show_picker.to_owned(),
                    state_value.to_owned(),
                    button,
                    ComponentWidgetEvent::CancelDatePicker { widget_id },
                    move |date| {
                        ComponentWidgetEvent::SubmitDatePicker {
                            widget_id,
                            value: date.to_string(),
                        }
                    }
                ).themed(DatePickerStyle::Default)
            }
            ComponentWidget::SelectItem { .. } => {
                panic!("parent select component takes care of rendering")
            }
            ComponentWidget::Select { children, .. } => {
                let items: Vec<_> = children.iter()
                    .map(|child| {
                        let (widget, _) = &*child.get();

                        let ComponentWidget::SelectItem { children, value } = widget else {
                            panic!("unexpected widget kind {:?}", widget)
                        };

                        let label = children.iter()
                            .map(|child| {
                                let (widget, _) = &*child.get();
                                let ComponentWidget::TextPart { value } = widget else {
                                    panic!("unexpected widget kind {:?}", widget)
                                };

                                value.to_owned()
                            })
                            .collect::<Vec<_>>()
                            .join("");

                        SelectItem {
                            value: value.to_owned(),
                            label
                        }
                    })
                    .collect();

                let ComponentWidgetState::Select { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let state_value = state_value.clone()
                    .map(|value| items.iter().find(|item| item.value == value))
                    .flatten()
                    .map(|value| value.clone());

                pick_list(
                    items,
                    state_value,
                    move |item| ComponentWidgetEvent::SelectPickList { widget_id, value: item.value }
                ).themed(PickListStyle::Default)
            }
            ComponentWidget::Separator => {
                horizontal_rule(1)
                    .into()
            }
            ComponentWidget::Form { children, isLoading: is_loading } => {
                let ComponentWidgetState::Form { show_action_panel } = *state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let items: Vec<Element<_>> = children.iter()
                    .flat_map(|child| {
                        let (widget, _) = &*child.get();

                        match widget {
                            ComponentWidget::Separator => Some(child.render_widget(ComponentRenderContext::None)),
                            ComponentWidget::ActionPanel { .. } => None,
                            _ => {
                                let label = match widget {
                                    ComponentWidget::TextField { label, .. } => label.clone(),
                                    ComponentWidget::PasswordField { label, .. } => label.clone(),
                                    ComponentWidget::Checkbox { label, .. } => label.clone(),
                                    ComponentWidget::DatePicker { label, .. } => label.clone(),
                                    ComponentWidget::Select { label, .. } => label.clone(),
                                    _ => None
                                };

                                let before_or_label: Element<_> = match label {
                                    None => {
                                        Space::with_width(Length::FillPortion(2))
                                            .into()
                                    }
                                    Some(label) => {
                                        let label: Element<_> = text(label)
                                            .horizontal_alignment(Horizontal::Right)
                                            .width(Length::Fill)
                                            .into();

                                        container(label)
                                            .width(Length::FillPortion(2))
                                            .themed(ContainerStyle::FormInputLabel)
                                    }
                                };

                                let form_input = container(child.render_widget(ComponentRenderContext::None))
                                    .width(Length::FillPortion(3))
                                    .into();

                                let after = Space::with_width(Length::FillPortion(2))
                                    .into();

                                let content = vec![
                                    before_or_label,
                                    form_input,
                                    after,
                                ];

                                let row: Element<_> = row(content)
                                    .align_items(Alignment::Center)
                                    .themed(RowStyle::FormInput);

                                Some(row)
                            }
                        }
                    })
                    .collect();

                let content: Element<_> = column(items)
                    .into();

                let content: Element<_> = container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::FormInner);

                let content: Element<_> = scrollable(content)
                    .width(Length::Fill)
                    .into();

                let content: Element<_> = container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::Form);

                render_plugin_root(show_action_panel, widget_id, children, content, context, is_loading.unwrap_or(false))
            }
            ComponentWidget::InlineSeparator { icon } => {
                match icon {
                    None => vertical_rule(1).into(),
                    Some(icon) => {
                        let top_rule: Element<_> = vertical_rule(1)
                            .into();

                        let top_rule = container(top_rule)
                            .center_x()
                            .into();

                        let icon = text(icon_to_bootstrap(icon))
                            .font(icons::BOOTSTRAP_FONT)
                            .size(45)
                            .themed(TextStyle::InlineSeparator);

                        let bot_rule: Element<_> = vertical_rule(1)
                            .into();

                        let bot_rule = container(bot_rule)
                            .center_x()
                            .into();

                        column([top_rule, icon, bot_rule])
                            .align_items(Alignment::Center)
                            .into()
                    }
                }
            }
            ComponentWidget::Inline { children } => {
                let ComponentRenderContext::InlineRoot { plugin_name, entrypoint_name } = context else {
                    panic!("not supposed to be passed to root item: {:?}", context)
                };

                let name: Element<_> = text(format!("{} - {}", plugin_name, entrypoint_name))
                    .themed(TextStyle::InlineName);

                let name: Element<_> = container(name)
                    .themed(ContainerStyle::InlineName);

                let content: Vec<Element<_>> = children
                    .into_iter()
                    .filter_map(|child| {
                        let (widget, _) = &*child.get();

                        match widget {
                            ComponentWidget::InlineSeparator { .. } => {
                                Some(child.render_widget(ComponentRenderContext::None))
                            }
                            ComponentWidget::Content { .. } => {
                                let element = child.render_widget(ComponentRenderContext::Inline);

                                let container = container(element)
                                    .width(Length::Fill)
                                    .into();

                                Some(container)
                            }
                            _ => None
                        }
                    })
                    .collect();

                let content: Element<_> = row(content)
                    .into();

                let content: Element<_> = container(content)
                    .themed(ContainerStyle::InlineInner);

                let content: Element<_> = column(vec![name, content])
                    .width(Length::Fill)
                    .into();

                let content: Element<_> = container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::Inline);

                content
            }
            ComponentWidget::EmptyView { title, description, image: empty_view_image } => {
                let image: Option<Element<_>> = empty_view_image.as_ref()
                    .map(|empty_view_image| {
                        match empty_view_image {
                            EmptyViewImage::_0(bytes) => {
                                image(Handle::from_memory(bytes.clone()))
                                    .themed(ImageStyle::EmptyViewImage)
                            }
                            EmptyViewImage::_1(icon) => {
                                text(icon_to_bootstrap(icon))
                                    .font(icons::BOOTSTRAP_FONT) // TODO size, height and width
                                    .into()
                            }
                        }
                    });

                let title: Element<_> = text(title)
                    .into();

                let subtitle: Element<_> = match description {
                    None => horizontal_space().into(),
                    Some(subtitle) => {
                        text(subtitle)
                            .themed(TextStyle::EmptyViewSubtitle)
                    }
                };

                let mut content = vec![title, subtitle];
                if let Some(image) = image {

                    let image: Element<_> = container(image)
                        .themed(ContainerStyle::EmptyViewImage);

                    content.insert(0, image)
                }

                let content: Element<_> = column(content)
                    .align_items(Alignment::Center)
                    .into();

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
            ComponentWidget::IconAccessory { icon, tooltip: tooltip_text } => {
                let icon: Element<_> = match icon {
                    IconAccessoryIcon::_0(bytes) => {
                        image(Handle::from_memory(bytes.clone()))
                            .into()
                    },
                    IconAccessoryIcon::_1(icon) => {
                        text(icon_to_bootstrap(icon))
                            .font(icons::BOOTSTRAP_FONT)
                            .themed(TextStyle::IconAccessory)
                    }
                };

                let content = container(icon)
                    .center_x()
                    .center_y()
                    .themed(ContainerStyle::IconAccessory);

                match tooltip_text.as_ref() {
                    None => content,
                    Some(tooltip_text) => {
                        let tooltip_text: Element<_> = text(tooltip_text)
                            .into();

                        tooltip(content, tooltip_text, Position::Top)
                            .themed(TooltipStyle::Tooltip)
                    }
                }
            },
            ComponentWidget::TextAccessory { text: text_value, icon, tooltip: tooltip_text } => {
                let icon: Option<Element<_>> = icon.as_ref()
                    .map(|icon| {
                        match icon {
                            TextAccessoryIcon::_0(bytes) => {
                                image(Handle::from_memory(bytes.clone()))
                                    .into()
                            },
                            TextAccessoryIcon::_1(icon) => {
                                let icon = icon.to_owned();
                                text(icon_to_bootstrap(icon))
                                    .font(icons::BOOTSTRAP_FONT)
                                    .themed(TextStyle::TextAccessory)
                            }
                        }
                    });

                let text_content: Element<_> = text(text_value)
                    .themed(TextStyle::TextAccessory);

                let mut content: Vec<Element<_>> = vec![];

                if let Some(icon) = icon {
                    let icon: Element<_> = container(icon)
                        .themed(ContainerStyle::TextAccessoryIcon);

                    content.push(icon)
                }

                content.push(text_content);

                let content: Element<_> = row(content)
                    .align_items(Alignment::Center)
                    .into();

                let content = container(content)
                    .center_x()
                    .center_y()
                    .themed(ContainerStyle::TextAccessory);

                match tooltip_text.as_ref() {
                    None => content,
                    Some(tooltip_text) => {
                        let tooltip_text: Element<_> = text(tooltip_text)
                            .into();

                        tooltip(content, tooltip_text, Position::Top)
                            .themed(TooltipStyle::Tooltip)
                    }
                }
            },
            ComponentWidget::ListItem { title, subtitle, icon, children } => {
                let accessories = render_children_by_type(children, |widget| matches!(widget, ComponentWidget::TextAccessory { .. } | ComponentWidget::IconAccessory { .. }), ComponentRenderContext::None);

                let icon: Option<Element<_>> = icon.as_ref()
                    .map(|icon| {
                        match icon {
                            ListItemIcon::_0(bytes) => {
                                image(Handle::from_memory(bytes.clone()))
                                    .into()
                            },
                            ListItemIcon::_1(icon) => {
                                let icon = icon.to_owned();
                                text(icon_to_bootstrap(icon))
                                    .font(icons::BOOTSTRAP_FONT)
                                    .into()
                            }
                        }
                    });

                let title: Element<_> = text(title)
                    .into();
                let title: Element<_> = container(title)
                    .themed(ContainerStyle::ListItemTitle);

                let mut content = vec![title];

                if let Some(icon) = icon {
                    let icon: Element<_> = container(icon)
                        .themed(ContainerStyle::ListItemIcon);

                    content.insert(0, icon)
                }

                if let Some(subtitle) = subtitle {
                    let subtitle: Element<_> = text(subtitle)
                        .themed(TextStyle::ListItemSubtitle);
                    let subtitle: Element<_> = container(subtitle)
                        .themed(ContainerStyle::ListItemSubtitle);

                    content.push(subtitle)
                }

                if accessories.len() > 0 {
                    let accessories: Element<_> = row(accessories)
                        .into();

                    let space = horizontal_space()
                        .into();

                    content.push(space);
                    content.push(accessories);
                }

                let content: Element<_> = row(content)
                    .align_items(Alignment::Center)
                    .into();

                button(content)
                    .on_press(ComponentWidgetEvent::ListItemClick { widget_id })
                    .width(Length::Fill)
                    .themed(ButtonStyle::ListItem)
            }
            ComponentWidget::ListSection { children, title, subtitle } => {
                let content = render_children(children, context);

                let content = column(content)
                    .into();

                render_section(content, Some(title), subtitle, RowStyle::ListSectionTitle, TextStyle::ListSectionTitle, TextStyle::ListSectionSubtitle)
            }
            ComponentWidget::List { children, isLoading: is_loading } => {
                let ComponentWidgetState::List { show_action_panel } = *state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let mut pending: Vec<ComponentWidgetWrapper> = vec![];
                let mut items: Vec<Element<_>> = vec![];

                for child in children {
                    let (widget, _) = &*child.get();

                    match widget {
                        ComponentWidget::ListItem { .. } => {
                            pending.push(child.clone())
                        },
                        ComponentWidget::ListSection { .. } => {
                            if !pending.is_empty() {
                                let content: Element<_> = column(render_children(&pending, ComponentRenderContext::List { widget_id }))
                                    .into();

                                items.push(content);

                                pending = vec![];
                            }

                            items.push(child.render_widget(ComponentRenderContext::List { widget_id }))
                        },
                        ComponentWidget::EmptyView { .. } | ComponentWidget::Detail { .. } => {},
                        _ => panic!("unexpected widget kind {:?}", widget)
                    }
                }

                if !pending.is_empty() {
                    let content: Element<_> = column(render_children(&pending, ComponentRenderContext::List { widget_id }))
                        .into();

                    items.push(content);
                }

                let content = if items.is_empty() {
                    if let Ok(empty_view) =  render_child_by_type(children, |child| matches!(child, ComponentWidget::EmptyView { .. }), ComponentRenderContext::None) {
                        empty_view
                    } else {
                        horizontal_space()
                            .into()
                    }
                } else {
                    let content: Element<_> = column(items)
                        .width(Length::Fill)
                        .into();

                    let content: Element<_> = container(content)
                        .width(Length::Fill)
                        .themed(ContainerStyle::ListInner);

                    let content: Element<_> = scrollable(content)
                        .width(Length::Fill)
                        .into();

                    let content: Element<_> = container(content)
                        .width(Length::FillPortion(3))
                        .themed(ContainerStyle::List);

                    content
                };

                let mut elements = vec![content];

                if let Ok(detail) = render_child_by_type(children, |child| matches!(child, ComponentWidget::Detail { .. }), ComponentRenderContext::List { widget_id }) {

                    let detail: Element<_> = container(detail)
                        .width(Length::FillPortion(5))
                        .into();

                    let separator: Element<_> = vertical_rule(1)
                        .into();

                    elements.push(separator);

                    elements.push(detail);
                }

                let content: Element<_> = row(elements)
                    .height(Length::Fill)
                    .into();

                render_plugin_root(show_action_panel, widget_id, children, content, context, is_loading.unwrap_or(false))
            }
            ComponentWidget::GridItem { children, title, subtitle } => {
                // TODO should be just one
                let accessories = render_children_by_type(children, |widget| matches!(widget, ComponentWidget::IconAccessory { .. }), ComponentRenderContext::None);

                let content: Element<_> = column(render_children_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::GridItem))
                    .height(130) // TODO dynamic height
                    .into();

                let content: Element<_> = button(content)
                    .on_press(ComponentWidgetEvent::GridItemClick { widget_id })
                    .width(Length::Fill)
                    .themed(ButtonStyle::GridItem);

                let mut sub_content_left = vec![];

                if let Some(title) = title {
                    // TODO text truncation when iced supports it
                    let title = text(title)
                        .themed(TextStyle::GridItemTitle);

                    sub_content_left.push(title);
                }

                if let Some(subtitle) = subtitle {
                    let subtitle = text(subtitle)
                        .themed(TextStyle::GridItemSubTitle);

                    sub_content_left.push(subtitle);
                }

                let mut sub_content_right = vec![];
                if accessories.len() > 0 {
                    let accessories: Element<_> = row(accessories)
                        .into();

                    sub_content_right.push(accessories);
                }

                let sub_content_left: Element<_> = column(sub_content_left)
                    .width(Length::Fill)
                    .into();

                let sub_content_right: Element<_> = column(sub_content_right)
                    .width(Length::Shrink)
                    .into();

                let sub_content: Element<_> = row(vec![sub_content_left, sub_content_right])
                    .themed(RowStyle::GridItemTitle);

                let content: Element<_> = column(vec![content, sub_content])
                    .width(Length::Fill)
                    .into();

                content
            }
            ComponentWidget::GridSection { children, title, subtitle, columns } => {
                let content = render_grid(children, columns, context);

                render_section(content, Some(title), subtitle, RowStyle::GridSectionTitle, TextStyle::GridSectionTitle, TextStyle::GridSectionSubtitle)
            }
            ComponentWidget::Grid { children, columns, isLoading: is_loading } => {
                let ComponentWidgetState::Grid { show_action_panel } = *state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let mut pending: Vec<ComponentWidgetWrapper> = vec![];
                let mut items: Vec<Element<_>> = vec![];

                for child in children {
                    let (widget, _) = &*child.get();

                    match widget {
                        ComponentWidget::GridItem { .. } => {
                            pending.push(child.clone())
                        },
                        ComponentWidget::GridSection { .. } => {
                            if !pending.is_empty() {
                                let content = render_grid(&pending, columns, ComponentRenderContext::Grid { widget_id });

                                items.push(content);

                                pending = vec![];
                            }

                            items.push(child.render_widget(ComponentRenderContext::Grid { widget_id }))
                        },
                        ComponentWidget::EmptyView { .. } => {},
                        _ => panic!("unexpected widget kind {:?}", widget)
                    }
                }

                if !pending.is_empty() {
                    let content = render_grid(&pending, columns, ComponentRenderContext::Grid { widget_id });

                    items.push(content);
                }

                let content: Element<_> = column(items)
                    .into();

                let content: Element<_> = container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::GridInner);

                let content: Element<_> = scrollable(content)
                    .width(Length::Fill)
                    .into();

                let content: Element<_> = container(content)
                    .width(Length::Fill)
                    .themed(ContainerStyle::Grid);

                render_plugin_root(show_action_panel, widget_id, children, content, context, is_loading.unwrap_or(false))
            }
        }
    }

    pub fn get_children(&self) -> anyhow::Result<Vec<ComponentWidgetWrapper>> {
        get_component_widget_children(&self)
    }

    pub fn set_children(&self, new_children: Vec<ComponentWidgetWrapper>) -> anyhow::Result<()> {
        set_component_widget_children(&self, new_children)
    }
}

fn create_top_panel<'a>() -> Element<'a, ComponentWidgetEvent> {
    let icon = text(icons::Bootstrap::ArrowLeft)
        .font(icons::BOOTSTRAP_FONT);

    let back_button: Element<_> = button(icon)
        .on_press(ComponentWidgetEvent::PreviousView)
        .themed(ButtonStyle::RootTopPanelBackButton);

    let space = Space::with_width(Length::FillPortion(3))
        .into();

    let top_panel: Element<_> = row(vec![back_button, space])
        .align_items(Alignment::Center)
        .into();

    let top_panel: Element<_> = container(top_panel)
        .width(Length::Fill)
        .themed(ContainerStyle::RootTopPanel);

    top_panel
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectItem {
    value: String,
    label: String
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}


fn render_metadata_item<'a>(label: &str, value: Element<'a, ComponentWidgetEvent>) -> Element<'a, ComponentWidgetEvent> {
    let label: Element<_> = text(label)
        .themed(TextStyle::MetadataItemLabel);

    let label = container(label)
        .themed(ContainerStyle::MetadataItemLabel);

    let value = container(value)
        .themed(ContainerStyle::MetadataItemValue);

    column(vec![label, value])
        .into()
}

fn render_grid<'a>(children: &[ComponentWidgetWrapper], /*aspect_ratio: Option<&str>,*/ columns: &Option<f64>, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
    // let (width, height) = match aspect_ratio {
    //     None => (1, 1),
    //     Some("1") => (1, 1),
    //     Some("3/2") => (3, 2),
    //     Some("2/3") => (2, 3),
    //     Some("4/3") => (4, 3),
    //     Some("3/4") => (3, 4),
    //     Some("16/9") => (16, 9),
    //     Some("9/16") => (9, 16),
    //     Some(value) => panic!("unsupported aspect_ratio {:?}", value)
    // };

    let row_length = columns.map(|value| value.trunc() as usize).unwrap_or(5);

    let rows: Vec<GridRow<_, _, _>> = render_children(children, context)
        .into_iter()
        .chunks(row_length)
        .into_iter()
        .map(|row_items| {
            let mut row_items: Vec<_> = row_items.collect();
            row_items.resize_with(row_length, || horizontal_space().into());

            grid_row(row_items).into()
        })
        .collect();

    let grid: Element<_> = grid(rows)
        .width(Length::Fill)
        .vertical_alignment(Vertical::Top)
        .themed(GridStyle::Default);

    grid
}

fn render_section<'a>(content: Element<'a, ComponentWidgetEvent>, title: Option<&str>, subtitle: &Option<String>, theme_kind_title: RowStyle, theme_kind_title_text: TextStyle, theme_kind_subtitle_text: TextStyle) -> Element<'a, ComponentWidgetEvent> {
    let mut title_content = vec![];

    if let Some(title) = title {
        let title: Element<_> = text(title)
            .size(15)
            .themed(theme_kind_title_text);

        title_content.push(title)
    }

    if let Some(subtitle) = subtitle {
        let subtitle: Element<_> = text(subtitle)
            .size(15)
            .themed(theme_kind_subtitle_text);

        title_content.push(subtitle)
    }

    if title_content.is_empty() {
        let space: Element<_> = horizontal_space()
            .height(40)
            .into();

        title_content.push(space)
    }

    let title_content = row(title_content)
        .themed(theme_kind_title);

    column([title_content, content])
        .into()
}

fn render_plugin_root<'a>(
    show_action_panel: bool,
    widget_id: UiWidgetId,
    children: &[ComponentWidgetWrapper],
    content: Element<'a, ComponentWidgetEvent>,
    context: ComponentRenderContext,
    is_loading: bool
) -> Element<'a, ComponentWidgetEvent>  {
    let ComponentRenderContext::Root { entrypoint_name, action_shortcuts, plugin_view_state } = context else {
        panic!("not supposed to be passed to root item: {:?}", context)
    };

    let top_panel = create_top_panel();

    let top_separator = if is_loading {
        LoadingBar::new()
            .into()
    } else {
        horizontal_rule(1)
            .into()
    };

    let mut action_panel = convert_action_panel(children, action_shortcuts);

    let primary_action = action_panel.as_mut()
        .map(|panel| panel.find_first())
        .flatten()
        .map(|label| {
            let shortcut = PhysicalShortcut {
                physical_key: PhysicalKey::Enter,
                modifier_shift: false,
                modifier_control: false,
                modifier_alt: false,
                modifier_meta: false
            };

            (label.to_string(), shortcut)
        });

    match plugin_view_state {
        PluginViewState::None => {
            render_root(
                show_action_panel,
                top_panel,
                top_separator,
                content,
                primary_action,
                action_panel,
                None::<&ScrollHandle<UiWidgetId>>,
                entrypoint_name,
                || ComponentWidgetEvent::ToggleActionPanel { widget_id },
                |widget_id| ComponentWidgetEvent::ActionClick { widget_id }
            )
        }
        PluginViewState::ActionPanel { focused_action_item } => {
            render_root(
                show_action_panel,
                top_panel,
                top_separator,
                content,
                primary_action,
                action_panel,
                Some(&focused_action_item),
                entrypoint_name,
                || ComponentWidgetEvent::ToggleActionPanel { widget_id },
                |widget_id| ComponentWidgetEvent::ActionClick { widget_id }
            )
        }
    }
}

#[derive(Debug)]
pub struct ActionPanel {
    pub title: Option<String>,
    pub items: Vec<ActionPanelItem>
}

impl ActionPanel {
    pub fn action_count(&self) -> usize {
        self.items.iter().map(|item| item.action_count()).sum()
    }

    pub fn find_first(&self) -> Option<String> {
        ActionPanelItem::find_first(&self.items)
    }
}

#[derive(Debug)]
pub enum ActionPanelItem {
    Action {
        label: String,
        widget_id: UiWidgetId,
        physical_shortcut: Option<PhysicalShortcut>
    },
    ActionSection {
        title: Option<String>,
        items: Vec<ActionPanelItem>
    }
}

impl ActionPanelItem {
    fn action_count(&self) -> usize {
        match self {
            ActionPanelItem::Action { .. } => 1,
            ActionPanelItem::ActionSection { items, .. } => {
                items.iter().map(|item| item.action_count()).sum()
            }
        }
    }

    fn find_first(items: &[ActionPanelItem]) -> Option<String> {
        for item in items {
            match item {
                ActionPanelItem::Action { label, .. } => {
                    return Some(label.to_string())
                }
                ActionPanelItem::ActionSection { items, .. } => {
                    if let Some(item) = Self::find_first(items) {
                        return Some(item)
                    }
                }
            }
        }

        None
    }
}

fn convert_action_panel(children: &[ComponentWidgetWrapper], action_shortcuts: HashMap<String, PhysicalShortcut>) -> Option<ActionPanel> {
    let action_panel: Vec<_> = children
        .into_iter()
        .filter(|child| {
            let (widget, _) = &*child.get();
            matches!(widget, ComponentWidget::ActionPanel { .. })
        })
        .collect();

    let action_panel = match action_panel[..] {
        [] => None?,
        [single] => single,
        [_, _, ..] => None?,
    };

    let (action_panel, _) = &*action_panel.get();

    match action_panel {
        ComponentWidget::ActionPanel { children, title } => {
            fn convert_to_items(children: &[ComponentWidgetWrapper], action_shortcuts: &HashMap<String, PhysicalShortcut>) -> Vec<ActionPanelItem> {
                let mut items = vec![];

                for child in children {
                    let widget_id = child.id;
                    let (widget, _) = &*child.get();

                    match widget {
                        ComponentWidget::Action { id, label } => {
                            let physical_shortcut: Option<PhysicalShortcut> = id.as_ref()
                                .map(|id| action_shortcuts.get(id))
                                .flatten()
                                .cloned();

                            items.push(ActionPanelItem::Action {
                                label: label.clone(),
                                widget_id,
                                physical_shortcut,
                            });
                        }
                        ComponentWidget::ActionPanelSection { children, title } => {
                            items.push(ActionPanelItem::ActionSection {
                                title: title.clone(),
                                items: convert_to_items(children, action_shortcuts),
                            });
                        }
                        _ => {
                            panic!("unexpected widget kind {:?}", widget)
                        }
                    };
                }

                items
            }

            Some(ActionPanel {
                title: title.clone(),
                items: convert_to_items(children, &action_shortcuts),
            })
        }
        _ => None
    }
}

fn render_action_panel_items<'a, T: 'a + Clone, ACTION>(
    title: Option<String>,
    items: Vec<ActionPanelItem>,
    action_panel_scroll_handle: &ScrollHandle<ACTION>,
    on_action_click: &dyn Fn(UiWidgetId) -> T,
    index_counter: &Cell<usize>
) -> Vec<Element<'a, T>> {
    let mut columns = vec![];

    if let Some(title) = title {
        let text: Element<_> = text(title)
            .font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            })
            .into();

        let text = container(text)
            .themed(ContainerStyle::ActionPanelTitle);

        columns.push(text)
    }

    let mut place_separator = false;

    for item in items {
        match item {
            ActionPanelItem::Action { label, widget_id, physical_shortcut } => {
                if place_separator {
                    let separator: Element<_> = horizontal_rule(1)
                        .themed(RuleStyle::ActionPanel);

                    columns.push(separator);

                    place_separator = false;
                }

                let physical_shortcut = match index_counter.get() {
                    0 => Some(PhysicalShortcut { // primary
                        physical_key: PhysicalKey::Enter,
                        modifier_shift: false,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: false,
                    }),
                    1 => Some(PhysicalShortcut { // secondary
                        physical_key: PhysicalKey::Enter,
                        modifier_shift: true,
                        modifier_control: false,
                        modifier_alt: false,
                        modifier_meta: false,
                    }),
                    _ => physical_shortcut
                };

                let shortcut_element: Option<Element<_>> = physical_shortcut.as_ref()
                    .map(|shortcut| render_shortcut(shortcut));

                let content: Element<_> = if let Some(shortcut_element) = shortcut_element {
                    let text: Element<_> = text(label)
                        .into();

                    let space: Element<_> = horizontal_space()
                        .into();

                    row([text, space, shortcut_element])
                        .align_items(Alignment::Center)
                        .into()
                } else {
                    text(label)
                        .into()
                };

                let style = match action_panel_scroll_handle.index {
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
                let separator: Element<_> = horizontal_rule(1)
                    .themed(RuleStyle::ActionPanel);

                columns.push(separator);

                let content = render_action_panel_items(title, items, action_panel_scroll_handle, on_action_click, index_counter);

                for content in content {
                    columns.push(content);
                }

                place_separator = true;
            }
        };
    }

    columns
}

fn render_action_panel<'a, T: 'a + Clone, F: Fn(UiWidgetId) -> T, ACTION>(
    action_panel: ActionPanel,
    on_action_click: F,
    action_panel_scroll_handle: &ScrollHandle<ACTION>,
) -> Element<'a, T> {
    let columns = render_action_panel_items(action_panel.title, action_panel.items, action_panel_scroll_handle, &on_action_click, &Cell::new(0));

    let actions: Element<_> = column(columns)
        .into();

    let actions: Element<_> = scrollable(actions)
        .id(action_panel_scroll_handle.scrollable_id.clone())
        .width(Length::Fill)
        .into();

    container(actions)
        .themed(ContainerStyle::ActionPanel)
}

pub fn render_root<'a, T: 'a + Clone, ACTION>(
    show_action_panel: bool,
    top_panel: Element<'a, T>,
    top_separator: Element<'a, T>,
    content: Element<'a, T>,
    primary_action: Option<(String, PhysicalShortcut)>,
    action_panel: Option<ActionPanel>,
    action_panel_scroll_handle: Option<&ScrollHandle<ACTION>>,
    entrypoint_name: String,
    on_panel_toggle_click: impl Fn() -> T,
    on_action_click: impl Fn(UiWidgetId) -> T,
) -> Element<'a, T>  {
    let entrypoint_name: Element<_> = text(entrypoint_name)
        .into();

    let panel_height = 16 + 8 + 2;  // TODO get value from theme

    let primary_action = match primary_action {
        Some((label, shortcut)) => {
            let label: Element<_> = text(label)
                .themed(TextStyle::RootBottomPanelPrimaryActionText);

            let label: Element<_> = container(label)
                .themed(ContainerStyle::RootBottomPanelPrimaryActionText);

            let shortcut = render_shortcut(&shortcut);

            let content: Element<_> = row(vec![label, shortcut])
                .themed(RowStyle::RootBottomPanelPrimaryAction);

            Some(content)
        }
        None => None
    };

    let (hide_action_panel, action_panel, bottom_panel) = match action_panel {
        Some(action_panel) => {
            let actions_text: Element<_> = text("Actions")
                .themed(TextStyle::RootBottomPanelActionToggleText);

            let actions_text: Element<_> = container(actions_text)
                .themed(ContainerStyle::RootBottomPanelActionToggleText);

            let shortcut = render_shortcut(&PhysicalShortcut {
                physical_key: PhysicalKey::KeyK,
                modifier_shift: false,
                modifier_control: false,
                modifier_alt: true,
                modifier_meta: false,
            });

            let mut bottom_panel_content = vec![entrypoint_name];

            let space = horizontal_space()
                .into();

            bottom_panel_content.push(space);

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);

                let rule: Element<_> = vertical_rule(1)
                    .style(RuleStyle::PrimaryActionSeparator)
                    .into();

                let rule: Element<_> = container(rule)
                    .width(Length::Shrink)
                    .height(panel_height)
                    .max_height(panel_height)
                    .into();

                bottom_panel_content.push(rule);
            }

            let action_panel_toggle_content: Element<_> = row(vec![actions_text, shortcut])
                .into();

            let action_panel_toggle: Element<_> = button(action_panel_toggle_content)
                .on_press(on_panel_toggle_click())
                .themed(ButtonStyle::RootBottomPanelActionToggleButton);

            bottom_panel_content.push(action_panel_toggle);

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_items(Alignment::Center)
                .themed(RowStyle::RootBottomPanel);

            (!show_action_panel, Some(action_panel), bottom_panel)
        }
        None => {
            let space: Element<_> = Space::new(Length::Fill, panel_height)
                .into();

            let mut bottom_panel_content = vec![entrypoint_name, space];

            if let Some(primary_action) = primary_action {
                bottom_panel_content.push(primary_action);
            }

            let bottom_panel: Element<_> = row(bottom_panel_content)
                .align_items(Alignment::Center)
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

    let content: Element<_> = column(vec![top_panel, top_separator, content, bottom_panel])
        .into();

    let content = if hide_action_panel {
        content
    } else {
        mouse_area(content)
            .on_press(on_panel_toggle_click())
            .into()
    };

    let action_panel_element = match (action_panel, action_panel_scroll_handle) {
        (Some(action_panel), Some(action_panel_scroll_handle)) => render_action_panel(action_panel, on_action_click, action_panel_scroll_handle),
        _ => Space::with_height(1).into(),
    };

    floating_element(content, action_panel_element)
        .offset(Offset::from([8.0, 48.0])) // TODO calculate based on theme
        .hide(hide_action_panel)
        .into()
}

fn render_text_part<'a>(value: &str, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
    let header = match context {
        ComponentRenderContext::None => None,
        ComponentRenderContext::H1 => Some(34),
        ComponentRenderContext::H2 => Some(30),
        ComponentRenderContext::H3 => Some(24),
        ComponentRenderContext::H4 => Some(20),
        ComponentRenderContext::H5 => Some(18),
        ComponentRenderContext::H6 => Some(16),
        ComponentRenderContext::List { .. } => panic!("not supposed to be passed to text part"),
        ComponentRenderContext::Grid { .. } => panic!("not supposed to be passed to text part"),
        ComponentRenderContext::Root { .. } => panic!("not supposed to be passed to text part"),
        ComponentRenderContext::Inline => panic!("not supposed to be passed to text part"),
        ComponentRenderContext::GridItem => panic!("not supposed to be passed to text part"),
        ComponentRenderContext::InlineRoot { .. } => panic!("not supposed to be passed to text part")
    };

    let mut text = text(value);

    if let Some(size) = header {
        text = text
            .size(size)
            .font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            })
    }

    text.into()
}

fn render_shortcut<'a, T: 'a>(shortcut: &PhysicalShortcut) -> Element<'a, T> {
    let mut result = vec![];

    let (
        key_name,
        alt_modifier_text,
        meta_modifier_text,
        control_modifier_text,
        shift_modifier_text
    ) = shortcut_to_text(shortcut);

    fn apply_modifier<'result, 'element, T: 'element>(
        result: &'result mut Vec<Element<'element, T>>,
        modifier: Option<Element<'element, T>>
    ) {
        if let Some(modifier) = modifier {
            let modifier: Element<_> = container(modifier)
                .themed(ContainerStyle::ActionShortcutModifier);

            let modifier: Element<_> = container(modifier)
                .themed(ContainerStyle::ActionShortcutModifiersInit);

            result.push(modifier);
        }
    }

    apply_modifier(&mut result, meta_modifier_text);
    apply_modifier(&mut result, control_modifier_text);
    apply_modifier(&mut result, shift_modifier_text);
    apply_modifier(&mut result, alt_modifier_text);

    let key_name: Element<_> = container(key_name)
        .themed(ContainerStyle::ActionShortcutModifier);

    result.push(key_name);

    row(result)
        .themed(RowStyle::ActionShortcut)
}

fn render_children_string<'a>(
    content: &[ComponentWidgetWrapper],
    context: ComponentRenderContext
) -> Element<'a, ComponentWidgetEvent> {
    let text_part = content
        .into_iter()
        .map(|child| {
            let (widget, _) = &*child.get();

            let ComponentWidget::TextPart { value } = widget else {
                panic!("unexpected widget kind {:?}", widget)
            };

            value.clone()
        })
        .join("");

    render_text_part(&text_part, context)
}


fn render_children<'a>(
    content: &[ComponentWidgetWrapper],
    context: ComponentRenderContext
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    content
        .into_iter()
        .map(|child| child.render_widget(context.clone()))
        .collect()
}

fn render_child_by_type<'a>(
    content: &[ComponentWidgetWrapper],
    predicate: impl Fn(&ComponentWidget) -> bool,
    context: ComponentRenderContext
) -> anyhow::Result<Element<'a, ComponentWidgetEvent>> {
    let vec: Vec<_> = content
        .into_iter()
        .filter(|child| {
            let (widget, _) = &*child.get();
            predicate(widget)
        })
        .collect();

    match vec[..] {
        [] => Err(anyhow::anyhow!("no child matching predicate found")),
        [single] => Ok(single.render_widget(context)),
        [_, _, ..] => Err(anyhow::anyhow!("more than 1 child matching predicate found")),
    }
}

fn render_children_by_type<'a>(
    content: &[ComponentWidgetWrapper], predicate: impl Fn(&ComponentWidget) -> bool,
    context: ComponentRenderContext
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    content
        .into_iter()
        .filter(|child| {
            let (widget, _) = &*child.get();
            predicate(widget)
        })
        .map(|child| child.render_widget(context.clone()))
        .collect()
}


#[derive(Clone, Debug)]
pub enum ComponentWidgetEvent {
    LinkClick {
        widget_id: UiWidgetId,
        href: String
    },
    TagClick {
        widget_id: UiWidgetId,
    },
    ActionClick {
        widget_id: UiWidgetId,
    },
    RunAction {
        widget_id: UiWidgetId
    },
    ToggleDatePicker {
        widget_id: UiWidgetId,
    },
    OnChangeTextField {
        widget_id: UiWidgetId,
        value: String
    },
    OnChangePasswordField {
        widget_id: UiWidgetId,
        value: String
    },
    SubmitDatePicker {
        widget_id: UiWidgetId,
        value: String
    },
    CancelDatePicker {
        widget_id: UiWidgetId,
    },
    ToggleCheckbox {
        widget_id: UiWidgetId,
        value: bool
    },
    SelectPickList {
        widget_id: UiWidgetId,
        value: String
    },
    ToggleActionPanel {
        widget_id: UiWidgetId,
    },
    ListItemClick {
        widget_id: UiWidgetId,
    },
    GridItemClick {
        widget_id: UiWidgetId,
    },
    PreviousView,
}

impl ComponentWidgetEvent {
    pub fn handle(self, _plugin_id: PluginId, widget: ComponentWidgetWrapper) -> Option<UiViewEvent> {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id: _, href } => {
                Some(UiViewEvent::Open {
                    href
                })
            }
            ComponentWidgetEvent::TagClick { widget_id } => {
                Some(create_metadata_tag_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::RunAction { widget_id } | ComponentWidgetEvent::ActionClick { widget_id } => {
                Some(create_action_on_action_event(widget_id))
            }
            ComponentWidgetEvent::ToggleDatePicker { .. } => {
                let (widget, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::DatePicker { state_value: _, show_picker } = state else {
                    panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                };

                *show_picker = !*show_picker;
                None
            }
            ComponentWidgetEvent::CancelDatePicker { .. } => {
                let (widget, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::DatePicker { state_value: _, show_picker } = state else {
                    panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                };

                *show_picker = false;
                None
            }
            ComponentWidgetEvent::SubmitDatePicker { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::DatePicker { state_value: _, show_picker,  } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *show_picker = false;
                }

                Some(create_date_picker_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::ToggleCheckbox { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::Checkbox { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = !*state_value;
                }

                Some(create_checkbox_on_change_event(widget_id, value))
            }
            ComponentWidgetEvent::SelectPickList { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::Select { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = Some(value.clone());
                }

                Some(create_select_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangeTextField { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::TextField { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_text_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::OnChangePasswordField { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::PasswordField { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = value.clone();
                }

                Some(create_password_field_on_change_event(widget_id, Some(value)))
            }
            ComponentWidgetEvent::ToggleActionPanel { .. } => {
                Some(UiViewEvent::AppEvent {
                    event: AppMsg::ToggleActionPanel { keyboard: false }
                })
            }
            ComponentWidgetEvent::ListItemClick { widget_id } => {
                Some(create_list_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::GridItemClick { widget_id } => {
                Some(create_grid_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::PreviousView => {
                panic!("handle event on PreviousView event is not supposed to be called")
            }
        }
    }

    pub fn widget_id(&self) -> UiWidgetId {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ActionClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::RunAction { widget_id, .. } => widget_id,
            ComponentWidgetEvent::TagClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SubmitDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::CancelDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleCheckbox { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SelectPickList { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangeTextField { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangePasswordField { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleActionPanel { widget_id } => widget_id,
            ComponentWidgetEvent::ListItemClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::GridItemClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::PreviousView => panic!("widget_id on PreviousView event is not supposed to be called"),
        }.to_owned()
    }
}

fn parse_optional_string(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<String>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_string().ok_or(anyhow::anyhow!("{} has to be a string", name))?.to_owned())),
    }
}

fn parse_string(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<String> {
    parse_optional_string(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_optional_number(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<f64>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_number().ok_or(anyhow::anyhow!("{} has to be a number", name))?.to_owned())),
    }
}

fn parse_number(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<f64> {
    parse_optional_number(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_optional_boolean(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<bool>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_bool().ok_or(anyhow::anyhow!("{} has to be a boolean", name))?.to_owned())),
    }
}
fn parse_boolean(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<bool> {
    parse_optional_boolean(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

pub fn parse_date(value: &str) -> Option<(i32, u32, u32)> {
    let ymd: Vec<_> = value.split("-")
        .collect();

    match ymd[..] {
        [year, month, day] => {
            let year = year.parse::<i32>();
            let month = month.parse::<u32>();
            let day = day.parse::<u32>();

            match (year, month, day) {
                (Ok(year), Ok(month), Ok(day)) => Some((year, month, day)),
                _ => None
            }
        }
        _ => None
    }
}

fn parse_bytes_optional(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_bytes().ok_or(anyhow::anyhow!("{} has to be a byte array", name))?.to_owned())),
    }
}

fn parse_bytes(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Vec<u8>> {
    parse_bytes_optional(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_enum_optional<T: FromStr<Err = strum::ParseError>>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<T>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => {
            let string = value.as_string().ok_or(anyhow::anyhow!("{} has to be a string", name))?.to_owned();
            let enum_value = T::from_str(&string)?;
            Ok(Some(enum_value))
        },
    }
}

fn parse_enum<T: FromStr<Err = strum::ParseError>>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<T> {
    parse_enum_optional(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}


fn parse_object_optional<T: UiPropertyValueToStruct>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<T>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => {
            let value = value.as_object().ok_or(anyhow::anyhow!("{} has to be an object", name))?;

            Ok(Some(value))
        },
    }
}

fn parse_object<T: UiPropertyValueToStruct>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<T> {
    parse_object_optional(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_array_optional<T: UiPropertyValueToStruct>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<Vec<T>>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => {
            let value = value.as_array().ok_or(anyhow::anyhow!("{} has to be an array", name))?;

            Ok(Some(value))
        },
    }
}

fn parse_array<T: UiPropertyValueToStruct>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Vec<T>> {
    parse_array_optional(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_union_optional<T: UiPropertyValueToEnum>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<Option<T>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_union()?)),
    }
}

fn parse_union<T: UiPropertyValueToEnum>(properties: &HashMap<String, UiPropertyValue>, name: &str) -> anyhow::Result<T> {
    parse_union_optional(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn icon_to_bootstrap(icon: &Icons) -> icons::Bootstrap {
    match icon {
        Icons::Airplane => icons::Bootstrap::Airplane,
        Icons::Alarm => icons::Bootstrap::Alarm,
        Icons::AlignCentre => icons::Bootstrap::AlignCenter,
        Icons::AlignLeft => icons::Bootstrap::AlignStart,
        Icons::AlignRight => icons::Bootstrap::AlignEnd,
        // Icons::Anchor => icons::Bootstrap::,
        Icons::ArrowClockwise => icons::Bootstrap::ArrowClockwise,
        Icons::ArrowCounterClockwise => icons::Bootstrap::ArrowCounterclockwise,
        Icons::ArrowDown => icons::Bootstrap::ArrowDown,
        Icons::ArrowLeft => icons::Bootstrap::ArrowLeft,
        Icons::ArrowRight => icons::Bootstrap::ArrowRight,
        Icons::ArrowUp => icons::Bootstrap::ArrowUp,
        Icons::ArrowLeftRight => icons::Bootstrap::ArrowLeftRight,
        Icons::ArrowsContract => icons::Bootstrap::ArrowsAngleContract,
        Icons::ArrowsExpand => icons::Bootstrap::ArrowsAngleExpand,
        Icons::AtSymbol => icons::Bootstrap::At,
        // Icons::BandAid => icons::Bootstrap::Bandaid,
        Icons::Cash => icons::Bootstrap::Cash,
        // Icons::BarChart => icons::Bootstrap::BarChart,
        // Icons::BarCode => icons::Bootstrap::,
        Icons::Battery => icons::Bootstrap::Battery,
        Icons::BatteryCharging => icons::Bootstrap::BatteryCharging,
        // Icons::BatteryDisabled => icons::Bootstrap::,
        Icons::Bell => icons::Bootstrap::Bell,
        Icons::BellDisabled => icons::Bootstrap::BellSlash,
        // Icons::Bike => icons::Bootstrap::Bicycle,
        // Icons::Binoculars => icons::Bootstrap::Binoculars,
        // Icons::Bird => icons::Bootstrap::,
        Icons::Bluetooth => icons::Bootstrap::Bluetooth,
        // Icons::Boat => icons::Bootstrap::,
        Icons::Bold => icons::Bootstrap::TypeBold,
        // Icons::Bolt => icons::Bootstrap::,
        // Icons::BoltDisabled => icons::Bootstrap::,
        Icons::Book => icons::Bootstrap::Book,
        Icons::Bookmark => icons::Bootstrap::Bookmark,
        Icons::Box => icons::Bootstrap::Box,
        // Icons::Brush => icons::Bootstrap::Brush,
        Icons::Bug => icons::Bootstrap::Bug,
        Icons::Building => icons::Bootstrap::Building,
        Icons::BulletPoints => icons::Bootstrap::ListUl,
        Icons::Calculator => icons::Bootstrap::Calculator,
        Icons::Calendar => icons::Bootstrap::Calendar,
        Icons::Camera => icons::Bootstrap::Camera,
        Icons::Car => icons::Bootstrap::CarFront,
        Icons::Cart => icons::Bootstrap::Cart,
        // Icons::Cd => icons::Bootstrap::,
        // Icons::Center => icons::Bootstrap::,
        Icons::Checkmark => icons::Bootstrap::Checktwo,
        // Icons::ChessPiece => icons::Bootstrap::,
        Icons::ChevronDown => icons::Bootstrap::ChevronDown,
        Icons::ChevronLeft => icons::Bootstrap::ChevronLeft,
        Icons::ChevronRight => icons::Bootstrap::ChevronRight,
        Icons::ChevronUp => icons::Bootstrap::ChevronUp,
        Icons::ChevronExpand => icons::Bootstrap::ChevronExpand,
        Icons::Circle => icons::Bootstrap::Circle,
        // Icons::CircleProgress100 => icons::Bootstrap::,
        // Icons::CircleProgress25 => icons::Bootstrap::,
        // Icons::CircleProgress50 => icons::Bootstrap::,
        // Icons::CircleProgress75 => icons::Bootstrap::,
        // Icons::ClearFormatting => icons::Bootstrap::,
        Icons::Clipboard => icons::Bootstrap::Clipboard,
        Icons::Clock => icons::Bootstrap::Clock,
        Icons::Cloud => icons::Bootstrap::Cloud,
        Icons::CloudLightning => icons::Bootstrap::CloudLightning,
        Icons::CloudRain => icons::Bootstrap::CloudRain,
        Icons::CloudSnow => icons::Bootstrap::CloudSnow,
        Icons::CloudSun => icons::Bootstrap::CloudSun,
        Icons::Code => icons::Bootstrap::Code,
        Icons::Gear => icons::Bootstrap::Gear,
        Icons::Coin => icons::Bootstrap::Coin,
        Icons::Command => icons::Bootstrap::Command,
        Icons::Compass => icons::Bootstrap::Compass,
        // Icons::ComputerChip => icons::Bootstrap::,
        // Icons::Contrast => icons::Bootstrap::,
        Icons::CreditCard => icons::Bootstrap::CreditCard,
        Icons::Crop => icons::Bootstrap::Crop,
        // Icons::Crown => icons::Bootstrap::,
        Icons::Document => icons::Bootstrap::FileEarmark,
        Icons::DocumentAdd => icons::Bootstrap::FileEarmarkPlus,
        Icons::DocumentDelete => icons::Bootstrap::FileEarmarkX,
        Icons::Dot => icons::Bootstrap::Dot,
        Icons::Download => icons::Bootstrap::Download,
        // Icons::Duplicate => icons::Bootstrap::,
        Icons::Eject => icons::Bootstrap::Eject,
        Icons::ThreeDots => icons::Bootstrap::ThreeDots,
        Icons::Envelope => icons::Bootstrap::Envelope,
        Icons::Eraser => icons::Bootstrap::Eraser,
        Icons::ExclamationMark => icons::Bootstrap::ExclamationLg,
        Icons::Eye => icons::Bootstrap::Eye,
        Icons::EyeDisabled => icons::Bootstrap::EyeSlash,
        Icons::EyeDropper => icons::Bootstrap::Eyedropper,
        Icons::Female => icons::Bootstrap::GenderFemale,
        Icons::Film => icons::Bootstrap::Film,
        Icons::Filter => icons::Bootstrap::Filter,
        Icons::Fingerprint => icons::Bootstrap::Fingerprint,
        Icons::Flag => icons::Bootstrap::Flag,
        Icons::Folder => icons::Bootstrap::Folder,
        Icons::FolderAdd => icons::Bootstrap::FolderPlus,
        Icons::FolderDelete => icons::Bootstrap::FolderMinus,
        Icons::Forward => icons::Bootstrap::Forward,
        Icons::GameController => icons::Bootstrap::Controller,
        Icons::Virus => icons::Bootstrap::Virus,
        Icons::Gift => icons::Bootstrap::Gift,
        Icons::Glasses => icons::Bootstrap::Eyeglasses,
        Icons::Globe => icons::Bootstrap::Globe,
        Icons::Hammer => icons::Bootstrap::Hammer,
        Icons::HardDrive => icons::Bootstrap::DeviceHdd,
        Icons::Headphones => icons::Bootstrap::Headphones,
        Icons::Heart => icons::Bootstrap::Heart,
        // Icons::HeartDisabled => icons::Bootstrap::,
        Icons::Heartbeat => icons::Bootstrap::Activity,
        Icons::Hourglass => icons::Bootstrap::Hourglass,
        Icons::House => icons::Bootstrap::House,
        Icons::Image => icons::Bootstrap::Image,
        Icons::Info => icons::Bootstrap::InfoLg,
        Icons::Italics => icons::Bootstrap::TypeItalic,
        Icons::Key => icons::Bootstrap::Key,
        Icons::Keyboard => icons::Bootstrap::Keyboard,
        Icons::Layers => icons::Bootstrap::Layers,
        // Icons::Leaf => icons::Bootstrap::,
        Icons::LightBulb => icons::Bootstrap::Lightbulb,
        Icons::LightBulbDisabled => icons::Bootstrap::LightbulbOff,
        Icons::Link => icons::Bootstrap::LinkFourfivedeg,
        Icons::List => icons::Bootstrap::List,
        Icons::Lock => icons::Bootstrap::Lock,
        // Icons::LockDisabled => icons::Bootstrap::,
        Icons::LockUnlocked => icons::Bootstrap::Unlock,
        // Icons::Logout => icons::Bootstrap::,
        // Icons::Lowercase => icons::Bootstrap::,
        // Icons::MagnifyingGlass => icons::Bootstrap::,
        Icons::Male => icons::Bootstrap::GenderMale,
        Icons::Map => icons::Bootstrap::Map,
        Icons::Maximize => icons::Bootstrap::Fullscreen,
        Icons::Megaphone => icons::Bootstrap::Megaphone,
        Icons::MemoryModule => icons::Bootstrap::Memory,
        Icons::MemoryStick => icons::Bootstrap::UsbDrive,
        Icons::Message => icons::Bootstrap::Chat,
        Icons::Microphone => icons::Bootstrap::Mic,
        Icons::MicrophoneDisabled => icons::Bootstrap::MicMute,
        Icons::Minimize => icons::Bootstrap::FullscreenExit,
        Icons::Minus => icons::Bootstrap::Dash,
        Icons::Mobile => icons::Bootstrap::Phone,
        // Icons::Monitor => icons::Bootstrap::,
        Icons::Moon => icons::Bootstrap::Moon,
        // Icons::Mountain => icons::Bootstrap::,
        Icons::Mouse => icons::Bootstrap::Mouse,
        Icons::Multiply => icons::Bootstrap::X,
        Icons::Music => icons::Bootstrap::MusicNoteBeamed,
        Icons::Network => icons::Bootstrap::BroadcastPin,
        Icons::Paperclip => icons::Bootstrap::Paperclip,
        Icons::Paragraph => icons::Bootstrap::TextParagraph,
        Icons::Pause => icons::Bootstrap::Pause,
        Icons::Pencil => icons::Bootstrap::Pencil,
        Icons::Person => icons::Bootstrap::Person,
        Icons::PersonAdd => icons::Bootstrap::PersonAdd,
        Icons::PersonRemove => icons::Bootstrap::PersonDash,
        Icons::Phone => icons::Bootstrap::Telephone,
        // Icons::PhoneRinging => icons::Bootstrap::,
        Icons::PieChart => icons::Bootstrap::PieChart,
        Icons::Capsule => icons::Bootstrap::Capsule,
        // Icons::Pin => icons::Bootstrap::,
        // Icons::PinDisabled => icons::Bootstrap::,
        Icons::Play => icons::Bootstrap::Play,
        Icons::Plug => icons::Bootstrap::Plug,
        Icons::Plus => icons::Bootstrap::Plus,
        // Icons::PlusMinusDivideMultiply => icons::Bootstrap::,
        Icons::Power => icons::Bootstrap::Power,
        Icons::Printer => icons::Bootstrap::Printer,
        Icons::QuestionMark => icons::Bootstrap::QuestionLg,
        Icons::Quotes => icons::Bootstrap::Quote,
        Icons::Receipt => icons::Bootstrap::Receipt,
        Icons::Repeat => icons::Bootstrap::Repeat,
        Icons::Reply => icons::Bootstrap::Reply,
        Icons::Rewind => icons::Bootstrap::Rewind,
        Icons::Rocket => icons::Bootstrap::Rocket,
        // Icons::Ruler => icons::Bootstrap::,
        Icons::Shield => icons::Bootstrap::Shield,
        Icons::Shuffle => icons::Bootstrap::Shuffle,
        Icons::Snippets => icons::Bootstrap::BodyText,
        Icons::Snowflake => icons::Bootstrap::Snow,
        // Icons::VolumeHigh => icons::Bootstrap::VolumeUp,
        // Icons::VolumeLow => icons::Bootstrap::VolumeDown,
        // Icons::VolumeOff => icons::Bootstrap::VolumeOff,
        // Icons::VolumeOn => icons::Bootstrap::,
        Icons::Star => icons::Bootstrap::Star,
        // Icons::StarDisabled => icons::Bootstrap::,
        Icons::Stop => icons::Bootstrap::Stop,
        Icons::Stopwatch => icons::Bootstrap::Stopwatch,
        Icons::StrikeThrough => icons::Bootstrap::TypeStrikethrough,
        Icons::Sun => icons::Bootstrap::SunFill, // TODO why Sun isn't in iced_aw?
        Icons::Scissors => icons::Bootstrap::Scissors,
        // Icons::Syringe => icons::Bootstrap::,
        Icons::Tag => icons::Bootstrap::Tag,
        Icons::Thermometer => icons::Bootstrap::Thermometer,
        Icons::Terminal => icons::Bootstrap::Terminal,
        Icons::Text => icons::Bootstrap::Fonts,
        Icons::TextCursor => icons::Bootstrap::CursorText,
        // Icons::TextSelection => icons::Bootstrap::,
        // Icons::Torch => icons::Bootstrap::,
        // Icons::Train => icons::Bootstrap::,
        Icons::Trash => icons::Bootstrap::Trash,
        Icons::Tree => icons::Bootstrap::Tree,
        Icons::Trophy => icons::Bootstrap::Trophy,
        Icons::People => icons::Bootstrap::People,
        Icons::Umbrella => icons::Bootstrap::Umbrella,
        Icons::Underline => icons::Bootstrap::TypeUnderline,
        Icons::Upload => icons::Bootstrap::Upload,
        // Icons::Uppercase => icons::Bootstrap::,
        Icons::Wallet => icons::Bootstrap::Wallet,
        Icons::Wand => icons::Bootstrap::Magic,
        // Icons::Warning => icons::Bootstrap::,
        // Icons::Weights => icons::Bootstrap::,
        Icons::Wifi => icons::Bootstrap::Wifi,
        Icons::WifiDisabled => icons::Bootstrap::WifiOff,
        Icons::Window => icons::Bootstrap::Window,
        Icons::Tools => icons::Bootstrap::Tools,
        Icons::Watch => icons::Bootstrap::Watch,
        Icons::XMark => icons::Bootstrap::XLg,
        Icons::Indent => icons::Bootstrap::Indent,
        Icons::Unindent => icons::Bootstrap::Unindent,
    }
}

impl UiPropertyValueToEnum for ListItemIcon {
    fn convert(value: &UiPropertyValue) -> anyhow::Result<ListItemIcon> {
        match value {
            UiPropertyValue::String(value) => Ok(ListItemIcon::_1(Icons::from_str(value)?)),
            UiPropertyValue::Bytes(value) => Ok(ListItemIcon::_0(value.clone())),
            UiPropertyValue::Number(_) => Err(anyhow!("unexpected type number")),
            UiPropertyValue::Bool(_) => Err(anyhow!("unexpected type bool")),
            UiPropertyValue::Object(_) => Err(anyhow!("unexpected type object")),
            UiPropertyValue::Array(_) => Err(anyhow!("unexpected type undefined")),
            UiPropertyValue::Undefined => Err(anyhow!("unexpected type undefined"))
        }
    }
}

impl UiPropertyValueToEnum for EmptyViewImage {
    fn convert(value: &UiPropertyValue) -> anyhow::Result<EmptyViewImage> {
        match value {
            UiPropertyValue::String(value) => Ok(EmptyViewImage::_1(Icons::from_str(value)?)),
            UiPropertyValue::Bytes(value) => Ok(EmptyViewImage::_0(value.clone())),
            UiPropertyValue::Number(_) => Err(anyhow!("unexpected type number")),
            UiPropertyValue::Bool(_) => Err(anyhow!("unexpected type bool")),
            UiPropertyValue::Object(_) => Err(anyhow!("unexpected type object")),
            UiPropertyValue::Array(_) => Err(anyhow!("unexpected type undefined")),
            UiPropertyValue::Undefined => Err(anyhow!("unexpected type undefined"))
        }
    }
}

impl UiPropertyValueToEnum for ImageSource {
    fn convert(value: &UiPropertyValue) -> anyhow::Result<ImageSource> {
        match value {
            UiPropertyValue::String(value) => Ok(ImageSource::_1(Icons::from_str(value)?)),
            UiPropertyValue::Bytes(value) => Ok(ImageSource::_0(value.clone())),
            UiPropertyValue::Number(_) => Err(anyhow!("unexpected type number")),
            UiPropertyValue::Bool(_) => Err(anyhow!("unexpected type bool")),
            UiPropertyValue::Object(_) => Err(anyhow!("unexpected type object")),
            UiPropertyValue::Array(_) => Err(anyhow!("unexpected type undefined")),
            UiPropertyValue::Undefined => Err(anyhow!("unexpected type undefined"))
        }
    }
}

impl UiPropertyValueToEnum for IconAccessoryIcon {
    fn convert(value: &UiPropertyValue) -> anyhow::Result<IconAccessoryIcon> {
        match value {
            UiPropertyValue::String(value) => Ok(IconAccessoryIcon::_1(Icons::from_str(value)?)),
            UiPropertyValue::Bytes(value) => Ok(IconAccessoryIcon::_0(value.clone())),
            UiPropertyValue::Number(_) => Err(anyhow!("unexpected type number")),
            UiPropertyValue::Bool(_) => Err(anyhow!("unexpected type bool")),
            UiPropertyValue::Object(_) => Err(anyhow!("unexpected type object")),
            UiPropertyValue::Array(_) => Err(anyhow!("unexpected type undefined")),
            UiPropertyValue::Undefined => Err(anyhow!("unexpected type undefined")),
        }
    }
}

impl UiPropertyValueToEnum for TextAccessoryIcon {
    fn convert(value: &UiPropertyValue) -> anyhow::Result<TextAccessoryIcon> {
        match value {
            UiPropertyValue::String(value) => Ok(TextAccessoryIcon::_1(Icons::from_str(value)?)),
            UiPropertyValue::Bytes(value) => Ok(TextAccessoryIcon::_0(value.clone())),
            UiPropertyValue::Number(_) => Err(anyhow!("unexpected type number")),
            UiPropertyValue::Bool(_) => Err(anyhow!("unexpected type bool")),
            UiPropertyValue::Object(_) => Err(anyhow!("unexpected type object")),
            UiPropertyValue::Array(_) => Err(anyhow!("unexpected type undefined")),
            UiPropertyValue::Undefined => Err(anyhow!("unexpected type undefined"))
        }
    }
}
