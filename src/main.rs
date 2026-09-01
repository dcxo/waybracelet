use std::time::Duration;

use iced::{
    Size, Subscription,
    keyboard::{Key, key},
    time::Instant,
    window::Id,
};
use iced_exwlshell::{
    daemon,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};
use lucide_icons::LUCIDE_FONT_BYTES;
use waybracelet::{components, theme, units};
use wb_dbus::{
    notifications::NotificationsCommand,
    sni::{StatusNotifierItemProxy, WatcherCommand},
};

mod daemon;
mod windows;

const TIME_INTERVAL: Duration = Duration::from_secs(20);

use daemon::Daemon;

use crate::windows::shell_window::Kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeLeavingMessage {
    Off,
    Reboot,
    LogOut,
    Suspend,
    LockOff,
    Hibernate,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    OpenSpotLight,
    Tick(Instant),
    Noop,
    Close(Id),
    CloseByKind(Kind),
    EscapePressed,
    WayEvent(iced_wayland_subscriber::shell::ShellEvent),

    // STATUSBAR
    TrayServer(WatcherCommand),
    TrayIconClick(StatusNotifierItemProxy<'static>),
    OpenLeaving,
    // SPOTLIGHT
    OnResize(Id, Size<f32>),
    ChangeInput(String),
    Exec(Vec<String>),
    // LEAVING
    LeaveAction(WeLeavingMessage),
    Deconfirm,
    // NOTIFICATION
    NotificationServer(NotificationsCommand),
    CloseNotification(u32),
    HoverNotification(u32),
    UnhoverNotification(u32),
    OnActionActivated(u32, String),
}

impl Message {
    pub fn is_notification_related(&self) -> bool {
        matches!(
            self,
            Self::NotificationServer(_)
                | Self::CloseNotification(_)
                | Self::HoverNotification(_)
                | Self::UnhoverNotification(_)
                | Self::OnActionActivated(_, _)
        )
    }

    pub fn is_spotlight_related(&self) -> bool {
        matches!(self, Self::ChangeInput(_))
    }
}

fn main() {
    let (shell_broadcast, shell_events) = iced_wayland_subscriber::shell::channel();

    daemon(
        move || Daemon::new(shell_events.clone()),
        "waybracelet",
        Daemon::update,
        Daemon::view,
    )
    .settings(iced_exwlshell::Settings {
        antialiasing: true,
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        shell_broadcast,
        layer_settings: LayerShellSettings {
            start_mode: StartMode::Background,
            ..Default::default()
        },
        ..Default::default()
    })
    .theme(theme::styx())
    .style(|_, _| theme::transparent_style())
    .default_font(theme::DEFAULT_FONT)
    .subscription(|daemon| {
        iced::Subscription::batch([
            if daemon.is_animating() {
                iced::window::frames().map(Message::Tick)
            } else {
                Subscription::none()
            },
            iced::time::every(TIME_INTERVAL).map(Message::Tick),
            Subscription::run(|| {
                iced::stream::channel(10, |output| {
                    wb_dbus::notifications::NotificationServer::new(output).run()
                })
            })
            .map(Message::NotificationServer),
            Subscription::run(|| {
                iced::stream::channel(10, |output| {
                    wb_dbus::sni::StatusNotifierWatcher::new(output).run()
                })
            })
            .map(Message::TrayServer),
            iced::keyboard::listen().map(|key| match key {
                iced::keyboard::Event::KeyPressed {
                    key: Key::Named(key::Named::Escape),
                    ..
                } => Message::EscapePressed,
                _ => Message::Noop,
            }),
            daemon.shell_events.listen().map(Message::WayEvent),
        ])
    })
    .run()
    .unwrap();
}
