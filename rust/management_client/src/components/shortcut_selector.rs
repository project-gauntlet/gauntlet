use iced::{alignment, Element, Event, Length, Padding, Rectangle, Renderer, Size};
use iced::advanced::{Clipboard, layout, Layout, mouse, renderer, Shell, Widget};
use iced::advanced::graphics::core::{event, keyboard};
use iced::advanced::widget::{Tree, tree};
use iced::mouse::Button;
use iced::widget::{container, row, text};
use iced::widget::container::{Appearance, draw_background, layout};

use common::model::PhysicalShortcut;
use common_ui::{physical_key_model, shortcut_to_text};

pub struct ShortcutSelector<'a, Message, Theme>
where
    Theme: StyleSheet + text::StyleSheet + container::StyleSheet
{
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,

    style: <Theme as StyleSheet>::Style,

    on_shortcut_captured: Box<dyn Fn(PhysicalShortcut) -> Message + 'a>,

    content: Element<'a, Message, Theme>,
}

impl<'a, Message: 'a, Theme> ShortcutSelector<'a, Message, Theme>
where
    Theme: StyleSheet + text::StyleSheet + container::StyleSheet + 'a
{
    pub fn new<F>(current_shortcut: &PhysicalShortcut, on_shortcut_captured: F, style: <Theme as StyleSheet>::Style) -> Self
        where
            F: 'a + Fn(PhysicalShortcut) -> Message,
    {

        let (key_name, alt_modifier_text, meta_modifier_text, control_modifier_text, shift_modifier_text) = shortcut_to_text(current_shortcut);

        let mut content: Vec<Element<Message, Theme>> = vec![];


        if let Some(meta_modifier_text) = meta_modifier_text {
            content.push(meta_modifier_text);
        }

        if let Some(control_modifier_text) = control_modifier_text {
            content.push(control_modifier_text);
        }

        if let Some(shift_modifier_text) = shift_modifier_text {
            content.push(shift_modifier_text);
        }

        if let Some(alt_modifier_text) = alt_modifier_text {
            content.push(alt_modifier_text);
        }

        content.push(key_name);

        let content: Element<_, _> = row(content)
            .spacing(8.0)
            .into();

        let content = container(content)
            .into();

        Self {
            padding: Padding::ZERO,
            width: Length::Fill,
            height: Length::Fill,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,

            style,

            on_shortcut_captured: Box::new(on_shortcut_captured),

            content,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_capturing: bool,
}


impl<'a, Message, Theme> Widget<Message, Theme, Renderer> for ShortcutSelector<'a, Message, Theme>
where
    Theme: StyleSheet + text::StyleSheet + container::StyleSheet
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| self.content.as_widget().layout(tree, renderer, limits),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let style = if state.is_capturing {
            theme.capturing(&self.style)
        } else {
            theme.active(&self.style)
        };

        draw_background(renderer, &style, layout.bounds());

        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                text_color: style
                    .text_color
                    .unwrap_or(renderer_style.text_color),
            },
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Keyboard(event) => {
                if state.is_capturing {
                    match event {
                        keyboard::Event::KeyReleased { physical_key, modifiers, .. } => {
                            match physical_key {
                                keyboard::key::PhysicalKey::Backspace | keyboard::key::PhysicalKey::Escape => {
                                    state.is_capturing = false;

                                    event::Status::Ignored
                                }
                                _ => {
                                    match physical_key_model(physical_key) {
                                        None => event::Status::Ignored,
                                        Some(physical_key) => {
                                            let message = (self.on_shortcut_captured)(
                                                PhysicalShortcut {
                                                    physical_key,
                                                    modifier_shift: modifiers.shift(),
                                                    modifier_control: modifiers.control(),
                                                    modifier_alt: modifiers.alt(),
                                                    modifier_meta: modifiers.logo(),
                                                }
                                            );
                                            shell.publish(message);

                                            state.is_capturing = false;

                                            event::Status::Captured
                                        }
                                    }

                                }
                            }
                        }
                        _ => event::Status::Ignored
                    }
                } else {
                    event::Status::Ignored
                }
            }
            Event::Mouse(event) => {
                match event {
                    mouse::Event::ButtonReleased(Button::Left) => {
                        if cursor.is_over(layout.bounds()) {
                            state.is_capturing = true;

                            event::Status::Captured
                        } else {
                            state.is_capturing = false;

                            event::Status::Ignored
                        }
                    }
                    _ => event::Status::Ignored
                }
            }
            _ => event::Status::Ignored
        }

    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme> From<ShortcutSelector<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: 'a,
    Theme: StyleSheet + text::StyleSheet + container::StyleSheet + 'a
{
    fn from(shortcut_selector: ShortcutSelector<'a, Message, Theme>) -> Self {
        Self::new(shortcut_selector)
    }
}

pub trait StyleSheet {
    type Style: Default;

    fn active(&self, style: &Self::Style) -> Appearance;
    fn capturing(&self, style: &Self::Style) -> Appearance;
}
