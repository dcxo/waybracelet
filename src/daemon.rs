use std::{iter::once, ops::IndexMut};

use crate::{
    Message,
    windows::{
        edge_bar::{EdgeBar, Position},
        leaving::WeLeaving,
        notifications::Notifications,
        shell_window::{Kind, ShellWindow, Window},
        spotlight::SpotLight,
        status_bar::{StatusBar, StatusBarComponents},
    },
};
use iced::widget::space;
use iced::{Element, Task, window};

pub struct Daemon {
    windows: Vec<Window>,
}

impl Daemon {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks: Vec<Task<Message>> = Vec::new();

        let status_bar = StatusBar::new();
        let (window, task) = status_bar.open();
        tasks.push(task);
        let mut windows = vec![window];

        let mut status_bar = StatusBar::new();
        status_bar.monitor = "DP-1".to_string();
        status_bar.components = StatusBarComponents::TIME;
        status_bar.is_main = false;
        let (window, task) = status_bar.open();
        tasks.push(task);
        windows.push(window);

        for pos in [Position::Left, Position::Right, Position::Bottom] {
            let eb = EdgeBar::new(pos);
            let (window, task) = eb.open();
            tasks.push(task);
            windows.push(window);
        }

        let spotlight = SpotLight::new();
        let (window, _) = spotlight.open();
        windows.push(window);

        (Daemon { windows }, Task::batch(tasks))
    }

    fn open_if_absent(&mut self, kind: Kind) -> Task<Message> {
        if self.windows.iter().any(|w| w.kind() == kind) {
            return Task::none();
        }

        match kind {
            Kind::EdgeBar | Kind::StatusBar | Kind::Leaving => todo!(),
            Kind::Notification => {
                let (window, task) = Notifications::new().open();
                self.windows.push(window);
                task
            }
            Kind::SpotLight => {
                let (window, task) = SpotLight::default().open();
                self.windows.push(window);
                task
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenLeaving => {
                let (window, task) = WeLeaving::default().open();

                self.windows.push(window);

                task
            }
            Message::Close(id) => {
                self.windows.retain(|w| w.id != id);
                Task::done(Message::RemoveWindow(id))
            }
            Message::CloseByKind(kind) => {
                let to_remove = self.windows.extract_if(.., |w| w.kind() == kind);

                Task::batch(
                    to_remove
                        .map(|w| w.id)
                        .map(Message::RemoveWindow)
                        .map(Task::done),
                )
            }
            Message::OpenSpotLight => {
                if let Some(spotlight) = self.windows.iter().find(|w| w.kind() == Kind::SpotLight) {
                    return Task::done(Message::NewLayerShell {
                        settings: spotlight.layer_shell_settings(),
                        id: spotlight.id,
                    });
                }

                Task::none()
            }
            msg => {
                let open_task = if msg.is_notification_related() {
                    self.open_if_absent(Kind::Notification)
                } else if msg.is_spotlight_related() {
                    self.open_if_absent(Kind::SpotLight)
                } else {
                    Task::none()
                };

                let tasks = self.windows.iter_mut().map(|w| w.update(w.id, &msg));
                open_task.chain(Task::batch(tasks))
            }
        }
    }

    pub fn is_animating(&self) -> bool {
        self.windows.iter().any(|w| w.is_animating())
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        self.windows
            .iter()
            .find_map(|w| if w.id == id { Some(w.view(id)) } else { None })
            .unwrap_or_else(|| space().into())
    }
}
