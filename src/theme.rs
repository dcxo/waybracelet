use iced::{Color, Font};
use mothscheme::themes::styx::{BLUE, GREEN, OVERLAY, RED, TEXT, YELLOW};

pub const DEFAULT_FONT: Font = Font::with_name("IBM Plex Serif");

pub fn styx() -> iced::Theme {
    iced::Theme::custom(
        "mothscheme-styx",
        iced::theme::Palette {
            background: OVERLAY,
            text: TEXT,
            primary: BLUE,
            success: GREEN,
            warning: YELLOW,
            danger: RED,
        },
    )
}

pub fn transparent_style() -> iced::theme::Style {
    iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    }
}
