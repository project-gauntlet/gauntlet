use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Shadow;
use iced::Size;
use iced::Vector;
use iced::advanced::Clipboard;
use iced::advanced::Layout;
use iced::advanced::Shell;
use iced::advanced::Widget;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::tree::Tag;
use iced::mouse::Cursor;
use iced::time::Duration;
use iced::time::Instant;
use iced::window;

pub struct Spinner {
    width: Length,
    height: Length,
    rate: Duration,
    circle_radius: f32,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            width: Length::Fixed(20.0),
            height: Length::Fixed(20.0),
            rate: Duration::from_secs_f32(1.0),
            circle_radius: 2.0,
        }
    }
}

impl Spinner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    #[must_use]
    pub fn circle_radius(mut self, radius: f32) -> Self {
        self.circle_radius = radius;
        self
    }
}

struct SpinnerState {
    last_update: Instant,
    t: f32,
}

fn is_visible(bounds: &Rectangle) -> bool {
    bounds.width > 0.0 && bounds.height > 0.0
}

fn fill_circle(renderer: &mut impl renderer::Renderer, position: Vector, radius: f32, color: Color) {
    if radius > 0. {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: position.x,
                    y: position.y,
                    width: radius * 2.0,
                    height: radius * 2.0,
                },
                border: Border {
                    radius: radius.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            },
            color,
        );
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Spinner
where
    Renderer: renderer::Renderer,
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
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if !is_visible(&bounds) {
            return;
        }

        let size = if bounds.width < bounds.height {
            bounds.width
        } else {
            bounds.height
        } / 2.0;
        let state = state.state.downcast_ref::<SpinnerState>();
        let center = bounds.center();
        let distance_from_center = size - self.circle_radius;
        let (y, x) = (state.t * std::f32::consts::PI * 2.0).sin_cos();
        let position = Vector::new(
            center.x + x * distance_from_center - self.circle_radius,
            center.y + y * distance_from_center - self.circle_radius,
        );

        fill_circle(renderer, position, self.circle_radius, style.text_color);
    }

    fn tag(&self) -> Tag {
        Tag::of::<SpinnerState>()
    }

    fn state(&self) -> State {
        State::new(SpinnerState {
            last_update: Instant::now(),
            t: 0.0,
        })
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        _cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        const FRAMES_PER_SECOND: u64 = 60;

        let bounds = layout.bounds();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if is_visible(&bounds) {
                let state = state.state.downcast_mut::<SpinnerState>();
                let duration = (*now - state.last_update).as_secs_f32();
                let increment = if self.rate == Duration::ZERO {
                    0.0
                } else {
                    duration * 1.0 / self.rate.as_secs_f32()
                };

                state.t += increment;

                if state.t > 1.0 {
                    state.t -= 1.0;
                }

                shell.request_redraw_at(window::RedrawRequest::At(
                    *now + Duration::from_millis(1000 / FRAMES_PER_SECOND),
                ));
                state.last_update = now.clone();
                shell.capture_event();
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Spinner> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
{
    fn from(spinner: Spinner) -> Self {
        Self::new(spinner)
    }
}
