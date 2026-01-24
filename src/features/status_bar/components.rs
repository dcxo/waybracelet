use chrono::{DateTime, Local};
use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Point, Renderer,
    widget::{
        canvas::{self, Action, Frame, LineCap, Program, Stroke, path},
        center, column, text,
    },
};

use crate::{components::bead_center, styles::BLACK_FONT};

pub fn workspace<'a, T: 'a>(workspace: i32) -> impl Into<Element<'a, T>> {
    bead_center(text!("{}", workspace).font(BLACK_FONT).size(24)).width(56)
}

fn clock_text<'a, T: 'a>(datetime: DateTime<Local>, format: &str) -> impl Into<Element<'a, T>> {
    text!("{}", datetime.format(format))
        .align_x(Center)
        .width(Fill)
        .font(BLACK_FONT)
}

pub fn clock<'a, T: 'a>(datetime: DateTime<Local>) -> impl Into<Element<'a, T>> {
    center(
        column![
            clock_text(datetime, "%H").into(),
            clock_text(datetime, "%M").into(),
        ]
        .spacing(-2.),
    )
    .width(56)
    .height(56)
}

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
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let stroke = Stroke::default()
            .with_color(theme.palette().background)
            .with_line_cap(LineCap::Round)
            .with_width(8.);
        let mut frame = Frame::new(renderer, bounds.size());
        if state.is_empty() {
            frame.stroke(
                &path::Path::line(
                    Point::new(0., bounds.height / 2.0),
                    Point::new(bounds.width, bounds.height / 2.0),
                ),
                stroke,
            );
        }

        #[cfg(all(debug_assertions, feature = "debug"))]
        let mut debug_frame = Frame::new(renderer, bounds.size());

        let mut path = path::Builder::new();
        let mut back = path::Builder::new();

        let start_point = Point::new(0., bounds.height / 2.);
        path.move_to(start_point);
        back.move_to(start_point);

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

                #[cfg(all(debug_assertions, feature = "debug"))]
                {
                    use iced::Color;

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

        let back_stroke = Stroke::default()
            .with_color(theme.palette().background.scale_alpha(0.6))
            .with_line_cap(LineCap::Round)
            .with_width(8.);
        frame.stroke(&back.build(), back_stroke);
        frame.stroke(&path.build(), stroke);

        vec![
            frame.into_geometry(),
            #[cfg(all(debug_assertions, feature = "debug"))]
            debug_frame.into_geometry(),
        ]
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
            let direction = if idx % 2 == 0 { -1. } else { 1. };
            Point::new(
                start_x + (idx + 1) as f32 * Self::WAVE_WIDTH,
                half_height - f * (half_height * 0.85 - 4.) * direction,
            )
        }));
        state.push(Point::new(end_x, half_height));

        Some(Action::request_redraw())
    }
}
