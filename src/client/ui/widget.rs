use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use iced::Element;
use iced::widget::{button, row, text};

use crate::client::model::{NativeUiPropertyValue, NativeUiWidgetId};

#[derive(Clone)]
pub struct BuiltInWidgetWrapper {
    id: NativeUiWidgetId,
    inner: Arc<RwLock<BuiltInWidget>>,
}

impl BuiltInWidgetWrapper {
    pub fn widget(id: NativeUiWidgetId, widget_type: &str, _properties: HashMap<String, NativeUiPropertyValue>) -> Self {
        let widget = match widget_type.as_ref() {
            "box" => BuiltInWidget::Container { children: vec![] },
            "button1" => BuiltInWidget::Button(widget_type.to_owned()),
            _ => panic!("widget_type {} not supported", widget_type)
        };

        BuiltInWidgetWrapper::new(id, widget)
    }

    pub fn empty_container(id: NativeUiWidgetId) -> Self {
        BuiltInWidgetWrapper::new(id, BuiltInWidget::Container { children: vec![] })
    }

    pub fn text(id: NativeUiWidgetId, text: &str) -> Self {
        BuiltInWidgetWrapper::new(id, BuiltInWidget::Text(text.to_owned()))
    }

    fn new(id: NativeUiWidgetId, widget: BuiltInWidget) -> Self {
        Self {
            id,
            inner: Arc::new(RwLock::new(widget)),
        }
    }

    fn get(&self) -> RwLockReadGuard<'_, BuiltInWidget> {
        self.inner.read().unwrap()
    }

    fn get_mut(&self) -> RwLockWriteGuard<'_, BuiltInWidget> {
        self.inner.write().unwrap()
    }

    pub fn render_widget<'a>(&self) -> Element<'a, BuiltInWidgetEvent> {
        let widget = self.get();
        match &*widget {
            BuiltInWidget::Container { children } => {
                let children: Vec<Element<_>> = children
                    .into_iter()
                    .map(|child| child.render_widget())
                    .collect();

                row(children)
                    .into()
            }
            BuiltInWidget::Button(text_content) => {
                let text: Element<_> = text(text_content)
                    .into();

                button(text)
                    .on_press(BuiltInWidgetEvent::ButtonClick { widget_id: self.id })
                    .into()
            }
            BuiltInWidget::Text(text_content) => {
                text(text_content)
                    .into()
            }
        }
    }

    pub fn append_child(&self, child: BuiltInWidgetWrapper) {
        let mut parent = self.get_mut();
        match *parent {
            BuiltInWidget::Container { ref mut children } => {
                children.push(child)
            }
            BuiltInWidget::Button(_) => {}
            _ => panic!("parent not supported")
        };
    }

    pub fn set_children(&self, new_children: Vec<BuiltInWidgetWrapper>) {
        let mut container = self.get_mut();
        match *container {
            BuiltInWidget::Container { ref mut children } => {
                *children = new_children
            }
            BuiltInWidget::Button(_) => {}
            _ => panic!("not supported parent")
        };
    }
}

enum BuiltInWidget {
    Container {
        children: Vec<BuiltInWidgetWrapper>
    },
    Button(String),
    Text(String),
}

#[derive(Clone, Debug)]
pub enum BuiltInWidgetEvent {
    ButtonClick {
        widget_id: NativeUiWidgetId
    }
}

impl BuiltInWidgetEvent {
    pub fn event_name(&self) -> String {
        match self {
            BuiltInWidgetEvent::ButtonClick { .. } => "onClick".to_owned()
        }
    }

    pub fn widget_id(&self) -> NativeUiWidgetId {
        match self {
            BuiltInWidgetEvent::ButtonClick { widget_id } => widget_id.clone()
        }
    }
}
