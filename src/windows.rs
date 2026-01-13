use std::{clone::Clone, convert::From, marker::PhantomData};

use chrono::{DateTime, Local};
use iced::{
    Color, Element,
    Length::Fill,
    Padding, Task,
    alignment::Vertical,
    widget::{bottom, button, container, container::Style, mouse_area, row, space},
    window,
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use lucide_icons::iced::icon_snowflake;

use super::Message;
use crate::components::{beads, clock, waves_player, workspace};

pub(super) struct Window<T>
where
    T: WindowView,
    for<'a> Element<'a, Message>: From<&'a T>,
{
    pub(super) id: window::Id,
    pub(super) view: T,
}

pub(super) trait WindowView: Sized
where
    for<'a> Element<'a, Message>: From<&'a Self>,
{
    fn open_window(self) -> (Window<Self>, Task<Message>) {
        let id = window::Id::unique();
        let settings = self.layer_shell_settings();

        (
            Window { id, view: self },
            Task::done(Message::NewLayerShell { settings, id }),
        )
    }
    fn layer_shell_settings(&self) -> NewLayerShellSettings;
}

#[derive(Default, Clone, Debug)]
pub(super) struct StatusBar {
    // output: String,
    pub(crate) cava_info: Vec<f32>,
    pub(crate) current_datetime: DateTime<Local>,
    pub(crate) current_worksapce: i32,
}

impl WindowView for StatusBar {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((0, 64)),
            layer: Layer::Top,
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            exclusive_zone: Some(64),
            margin: None,
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::None,
            events_transparent: false,
            namespace: Some("bar".to_string()),
        }
    }
}

impl<'a> From<&'a StatusBar> for Element<'a, Message> {
    fn from(value: &'a StatusBar) -> Self {
        bottom(
            row![
                beads::BeadsChord::W24,
                workspace(value.current_worksapce).into(),
                beads::BeadsChord::W24,
                waves_player::CavaPlayer(&value.cava_info),
                beads::BeadsChord::FILL,
                beads::bead(
                    row![
                        clock(&value.current_datetime).into(),
                        button(icon_snowflake().size(36).center())
                            .style(|theme, status| {
                                let mut style = button::primary(theme, status);
                                style.text_color = mothscheme::TEXT_L20;
                                style.with_background(Color::TRANSPARENT)
                            })
                            .width(56)
                            .height(56)
                    ]
                    .spacing(-12.)
                )
                .padding(Padding::ZERO.right(8).left(8)),
                beads::BeadsChord::W24
            ]
            .align_y(Vertical::Center)
            .width(Fill),
        )
        .height(Fill)
        .align_y(Vertical::Bottom)
        .into()
    }
}

impl StatusBar {
    pub(super) fn new(_output: impl Into<String>) -> Self {
        Self {
            // output,
            cava_info: Vec::with_capacity(12),
            current_datetime: Local::now(),
            current_worksapce: 1,
        }
    }
}

pub struct VolumeOSD;

impl WindowView for VolumeOSD {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((500, 300)),
            layer: Layer::Top,
            anchor: Anchor::Bottom | Anchor::Right,
            exclusive_zone: None,
            margin: None,
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::None,
            events_transparent: true,
            namespace: None,
        }
    }
}

impl<'a> From<&VolumeOSD> for Element<'a, Message> {
    fn from(_: &VolumeOSD) -> Self {
        container(space())
            .width(Fill)
            .height(Fill)
            .style(|_| Style {
                background: Some(mothscheme::TEXT_L10.into()),
                ..Default::default()
            })
            .into()
    }
}

struct PowerMenu;

impl WindowView for PowerMenu {
    fn layer_shell_settings(&self) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((0, 0)),
            layer: Layer::Overlay,
            anchor: Anchor::all(),
            exclusive_zone: None,
            margin: None,
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::None,
            events_transparent: false,
            namespace: Some("power_menu".to_string()),
        }
    }
}

impl<'a> From<&'a PowerMenu> for Element<'a, Message> {
    fn from(_: &'a PowerMenu) -> Self {
        mouse_area(container(space()).width(Fill).height(Fill)).into()
    }
}
