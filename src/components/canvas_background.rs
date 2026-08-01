use crate::units;
use iced::Element;
use iced::Length::Fill;
use iced::widget::{Container, container as iced_container};

pub fn pill<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    iced_container(content)
        .center(Fill)
        .style(pill_background_style)
}

pub fn ring<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    iced_container(content)
        .style(ring_outline)
        .padding(units::MARGIN)
}

pub(crate) fn pill_background_style(theme: &iced::Theme) -> iced_container::Style {
    iced_container::Style::default()
        .background(theme.palette().background)
        .border(iced::Border::default().rounded(units::INNER_CORNER_RADIUS))
}

fn ring_outline(theme: &iced::Theme) -> iced_container::Style {
    iced_container::Style {
        background: None,
        text_color: None,
        border: iced::Border::default()
            .color(theme.palette().background)
            .width(units::STROKE_WIDTH)
            .rounded(units::CORNER_RADIUS),
        shadow: iced::Shadow::default(),
        snap: false,
    }
}
