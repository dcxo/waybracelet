use iced::widget::{container, space};
use iced::{Length, Length::Fill};

use crate::units;

pub fn colored_bar_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(theme.palette().background)),
        ..Default::default()
    }
}

pub fn h24<'a, Message: 'a>() -> container::Container<'a, Message> {
    horizontal(units::STROKE_WIDTH, units::DESIGN_UNIT)
}

pub fn h8<'a, Message: 'a>() -> container::Container<'a, Message> {
    horizontal(units::STROKE_WIDTH, units::MARGIN)
}

pub fn horizontal<'a, Message: 'a>(
    stroke_width: f32,
    width: f32,
) -> container::Container<'a, Message> {
    container(space())
        .width(Length::Fixed(width))
        .height(Length::Fixed(stroke_width))
        .style(colored_bar_style)
}

pub fn horizontal_fill<'a, Message: 'a>(stroke_width: f32) -> container::Container<'a, Message> {
    container(space())
        .width(Fill)
        .height(Length::Fixed(stroke_width))
        .style(colored_bar_style)
}

pub fn vertical<'a, Message: 'a>(
    stroke_width: f32,
    height: f32,
) -> container::Container<'a, Message> {
    container(space())
        .width(Length::Fixed(stroke_width))
        .height(Length::Fixed(height))
        .style(colored_bar_style)
}

pub fn vertical_fill<'a, Message: 'a>(stroke_width: f32) -> container::Container<'a, Message> {
    container(space())
        .width(Length::Fixed(stroke_width))
        .height(Fill)
        .style(colored_bar_style)
}
