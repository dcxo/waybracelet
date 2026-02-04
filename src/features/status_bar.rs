use std::time::Instant;

use chrono::{DateTime, Local};
use iced::{
    Color,
    Length::{Fill, Shrink},
    Padding, Subscription, Task,
    alignment::Vertical,
    widget::{button, container, row},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use lucide_icons::iced::icon_box;
use wayland_client::protocol::wl_output::{self, WlOutput};

use crate::{FeatureSelector, Message, features::Feature};

mod components;
mod subscriptions;

#[derive(Clone, Debug)]
pub struct StatusBar {
    now: Instant,
    pub output: String,
    pub wloutput: WlOutput,
    pub(crate) cava_info: Vec<f32>,
    pub(crate) current_datetime: DateTime<Local>,
    pub(crate) current_workspace: i32,
}

impl StatusBar {
    pub fn new(
        output: impl Into<String>,
        wloutput: WlOutput,
        current_workspace: i32,
        now: Instant,
    ) -> Self {
        Self {
            now,
            output: output.into(),
            wloutput,
            cava_info: Vec::with_capacity(12),
            current_datetime: Local::now(),
            current_workspace,
        }
    }

    fn is_in_main(&self) -> bool {
        self.output == "DP-3"
    }
}

#[derive(Debug, Clone)]
pub enum StatusBarMessage {
    CavaInfo(Vec<f32>),
    UpdateDatetime(DateTime<Local>),
    UpdateCurrenWorkspace(String, i32),
}

impl Feature for StatusBar {
    type InnerMessage = StatusBarMessage;

    fn layer_settings(&self) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((0, 56)),
            layer: Layer::Top,
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            margin: Some((8, 0, 0, 0)),
            exclusive_zone: Some(56),
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::Output(self.wloutput.clone()),
            ..Default::default()
        }
    }

    fn update(&mut self, message: StatusBarMessage) -> iced::Task<Message> {
        match message {
            StatusBarMessage::CavaInfo(info) => {
                self.cava_info = info;
                Task::none()
            }
            StatusBarMessage::UpdateDatetime(datetime) => {
                self.current_datetime = datetime;
                Task::none()
            }
            StatusBarMessage::UpdateCurrenWorkspace(output, workspace) => {
                if output == self.output {
                    self.current_workspace = workspace;
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> impl Into<iced::Element<'_, Message>> {
        let padding = if self.is_in_main() { 8 } else { 0 };

        container(
            row![
                crate::components::BeadsChord::W24,
                components::workspace(self.current_workspace).into(),
                crate::components::BeadsChord::W24,
                components::CavaPlayer(&self.cava_info),
                crate::components::BeadsChord::FILL,
                crate::components::bead(
                    row![
                        components::clock(self.current_datetime).into(),
                        Some(
                            button(icon_box().size(32).center())
                                .style(|theme, status| {
                                    let mut style = button::primary(theme, status);
                                    style.text_color = theme.palette().text;
                                    style.with_background(Color::TRANSPARENT)
                                })
                                .on_press(Message::Open(FeatureSelector::PowerMenu))
                                .width(56)
                                .height(56)
                        )
                        .filter(|_| self.is_in_main())
                    ]
                    .spacing(-12.)
                )
                .padding(Padding::ZERO.right(padding).left(padding)),
                crate::components::BeadsChord::W24
            ]
            .align_y(Vertical::Center),
        )
        .height(Shrink)
        .width(Fill)
    }

    fn subscriptions(&self) -> iced::Subscription<Message> {
        Subscription::batch([
            subscriptions::cava_subscription(),
            subscriptions::clock_subscription(),
            subscriptions::hyprland_subscription(self.output.clone()),
        ])
        .map(Message::StatusBar)
    }

    fn set_now(&mut self, now: Instant) {
        self.now = now;
    }
}
