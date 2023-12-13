use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::Element;
use iced::theme::Button;
use iced::widget::{button, column, row, text};
use zbus::SignalContext;

use common::dbus::DbusEventViewEvent;
use common::model::PluginId;

use crate::dbus::DbusClient;
use crate::model::{NativeUiPropertyValue, NativeUiWidget, NativeUiWidgetId};

#[derive(Clone, Debug)]
pub struct ComponentWidgetWrapper {
    id: NativeUiWidgetId,
    inner: Arc<RwLock<ComponentWidget>>,
}

include!(concat!(env!("OUT_DIR"), "/components.rs"));

impl ComponentWidgetWrapper {
    pub fn widget(id: NativeUiWidgetId, widget_type: &str, properties: HashMap<String, NativeUiPropertyValue>) -> anyhow::Result<Self> {
        Ok(ComponentWidgetWrapper::new(id, create_component_widget(widget_type, properties)?))
    }

    pub fn root(id: NativeUiWidgetId) -> Self {
        ComponentWidgetWrapper::new(id, ComponentWidget::Root { children: vec![] })
    }

    pub fn text_part(id: NativeUiWidgetId, text: &str) -> anyhow::Result<Self> {
        Ok(ComponentWidgetWrapper::new(id, ComponentWidget::TextPart(text.to_owned())))
    }

    fn new(id: NativeUiWidgetId, widget: ComponentWidget) -> Self {
        Self {
            id,
            inner: Arc::new(RwLock::new(widget)),
        }
    }

    fn get(&self) -> RwLockReadGuard<'_, ComponentWidget> {
        self.inner.read().expect("lock is poisoned")
    }

    fn get_mut(&self) -> RwLockWriteGuard<'_, ComponentWidget> {
        self.inner.write().expect("lock is poisoned")
    }

    pub fn render_widget<'a>(&self) -> Element<'a, ComponentWidgetEvent> {
        let widget = self.get();
        match &*widget {
            ComponentWidget::TextPart(text_content) => {
                text(text_content).into()
            }
            ComponentWidget::Text { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::Link { children, href } => {
                let content: Element<_> = row(render_children(children))
                    .into();

                button(content)
                    .style(Button::Text)
                    .on_press(ComponentWidgetEvent::LinkClick { href: href.to_owned() })
                    .into()
            }
            ComponentWidget::Tag { children, onClick: _, color: _ } => {
                let content: Element<_> = row(render_children(children))
                    .into();

                button(content)
                    .on_press(ComponentWidgetEvent::TagClick { widget: self.as_native_widget() })
                    .into()
            }
            ComponentWidget::MetadataItem { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::Separator => {
                text("Separator").into()
            }
            ComponentWidget::Metadata { children } => {
                column(render_children(children))
                    .into()
            }
            ComponentWidget::Image => {
                text("Image").into()
            }
            ComponentWidget::H1 { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::H2 { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::H3 { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::H4 { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::H5 { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::H6 { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::HorizontalBreak => {
                text("HorizontalBreak").into()
            }
            ComponentWidget::CodeBlock { children } => {
                text("CodeBlock").into()
            }
            ComponentWidget::Code { children } => {
                text("Code").into()
            }
            ComponentWidget::Content { children } => {
                column(render_children(children))
                    .into()
            }
            ComponentWidget::Detail { children } => {
                row(render_children(children))
                    .into()
            }
            ComponentWidget::Root { children } => {
                row(render_children(children))
                    .into()
            }
        }
    }

    pub fn append_child(&self, child: ComponentWidgetWrapper) -> anyhow::Result<()> {
        append_component_widget_child(&self, child)
    }

    pub fn get_children(&self) -> anyhow::Result<Vec<ComponentWidgetWrapper>> {
        get_component_widget_children(&self)
    }

    pub fn set_children(&self, new_children: Vec<ComponentWidgetWrapper>) -> anyhow::Result<()> {
        set_component_widget_children(&self, new_children)
    }

    pub fn as_native_widget(&self) -> NativeUiWidget {
        let (internal_name, _) = get_component_widget_type(&self);
        NativeUiWidget {
            widget_id: self.id,
            widget_type: internal_name.to_owned()
        }
    }
}

pub fn render_children<'a>(
    content: &[ComponentWidgetWrapper]
) -> Vec<Element<'a, ComponentWidgetEvent>> {
    return content
        .into_iter()
        .map(|child| child.render_widget())
        .collect();
}


#[derive(Clone, Debug)]
pub enum ComponentWidgetEvent {
    LinkClick {
        href: String
    },
    TagClick {
        widget: NativeUiWidget
    },
}

impl ComponentWidgetEvent {
    pub async fn handle(self, signal_context: &SignalContext<'_>, plugin_id: PluginId) {
        match self {
            ComponentWidgetEvent::LinkClick { href } => {
                todo!("href {:?}", href)
            }
            ComponentWidgetEvent::TagClick { widget } => {
                let event_view_event = DbusEventViewEvent {
                    event_name: "onClick".to_owned(),
                    widget: widget.into(),
                };

                DbusClient::view_event_signal(signal_context, &plugin_id.to_string(), event_view_event)
                    .await
                    .unwrap();
            }
        }
    }
}
