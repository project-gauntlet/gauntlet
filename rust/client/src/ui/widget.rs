use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::{Font, Length, Padding};
use iced::alignment::Horizontal;
use iced::font::Weight;
use iced::widget::{button, checkbox, column, container, horizontal_rule, horizontal_space, image, pick_list, row, scrollable, Space, text, text_input, tooltip, vertical_rule, vertical_space};
use iced::widget::image::Handle;
use iced::widget::tooltip::Position;
use iced_aw::{floating_element, GridRow};
use iced_aw::core::icons;
use iced_aw::date_picker::Date;
use iced_aw::floating_element::Offset;
use iced_aw::helpers::{date_picker, grid, grid_row, wrap_horizontal};
use itertools::Itertools;

use common::model::PluginId;

use crate::model::{NativeUiPropertyValue, NativeUiViewEvent, NativeUiWidgetId};
use crate::ui::theme::{ButtonStyle, ContainerStyle, Element, TextInputStyle, TextStyle};

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
    Form {
        show_action_panel: bool
    },
    List {
        show_action_panel: bool
    },
    Grid {
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
    List {
        widget_id: NativeUiWidgetId
    },
    Grid {
        widget_id: NativeUiWidgetId
    },
    Root {
        entrypoint_name: String,
    }
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
            ComponentWidget::TextPart { value } => render_text_part(value, context),
            ComponentWidget::Action { title, .. } => {
                button(text(title))
                    .on_press(ComponentWidgetEvent::ActionClick { widget_id })
                    .style(ButtonStyle::GauntletButton)
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
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let tag: Element<_> = button(content)
                    .on_press(ComponentWidgetEvent::TagClick { widget_id })
                    .into();

                container(tag)
                    .padding(Padding::new(5.0))
                    .into()
            }
            ComponentWidget::MetadataTagList { label,  children } => {
                let value = wrap_horizontal(render_children(children, ComponentRenderContext::None))
                    .into();

                render_metadata_item(label, value)
                    .into()
            }
            ComponentWidget::MetadataLink { label, children, href } => {
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let link: Element<_> = button(content)
                    .style(ButtonStyle::Link)
                    .on_press(ComponentWidgetEvent::LinkClick { widget_id, href: href.to_owned() })
                    .into();

                let content: Element<_> = if href.is_empty() {
                    link
                } else {
                    let href: Element<_> = text(href)
                        .into();

                    tooltip(link, href, Position::Top)
                        .style(ContainerStyle::Background)
                        .into()
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
                let paragraph: Element<_> = render_children_string(children, context);

                container(paragraph)
                    .width(Length::Fill)
                    .padding(Padding::new(5.0))
                    .into()
            }
            ComponentWidget::Link { children, href } => {
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let content: Element<_> = button(content)
                    .style(ButtonStyle::Link)
                    .on_press(ComponentWidgetEvent::LinkClick { widget_id, href: href.to_owned() })
                    .into();

                if href.is_empty() {
                    content
                } else {
                    let href: Element<_> = text(href)
                        .into();

                    tooltip(content, href, Position::Top)
                        .style(ContainerStyle::Background)
                        .into()
                }
            }
            ComponentWidget::Image { source } => {
                image(Handle::from_memory(source.clone())) // FIXME really expensive clone
                    .into()
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
                    .padding(Padding::from([10.0, 0.0]))
                    .into()
            }
            ComponentWidget::CodeBlock { children } => {
                let content: Element<_> = render_children_string(children, ComponentRenderContext::None);

                let content: Element<_> = container(content)
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
                column(render_children(children, ComponentRenderContext::None))
                    .into()
            }
            ComponentWidget::Detail { children } => {
                let ComponentWidgetState::Detail { show_action_panel } = *state else {
                    panic!("unexpected state kind {:?}", state)
                };

                let metadata_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Metadata { .. }), ComponentRenderContext::None)
                    .map(|metadata_element| {
                        container(metadata_element)
                            .width(Length::FillPortion(2))
                            .padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
                            .into()
                    })
                    .ok();

                let content_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::None)
                    .map(|content_element| {
                        let content_element: Element<_> = container(content_element)
                            .width(Length::Fill)
                            .padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
                            .into();

                        let content_element: Element<_> = scrollable(content_element)
                            .width(Length::FillPortion(3))
                            .into();

                        content_element
                    })
                    .ok();

                let separator = vertical_rule(1)
                    .into();

                let content: Element<_> = match (content_element, metadata_element) {
                    (Some(content_element), Some(metadata_element)) => {
                        row(vec![content_element, separator, metadata_element])
                            .into()
                    }
                    (Some(content_element), None) => {
                        row(vec![content_element])
                            .into()
                    }
                    (None, Some(metadata_element)) => {
                        let content_element = vertical_space()
                            .into();

                        row(vec![content_element, separator, metadata_element])
                            .into()
                    }
                    (None, None) => {
                        row(vec![])
                            .into()
                    }
                };

                render_root(show_action_panel, widget_id, children, content, context)
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
                    .style(TextInputStyle::Form)
                    .into()
            }
            ComponentWidget::PasswordField { .. } => {
                let ComponentWidgetState::PasswordField { state_value } = state else {
                    panic!("unexpected state kind {:?}", state)
                };

                text_input("", state_value)
                    .secure(true)
                    .on_input(move |value| ComponentWidgetEvent::OnChangePasswordField { widget_id, value })
                    .style(TextInputStyle::Form)
                    .into()
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
                                            .padding(Padding::from([5.0, 10.0]))
                                            .into()
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
                                    .padding(Padding::new(10.0))
                                    .into();

                                Some(row)
                            }
                        }
                    })
                    .collect();

                let content: Element<_> = column(items)
                    .into();

                let content: Element<_> = scrollable(content)
                    .width(Length::Fill)
                    .into();

                render_root(show_action_panel, widget_id, children, content, context)
            }
            ComponentWidget::InlineSeparator => {
                vertical_rule(1)
                    .into()
            }
            ComponentWidget::Inline { children } => {
                let contents: Vec<_> = render_children_by_type(children, |widget| matches!(widget, ComponentWidget::Content { .. }), ComponentRenderContext::None)
                    .into_iter()
                    .map(|content_element| {
                        container(content_element)
                            .width(Length::FillPortion(3))
                            // .padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
                            .into()
                    })
                    .collect();

                // let mut separators: Vec<_> = render_children_by_type(children, |widget| matches!(widget, ComponentWidget::InlineSeparator { .. }), ComponentRenderContext::None);

                // let mut left = contents.len();

                let contents: Vec<_> = contents.into_iter()
                    .flat_map(|i| {
                        // if left > 1 {
                        //     left = left - 1;
                        //     if separators.is_empty() {
                        //         let separator = vertical_rule(1).into();
                        //         vec![i, separator]
                        //     } else {
                        //         let separator = separators.remove(0);
                        //         vec![i, separator]
                        //     }
                        // } else {
                            vec![i]
                        // }
                    })
                    .collect();

                let content: Element<_> = row(contents)
                    .into();

                container(content)
                    .padding(Padding::new(5.0))
                    .into()
            }
            ComponentWidget::EmptyView { title, description, image } => {
                let image: Option<Element<_>> = image.clone()  // FIXME really expensive clone
                    .map(|image| iced::widget::image(Handle::from_memory(image)).into());

                let title: Element<_> = text(title)
                    .into();

                let subtitle: Element<_> = match description {
                    None => horizontal_space().into(),
                    Some(subtitle) => text(subtitle).into(),
                };

                let mut content = vec![title, subtitle];
                if let Some(icon) = image {
                    content.insert(0, icon)
                }

                let content: Element<_> = column(content)
                    .into();

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
            ComponentWidget::ListItem { id, title, subtitle, icon } => {
                let ComponentRenderContext::List { widget_id: list_widget_id } = context else {
                    panic!("not supposed to be passed to list item: {:?}", context)
                };

                let icon: Option<Element<_>> = icon.clone()  // FIXME really expensive clone
                    .map(|icon| image(Handle::from_memory(icon)).into());

                let title: Element<_> = text(title)
                    .into();
                let title: Element<_> = container(title)
                    .padding(Padding::new(3.0))
                    .into();

                let mut content = vec![title];

                if let Some(icon) = icon {
                    content.insert(0, icon)
                }

                if let Some(subtitle) = subtitle {
                    let subtitle: Element<_> = text(subtitle)
                        .style(TextStyle::Subtext)
                        .into();
                    let subtitle: Element<_> = container(subtitle)
                        .padding(Padding::new(3.0))
                        .into();

                    content.push(subtitle)
                }
                let content: Element<_> = row(content)
                    .into();

                button(content)
                    .on_press(ComponentWidgetEvent::SelectListItem { list_widget_id, item_id: id.to_owned() })
                    .style(ButtonStyle::GauntletButton)
                    .width(Length::Fill)
                    .padding(Padding::new(5.0))
                    .into()
            }
            ComponentWidget::ListSection { children, title, subtitle } => {
                let content = render_children(children, context);

                let content = column(content)
                    .into();

                render_section(content, Some(title), subtitle)
            }
            ComponentWidget::List { children } => {
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
                        _ => panic!("unexpected widget kind {:?}", widget)
                    }
                }

                let content: Element<_> = column(items)
                    .width(Length::Fill)
                    .into();

                let content: Element<_> = scrollable(content)
                    .width(Length::Fill)
                    .into();

                render_root(show_action_panel, widget_id, children, content, context)
            }
            ComponentWidget::GridItem { children, id, title, subtitle } => {
                let ComponentRenderContext::Grid { widget_id: grid_widget_id } = context else {
                    panic!("not supposed to be passed to grid item: {:?}", context)
                };

                let content: Element<_> = column(render_children(children, ComponentRenderContext::None))
                    .into();

                let title: Element<_> = text(title)
                    .into();

                let subtitle: Option<Element<_>> = subtitle.as_ref()
                    .map(|subtitle| text(subtitle).into());

                let mut content = vec![content, title];
                if let Some(subtitle) = subtitle {
                    content.push(subtitle);
                }

                let content: Element<_> = column(content)
                    .into();

                let content: Element<_> = button(content)
                    .on_press(ComponentWidgetEvent::SelectGridItem { grid_widget_id, item_id: id.to_owned() })
                    .style(ButtonStyle::GauntletGridButton)
                    .padding(Padding::new(5.0))
                    .height(150) // TODO dynamic height
                    .into();

                content
            }
            ComponentWidget::GridSection { children, title, subtitle, columns } => {
                let content = render_grid(children, columns, context);

                render_section(content, Some(title), subtitle)
            }
            ComponentWidget::Grid { children, columns } => {
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
                        _ => panic!("unexpected widget kind {:?}", widget)
                    }
                }

                let content: Element<_> = column(items)
                    .into();

                let content: Element<_> = scrollable(content)
                    .width(Length::Fill)
                    .into();

                render_root(show_action_panel, widget_id, children, content, context)
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
        .padding(Padding::from([3.0, 5.0]))
        .style(ButtonStyle::Secondary)
        .on_press(ComponentWidgetEvent::PreviousView)
        .into();

    let space = Space::with_width(Length::FillPortion(3))
        .into();

    let top_panel: Element<_> = row(vec![back_button, space])
        .into();

    let top_panel: Element<_> = container(top_panel)
        .padding(Padding::new(10.0))
        .width(Length::Fill)
        .into();

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
        .spacing(10.0)
        .into();

    grid
}

fn render_section<'a>(content: Element<'a, ComponentWidgetEvent>, title: Option<&str>, subtitle: &Option<String>) -> Element<'a, ComponentWidgetEvent> {
    let mut title_content = vec![];

    if let Some(title) = title {
        let title: Element<_> = text(title)
            .size(15)
            .style(TextStyle::Subtext)
            .into();

        title_content.push(title)
    }

    if let Some(subtitle) = subtitle {
        let subtitle: Element<_> = text(subtitle)
            .size(15)
            .style(TextStyle::Subtext)
            .into();

        title_content.push(subtitle)
    }

    if title_content.is_empty() {
        let space: Element<_> = horizontal_space()
            .height(40)
            .into();

        title_content.push(space)
    }

    let title_content = row(title_content)
        .padding(Padding::from([5.0, 8.0])) // 5 + 3 to line up a section with items
        .into();

    column([title_content, content])
        .into()
}

fn render_root<'a>(
    show_action_panel: bool,
    widget_id: NativeUiWidgetId,
    children: &[ComponentWidgetWrapper],
    content: Element<'a, ComponentWidgetEvent>,
    context: ComponentRenderContext
) -> Element<'a, ComponentWidgetEvent>  {
    let ComponentRenderContext::Root { entrypoint_name } = context else {
        panic!("not supposed to be passed to root item: {:?}", context)
    };

    let entrypoint_name: Element<_> = text(entrypoint_name)
        .into();

    let space = Space::with_width(Length::FillPortion(3))
        .into();

    let action_panel_element = render_child_by_type(children, |widget| matches!(widget, ComponentWidget::ActionPanel { .. }), ComponentRenderContext::None)
        .ok();

    let (hide_action_panel, action_panel_element, bottom_panel) = if let Some(action_panel_element) = action_panel_element {
        let action_panel_toggle: Element<_> = button(text("Actions"))
            .padding(Padding::from([0.0, 5.0]))
            .style(ButtonStyle::Secondary)
            .on_press(ComponentWidgetEvent::ToggleActionPanel { widget_id })
            .into();

        let bottom_panel: Element<_> = row(vec![entrypoint_name, space, action_panel_toggle])
            .into();

        (!show_action_panel, action_panel_element, bottom_panel)
    } else {
        let bottom_panel: Element<_> = row(vec![entrypoint_name, space])
            .into();

        (true, Space::with_height(1).into(), bottom_panel)
    };

    let top_panel = create_top_panel();

    let bottom_panel: Element<_> = container(bottom_panel)
        .padding(Padding::new(5.0))
        .width(Length::Fill)
        .into();

    let top_separator = horizontal_rule(1)
        .into();

    let bottom_separator = horizontal_rule(1)
        .into();

    let content: Element<_> = container(content)
        .width(Length::Fill)
        .height(Length::Fill) // TODO remove after https://github.com/iced-rs/iced/issues/2186 is resolved
        .padding(Padding::from([5.0, 5.0, 0.0, 5.0]))
        .into();

    let content: Element<_> = column(vec![top_panel, top_separator, content, bottom_separator, bottom_panel])
        .into();

    floating_element(content, action_panel_element)
        .offset(Offset::from([5.0, 35.0]))
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
        ComponentRenderContext::Root { .. } => panic!("not supposed to be passed to text part")
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

    return render_text_part(&text_part, context);
}


fn render_children<'a>(
    content: &[ComponentWidgetWrapper],
    context: ComponentRenderContext
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    return content
        .into_iter()
        .map(|child| child.render_widget(context.clone()))
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
        .map(|child| child.render_widget(context.clone()))
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
    SelectListItem {
        list_widget_id: NativeUiWidgetId,
        item_id: String,
    },
    SelectGridItem {
        grid_widget_id: NativeUiWidgetId,
        item_id: String,
    },
    PreviousView,
}

impl ComponentWidgetEvent {
    pub fn handle(self, plugin_id: PluginId, widget: ComponentWidgetWrapper) -> Option<NativeUiViewEvent> {
        match self {
            ComponentWidgetEvent::LinkClick { widget_id: _, href } => {
                todo!("href {:?}", href);
            }
            ComponentWidgetEvent::TagClick { widget_id } => {
                Some(create_metadata_tag_item_on_click_event(widget_id))
            }
            ComponentWidgetEvent::ActionClick { widget_id } => {
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
                    let ComponentWidgetState::DatePicker { state_value, show_picker,  } = state else {
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
                let (widget, ref mut state) = &mut *widget.get_mut();

                let show_action_panel = match state {
                    ComponentWidgetState::Detail { show_action_panel } => show_action_panel,
                    ComponentWidgetState::Form { show_action_panel } => show_action_panel,
                    ComponentWidgetState::List { show_action_panel } => show_action_panel,
                    ComponentWidgetState::Grid { show_action_panel } => show_action_panel,
                    _ => panic!("unexpected state kind, widget: {:?} state: {:?}", widget, state)
                };

                *show_action_panel = !*show_action_panel;

                None
            }
            ComponentWidgetEvent::SelectListItem { list_widget_id, item_id } => {
                Some(create_list_on_selection_change_event(list_widget_id, item_id))
            }
            ComponentWidgetEvent::SelectGridItem { grid_widget_id, item_id } => {
                Some(create_grid_on_selection_change_event(grid_widget_id, item_id))
            }
            ComponentWidgetEvent::PreviousView => {
                panic!("handle event on PreviousView event is not supposed to be called")
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
            ComponentWidgetEvent::SelectListItem { list_widget_id, .. } => list_widget_id,
            ComponentWidgetEvent::SelectGridItem { grid_widget_id, .. } => grid_widget_id,
            ComponentWidgetEvent::PreviousView => panic!("widget_id on PreviousView event is not supposed to be called"),
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

fn parse_bytes_optional(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
    match properties.get(name) {
        None => Ok(None),
        Some(value) => Ok(Some(value.as_bytes().ok_or(anyhow::anyhow!("{} has to be a string", name))?.to_owned())),
    }
}

fn parse_bytes(properties: &HashMap<String, NativeUiPropertyValue>, name: &str) -> anyhow::Result<Vec<u8>> {
    parse_bytes_optional(properties, name)?.ok_or(anyhow::anyhow!("{} is required", name))
}
