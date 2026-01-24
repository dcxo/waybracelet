use iced::{
    Alignment::Center,
    Element,
    Length::{self},
    widget::{Container, container, space},
};

use crate::styles;

pub struct BeadsChord {
    pub length: Length,
}

impl BeadsChord {
    pub const FILL: BeadsChord = BeadsChord {
        length: Length::Fill,
    };
    pub const W24: BeadsChord = BeadsChord {
        length: Length::Fixed(24.),
    };
}

impl<'a, T: 'a> From<BeadsChord> for Element<'a, T> {
    fn from(val: BeadsChord) -> Self {
        container(space())
            .height(8)
            .width(val.length)
            .style(styles::chord_style)
            .into()
    }
}

pub fn bead<'a, T: 'a>(content: impl Into<Element<'a, T>>) -> Container<'a, T> {
    container(content).style(styles::bead_style).height(56)
}

pub fn bead_center<'a, T: 'a>(content: impl Into<Element<'a, T>>) -> Container<'a, T> {
    bead(content).align_y(Center).align_x(Center)
}
