use iced::widget::{Canvas, canvas};
use iced::{Point, Renderer};

pub struct CornerCurve {
    start: Point,
    end: Point,
    stroke_width: f32,
}

impl CornerCurve {
    pub fn new(start: Point, end: Point, stroke_width: f32) -> Self {
        Self {
            start,
            end,
            stroke_width,
        }
    }

    pub fn into_canvas<Message>(self, width: f32, height: f32) -> Canvas<Self, Message> {
        canvas(self).width(width).height(height)
    }

    pub fn top_left<Message>(size: f32, stroke_width: f32) -> Canvas<Self, Message> {
        Self::new(
            Point::new(size / 2., 0.),
            Point::new(0., size / 2.),
            stroke_width,
        )
        .into_canvas(size, size)
    }

    pub fn top_right<Message>(size: f32, stroke_width: f32) -> Canvas<Self, Message> {
        Self::new(
            Point::new(size / 2., 0.),
            Point::new(size, size / 2.),
            stroke_width,
        )
        .into_canvas(size, size)
    }
}

impl<Message> canvas::Program<Message> for CornerCurve {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let stroke = canvas::Stroke {
            width: self.stroke_width,
            style: canvas::Style::Solid(theme.palette().background),
            line_cap: canvas::LineCap::Round,
            ..Default::default()
        };

        let path = canvas::Path::new(|builder| {
            builder.move_to(self.start);
            builder.quadratic_curve_to(Point::new(self.start.x, self.end.y), self.end);
        });

        frame.stroke(&path, stroke);

        vec![frame.into_geometry()]
    }
}
