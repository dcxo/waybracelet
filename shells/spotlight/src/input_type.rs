use std::{convert::Infallible, process::Command};

use iced::{
    Alignment::Center,
    Element, Font,
    Length::{Fill, Shrink},
    widget::{button, column, grid, scrollable, text},
};

use crate::{desktop_entries, sort_ids_by_frecency};

use super::{SpotLight, SpotLightMessage};

#[derive(Debug)]
pub enum InputType {
    DesktopEntries(Vec<String>),
    Math(f64),
    Projects(Vec<String>),
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
        InputType::Projects(items) => {
            let children = items
                .iter()
                .map(|p| {
                    button(text(p.replace("%%", "/")))
                        .width(Fill)
                        .on_press(SpotLightMessage::OpenProject(p.clone()))
                })
                .map(Into::into);
            scrollable(column(children)).width(Fill).height(Fill).into()
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Input Type Error")]
pub struct InputTypeError;

impl<'a> TryFrom<&SpotLight> for InputType {
    type Error = InputTypeError;

    fn try_from(value: &SpotLight) -> Result<Self, Self::Error> {
        let raw_input = &value.raw_input;
        match raw_input.split_at_checked(1) {
            Some(("=", math)) => {
                if let Ok(res) = meval::eval_str(math) {
                    Ok(InputType::Math(res))
                } else {
                    Err(InputTypeError)
                }
            }
            Some(("@", project_name)) => {
                let child = Command::new("project-opener")
                    .arg("list")
                    .arg("--json")
                    .output()
                    .unwrap()
                    .stdout;

                let v: Vec<String> = serde_json::from_slice(&child).unwrap();

                Ok(InputType::Projects(v))
            }
            Some(_) => {
                let mut ids: Vec<_> = value
                    .all_entries
                    .values()
                    .filter(|de| {
                        value
                            .raw_input
                            .chars()
                            .all(|c| de.title.chars().any(|c2| c.eq_ignore_ascii_case(&c2)))
                    })
                    .map(desktop_entries::DesktopEntry::id)
                    .collect();

                sort_ids_by_frecency(&mut ids, &value.all_entries);

                Ok(InputType::DesktopEntries(ids))
            }
            _ => {
                let mut collect: Vec<_> = value.all_entries.keys().map(Into::into).collect();
                sort_ids_by_frecency(&mut collect, &value.all_entries);
                Ok(InputType::DesktopEntries(collect))
            }
        }
    }
}
