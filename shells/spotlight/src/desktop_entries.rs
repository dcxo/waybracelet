use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    widget::{button, column, container, image, svg, text},
};

use super::SpotLightMessage;

use crate::frecency;

#[derive(Debug, thiserror::Error)]
pub enum DesktopEntryError {
    #[error("The title was not found")]
    TitleNotFound,

    #[error("Could not parse exec for desktop entry")]
    ExecError(#[from] freedesktop_desktop_entry::ExecError),

    #[error("Should have an icon")]
    IconNotFound,
}

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    pub score: frecency::FrecencyScore,
    pub id: String,
    pub title: String,
    pub exec: Vec<String>,
    pub dbus_activatable: bool,
    pub terminal: bool,
    pub icon: String,
}

impl DesktopEntry {
    pub(crate) fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn should_show(de: &freedesktop_desktop_entry::DesktopEntry) -> bool {
        if de.no_display()
            || de.type_() != Some("Application")
            || de.hidden()
            || de.exec().is_none()
        {
            return false;
        }
        if let Some(only_show) = de.only_show_in() {
            let current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
            return only_show.iter().any(|os| os == &current_desktop);
        }
        if let Some(not_show) = de.not_show_in() {
            let current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
            return !not_show.iter().any(|os| os == &current_desktop);
        }

        true
    }
}

impl TryFrom<freedesktop_desktop_entry::DesktopEntry> for DesktopEntry {
    type Error = DesktopEntryError;

    fn try_from(value: freedesktop_desktop_entry::DesktopEntry) -> Result<Self, Self::Error> {
        let icon = {
            let icon_name = value.icon().ok_or(DesktopEntryError::IconNotFound)?;

            let icon = freedesktop_icons::lookup(icon_name)
                .with_size(64)
                .find()
                .ok_or(DesktopEntryError::IconNotFound)?;

            icon.display().to_string()
        };
        let terminal = value.terminal();
        let title = value
            .name(&freedesktop_desktop_entry::get_languages_from_env())
            .ok_or(DesktopEntryError::TitleNotFound)?
            .into();
        let exec = value.parse_exec()?;
        let dbus_activatable = value.dbus_activatable();

        Ok(DesktopEntry {
            score: frecency::FrecencyScore::new(),
            id: value.appid,
            title,
            exec,
            dbus_activatable,
            terminal,
            icon,
        })
    }
}

impl<'a> From<&'a DesktopEntry> for Element<'a, SpotLightMessage> {
    fn from(value: &'a DesktopEntry) -> Self {
        let image: Element<'_, _> = if value.icon.ends_with(".svg") {
            svg(&value.icon)
                .height(Fill)
                .width(Fill)
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else {
            image(&value.icon)
                .width(Fill)
                .height(Fill)
                .filter_method(image::FilterMethod::Nearest)
                .content_fit(iced::ContentFit::Contain)
                .into()
        };

        button(
            column![
                image,
                container(text(&value.title).wrapping(text::Wrapping::None)).width(Shrink)
            ]
            .spacing(12)
            .width(Fill)
            .height(Fill)
            .align_x(Center),
        )
        .clip(true)
        .on_press_with(|| SpotLightMessage::Exec(value.id.clone()))
        .width(Fill)
        .into()
    }
}
