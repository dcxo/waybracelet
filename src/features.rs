use std::time::Instant;

use iced::{Element, Subscription, Task, window};
use iced_layershell::reexport::NewLayerShellSettings;

use crate::{Message, Window};

pub trait Feature: Sized {
    type InnerMessage;

    fn layer_settings(&self) -> NewLayerShellSettings;

    fn update(&mut self, message: Self::InnerMessage) -> Task<Message>;
    fn view(&self) -> impl Into<Element<'_, Message>>;

    fn is_animating(&self) -> bool {
        false
    }

    fn open(self) -> (Window<Self>, Task<Message>) {
        let id = window::Id::unique();
        dbg!(id);
        let settings = self.layer_settings();

        (
            Window { id, view: self },
            Task::done(Message::NewLayerShell { settings, id }),
        )
    }

    fn subscriptions(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn set_now(&mut self, now: Instant);
}

pub mod notifications;
pub mod power_menu;
pub mod status_bar;
pub mod volume_osd;
