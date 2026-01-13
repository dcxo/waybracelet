use chrono::{DateTime, Local};
use iced::{
    Alignment::Center,
    Element, Font,
    Length::Fill,
    font::{Family, Stretch, Style, Weight},
    widget::{center, column, text},
};

pub mod beads;
mod styles;
pub mod waves_player;

const BLACK_FONT: Font = Font {
    family: Family::SansSerif,
    weight: Weight::Black,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub fn workspace<'a, T: 'a>(workspace: i32) -> impl Into<Element<'a, T>> {
    beads::bead_center(text!("{}", workspace).font(BLACK_FONT).size(24)).width(56)
}

fn clock_text<'a, T: 'a>(datetime: &DateTime<Local>, format: &str) -> impl Into<Element<'a, T>> {
    text!("{}", datetime.format(format))
        .align_x(Center)
        .width(Fill)
        .font(BLACK_FONT)
}

pub fn clock<'a, T: 'a>(datetime: &DateTime<Local>) -> impl Into<Element<'a, T>> {
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
