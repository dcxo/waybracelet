use core::convert::Into;
use std::{
    borrow::Cow,
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
};

use iced::{
    Alignment::Center,
    Background, Border, Element, Function,
    Length::{Fill, Shrink},
    Task, Theme,
    border::Radius,
    futures::lock::MutexGuard,
    widget::{button, column, row, scrollable, sensor, space, text, text_input},
};
use iced_layershell::{
    actions::ActionCallback,
    reexport::{Anchor, BlurOption, KeyboardInteractivity, Layer, OutputOption},
    settings::LayerSize,
};

use crate::{
    Message,
    components::{
        canvas_background::{pill, ring},
        colored_bar,
    },
    units,
    windows::shell_window::ShellWindow,
};

pub enum SpotLightInput {
    DesktopEntries(String),
    Math(String),
}

impl Default for SpotLightInput {
    fn default() -> Self {
        Self::DesktopEntries(String::new())
    }
}

#[derive(Default)]
pub struct SpotLight {
    input: SpotLightInput,
    desktop_entries: Vec<freedesktop_desktop_entry::DesktopEntry>,
}

impl SpotLight {
    pub fn new() -> Self {
        let locales = freedesktop_desktop_entry::get_languages_from_env();
        Self {
            input: SpotLightInput::default(),
            desktop_entries: freedesktop_desktop_entry::desktop_entries(&locales),
        }
    }
}

impl ShellWindow for SpotLight {
    fn layer_shell_settings(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        iced_layershell::reexport::NewLayerShellSettings {
            size: LayerSize::px(700, 500),
            layer: Layer::Overlay,
            anchor: Anchor::empty(),
            exclusive_zone: None,
            margin: Some((0, 0, units::DESIGN_UNIT as i32, 0)),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            blur_option: BlurOption::None,
            output_option: OutputOption::Active,
            events_transparent: false,
            namespace: Some("SpotLight".into()),
        }
    }

    fn update(&mut self, id: iced::window::Id, msg: &Message) -> iced::Task<Message> {
        match msg {
            Message::ChangeInput(search) => {
                self.input = SpotLightInput::DesktopEntries(search.clone());
                Task::none()
            }
            Message::OnResize(rid, size) if *rid == id => {
                let size = *size;
                Task::done(Message::SetInputRegion {
                    id,
                    callback: ActionCallback::new(move |r| {
                        r.add(
                            0,
                            250 - (size.height / 2.) as i32,
                            size.width as i32,
                            size.height as i32,
                        );
                    }),
                })
            }
            Message::Exec(exec) => {
                let _ = Command::new("setsid")
                    .arg("-f")
                    .arg("sh")
                    .arg("-c")
                    .arg(exec.join(" "))
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
                let _ = std::mem::take(&mut self.input);
                Task::done(Message::RemoveWindow(id))
            }
            Message::EscapePressed => {
                let _ = std::mem::take(&mut self.input);
                Task::done(Message::RemoveWindow(id))
            }
            _ => Task::none(),
        }
    }

    fn view(&self, id: iced::window::Id) -> iced::Element<'_, Message> {
        let text_input = ring(
            text_input(
                "Type in here…",
                match self.input {
                    SpotLightInput::DesktopEntries(ref input) => input,
                    SpotLightInput::Math(ref input) => input,
                },
            )
            .on_input(Message::ChangeInput)
            .style(|theme: &Theme, _| text_input::Style {
                background: Background::from(theme.palette().background),
                border: Border::default().rounded(Radius::new(units::INNER_CORNER_RADIUS)),
                icon: theme.palette().text,
                placeholder: mothscheme::themes::styx::SUBTLE,
                value: theme.palette().text,
                selection: mothscheme::swatches::BLUE_L40,
            })
            .padding([6, 16]),
        )
        .width(Fill)
        .height(Shrink);

        let locales = freedesktop_desktop_entry::get_languages_from_env();
        let mut desktop_entries = self
            .desktop_entries
            .iter()
            .filter(should_show)
            // .inspect(|de| {
            //     dbg!((de.name(&locales), de.exec(), de.type_(), de.terminal()));
            // })
            .filter_map(|de| Some((de.parse_exec().ok()?, de.name(&locales)?)))
            .peekable();
        // let grid = grid(vec![space().into()]).columns(5);

        // let container = ring(pill(space()).width(Fill).height(300.));

        let mut entries_column = column![text_input,];

        if desktop_entries.peek().is_some() {
            let child = ring(
                pill(scrollable(
                    column(desktop_entries.map(desktop_entry_view))
                        .width(Fill)
                        .spacing(8),
                ))
                .padding(8),
            )
            .width(Fill);
            entries_column = entries_column.push(colored_bar::vertical(units::STROKE_WIDTH, 8.));
            entries_column = entries_column.push(child);
        }

        column!(
            space().height(Fill),
            sensor(entries_column.height(Shrink).align_x(Center),)
                .on_resize(Message::OnResize.with(id)),
            space().height(Fill)
        )
        .into()
    }

    fn kind(&self) -> super::shell_window::Kind {
        super::shell_window::Kind::SpotLight
    }
}

fn desktop_entry_view<'a>((exec, name): (Vec<String>, Cow<'a, str>)) -> Element<'a, Message> {
    button(text(name))
        .width(Fill)
        .on_press(Message::Exec(exec.clone()))
        .into()
}

fn should_show(de: &&freedesktop_desktop_entry::DesktopEntry) -> bool {
    if de.type_() != Some("Application") {
        return false;
    }
    if de.hidden() {
        return false;
    }
    if de.no_display() {
        return false;
    }
    if de.exec().is_none() {
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
