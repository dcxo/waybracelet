use iced::{
    Color, Element, Point, Renderer,
    widget::canvas::{self, Action, Frame, LineCap, Program, Stroke, path},
};

pub struct CavaPlayer<'a>(pub &'a [f32]);

impl<'a> CavaPlayer<'a> {
    pub(crate) const WAVE_WIDTH: f32 = 24.;
}

impl<'a, T: 'a> From<CavaPlayer<'a>> for Element<'a, T> {
    fn from(value: CavaPlayer<'a>) -> Self {
        let len = value.0.len();
        canvas::Canvas::new(value)
            .width(CavaPlayer::WAVE_WIDTH * (len + 1) as f32)
            .into()
    }
}

impl<'a, T> Program<T> for CavaPlayer<'a> {
    type State = Vec<Point>;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let mut debug_frame = Frame::new(renderer, bounds.size());

        let mut path = path::Builder::new();
        let mut back = path::Builder::new();

        path.move_to(Point::new(0., bounds.height / 2.));
        back.move_to(Point::new(0., bounds.height / 2.));

        state.windows(2).for_each(|p| {
            if let [current_point, to] = p {
                let a = Point::new((current_point.x + to.x) / 2., current_point.y);
                let b = Point::new((current_point.x + to.x) / 2., to.y);

                back.bezier_curve_to(
                    Point::new(a.x, -a.y + bounds.height),
                    Point::new(b.x, -b.y + bounds.height),
                    Point::new(to.x, -to.y + bounds.height),
                );
                path.bezier_curve_to(a, b, *to);

                #[cfg(debug_assertions)]
                {
                    const RADIUS: f32 = 2.;
                    debug_frame.fill(
                        &canvas::Path::circle(a, RADIUS),
                        Color::from_rgb(1., 0., 0.),
                    );
                    debug_frame.fill(
                        &canvas::Path::circle(b, RADIUS),
                        Color::from_rgb(0., 1., 0.),
                    );
                    debug_frame.fill(
                        &canvas::Path::circle(*to, RADIUS),
                        Color::from_rgb(0., 0.5, 1.),
                    );
                };
            }
        });

        frame.stroke(
            &back.build(),
            Stroke::default()
                .with_color(mothscheme::BACKGROUND_L80.scale_alpha(0.6))
                .with_line_cap(LineCap::Round)
                .with_width(8.),
        );
        frame.stroke(
            &path.build(),
            Stroke::default()
                .with_color(mothscheme::BACKGROUND_L80)
                .with_line_cap(LineCap::Round)
                .with_width(8.),
        );

        vec![frame.into_geometry(), debug_frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        _event: &iced::Event,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Option<Action<T>> {
        if self.0.is_empty() {
            return None;
        }

        let half_height = bounds.height / 2.;
        let start_x = 0.;
        let end_x = bounds.width;

        state.clear();
        state.push(Point::new(start_x, half_height));
        state.extend(self.0.iter().enumerate().map(|(idx, f)| {
            Point::new(
                start_x + (idx + 1) as f32 * Self::WAVE_WIDTH,
                half_height - f * (half_height * 0.85 - 4.) * if idx % 2 == 0 { -1. } else { 1. },
            )
        }));
        state.push(Point::new(end_x, half_height));

        Some(Action::capture())
    }
}
