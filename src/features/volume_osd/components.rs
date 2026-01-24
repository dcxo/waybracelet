use iced::{
    Element,
    Length::Fill,
    Point, Rectangle, Renderer, Theme, Vector,
    advanced::mouse,
    widget::{
        Canvas,
        canvas::{self, LineCap, Path, Program, Stroke},
    },
};

use crate::Message;

pub struct Volume {
    pub volume: f32,
    pub alpha: f32,
}

impl Volume {
    const MARGIN: f32 = 68.;
}

impl<'a> From<Volume> for Element<'a, Message> {
    fn from(value: Volume) -> Self {
        Canvas::new(value).width(Fill).height(Fill).into()
    }
}

impl<T> Program<T> for Volume {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let alpha = self.alpha;

        let va1 = Vector::new(0., -Self::MARGIN);
        let va2 = Vector::new(Self::MARGIN, 0.);
        let vb = Vector::new(Self::MARGIN, -Self::MARGIN);

        let start_point = Point::new(Volume::MARGIN, bounds.height);
        let (a1, b1) = (start_point + va1, start_point + vb);
        let next_point = Point::new(bounds.width - Self::MARGIN * 2.5, b1.y);
        let (a2, b2) = (next_point + va2, next_point + vb);
        let (a3, b3) = (b2 + va1, b2 + vb);
        let last_point = Point::new(bounds.width, b3.y);

        let background = theme.palette().background;
        let primary = theme.palette().primary;

        frame.stroke(
            &Path::new(|b| {
                b.move_to(start_point);
                b.quadratic_curve_to(a1, b1);
                b.line_to(next_point);
                b.quadratic_curve_to(a2, b2);
                b.quadratic_curve_to(a3, b3);
                b.line_to(last_point);
            }),
            Stroke::default()
                .with_width(8.)
                .with_line_cap(LineCap::Round)
                .with_color(background.scale_alpha(alpha)),
        );

        frame.stroke(
            &Path::new(|b| {
                b.move_to(b1);
                b.line_to(next_point);
            }),
            Stroke::default()
                .with_width(40.)
                .with_line_cap(LineCap::Round)
                .with_color(background.scale_alpha(alpha)),
        );

        frame.stroke(
            &Path::new(|b| {
                b.move_to(b1);
                b.line_to(next_point);
            }),
            Stroke::default()
                .with_width(8.)
                .with_line_cap(LineCap::Round)
                .with_color(primary.scale_alpha(alpha)),
        );

        let db1n = b1.distance(next_point);
        frame.stroke(
            &Path::new(|b| {
                b.move_to(b1);
                b.line_to(next_point - Vector::new(db1n * 0.6, 0.));
            }),
            Stroke::default()
                .with_width(32.)
                .with_line_cap(LineCap::Round)
                .with_color(primary.scale_alpha(alpha)),
        );

        vec![frame.into_geometry()]
    }
}
