use std::collections::HashMap;
use std::future::pending;
use std::time::Duration;

use futures_channel::mpsc;
use zbus::connection;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

pub const IFACE: &str = "org.freedesktop.Notifications";
pub const PATH: &str = "/org/freedesktop/Notifications";

#[derive(Clone)]
pub struct NotificationServer {
    notifications: HashMap<u32, ()>,
    sender: mpsc::Sender<NotificationsCommand>,
    next_id: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Actions {
    pub default: Option<String>,
    pub rest: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Option<Actions>,
    // pub hints: HashMap<String, String>,
    pub timeout_secs: Option<Duration>,
}

fn resolve_timeout(expire_timeout: i32) -> Option<Duration> {
    match expire_timeout {
        -1 => Some(Duration::from_secs(13)),
        0 => None,
        ms if ms > 0 => Some(Duration::from_millis(ms as u64)),
        _ => Some(Duration::from_secs(13)),
    }
}

#[derive(Debug, Clone)]
pub enum NotificationsCommand {
    NewNotification(Notification),
    CloseNotification(u32),
    ReplaceNotification(u32, Notification),
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &mut self,
        #[zbus(signal_emitter)] _emitter: SignalEmitter<'_>,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        _hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let is_replacing = replaces_id != 0;
        let id = if is_replacing {
            replaces_id
        } else {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
            id
        };

        let actions = actions.as_chunks::<2>().0;
        let actions = if actions.is_empty() {
            None
        } else {
            Some(
                actions
                    .iter()
                    .fold(Actions::default(), |mut acc, [key, value]| {
                        if acc.default.is_none() && key == "default" {
                            acc.default = Some(value.clone());
                        } else {
                            acc.rest.push((key.clone(), value.clone()));
                        }

                        acc
                    }),
            )
        };
        // let hints = hints.into_iter().map(|(s, v)| (s, v.to_string())).collect();

        let notification = Notification {
            id,
            app_name: app_name.into(),
            app_icon: app_icon.into(),
            summary: summary.into(),
            body: body.into(),
            actions,
            // hints,
            timeout_secs: resolve_timeout(expire_timeout),
        };

        self.sender
            .start_send(if is_replacing {
                NotificationsCommand::ReplaceNotification(id, notification)
            } else {
                NotificationsCommand::NewNotification(notification)
            })
            .unwrap();

        self.notifications.insert(id, ());
        id
    }

    async fn close_notification(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        id: u32,
    ) {
        if self.notifications.remove(&id).is_some() {
            let _ = Self::notification_closed(&emitter, id, 3).await;
        }
    }

    #[zbus(signal)]
    async fn notification_closed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;

    async fn get_capabilities(&self) -> Vec<&str> {
        vec!["body", "actions", "icon-static"]
    }

    async fn get_server_information(&self) -> (&str, &str, &str, &str) {
        (
            "waybracelet",
            "waybracelet",
            env!("CARGO_PKG_VERSION"),
            "1.2",
        )
    }
}

impl NotificationServer {
    pub fn new(sender: mpsc::Sender<NotificationsCommand>) -> Self {
        Self {
            notifications: HashMap::new(),
            sender,
            next_id: 1,
        }
    }

    pub async fn run(self) {
        let conn = connection::Builder::session()
            .unwrap()
            .name(IFACE)
            .unwrap()
            .serve_at(PATH, self)
            .unwrap()
            .build()
            .await
            .unwrap();

        super::DBUS_CONN.set(conn).ok();
        pending::<()>().await;
    }
}
