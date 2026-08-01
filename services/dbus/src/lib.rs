use std::sync::OnceLock;

static DBUS_CONN: OnceLock<zbus::Connection> = OnceLock::new();

pub fn connection() -> Option<&'static zbus::Connection> {
    DBUS_CONN.get()
}

pub use zbus::object_server::SignalEmitter;

pub mod notifications;
pub mod sni;
