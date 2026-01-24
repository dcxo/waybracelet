use iced::{
    Border, Font, Shadow, Theme, border,
    font::{Family, Stretch, Style, Weight},
    theme::Palette,
    widget::container,
};

pub fn chord_style(theme: &iced::Theme) -> container::Style {
    container::Style {
        text_color: Some(theme.palette().text),
        background: Some(theme.palette().background.into()),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn bead_style(t: &iced::Theme) -> container::Style {
    chord_style(t).border(border::rounded(i32::MAX))
}

pub const DARK_PALETTE: Palette = Palette {
    background: mothscheme::BACKGROUND_L75,
    text: mothscheme::TEXT_L25,
    primary: mothscheme::ORANGE_L40,
    success: mothscheme::GREEN_L40,
    warning: mothscheme::YELLOW_L40,
    danger: mothscheme::RED_L40,
};

pub const LIGHT_PALETTE: Palette = Palette {
    background: mothscheme::BACKGROUND_L20,
    text: mothscheme::TEXT_L80,
    primary: mothscheme::ORANGE_L60,
    success: mothscheme::GREEN_L60,
    warning: mothscheme::YELLOW_L60,
    danger: mothscheme::RED_L60,
};

pub fn dark_theme() -> Option<Theme> {
    Some(Theme::custom("MothschemeStyx", DARK_PALETTE))
}

pub fn light_theme() -> Option<Theme> {
    Some(Theme::custom("MothschemeBombyx", LIGHT_PALETTE))
}

pub const BLACK_FONT: Font = Font {
    family: Family::SansSerif,
    weight: Weight::Black,
    stretch: Stretch::Normal,
    style: Style::Normal,
};
