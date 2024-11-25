use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::renderer;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::tree::Tag;
use iced::advanced::widget::Tree;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::event::Status;
use iced::mouse::Cursor;
use iced::Border;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Shadow;
use iced::Size;
use iced::{window, Color};
use std::time::{Duration, Instant};

pub struct LoadingBar<'a, Theme>
where
    Theme: Catalog,
{
    width: Length,
    segment_width: f32,
    height: Length,
    rate: Duration,
    class: <Theme as Catalog>::Class<'a>,
}

impl<'a, Theme> Default for LoadingBar<'a, Theme>
where
    Theme: Catalog,
{
    fn default() -> Self {
        Self {
            width: Length::Fill,
            segment_width: 200.0,
            height: Length::Fixed(1.0),
            rate: Duration::from_secs_f32(1.0),
            class: <Theme as Catalog>::Class::default(),
        }
    }
}

impl<'a, Theme> LoadingBar<'a, Theme>
where
    Theme: Catalog,
{
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub fn segment_width(mut self, segment_width: f32) -> Self {
        self.segment_width = segment_width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    #[must_use]
    pub fn class(mut self, class: <Theme as Catalog>::Class<'a>) -> Self {
        self.class = class;
        self
    }
}

struct LoadingBarState {
    last_update: Instant,
    t: f32,
}

fn is_visible(bounds: &Rectangle) -> bool {
    bounds.width > 0.0 && bounds.height > 0.0
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for LoadingBar<'a, Theme>
where
    Renderer: renderer::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(limits.width(self.width).height(self.height).resolve(
            self.width,
            self.height,
            Size::new(f32::INFINITY, f32::INFINITY),
        ))
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if !is_visible(&bounds) {
            return;
        }

        let position = bounds.position();
        let size = bounds.size();
        let styling = Catalog::style(theme, &self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: position.x,
                    y: position.y,
                    width: size.width,
                    height: size.height,
                },
                border: Border::default(),
                shadow: Shadow::default(),
            },
            styling.background_color,
        );

        let state = state.state.downcast_ref::<LoadingBarState>();

        // works but quick and hacky
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: position.x + (size.width * state.t * 1.3) - self.segment_width,
                    y: position.y,
                    width: self.segment_width,
                    height: size.height,
                },
                border: Border::default(),
                shadow: Shadow::default(),
            },
            styling.loading_bar_color,
        );
    }

    fn tag(&self) -> Tag {
        Tag::of::<LoadingBarState>()
    }

    fn state(&self) -> State {
        State::new(LoadingBarState {
            last_update: Instant::now(),
            t: 0.0,
        })
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        const FRAMES_PER_SECOND: u64 = 60;

        let bounds = layout.bounds();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if is_visible(&bounds) {
                let state = state.state.downcast_mut::<LoadingBarState>();
                let duration = (now - state.last_update).as_secs_f32();
                let increment = if self.rate == Duration::ZERO {
                    0.0
                } else {
                    duration * 1.0 / self.rate.as_secs_f32()
                };

                state.t += increment;

                if state.t > 1.0 {
                    state.t -= 1.0;
                }

                shell.request_redraw(window::RedrawRequest::At(
                    now + Duration::from_millis(1000 / FRAMES_PER_SECOND),
                ));
                state.last_update = now;

                return Status::Captured;
            }
        }

        Status::Ignored
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub background_color: Color,
    pub loading_bar_color: Color,
}

pub trait Catalog {
    type Class<'a>: Default;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}


impl<'a, Message, Theme, Renderer> From<LoadingBar<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Theme: 'a + Catalog,
{
    fn from(spinner: LoadingBar<'a, Theme>) -> Self {
        Self::new(spinner)
    }
}
