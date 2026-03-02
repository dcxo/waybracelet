#![warn(unused_extern_crates)]
#![allow(mismatched_lifetime_syntaxes)]

use std::{fs::File, iter};

use hyprland::{data::Monitors, shared::HyprData};
use iced::{
    Color, Element, Size, Subscription, Task,
    theme::Style,
    time::Instant,
    widget::{container, space},
    window,
};
use iced_layershell::{
    Settings,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};
use iced_wayland_subscriber::{OutputInfo, WaylandEvent};
use tracing::Level;
use tracing_panic::panic_hook;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use wayland_client::Connection;

use crate::{
    features::{
        Feature,
        notifications::{self, Notifications},
        power_menu::PowerMenu,
        status_bar::StatusBar,
        volume_osd::VolumeOSD,
    },
    styles::dark_theme,
    windows::Window,
};

mod components;
mod features;
mod styles;
mod windows;

fn main() {
    std::panic::set_hook(Box::new(panic_hook));

    let file = File::create("/home/dcxo/debug.wb.log").unwrap();
    let log = fmt::layer()
        .with_writer(file)
        .with_filter(EnvFilter::from("info,iced_layershell=warn,calloop=warn"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(EnvFilter::from_default_env()))
        .with(log)
        .init();

    let conn = Connection::connect_to_env().unwrap();
    let conn2 = conn.clone();

    iced_layershell::daemon(
        move || Daemon::new(conn.clone()),
        || "WayBracelet".into(),
        Daemon::update,
        Daemon::view,
    )
    .subscription(Daemon::subscriptions)
    .theme(|_: &Daemon, _| dark_theme())
    .style(|_, theme| Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    })
    .settings(Settings {
        antialiasing: true,
        with_connection: Some(conn2),
        layer_settings: LayerShellSettings {
            start_mode: StartMode::Background,
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
    StatusBar(features::status_bar::StatusBarMessage),
    PowerMenu(features::power_menu::PowerMenuMessage),
    VolumeOSD(features::volume_osd::VolumeOsdMessage),
    Notifications(features::notifications::NotificationsMessage),

    Open(FeatureSelector),
    Hide(FeatureSelector),
    Remove(FeatureSelector),
    ChangeSize(FeatureSelector, Size),

    DisplayInserted(OutputInfo),
    Animation,
    ChangeTheme,
}

#[derive(Debug, Clone)]
pub enum FeatureSelector {
    StatusBar,
    PowerMenu,
    VolumeOSD,
    Notifications,
}

struct Daemon {
    statuses_bar: Vec<Window<StatusBar>>,
    volume_osd: Window<VolumeOSD>,
    power_menu: Option<Window<PowerMenu>>,
    notifications: Option<Window<Notifications>>,
    connection: Connection,
    now: Instant,
}

impl Daemon {
    fn new(connection: Connection) -> (Self, Task<Message>) {
        let now = Instant::now();
        let (volume_osd, volume_open_task) = VolumeOSD::new(now).open();

        // let (statuses_bar, mut open_tasks) = Monitors::get()
        //     .unwrap()
        //     .iter()
        //     .map(|m| StatusBar::new(m.name.clone(), m.active_workspace.id, now))
        //     .map(StatusBar::open)
        //     .unzip::<Window<StatusBar>, Task<Message>, Vec<_>, Vec<_>>();

        (
            Self {
                connection,
                statuses_bar: vec![],
                volume_osd,
                power_menu: None,
                notifications: None,
                now,
            },
            volume_open_task,
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.set_now();

        match message {
            Message::StatusBar(message) => self
                .statuses_bar
                .iter_mut()
                .map(move |sb| sb.update(message.clone()))
                .fold(Task::none(), |mt, t| mt.chain(t)),
            Message::VolumeOSD(message) => self.volume_osd.update(message),
            Message::Notifications(message) => {
                if let Some(ns) = self.notifications.as_mut() {
                    ns.update(message)
                } else {
                    Task::none()
                }
            }
            Message::PowerMenu(message) => self
                .power_menu
                .as_mut()
                .map(|pm| pm.update(message))
                .unwrap_or(Task::none()),

            Message::Open(feature) => match feature {
                FeatureSelector::PowerMenu if self.power_menu.is_none() => {
                    let (window, open_task) = PowerMenu::new(self.now).open();
                    self.power_menu.replace(window);

                    open_task
                }
                FeatureSelector::Notifications if self.notifications.is_none() => {
                    let (window, open_task) =
                        Notifications::new(Default::default(), self.now).open();
                    self.notifications.replace(window);

                    open_task
                }

                _ => Task::none(),
            },

            Message::Hide(feature) => {
                let id = match feature {
                    FeatureSelector::PowerMenu => self.power_menu.as_ref().map(|pm| pm.id),
                    FeatureSelector::Notifications => self.notifications.as_ref().map(|ns| ns.id),
                    _ => unreachable!(),
                };
                if let Some(id) = id {
                    Task::done(Message::RemoveWindow(id))
                        .chain(Task::done(Message::Remove(feature)))
                } else {
                    Task::none()
                }
            }
            Message::Remove(feature) => {
                match feature {
                    FeatureSelector::Notifications => {
                        self.notifications.take();
                    }
                    FeatureSelector::PowerMenu => {
                        self.power_menu.take();
                    }
                    _ => unreachable!(),
                };
                Task::none()
            }

            Message::ChangeSize(f, s) => {
                let Some(id) = (match f {
                    FeatureSelector::Notifications => self.notifications.as_ref().map(|f| f.id),
                    FeatureSelector::PowerMenu => todo!(),
                    FeatureSelector::VolumeOSD => todo!(),
                    FeatureSelector::StatusBar => todo!(),
                }) else {
                    return Task::none();
                };

                Task::done(Message::SizeChange {
                    id,
                    size: (s.width as u32, s.height as u32),
                })
            }

            Message::DisplayInserted(info) => {
                let m = Monitors::get()
                    .unwrap()
                    .iter()
                    .find(|m| m.name == info.name)
                    .map(|m| m.active_workspace.id);

                let (window, task) =
                    StatusBar::new(info.name, info.wl_output, m.unwrap_or(1), self.now).open();

                let current_status_bar = self
                    .statuses_bar
                    .iter_mut()
                    .find(|sb| sb.output == window.output);

                if let Some(current_status_bar) = current_status_bar {
                    let _ = std::mem::replace(current_status_bar, window);
                } else {
                    self.statuses_bar.push(window);
                }

                task
            }

            _ => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(window) = self.statuses_bar.iter().find(|sb| sb.id == window_id) {
            window.view().into()
        } else if let Some(window) = self.notifications.as_ref().filter(|ns| window_id == ns.id) {
            window.view().into()
        } else if window_id == self.volume_osd.id {
            self.volume_osd.view().into()
        } else if let Some(window) = self.power_menu.as_ref().filter(|pm| pm.id == window_id) {
            window.view().into()
        } else {
            container(space()).into()
        }
    }

    fn subscriptions(&self) -> Subscription<Message> {
        let frames = if self.is_animating() {
            iced::window::frames().map(|_| Message::Animation)
        } else {
            Subscription::none()
        };

        Subscription::batch(
            self.statuses_bar
                .iter()
                .map(|sb| sb.subscriptions())
                .chain(self.power_menu.as_ref().map(|pm| pm.subscriptions()))
                // .chain(self.notifications.as_ref().map(|pm| pm.subscriptions()))
                .chain(iter::once(
                    notifications::subscriptions::notifications_subscription(),
                ))
                .chain(iter::once(self.volume_osd.subscriptions()))
                .chain([
                    frames,
                    iced_wayland_subscriber::listen(self.connection.clone())
                        .filter_map(|event| match event {
                            WaylandEvent::OutputInsert(oi) => Some(oi),
                            _ => None,
                        })
                        .map(Message::DisplayInserted),
                ]), // .chain([frames, change_theme_subscription()]),
        )
    }

    fn is_animating(&self) -> bool {
        self.notifications
            .as_ref()
            .is_some_and(|ns| ns.is_animating())
            || self.power_menu.as_ref().is_some_and(|pm| pm.is_animating())
            || self.statuses_bar.iter().any(|sb| sb.is_animating())
            || self.volume_osd.is_animating()
    }

    fn set_now(&mut self) {
        self.now = Instant::now();
        self.statuses_bar
            .iter_mut()
            .for_each(|sb| sb.set_now(self.now));
        self.volume_osd.set_now(self.now);
        self.notifications
            .as_mut()
            .iter_mut()
            .for_each(|ns| ns.set_now(self.now));

        self.power_menu
            .as_mut()
            .iter_mut()
            .for_each(|pm| pm.set_now(self.now));
    }
}

// fn change_theme_subscription() -> Subscription<Message> {
//     Subscription::run(|| {
//         SignalStream::new(signal(SignalKind::user_defined1()).unwrap())
//             .map(|_| Message::ChangeTheme)
//     })
// }
