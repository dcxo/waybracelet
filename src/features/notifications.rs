use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use iced::{
    Animation, Color,
    Length::{Fill, Shrink},
    Padding, Shadow, Task, Vector,
    alignment::Horizontal,
    animation::Easing,
    border::{Radius, rounded},
    time::{milliseconds, seconds},
    widget::{container, float, keyed::Column, sensor, stack, value},
};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use smol::{Timer, channel::Sender};
use zbus::zvariant;

use crate::{Message, components::bead_center, features::Feature};

mod components;
pub mod subscriptions;

#[derive(Debug, Clone, Copy)]
pub enum ExpireTimeout {
    ServerDefault,
    NotificationSpecific(i32),
    ManualClosing,
}

impl From<ExpireTimeout> for Option<Duration> {
    fn from(val: ExpireTimeout) -> Self {
        match val {
            ExpireTimeout::ServerDefault => Some(seconds(5)),
            ExpireTimeout::NotificationSpecific(m) => Some(milliseconds(m as u64)),
            ExpireTimeout::ManualClosing => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(String, String)>,
    pub hints: HashMap<String, zvariant::OwnedValue>,
    pub expire_timeout: ExpireTimeout,
}

#[derive(Debug)]
pub struct AnimatedNotification {
    notification: Notification,
    animation: Animation<f32>,
}

impl AnimatedNotification {
    pub fn new(notification: Notification) -> Self {
        Self {
            notification,
            animation: Animation::new(1.).quick().easing(Easing::Linear),
        }
    }
}

pub struct Notifications {
    notifications: BTreeMap<u32, AnimatedNotification>,
    animation: Animation<f32>,
    dbus_sender: Option<Sender<DbusEvents>>,
    pub now: Instant,
}

impl Notifications {
    const PAD: f32 = 24.;
    const WIDTH: f32 = 500.;
    const ROUND: f32 = Self::PAD * 3.;

    pub fn new(notifications: BTreeMap<u32, AnimatedNotification>, now: Instant) -> Self {
        Self {
            notifications,
            animation: Animation::new(0.).quick().easing(Easing::Linear),
            dbus_sender: None,
            now,
        }
    }

    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications
            .insert(notification.id, AnimatedNotification::new(notification));
    }

    pub fn start_animation(&mut self, id: u32) {
        self.animation.go_mut(0., self.now);
        if let Some(an) = self.notifications.get_mut(&id) {
            an.animation.go_mut(0., self.now);
        }
    }

    pub fn remove_notification(&mut self, id: u32) -> Option<Notification> {
        self.notifications.remove(&id).map(|n| n.notification)
    }
}

#[derive(Debug, Clone)]
pub enum NotificationsMessage {
    New(Notification),
    Close(u32),
    PopUp(u32),
    Remove(u32),

    DbusInterfaceReady(Sender<DbusEvents>),
}

pub enum DbusEvents {
    ActionInvoked(String, u32),
    CloseNotification(u32),
}

impl Feature for Notifications {
    type InnerMessage = NotificationsMessage;

    fn layer_settings(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((500, 0)),
            layer: Layer::Top,
            margin: Some((16, 0, 32, 0)),
            anchor: Anchor::Right | Anchor::Top | Anchor::Bottom,
            keyboard_interactivity: KeyboardInteractivity::None,
            ..Default::default()
        }
    }

    fn update(&mut self, message: NotificationsMessage) -> iced::Task<Message> {
        match message {
            NotificationsMessage::New(notification) => {
                self.add_notification(notification);
                Task::none()
            }
            NotificationsMessage::PopUp(id) => {
                self.start_animation(id);
                if let Some(duration) = self
                    .notifications
                    .get(&id)
                    .as_ref()
                    .map(|n| n.notification.expire_timeout)
                    .and_then(Into::into)
                {
                    Task::future(async move {
                        Timer::after(dbg!(duration)).await;
                        Message::Notifications(NotificationsMessage::Close(id))
                    })
                } else {
                    Task::none()
                }
            }
            NotificationsMessage::Close(id) => {
                if let Some(notification) = self.notifications.get_mut(&id) {
                    notification.animation.go_mut(1., self.now);
                }

                Task::done(Message::Notifications(NotificationsMessage::Remove(id)))
            }

            NotificationsMessage::Remove(id) => {
                self.remove_notification(id);
                if self.notifications.is_empty() {
                    self.animation.go_mut(1.0, self.now);
                    Task::done(Message::Hide(crate::FeatureSelector::Notifications))
                } else {
                    Task::none()
                }
            }

            NotificationsMessage::DbusInterfaceReady(sender) => {
                self.dbus_sender = Some(sender);
                Task::none()
            }
        }
    }

    fn view(&self) -> impl Into<iced::Element<'_, Message>> {
        let a = self
            .notifications
            .iter()
            .map(|(id, n)| (*id, components::notification(self.now, n)));

        float(stack![
            container(Column::with_children(a).spacing(Self::PAD))
                .width(Fill)
                .height(Fill)
                .style(|theme| container::Style {
                    text_color: Some(theme.palette().text),
                    background: Some(Color::TRANSPARENT.into()),
                    border: rounded(Radius::new(0).left(Notifications::ROUND))
                        .width(8.)
                        .color(theme.palette().background),
                    shadow: Shadow::default(),
                    snap: true,
                })
                .padding(
                    Padding::new(Notifications::PAD)
                        .right(0.)
                        .left(Notifications::PAD + 56. / 2. - 8.)
                )
                .width(Notifications::WIDTH - 56. / 2.)
                .align_x(Horizontal::Right)
                .height(Shrink),
            float(
                bead_center(value(self.notifications.len()))
                    .width(56.)
                    .height(56.)
            )
            .translate(|b, _| Vector::new(-b.width / 2. + 4., Notifications::ROUND * 2. / 3.)),
        ])
        .translate(|b, _| {
            Vector::new(
                8. + 56. / 2. + self.animation.interpolate_with(|f| f * b.width, self.now),
                0.,
            )
        })
    }

    // fn subscriptions(&self) -> iced::Subscription<Message> {
    //     subscriptions::notifications_subscription().map(Message::Notifications)
    // }

    fn is_animating(&self) -> bool {
        self.animation.is_animating(self.now)
            || self
                .notifications
                .iter()
                .any(|(_, n)| n.animation.is_animating(self.now))
    }

    fn set_now(&mut self, now: Instant) {
        self.now = now;
    }
}
