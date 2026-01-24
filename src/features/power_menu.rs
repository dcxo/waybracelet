use std::time::Instant;

use iced::{
    Element,
    Length::Fill,
    Task,
    alignment::Vertical,
    widget::{center, container, mouse_area, row, space},
};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use lucide_icons::iced::{icon_log_out, icon_moon, icon_power, icon_rotate_ccw, icon_rotate_cw};
use smol::process::Command;

use crate::{Message, components::BeadsChord, features::Feature};

mod components;

use components::power_button;

#[derive(Debug)]
pub struct PowerMenu {
    now: Instant,
}

#[derive(Debug, Clone)]
pub enum PowerMenuMessage {
    Shutdown,
    Reboot,
    Suspend,
    Logout,
}

impl PowerMenu {
    pub fn new(now: Instant) -> PowerMenu {
        PowerMenu { now }
    }

    fn execute_command(mut cmd: Command) -> Task<Message> {
        Task::future(async move {
            let mut child = cmd.spawn().unwrap();
            child.status().await.unwrap();

            Message::Animation
        })
    }
}

impl Feature for PowerMenu {
    type InnerMessage = PowerMenuMessage;

    fn layer_settings(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((0, 0)),
            layer: Layer::Overlay,
            anchor: Anchor::all(),
            exclusive_zone: Some(-1),
            namespace: Some("power_menu".to_string()),
            keyboard_interactivity: KeyboardInteractivity::None,
            ..Default::default()
        }
    }

    fn update(&mut self, message: PowerMenuMessage) -> iced::Task<Message> {
        match message {
            PowerMenuMessage::Shutdown => Self::execute_command({
                let mut cmd = Command::new("shutdown");
                cmd.arg("now");
                cmd
            }),
            PowerMenuMessage::Reboot => Self::execute_command(Command::new("reboot")),
            PowerMenuMessage::Suspend => Self::execute_command({
                let mut cmd = Command::new("systemctl");
                cmd.arg("hybrid-sleep");
                cmd
            }),
            PowerMenuMessage::Logout => Self::execute_command({
                let mut cmd = Command::new("loginctl");
                cmd.arg("terminate-session");
                cmd.arg(std::env::var("XDG_SESSION_ID").unwrap());
                cmd
            }),
        }
    }

    fn view(&self) -> impl Into<Element<'_, Message>> {
        mouse_area(
            center(
                row![
                    BeadsChord::FILL,
                    power_button(icon_log_out(), PowerMenuMessage::Logout).into(),
                    BeadsChord::W24,
                    power_button(icon_power(), PowerMenuMessage::Shutdown).into(),
                    BeadsChord::W24,
                    power_button(icon_rotate_cw(), PowerMenuMessage::Reboot).into(),
                    BeadsChord::W24,
                    power_button(icon_moon(), PowerMenuMessage::Suspend).into(),
                    BeadsChord::FILL,
                ]
                .align_y(Vertical::Center),
            )
            .width(Fill)
            .height(Fill)
            .style(|theme| container::Style {
                background: Some(theme.palette().text.scale_alpha(0.35).into()),
                ..Default::default()
            }),
        )
        .on_press(Message::Hide(crate::FeatureSelector::PowerMenu))
    }

    fn set_now(&mut self, now: std::time::Instant) {
        self.now = now;
    }
}
