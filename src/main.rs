use iced::Color;
use iced::Font;
use iced::Size;
use iced::Subscription;
use iced::futures::SinkExt;
use iced::keyboard::Key;
use iced::keyboard::key;
use iced::time::Instant;
use iced::window::Id;
use iced_exwlshell::{
    daemon,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};
use lucide_icons::LUCIDE_FONT_BYTES;
use mothscheme::themes::styx as theme;
use signalfut::Signal;
use signalfut::SignalFut;
use std::time::Duration;
use wb_dbus::notifications::NotificationsCommand;
use wb_dbus::sni::StatusNotifierItemProxy;
use wb_dbus::sni::WatcherCommand;

mod components;
mod daemon;
mod units;
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
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::temp_dir().join("waybracelet.log"))
        .expect("failed to open log file");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .with_target(true)
        .with_writer(log_file)
        .init();

    tracing::info!("waybracelet starting");
    daemon(Daemon::new, "waybracelet", Daemon::update, Daemon::view)
        .settings(iced_exwlshell::Settings {
            antialiasing: true,
            fonts: vec![LUCIDE_FONT_BYTES.into()],
            ..Default::default()
        })
        .layer_settings(LayerShellSettings {
            start_mode: StartMode::Background,
            ..Default::default()
        })
        .theme(iced::Theme::custom(
            "mothscheme-styx",
            iced::theme::Palette {
                background: theme::OVERLAY,
                text: theme::TEXT,
                primary: theme::BLUE,
                success: theme::GREEN,
                warning: theme::YELLOW,
                danger: theme::RED,
            },
        ))
        .style(|_, _| iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
        })
        .default_font(Font::with_name("IBM Plex Serif"))
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
                Subscription::run(|| {
                    iced::stream::channel(1, async |mut output| {
                        loop {
                            SignalFut::new(Signal::SIGUSR1).await;
                            let _ = output.send(Message::OpenSpotLight).await;
                        }
                    })
                }),
                iced::keyboard::listen().map(|key| match key {
                    iced::keyboard::Event::KeyPressed {
                        key: Key::Named(key::Named::Escape),
                        ..
                    } => Message::EscapePressed,
                    _ => Message::Noop,
                }),
            ])
        })
        .run()
        .unwrap();
}
