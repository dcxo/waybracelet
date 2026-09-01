use iced::{
    Alignment::Center,
    Element, Font,
    Length::{Fill, Shrink},
    widget::{column, grid, scrollable, text},
};

use super::{SpotLight, SpotLightMessage};

#[derive(Debug)]
pub enum InputType {
    DesktopEntries(Vec<String>),
    Math(f64),
}

pub fn input_type_to_view<'a>(
    input: &'a InputType,
    spotlight: &'a SpotLight,
) -> Element<'a, SpotLightMessage> {
    match input {
        InputType::DesktopEntries(s) => {
            let children = s
                .iter()
                .filter_map(|id| spotlight.all_entries.get(id))
                .map(Into::into);

            scrollable(grid(children).columns(5))
                .width(Fill)
                .height(Shrink)
                .into()
        }
        InputType::Math(math) => {
            let f = numfmt::Formatter::new()
                .precision(numfmt::Precision::Significance(5))
                .separator('_')
                .unwrap();

            column![
                text(format!("{math} ="))
                    .size(16)
                    .font(Font::MONOSPACE)
                    .color(mothscheme::swatches::BLUE_L40),
                text(f.fmt_string(*math))
                    .size(40)
                    .font(Font {
                        weight: iced::font::Weight::ExtraBold,
                        ..Font::with_name("Inter")
                    })
                    .wrapping(text::Wrapping::Glyph)
            ]
            .width(Fill)
            .align_x(Center)
            .into()
        }
    }
}
