use std::{collections::HashMap, fmt::Debug, fs, process::Stdio, time::SystemTime};

use iced::{
    Alignment::Center,
    Border, Color, Element,
    Length::Fill,
    Task,
    widget::{column, text_input},
};
use iced_exwlshell::{
    layershell::application,
    reexport::{Anchor, Layer},
    settings::{LayerShellSettings, LayerSize},
};
use smol::process::Command;
use waybracelet::units;

use crate::{desktop_entries::DesktopEntry, frecency::FrecencyScore};

mod desktop_entries;
mod frecency;
mod input_type;

const TEXT_INPUT: &str = "text_input";

fn main() {
    application(
        SpotLight::new,
        "spotlight",
        SpotLight::update,
        SpotLight::view,
    )
    .settings(iced_exwlshell::Settings {
        layer_settings: LayerShellSettings {
            size: LayerSize::px(720, 480),
            anchor: Anchor::empty(),
            layer: Layer::Top,
            ..Default::default()
        },
        keep_compositor_alive: false,
        ..Default::default()
    })
    .subscription(|_| iced::event::listen().map(SpotLightMessage::IcedEvent))
    .theme(waybracelet::theme::styx())
    .style(|_, _| waybracelet::theme::transparent_style())
    .run()
    .unwrap();
}

#[iced_exwlshell::to_exwlshell_message]
#[derive(Debug, Clone)]
enum SpotLightMessage {
    ChangeInput(String),
    Exec(String),
    IcedEvent(iced::Event),
    FocusTextInput,
}

struct SpotLight {
    raw_input: String,
    pub input: input_type::InputType,
    pub all_entries: HashMap<String, desktop_entries::DesktopEntry>,
    db: sled::Db,
}

impl SpotLight {
    fn new() -> (Self, Task<SpotLightMessage>) {
        let data_dir = dirs::data_dir()
            .unwrap()
            .join("waybracelet")
            .join("spotlight_sled");
        fs::create_dir_all(&data_dir).unwrap();
        let db = sled::open(data_dir).unwrap();
        let entries = freedesktop_desktop_entry::desktop_entries(
            &freedesktop_desktop_entry::get_languages_from_env(),
        );

        let mut entries: HashMap<String, desktop_entries::DesktopEntry> = entries
            .into_iter()
            .filter(DesktopEntry::should_show)
            .filter_map(|d| d.try_into().ok())
            .map(|de: desktop_entries::DesktopEntry| (de.id.clone(), de))
            .collect();

        for entry in entries.values_mut() {
            if let Ok(Some(fs)) = db.get(&entry.id) {
                let fs = FrecencyScore::from_bytes(&fs);
                entry.score.replace(fs);
            }
        }
        let mut ids: Vec<String> = entries.keys().cloned().collect();
        sort_ids_by_frecency(&mut ids, &entries);

        (
            Self {
                raw_input: String::new(),
                input: input_type::InputType::DesktopEntries(ids),
                all_entries: entries,
                db,
            },
            Task::done(SpotLightMessage::FocusTextInput),
        )
    }

    fn update_input(&mut self, new_input: String) -> Task<SpotLightMessage> {
        self.raw_input = new_input;

        if let Some(math) = self.raw_input.strip_prefix('=') {
            if let Ok(res) = meval::eval_str(math) {
                self.input = input_type::InputType::Math(res);
            }
        } else {
            let mut ids: Vec<_> = self
                .all_entries
                .values()
                .filter(|de| {
                    self.raw_input
                        .chars()
                        .all(|c| de.title.chars().any(|c2| c.eq_ignore_ascii_case(&c2)))
                })
                .map(desktop_entries::DesktopEntry::id)
                .collect();

            sort_ids_by_frecency(&mut ids, &self.all_entries);

            self.input = input_type::InputType::DesktopEntries(ids)
        }

        Task::none()
    }

    fn update(&mut self, message: SpotLightMessage) -> Task<SpotLightMessage> {
        match message {
            SpotLightMessage::ChangeInput(new_input) => self.update_input(new_input),
            SpotLightMessage::FocusTextInput => iced::widget::operation::focus(TEXT_INPUT),
            SpotLightMessage::Exec(id) => {
                let now = SystemTime::now();
                if let Some(entry) = self.all_entries.get_mut(&id) {
                    entry.score.update_at(now);
                    let exec = &entry.exec;
                    let _ = self.db.insert(&entry.id, &entry.score.to_bytes());

                    Command::new(&exec[0])
                        .args(&exec[1..])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap();

                    return iced::exit();
                }

                Task::none()
            }
            SpotLightMessage::IcedEvent(iced::Event::Window(iced::window::Event::Unfocused)) => {
                iced::exit()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> impl Into<Element<'_, SpotLightMessage>> {
        let text_input = text_input("...", &self.raw_input)
            .on_input(SpotLightMessage::ChangeInput)
            .style(|theme: &iced::Theme, _| text_input::Style {
                background: theme.palette().background.into(),
                border: Border::default().rounded(1000),
                icon: Color::BLACK,
                placeholder: Color::BLACK,
                value: Color::WHITE,
                selection: Color::BLACK,
            })
            .padding([8, 12])
            .id(TEXT_INPUT);

        column!(
            waybracelet::components::canvas_background::ring(text_input),
            waybracelet::components::colored_bar::vertical(units::STROKE_WIDTH, 8.),
            waybracelet::components::canvas_background::ring(
                waybracelet::components::canvas_background::pill(input_type::input_type_to_view(
                    &self.input,
                    self
                ))
                .clip(true)
            )
        )
        .align_x(Center)
        .width(Fill)
        .height(Fill)
    }
}

fn sort_ids_by_frecency(ids: &mut [String], entries: &HashMap<String, DesktopEntry>) {
    let now = SystemTime::now();
    ids.sort_by(|de1, de2| {
        let de1 = &entries[de1];
        let de2 = &entries[de2];

        de1.score
            .score_at(now)
            .total_cmp(&de2.score.score_at(now))
            .reverse()
            .then_with(|| de1.title.cmp(&de2.title))
    });
}
