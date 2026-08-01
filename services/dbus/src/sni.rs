pub const IFACE: &str = "org.kde.StatusNotifierWatcher";
pub const PATH: &str = "/StatusNotifierWatcher";

use core::future::pending;
use std::collections::HashSet;

use futures_channel::mpsc;
use futures_lite::StreamExt;
use zbus::{
    connection, interface, message::Header, object_server::SignalEmitter, proxy,
    zvariant::OwnedObjectPath,
};

#[derive(Clone, Debug)]
pub enum WatcherCommand {
    ItemRegistered(NewTrayItem),
    ItemUnregistered(String),
}

#[derive(Clone, Debug)]
pub struct RegisteredItem {
    pub bus_name: String,
    pub path: String,
}

#[derive(Clone)]
pub enum TrayItemIcon {
    IconData(i32, i32, Vec<u8>),
    IconName(String),
    NoIcon,
}

impl std::fmt::Debug for TrayItemIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IconData(w, h, data) => f
                .debug_tuple("IconData")
                .field(w)
                .field(h)
                .field(&data.len())
                .finish(),
            Self::IconName(name) => f.debug_tuple("IconName").field(name).finish(),
            Self::NoIcon => write!(f, "NoIcon"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewTrayItem {
    pub id: String,
    pub title: String,
    pub icon: TrayItemIcon,
    pub proxy: StatusNotifierItemProxy<'static>,
}

pub async fn fetch_item(registered_item: RegisteredItem) -> zbus::Result<Option<NewTrayItem>> {
    let Some(conn) = super::connection() else {
        return Ok(None);
    };
    let sni =
        StatusNotifierItemProxy::new(conn, registered_item.bus_name, registered_item.path).await?;

    let icon = match sni.icon_name().await {
        Ok(name) if !name.is_empty() => TrayItemIcon::IconName(name),
        _ => match sni.icon_pixmap().await {
            Ok(pixmap) => {
                let (w, h, data) = pixmap.first().cloned().unwrap_or_default();
                TrayItemIcon::IconData(w, h, data)
            }
            Err(_) => TrayItemIcon::NoIcon,
        },
    };

    let item = NewTrayItem {
        id: sni.id().await?,
        title: sni.title().await?,
        icon,
        proxy: sni,
    };

    Ok(Some(item))
}

pub struct StatusNotifierWatcher {
    items: HashSet<String>,
    hosts: HashSet<String>,
    sender: mpsc::Sender<WatcherCommand>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        #[zbus(header)] header: Header<'_>,
        service: &str,
    ) {
        let bus_name = header
            .sender()
            .map(|un| un.to_string())
            .unwrap_or(service.to_string());
        let path = if service.starts_with("/") {
            service.to_string()
        } else {
            "/StatusNotifierItem".to_string()
        };

        let item = RegisteredItem { bus_name, path };

        self.items.insert(item.path.clone());
        let _ = Self::status_notifier_item_registered(&emitter, &item.path).await;

        match fetch_item(item).await {
            Ok(Some(tray_item)) => {
                let mut sender = self.sender.clone();
                let bus_name = tray_item.proxy.inner().destination().to_string();
                let mut owner_stream = tray_item
                    .proxy
                    .inner()
                    .receive_owner_changed()
                    .await
                    .unwrap();

                smol::spawn(async move {
                    while let Some(new_owner) = owner_stream.next().await {
                        if new_owner.is_none() {
                            // FIX: Remove from self.items too
                            let _ = sender.start_send(WatcherCommand::ItemUnregistered(bus_name));
                            break;
                        }
                    }
                })
                .detach();

                let _ = self
                    .sender
                    .start_send(WatcherCommand::ItemRegistered(tray_item));
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Failed to fetch SNI item: {e}");
            }
        }
    }

    async fn register_status_notifier_host(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        service: &str,
    ) {
        if self.hosts.insert(service.to_string()) {
            let _ = Self::status_notifier_host_registered(&emitter).await;
        }
    }

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.iter().cloned().collect()
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        !self.hosts.is_empty()
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

impl StatusNotifierWatcher {
    pub fn new(sender: mpsc::Sender<WatcherCommand>) -> Self {
        Self {
            items: HashSet::new(),
            hosts: HashSet::new(),
            sender,
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

        let host_name = format!("org.kde.StatusNotifierHost-{}", std::process::id());
        let host_name = zbus::names::WellKnownName::try_from(host_name).unwrap();
        conn.request_name(host_name.clone()).await.unwrap();
        conn.call_method(
            Some("org.kde.StatusNotifierWatcher"),
            "/StatusNotifierWatcher",
            Some("org.kde.StatusNotifierWatcher"),
            "RegisterStatusNotifierHost",
            &(host_name.as_str(),),
        )
        .await
        .unwrap();

        pending::<()>().await;
    }
}

#[proxy(interface = "org.kde.StatusNotifierItem", gen_blocking = false)]
pub trait StatusNotifierItem {
    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn context_menu(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn secondary_activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn scroll(&self, delta: i32, orientation: &str) -> zbus::Result<()>;

    #[zbus(property)]
    fn category(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;
    #[zbus(property)]
    fn attention_icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn overlay_icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn item_is_menu(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn menu(&self) -> zbus::Result<OwnedObjectPath>;

    #[zbus(signal)]
    fn new_icon(&self) -> zbus::Result<()>;
    #[zbus(signal)]
    fn new_status(&self, status: &str) -> zbus::Result<()>;
    #[zbus(signal)]
    fn new_title(&self) -> zbus::Result<()>;
    #[zbus(signal)]
    fn new_attention_icon(&self) -> zbus::Result<()>;
}
