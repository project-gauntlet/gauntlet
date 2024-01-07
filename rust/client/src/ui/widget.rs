use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::{Font, Length, Padding};
use iced::advanced::Widget;
use iced::alignment::Horizontal;
use iced::font::Weight;
use iced::widget::{button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable, text, text_input, tooltip, vertical_rule, vertical_space};
use iced::widget::tooltip::Position;
use iced_aw::date_picker::Date;
use iced_aw::floating_element;
use iced_aw::floating_element::Offset;
use iced_aw::helpers::date_picker;
use zbus::SignalContext;

use common::model::PluginId;

use crate::model::{NativeUiPropertyValue, NativeUiWidgetId};
use crate::ui::theme::{ButtonStyle, ContainerStyle, Element, TextInputStyle};

#[derive(Clone, Debug)]
pub struct ComponentWidgetWrapper {
    id: NativeUiWidgetId,
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
        show_action_panel: bool
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
            _ => ComponentWidgetState::None
        }
    }
}

#[derive(Clone, Copy)]
pub enum ComponentRenderContext {
    None,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl ComponentWidgetWrapper {
    pub fn widget(
        id: NativeUiWidgetId,
        widget_type: impl Into<String>,
        properties: HashMap<String, NativeUiPropertyValue>,
        children: Vec<ComponentWidgetWrapper>
    ) -> anyhow::Result<Self> {
        let widget_type = widget_type.into();
        let widget = create_component_widget(&widget_type, properties, children)?;
        let widget_state = ComponentWidgetState::create(&widget);
        let widget = ComponentWidgetWrapper::new(id, widget, widget_state);

        Ok(widget)
    }

    pub fn root(id: NativeUiWidgetId) -> Self {
        ComponentWidgetWrapper::new(id, ComponentWidget::Root { children: vec![] }, ComponentWidgetState::None)
    }

    fn new(id: NativeUiWidgetId, widget: ComponentWidget, state: ComponentWidgetState) -> Self {
        Self {
            id,
            inner: Arc::new(RwLock::new((widget, state))),
        }
    }

    pub fn find_child_with_id(&self, widget_id: NativeUiWidgetId) -> Option<ComponentWidgetWrapper> {
        // TODO not the most performant solution but works for now?

        if self.id == widget_id {
            return Some(self.clone())
        }

        self.get_children()
            .unwrap_or(vec![])
            .iter()
            .find_map(|child| child.find_child_with_id(widget_id))
            .map(|widget| widget.clone())
    }

    fn get(&self) -> RwLockReadGuard<'_, (ComponentWidget, ComponentWidgetState)> {
        self.inner.read().expect("lock is poisoned")
    }

    fn get_mut(&self) -> RwLockWriteGuard<'_, (ComponentWidget, ComponentWidgetState)> {
        self.inner.write().expect("lock is poisoned")
    }

    pub fn render_widget<'a>(&self, context: ComponentRenderContext) -> Element<'a, ComponentWidgetEvent> {
        let widget_id = self.id;
        let (widget, state) = &*self.get();
        match widget {
            ComponentWidget::TextPart { value } => {
                let size = match context {
                    ComponentRenderContext::None => None,
                    ComponentRenderContext::H1 => Some(34),
                    ComponentRenderContext::H2 => Some(30),
                    ComponentRenderContext::H3 => Some(24),
                    ComponentRenderContext::H4 => Some(20),
                    ComponentRenderContext::H5 => Some(18),
                    ComponentRenderContext::H6 => Some(16),
                };

                let mut text = text(value);

                if let Some(size) = size {
                    text = text
                        .size(size)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Font::DEFAULT
                        })
                }

                text.into()
            }
            ComponentWidget::Action { title } => {
                button(text(title))
                    .on_press(ComponentWidgetEvent::ActionClick { widget_id })
                    .style(ButtonStyle::EntrypointItem)
                    .width(Length::Fill)
                    .into()
            }
            ComponentWidget::ActionPanelSection { children, .. } => {
                column(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::ActionPanel { children, title } => {
                let mut columns = vec![];
                if let Some(title) = title {
                    columns.push(
                        text(title)
                            .font(Font {
                                weight: Weight::Bold,
                                ..Font::DEFAULT
                            })
                            .into()
                    )
                }

                let mut place_separator = false;

                for child in children {
                    let (widget, _) = &*child.get();

                    match widget {
                        ComponentWidget::Action { .. } => {
                            if place_separator {
                                let separator: Element<_> = horizontal_rule(1)
                                    .into();
                                columns.push(separator);

                                place_separator = false;
                            }

                            columns.push(child.render_widget(ComponentRenderContext::None));
                        }
                        ComponentWidget::ActionPanelSection { .. } => {
                            let separator: Element<_> = horizontal_rule(1)
                                .into();
                            columns.push(separator);

                            columns.push(child.render_widget(ComponentRenderContext::None));

                            place_separator = true;
                        }
                        _ => {
                            panic!("unexpected widget kind {:?}", widget)
                        }
                    };

                }

                let actions: Element<_> = column(columns)
                    .into();

                let actions: Element<_> = scrollable(actions)
                    .width(Length::Fill)
                    .into();

                container(actions)
                    .padding(Padding::new(10.0))
                    .style(ContainerStyle::Background)
                    .height(Length::Fixed(200.0))
                    .width(Length::Fixed(300.0))
                    .into()
            }
            ComponentWidget::MetadataTagItem { children } => {
                let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
                    .into();

                let tag: Element<_> = button(content)
                    .on_press(ComponentWidgetEvent::TagClick { widget_id })
                    .into();

                container(tag)
                    .padding(Padding::new(5.0))
                    .into()
            }
            ComponentWidget::MetadataTagList { label,  children } => {
                let value = row(render_children(children, ComponentRenderContext::None))
                    .into();

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataLink { label, children, href } => {
                let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
                    .into();

                let link: Element<_> = button(content)
                    .style(ButtonStyle::Link)
                    .on_press(ComponentWidgetEvent::LinkClick { widget_id, href: href.to_owned() })
                    .into();

                let content: Element<_> = if href.is_empty() {
                    link
                } else {
                    tooltip(link, href, Position::Top)
                        .style(ContainerStyle::Background)
                        .into()
                };

                render_metadata_item(label, content)
                    .into()
            }
            ComponentWidget::MetadataValue { label, children} => {
                let value = row(render_children(children, ComponentRenderContext::None))
                    .into();

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataIcon { label, icon} => {
                let value = text(icon).into();

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataSeparator => {
                let separator: Element<_> = horizontal_rule(1)
                    .into();

                container(separator)
                    .width(Length::Fill)
                    .padding(Padding::from([10.0, 0.0]))
                    .into()
            }
            ComponentWidget::Metadata { children } => {
                let metadata: Element<_> = column(render_children(children, ComponentRenderContext::None))
                    .into();

                scrollable(metadata)
                    .width(Length::Fill)
                    .into()
            }
            ComponentWidget::Paragraph { children } => {
                let paragraph: Element<_> = row(render_children(children, context))
                    .into();

                container(paragraph)
                    .width(Length::Fill)
                    .padding(Padding::new(5.0))
                    .into()
            }
            ComponentWidget::Link { children, href } => {
                let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
                    .into();

                let content: Element<_> = button(content)
                    .style(ButtonStyle::Link)
                    .on_press(ComponentWidgetEvent::LinkClick { widget_id, href: href.to_owned() })
                    .into();

                if href.is_empty() {
                    content
                } else {
                    tooltip(content, href, Position::Top)
                        .style(ContainerStyle::Background)
                        .into()
                }
            }
            ComponentWidget::Image => {
                text("Image").into()
            }
            ComponentWidget::H1 { children } => {
                row(render_children(children, ComponentRenderContext::H1))
                    .into()
            }
            ComponentWidget::H2 { children } => {
                row(render_children(children, ComponentRenderContext::H2))
                    .into()
            }
            ComponentWidget::H3 { children } => {
                row(render_children(children, ComponentRenderContext::H3))
                    .into()
            }
            ComponentWidget::H4 { children } => {
                row(render_children(children, ComponentRenderContext::H4))
                    .into()
            }
            ComponentWidget::H5 { children } => {
                row(render_children(children, ComponentRenderContext::H5))
                    .into()
            }
            ComponentWidget::H6 { children } => {
                row(render_children(children, ComponentRenderContext::H6))
                    .into()
            }
            ComponentWidget::HorizontalBreak => {
                let separator: Element<_> = horizontal_rule(1).into();

                container(separator)
                    .width(Length::Fill)
                    .padding(Padding::from([10.0, 0.0]))
                    .into()
            }
            ComponentWidget::CodeBlock { children } => {
                let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
                    .padding(Padding::from([3.0, 5.0]))
                    .into();

                container(content)
                    .width(Length::Fill)
                    .style(ContainerStyle::Code)
                    .into()
            }
            // ComponentWidget::Code { children } => {
            //     let content: Element<_> = row(render_children(children, ComponentRenderContext::None))
            //         .padding(Padding::from([3.0, 5.0]))
            //         .into();
            //
            //     container(content)
            //         .style(ContainerStyle::Code)
            //         .into()
            // }
            ComponentWidget::Content { children } => {
                let content: Element<_> = column(render_children(children, ComponentRenderContext::None))
                    .into();

                scrollable(content)
                    .width(Length::Fill)
                    .into()
            }
            ComponentWidget::Detail { children } => {
                let ComponentWidgetState::Detail { show_action_panel } = *state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let metadata_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Metadata { .. }), ComponentRenderContext::None)
                    .unwrap();

                let metadata_element = container(metadata_element)
                    .width(Length::FillPortion(2))
                    .padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
                    .into();

                let content_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::None)
                    .unwrap();

                let content_element = container(content_element)
                    .width(Length::FillPortion(3))
                    .padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
                    .into();

                let separator = vertical_rule(1)
                    .into();

                let content: Element<_> = row(vec![content_element, separator, metadata_element])
                    .into();

                let space = horizontal_space(Length::FillPortion(3))
                    .into();

                let action_panel_toggle: Element<_> = button(text("Alt + K"))
                    .padding(Padding::from([0.0, 5.0]))
                    .style(ButtonStyle::Secondary)
                    .on_press(ComponentWidgetEvent::ToggleActionPanel { widget_id })
                    .into();

                let bottom_panel: Element<_> = row(vec![space, action_panel_toggle])
                    .into();

                let bottom_panel: Element<_> = container(bottom_panel)
                    .padding(Padding::new(5.0))
                    .width(Length::Fill)
                    .into();

                let separator = horizontal_rule(1)
                    .into();

                let content: Element<_> = container(content)
                    .width(Length::Fill)
                    .height(Length::Fill) // TODO remove after https://github.com/iced-rs/iced/issues/2186 is resolved
                    .padding(Padding::from([10.0, 10.0, 0.0, 10.0]))
                    .into();

                let content: Element<_> = column(vec![content, separator, bottom_panel])
                    .into();

                let action_panel_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::ActionPanel { .. }), ComponentRenderContext::None)
                    .unwrap();

                floating_element(content, action_panel_element)
                    .offset(Offset::from([5.0, 35.0]))
                    .hide(!show_action_panel)
                    .into()
            }
            ComponentWidget::Root { children } => {
                row(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::TextField { .. } => {
                let ComponentWidgetState::TextField { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                text_input("", state_value)
                    .on_input(move |value| ComponentWidgetEvent::OnChangeTextField { widget_id, value })
                    .style(TextInputStyle::Form)
                    .into()
            }
            ComponentWidget::PasswordField { .. } => {
                let ComponentWidgetState::PasswordField { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                text_input("", state_value)
                    .password()
                    .on_input(move |value| ComponentWidgetEvent::OnChangePasswordField { widget_id, value })
                    .style(TextInputStyle::Form)
                    .into()
            }
            ComponentWidget::Checkbox { .. } => {
                let ComponentWidgetState::Checkbox { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                checkbox("", state_value.to_owned(), move|value| ComponentWidgetEvent::ToggleCheckbox { widget_id, value })
                    .into()
            }
            ComponentWidget::DatePicker { .. } => {
                let ComponentWidgetState::DatePicker { state_value, show_picker } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let button = button(text(state_value.to_string()))
                    .on_press(ComponentWidgetEvent::ToggleDatePicker { widget_id });

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
                ).into()
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
                ).into()
            }
            ComponentWidget::Separator => {
                horizontal_rule(1)
                    .into()
            }
            ComponentWidget::Form { children } => {
                let items: Vec<Element<_>> = children.iter()
                    .map(|child| {
                        let (widget, _) = &*child.get();

                        match widget {
                            ComponentWidget::Separator => child.render_widget(context),
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
                                        horizontal_space(Length::FillPortion(2))
                                            .into()
                                    }
                                    Some(label) => {
                                        let label: Element<_> = text(label)
                                            .horizontal_alignment(Horizontal::Right)
                                            .width(Length::Fill)
                                            .into();

                                        container(label)
                                            .width(Length::FillPortion(2))
                                            .padding(Padding::from([5.0, 10.0]))
                                            .into()
                                    }
                                };

                                let form_input = container(child.render_widget(context))
                                    .width(Length::FillPortion(3))
                                    .into();

                                let after = horizontal_space(Length::FillPortion(2))
                                    .into();

                                let content = vec![
                                    before_or_label,
                                    form_input,
                                    after,
                                ];

                                row(content)
                                    .padding(Padding::new(10.0))
                                    .into()
                            }
                        }
                    })
                    .collect();

                let content: Element<_> = column(items)
                    .into();

                let scrollable_content: Element<_> = scrollable(content)
                    .width(Length::Fill)
                    .into();

                container(scrollable_content)
                    .padding(Padding::new(10.0))
                    .into()
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
    let bold_font = Font {
        weight: Weight::Bold,
        ..Font::DEFAULT
    };

    let label: Element<_> = text(label)
        .font(bold_font)
        .into();

    let value = container(value)
        .padding(Padding::new(5.0))
        .into();

    column(vec![label, value])
        .into()
}

fn render_children<'a>(
    content: &[ComponentWidgetWrapper],
    context: ComponentRenderContext
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    return content
        .into_iter()
        .map(|child| child.render_widget(context))
        .collect();
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
    return content
        .into_iter()
        .filter(|child| {
            let (widget, _) = &*child.get();
            predicate(widget)
        })
        .map(|child| child.render_widget(context))
        .collect();
}


#[derive(Clone, Debug)]
pub enum ComponentWidgetEvent {
    LinkClick {
        widget_id: NativeUiWidgetId,
        href: String
    },
    TagClick {
        widget_id: NativeUiWidgetId,
    },
    ActionClick {
        widget_id: NativeUiWidgetId,
    },
    ToggleDatePicker {
        widget_id: NativeUiWidgetId,
    },
    OnChangeTextField {
        widget_id: NativeUiWidgetId,
        value: String
    },
    OnChangePasswordField {
        widget_id: NativeUiWidgetId,
        value: String
    },
    SubmitDatePicker {
        widget_id: NativeUiWidgetId,
        value: String
    },
    CancelDatePicker {
        widget_id: NativeUiWidgetId,
    },
    ToggleCheckbox {
        widget_id: NativeUiWidgetId,
        value: bool
    },
    SelectPickList {
        widget_id: NativeUiWidgetId,
        value: String
    },
    ToggleActionPanel {
        widget_id: NativeUiWidgetId,
    },
}

impl ComponentWidgetEvent {
    pub async fn handle<'a>(self, signal_context: &SignalContext<'_>, plugin_id: PluginId, widget: ComponentWidgetWrapper) {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id: _, href } => {
                todo!("href {:?}", href)
            }
            ComponentWidgetEvent::TagClick { widget_id } => {
                send_metadata_tag_item_on_click_dbus_event(signal_context, plugin_id, widget_id).await
            }
            ComponentWidgetEvent::ActionClick { widget_id } => {
                send_action_on_action_dbus_event(signal_context, plugin_id, widget_id).await
            }
            ComponentWidgetEvent::ToggleDatePicker { .. } => {
                let (widget, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::DatePicker { state_value: _, show_picker } = state else {
                    panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                };

                *show_picker = !*show_picker
            }
            ComponentWidgetEvent::CancelDatePicker { .. } => {
                let (widget, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::DatePicker { state_value: _, show_picker } = state else {
                    panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                };

                *show_picker = false
            }
            ComponentWidgetEvent::SubmitDatePicker { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::DatePicker { state_value, show_picker,  } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *show_picker = false;
                }

                send_date_picker_on_change_dbus_event(signal_context, plugin_id, widget_id, Some(value)).await
            }
            ComponentWidgetEvent::ToggleCheckbox { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::Checkbox { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = !*state_value;
                }

                send_checkbox_on_change_dbus_event(signal_context, plugin_id, widget_id, value).await
            }
            ComponentWidgetEvent::SelectPickList { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::Select { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = Some(value.clone());
                }

                send_select_on_change_dbus_event(signal_context, plugin_id, widget_id, Some(value)).await
            }
            ComponentWidgetEvent::OnChangeTextField { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::TextField { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = value.clone();
                }

                send_text_field_on_change_dbus_event(signal_context, plugin_id, widget_id, Some(value)).await
            }
            ComponentWidgetEvent::OnChangePasswordField { widget_id, value } => {
                {
                    let (widget, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::PasswordField { state_value } = state else {
                        panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                    };

                    *state_value = value.clone();
                }

                send_password_field_on_change_dbus_event(signal_context, plugin_id, widget_id, Some(value)).await
            }
            ComponentWidgetEvent::ToggleActionPanel { .. } => {
                let (widget, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::Detail { show_action_panel } = state else {
                    panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                };

                *show_action_panel = !*show_action_panel;
            }
        }
    }

    pub fn widget_id(&self) -> NativeUiWidgetId {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ActionClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::TagClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SubmitDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::CancelDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleCheckbox { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SelectPickList { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangeTextField { widget_id, .. } => widget_id,
            ComponentWidgetEvent::OnChangePasswordField { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleActionPanel { widget_id } => widget_id,
        }.to_owned()
    }
}

fn parse_optional_string(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<String>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_string().ok_or(anyhow::anyhow!("{} has to be a string", name))?.to_owned())),
    }
}

fn parse_string(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<String> {
    parse_optional_string(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_optional_number(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<f64>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_number().ok_or(anyhow::anyhow!("{} has to be a number", name))?.to_owned())),
    }
}

fn parse_number(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<f64> {
    parse_optional_number(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}

fn parse_optional_boolean(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<bool>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_bool().ok_or(anyhow::anyhow!("{} has to be a boolean", name))?.to_owned())),
    }
}
fn parse_boolean(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<bool> {
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