use iced::{
    Border, Shadow,
    border::{self, Radius},
    widget::container,
};

pub(super) fn chord_style(_: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(mothscheme::TEXT_L20),
        background: Some(mothscheme::BACKGROUND_L80.into()),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub(super) fn bead_style(t: &iced::Theme) -> container::Style {
    chord_style(t).border(border::rounded(i32::MAX))
}
