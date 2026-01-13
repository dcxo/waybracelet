#![allow(mismatched_lifetime_syntaxes)]

use chrono::{DateTime, Local};
use iced::{
    Color, Element,
    Length::Fill,
    Subscription, Task,
    theme::Style,
    widget::{container, space},
    window,
};
use iced_layershell::{
    Settings,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};

use crate::windows::{VolumeOSD, WindowView};

mod components;
mod subscriptions;
mod windows;

fn main() {
    let conn = wayland_client::Connection::connect_to_env().unwrap();
    iced_layershell::daemon(Daemon::new, || "Holi".into(), Daemon::update, Daemon::view)
        .subscription(Daemon::subscriptions)
        .style(|_, __| Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
        })
        .settings(Settings {
            antialiasing: true,
            with_connection: Some(conn),
            layer_settings: LayerShellSettings {
                // anchor: Anchor::Top | Anchor::Left | Anchor::Right,
                // size: Some((0, 64)),
                // exclusive_zone: 64,
                start_mode: StartMode::Background,
                //TargetScreen("DP-3".to_string()),
                // layer: Layer::Bottom,
                // keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
        .unwrap()
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    CavaInfo(Vec<f32>),
    UpdateDatetime(DateTime<Local>),
    UpdateCurrenWorkspace(i32),
    PlayerState(bool),
}

struct Daemon {
    main_status_bar: windows::Window<windows::StatusBar>,
    volume_osd: windows::Window<windows::VolumeOSD>,
}

impl Daemon {
    fn new() -> (Self, Task<Message>) {
        let status_bar = windows::StatusBar::new("DP-3");
        let (main_status_bar, open_task) = status_bar.open_window();

        let (volume_osd, volume_open_task) = VolumeOSD.open_window();
        let open_task = open_task.chain(volume_open_task);

        (
            Self {
                main_status_bar,
                volume_osd,
            },
            open_task,
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CavaInfo(info) => {
                self.main_status_bar.view.cava_info = info;
                Task::none()
            }
            Message::UpdateDatetime(datetime) => {
                self.main_status_bar.view.current_datetime = datetime;
                Task::none()
            }
            Message::UpdateCurrenWorkspace(workspace) => {
                self.main_status_bar.view.current_worksapce = workspace;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        match window_id {
            x if x == self.main_status_bar.id => (&self.main_status_bar.view).into(),
            x if x == self.volume_osd.id => (&self.volume_osd.view).into(),
            _ => container(space()).into(),
        }
    }

    fn subscriptions(&self) -> Subscription<Message> {
        Subscription::batch([
            subscriptions::cava_subscription(),
            subscriptions::clock_subscription(),
            subscriptions::hyprland_subscription("DP-3"),
        ])
    }
}
