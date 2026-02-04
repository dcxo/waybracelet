use std::collections::HashMap;

use iced::futures::SinkExt;
use iced::{Subscription, stream};
use smol::channel::{Sender, unbounded};
use zbus::conn::Builder;
use zbus::object_server::SignalEmitter;
use zbus::{interface, zvariant};

use crate::{FeatureSelector, Message};

use super::{ExpireTimeout, Notification, NotificationsMessage};

#[derive(Debug)]
struct NotificationsManager {
    sender: Sender<NotificationsMessage>,
    current_id: u32,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationsManager {
    #[allow(clippy::too_many_arguments)]
    pub async fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, zvariant::OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id == 0 {
            self.current_id = self.current_id.wrapping_add(1);
            self.current_id
        } else {
            replaces_id
        };

        let expire_timeout = match expire_timeout {
            -1 => ExpireTimeout::ServerDefault,
            0 => ExpireTimeout::ManualClosing,
            x => ExpireTimeout::NotificationSpecific(x),
        };

        let notification = Notification {
            id,
            app_name,
            app_icon,
            summary,
            body,
            actions: actions
                .chunks(2)
                .filter_map(|a| {
                    if let [key, text] = a {
                        Some((key.to_string(), text.to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
            hints,
            expire_timeout,
        };

        let _ = self
            .sender
            .send(NotificationsMessage::New(notification))
            .await;

        id
    }

    fn get_capabilities(&self) -> Vec<&'static str> {
        vec![
            "action-icons",
            "actions",
            "body",
            "body-hyperlinks",
            "body-images",
            "body-markup",
            "icon-multi",
            "icon-static",
            "persistence",
            "sound",
        ]
    }

    async fn close_notification(&mut self, id: u32) {
        let _ = self.sender.send(NotificationsMessage::Close(id)).await;
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "WayBracelet".to_string(),
            "dcxo".to_string(),
            "0.1.0".to_string(),
            "1.3".to_string(),
        )
    }

    #[zbus(signal)]
    async fn notification_closed(
        ctx: &SignalEmitter<'_>,
        id: u32,
        reason: String,
    ) -> Result<(), zbus::Error>;

    #[zbus(signal)]
    async fn action_invoked(
        ctx: &SignalEmitter<'_>,
        id: u32,
        action_key: String,
    ) -> Result<(), zbus::Error>;

    #[zbus(signal)]
    async fn activation_token(
        ctx: &SignalEmitter<'_>,
        id: u32,
        activation_token: String,
    ) -> Result<(), zbus::Error>;
}

pub fn notifications_subscription() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(1, async |mut output| {
            let (tx, mut rx) = unbounded();

            let iface = NotificationsManager {
                sender: tx,
                current_id: 0,
            };
            let connection = Builder::session()
                .unwrap()
                .name("org.freedesktop.Notifications")
                .unwrap()
                .serve_at("/org/freedesktop/Notifications", iface)
                .unwrap()
                .build()
                .await
                .unwrap();

            connection
                .request_name("org.freedesktop.Notifications")
                .await
                .unwrap();

            let (tx_id, mut rx_id) = unbounded();

            let _ = output
                .send(Message::Notifications(
                    NotificationsMessage::DbusInterfaceReady(tx_id),
                ))
                .await;

            loop {
                if let Ok(event) = rx.recv().await {
                    let _ = output
                        .send(Message::Open(FeatureSelector::Notifications))
                        .await;
                    let _ = output.send(Message::Notifications(event)).await;
                }
                // select! {
                //     Some(event) = rx_id.recv() => {
                //
                //     },
                //     else => continue
                // };
            }
        })
    })
}
