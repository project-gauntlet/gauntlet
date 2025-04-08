use gauntlet_common::model::PhysicalShortcut;
use gauntlet_common_ui::physical_key_model;
use gauntlet_common_ui::shortcut_to_text;
use iced::advanced::graphics::core::event;
use iced::advanced::graphics::core::keyboard;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::tree;
use iced::advanced::widget::Tree;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::keyboard::key::Physical;
use iced::mouse::Button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::container::draw_background;
use iced::widget::row;
use iced::widget::text;
use iced::Alignment;
use iced::Event;
use iced::Length;
use iced::Padding;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Vector;

use crate::theme::text::TextStyle;
use crate::theme::Element;
use crate::theme::GauntletSettingsTheme;

pub struct ShortcutData {
    pub shortcut: Option<PhysicalShortcut>,
    pub error: Option<String>,
}

pub fn shortcut_selector<'a, 'b: 'a, 'c, Message: 'a, F>(
    current_shortcut: &'b ShortcutData,
    on_shortcut_captured: F,
    overlay_class: <GauntletSettingsTheme as container::Catalog>::Class<'a>,
    in_table: bool,
) -> Element<'a, Message>
where
    F: 'a + Fn(Option<PhysicalShortcut>) -> Message,
{
    Element::new(ShortcutSelector::new::<F>(
        current_shortcut,
        on_shortcut_captured,
        overlay_class,
        in_table,
    ))
}

pub struct ShortcutSelector<'a, 'b, Message> {
    on_shortcut_captured: Box<dyn Fn(Option<PhysicalShortcut>) -> Message + 'a>,

    current_shortcut: &'b ShortcutData,

    content: Element<'a, Message>,
    popup: Element<'a, Message>,
    overlay_class: <GauntletSettingsTheme as container::Catalog>::Class<'a>,
    in_table: bool,
}

impl<'a, 'b, 'c, Message: 'a> ShortcutSelector<'a, 'b, Message> {
    pub fn new<F>(
        current_shortcut: &'b ShortcutData,
        on_shortcut_captured: F,
        overlay_class: <GauntletSettingsTheme as container::Catalog>::Class<'a>,
        in_table: bool,
    ) -> Self
    where
        F: 'a + Fn(Option<PhysicalShortcut>) -> Message,
    {
        let content = render_shortcut(&current_shortcut.shortcut, in_table);

        let mut content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill);

        if !in_table {
            content = content.center_x(Length::Fill);
        }

        let content = content.into();

        let recording_text: Element<_> = text("Recording shortcut...").into();

        let backspace_test: Element<_> = text("Backspace - Unset Shortcut").class(TextStyle::Subtitle).into();

        let escape_test: Element<_> = text("Escape - Stop Capturing").class(TextStyle::Subtitle).into();

        let popup: Element<_> = column(vec![recording_text, backspace_test, escape_test])
            .align_x(Alignment::Center)
            .into();

        let popup = container(popup)
            .max_height(80)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .max_width(300)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        Self {
            on_shortcut_captured: Box::new(on_shortcut_captured),

            current_shortcut,
            content,
            popup,

            overlay_class,
            in_table,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_capturing: bool,
    is_hovering: bool,
}

impl<'a, 'b, Message: 'a> Widget<Message, GauntletSettingsTheme, Renderer> for ShortcutSelector<'a, 'b, Message> {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &GauntletSettingsTheme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let style = if state.is_capturing {
            Status::Capturing
        } else {
            if state.is_hovering {
                Status::Hovered
            } else {
                Status::Active
            }
        };

        let style = Catalog::style(
            theme,
            &<GauntletSettingsTheme as Catalog>::default(),
            style,
            self.in_table,
        );

        draw_background(renderer, &style, layout.bounds());

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
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
        vec![Tree::new(&self.content), Tree::new(&self.popup)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget(), self.popup.as_widget()]);
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
                        keyboard::Event::KeyReleased {
                            physical_key,
                            modifiers,
                            ..
                        } => {
                            match physical_key {
                                Physical::Code(code) => {
                                    match code {
                                        keyboard::key::Code::Backspace if modifiers.is_empty() => {
                                            state.is_capturing = false;

                                            let message = (self.on_shortcut_captured)(None);
                                            shell.publish(message);

                                            event::Status::Ignored
                                        }
                                        keyboard::key::Code::Escape if modifiers.is_empty() => {
                                            state.is_capturing = false;

                                            let message = (self.on_shortcut_captured)(None);
                                            shell.publish(message);

                                            event::Status::Ignored
                                        }
                                        _ => {
                                            match physical_key_model(code, modifiers) {
                                                None => event::Status::Ignored,
                                                Some(shortcut) => {
                                                    state.is_capturing = false;

                                                    let message = (self.on_shortcut_captured)(Some(shortcut));
                                                    shell.publish(message);

                                                    event::Status::Captured
                                                }
                                            }
                                        }
                                    }
                                }
                                Physical::Unidentified(_) => event::Status::Ignored,
                            }
                        }
                        _ => event::Status::Ignored,
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
                    mouse::Event::CursorMoved { .. } => {
                        state.is_hovering = cursor.is_over(layout.bounds());

                        event::Status::Captured
                    }
                    _ => event::Status::Ignored,
                }
            }
            _ => event::Status::Ignored,
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

    fn overlay<'c>(
        &'c mut self,
        tree: &'c mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'c, Message, GauntletSettingsTheme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        let mut children = tree.children.iter_mut();

        let content = self
            .content
            .as_widget_mut()
            .overlay(children.next().unwrap(), layout, renderer, translation);

        let popup = if state.is_capturing {
            Some(overlay::Element::new(Box::new(Overlay {
                position: layout.position() + translation,
                popup: &self.popup,
                state: children.next().unwrap(),
                content_bounds: layout.bounds(),
                class: &self.overlay_class,
            })))
        } else {
            None
        };

        if content.is_some() || popup.is_some() {
            Some(overlay::Group::with_children(content.into_iter().chain(popup).collect()).overlay())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Hovered,
    Capturing,
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>, status: Status, transparent_background: bool) -> container::Style;
}

pub fn render_shortcut<'a, Message: 'a>(shortcut: &Option<PhysicalShortcut>, in_table: bool) -> Element<'a, Message> {
    let mut content: Vec<Element<Message>> = vec![];

    if let Some(current_shortcut) = shortcut {
        let (key_name, alt_modifier_text, meta_modifier_text, control_modifier_text, shift_modifier_text) =
            shortcut_to_text(current_shortcut);

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
    } else {
        if in_table {
            content.push(text("Record Shortcut").class(TextStyle::Subtitle).into());
        }
    }

    let content: Element<Message> = row(content).padding(8.0).spacing(8.0).into();

    content
}

struct Overlay<'a, 'b, Message> {
    position: Point,
    popup: &'b Element<'a, Message>,
    state: &'b mut Tree,
    content_bounds: Rectangle,
    class: &'b <GauntletSettingsTheme as container::Catalog>::Class<'a>,
}

impl<'a, 'b, Message> overlay::Overlay<Message, GauntletSettingsTheme, Renderer> for Overlay<'a, 'b, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let padding = 2.0;
        let gap = 10.0;

        let viewport = Rectangle::with_size(bounds);

        let popup_layout = self.popup.as_widget().layout(
            self.state,
            renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()).shrink(Padding::new(padding)),
        );

        let text_bounds = popup_layout.bounds();
        let x_center = self.position.x + (self.content_bounds.width - text_bounds.width) / 2.0;

        let mut tooltip_bounds = {
            let offset = Vector::new(x_center, self.position.y + self.content_bounds.height + gap + padding);

            Rectangle {
                x: offset.x - padding,
                y: offset.y - padding,
                width: text_bounds.width + padding * 2.0,
                height: text_bounds.height + padding * 2.0,
            }
        };

        // snap_within_viewport
        if tooltip_bounds.x < viewport.x {
            tooltip_bounds.x = viewport.x;
        } else if viewport.x + viewport.width < tooltip_bounds.x + tooltip_bounds.width {
            tooltip_bounds.x = viewport.x + viewport.width - tooltip_bounds.width;
        }

        if tooltip_bounds.y < viewport.y {
            tooltip_bounds.y = viewport.y;
        } else if viewport.y + viewport.height < tooltip_bounds.y + tooltip_bounds.height {
            tooltip_bounds.y = viewport.y + viewport.height - tooltip_bounds.height;
        }

        layout::Node::with_children(
            tooltip_bounds.size(),
            vec![popup_layout.translate(Vector::new(padding, padding))],
        )
        .translate(Vector::new(tooltip_bounds.x, tooltip_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &GauntletSettingsTheme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        let style = <GauntletSettingsTheme as container::Catalog>::style(theme, self.class);

        draw_background(renderer, &style, layout.bounds());

        let defaults = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        self.popup.as_widget().draw(
            self.state,
            renderer,
            theme,
            &defaults,
            layout.children().next().unwrap(),
            cursor_position,
            &Rectangle::with_size(Size::INFINITY),
        );
    }

    fn is_over(&self, _layout: Layout<'_>, _renderer: &Renderer, _cursor_position: Point) -> bool {
        false
    }
}
