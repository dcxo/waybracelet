use iced::{Animation, Length::Fill, Task, time::Instant, widget::container};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{Message, features::Feature};

mod components;

pub struct VolumeOSD {
    pub animation: Animation<f32>,
    now: Instant,
}

impl VolumeOSD {
    pub fn new(now: Instant) -> Self {
        Self {
            animation: Animation::new(0.).quick(),
            now,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VolumeOsdMessage {
    VolumeAppear,
    VolumeDissapear,
}

impl Feature for VolumeOSD {
    type InnerMessage = VolumeOsdMessage;

    fn layer_settings(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((500, 300)),
            layer: Layer::Overlay,
            anchor: Anchor::Bottom | Anchor::Right,
            events_transparent: true,
            keyboard_interactivity: KeyboardInteractivity::None,
            output_option: OutputOption::OutputName("DP-3".into()),
            exclusive_zone: Some(-1),
            ..Default::default()
        }
    }

    fn update(&mut self, message: VolumeOsdMessage) -> iced::Task<Message> {
        match message {
            VolumeOsdMessage::VolumeAppear => {
                self.animation.go_mut(1.0, self.now);
                Task::none()
            }
            VolumeOsdMessage::VolumeDissapear => {
                self.animation.go_mut(0.0, self.now);
                Task::none()
            }
        }
    }

    fn view(&self) -> impl Into<iced::Element<'_, Message>> {
        container(components::Volume {
            volume: 0.3,
            alpha: self
                .animation
                .interpolate_with(|f| f, std::time::Instant::now()),
        })
        .width(Fill)
        .height(Fill)
        .style(|_| container::Style {
            ..Default::default()
        })
    }

    fn set_now(&mut self, now: Instant) {
        self.now = now;
    }

    fn is_animating(&self) -> bool {
        self.animation.is_animating(self.now)
    }
}
