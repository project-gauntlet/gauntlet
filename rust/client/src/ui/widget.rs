use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::{Font, Length, Padding};
use iced::font::Weight;
use iced::widget::{button, checkbox, column, container, horizontal_rule, pick_list, row, scrollable, text, text_input, tooltip, vertical_rule};
use iced::widget::tooltip::Position;
use iced_aw::date_picker::Date;
use iced_aw::helpers::date_picker;
use zbus::SignalContext;

use common::model::PluginId;

use crate::model::{NativeUiPropertyValue, NativeUiWidgetId};
use crate::ui::theme::{ButtonStyle, ContainerStyle, Element};

#[derive(Clone, Debug)]
pub struct ComponentWidgetWrapper {
    id: NativeUiWidgetId,
    inner: Arc<RwLock<(ComponentWidget, ComponentWidgetState)>>,
}

include!(concat!(env!("OUT_DIR"), "/components.rs"));

#[derive(Clone, Debug)]
pub enum ComponentWidgetState {
    TextField {
        value: String
    },
    PasswordField {
        value: String
    },
    Checkbox {
        value: bool
    },
    DatePicker {
        value: Date,
        show_picker: bool,
    },
    Select {
        value: Option<String>
    },
    None
}

impl ComponentWidgetState {
    fn create(component_widget: &ComponentWidget) -> Self {
        match component_widget {
            ComponentWidget::TextField { .. } => ComponentWidgetState::TextField {
                value: Default::default()
            },
            ComponentWidget::PasswordField { .. } => ComponentWidgetState::PasswordField {
                value: Default::default()
            },
            ComponentWidget::Checkbox { .. } => ComponentWidgetState::Checkbox {
                value: Default::default()
            },
            ComponentWidget::DatePicker { value } => {
                let value = value
                    .clone()
                    .map(|value| parse_date(&value))
                    .flatten()
                    .map(|(year, month, day)| Date::from_ymd(year, month, day))
                    .unwrap_or(Date::today());

                ComponentWidgetState::DatePicker {
                    value,
                    show_picker: false,
                }
            },
            ComponentWidget::Select { .. } => ComponentWidgetState::Select {
                value: Default::default()
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
                let metadata_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Metadata { .. }), ComponentRenderContext::None)
                    .unwrap();

                let metadata_element = container(metadata_element)
                    .width(Length::FillPortion(2))
                    .padding(Padding::new(5.0))
                    .into();

                let content_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::None)
                    .unwrap();

                let content_element = container(content_element)
                    .width(Length::FillPortion(3))
                    .padding(Padding::new(5.0))
                    .into();

                let separator = vertical_rule(1)
                    .into();

                let content: Element<_> = row(vec![content_element, separator, metadata_element])
                    .into();

                container(content)
                    .width(Length::Fill)
                    .padding(Padding::new(10.0))
                    .into()
            }
            ComponentWidget::Root { children } => {
                row(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::TextField => {
                text_input("", "")
                    .into()
            }
            ComponentWidget::PasswordField => {
                text_input("", "")
                    .password()
                    .into()
            }
            ComponentWidget::Checkbox => {
                let ComponentWidgetState::Checkbox { value } = state else {
                    panic!("unexpected state kind")
                };

                checkbox("checkbox label", value.to_owned(), move|value| ComponentWidgetEvent::ToggleCheckbox { widget_id, value })
                    .into()
            }
            ComponentWidget::DatePicker { value: _ } => {
                let ComponentWidgetState::DatePicker { value, show_picker } = state else {
                    panic!("unexpected state kind")
                };

                let button = button(text("Set Date"))
                    .on_press(ComponentWidgetEvent::ToggleDatePicker { widget_id });

                date_picker(
                    show_picker.to_owned(),
                    value.to_owned(),
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
            ComponentWidget::Select => {
                let ComponentWidgetState::Select { value } = state else {
                    panic!("unexpected state kind")
                };

                pick_list(
                    vec![],
                    value.to_owned(),
                    move |value| ComponentWidgetEvent::SelectPickList { widget_id, value }
                ).into()
            }
            ComponentWidget::Separator => {
                horizontal_rule(1)
                    .into()
            }
            ComponentWidget::Form { children } => {
                column(render_children(children, ComponentRenderContext::None))
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
    ToggleDatePicker {
        widget_id: NativeUiWidgetId,
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
            ComponentWidgetEvent::ToggleDatePicker { .. } => {
                let (_, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::DatePicker { show_picker, .. } = state else {
                    panic!("unexpected state kind")
                };

                *show_picker = !*show_picker
            }
            ComponentWidgetEvent::CancelDatePicker { .. } => {
                let (_, ref mut state) = &mut *widget.get_mut();
                let ComponentWidgetState::DatePicker { show_picker, .. } = state else {
                    panic!("unexpected state kind")
                };

                *show_picker = false
            }
            ComponentWidgetEvent::SubmitDatePicker { widget_id, value } => {
                {
                    let (_, ref mut state) = &mut *widget.get_mut();
                    let ComponentWidgetState::DatePicker { show_picker, .. } = state else {
                        panic!("unexpected state kind")
                    };

                    *show_picker = false;
                }

                send_date_picker_on_change_dbus_event(signal_context, plugin_id, widget_id, Some(value)).await
            }
            ComponentWidgetEvent::ToggleCheckbox { .. } => {}
            ComponentWidgetEvent::SelectPickList { .. } => {}
        }
    }

    pub fn widget_id(&self) -> NativeUiWidgetId {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::TagClick { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SubmitDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::CancelDatePicker { widget_id, .. } => widget_id,
            ComponentWidgetEvent::ToggleCheckbox { widget_id, .. } => widget_id,
            ComponentWidgetEvent::SelectPickList { widget_id, .. } => widget_id,
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