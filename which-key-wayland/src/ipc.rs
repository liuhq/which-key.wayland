use std::os::fd::OwnedFd;
use std::sync::mpsc;

use zbus::blocking::{Connection, MessageIterator, connection::Builder};
use zbus::{self, interface};

pub const DBUS_NAME: &str = "com.hrtius.WhichKey";
pub const DBUS_PATH: &str = "/com/hrtius/WhichKey";

#[derive(Debug)]
pub enum DBusCommand {
    Show,
    Quit,
}

struct WhichKeyIface {
    tx: mpsc::Sender<DBusCommand>,
    wake_fd: OwnedFd,
}

#[interface(name = "com.hrtius.WhichKey")]
impl WhichKeyIface {
    fn show(&self) -> zbus::fdo::Result<()> {
        self.send(DBusCommand::Show);
        Ok(())
    }

    fn quit(&self) -> zbus::fdo::Result<()> {
        self.send(DBusCommand::Quit);
        Ok(())
    }
}

impl WhichKeyIface {
    fn send(&self, cmd: DBusCommand) {
        let _ = self.tx.send(cmd);
        let val: u64 = 1;
        let _ = rustix::io::write(&self.wake_fd, &val.to_ne_bytes());
    }
}

pub fn init() -> Option<(mpsc::Receiver<DBusCommand>, OwnedFd)> {
    let (tx, rx) = mpsc::channel();

    let wake_fd = match rustix::event::eventfd(
        0,
        rustix::event::EventfdFlags::NONBLOCK | rustix::event::EventfdFlags::CLOEXEC,
    ) {
        Ok(fd) => fd,
        Err(e) => {
            log::error!("Failed to create eventfd: {e}");
            return None;
        }
    };

    let wake_fd_dbus = match wake_fd.try_clone() {
        Ok(fd) => fd,
        Err(e) => {
            log::error!("Failed to clone eventfd: {e}");
            return None;
        }
    };

    let iface = WhichKeyIface {
        tx,
        wake_fd: wake_fd_dbus,
    };

    let conn = match Builder::session() {
        Ok(b) => b,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return None;
        }
    };

    let conn = match conn
        .name(DBUS_NAME)
        .and_then(|b| b.serve_at(DBUS_PATH, iface))
    {
        Ok(b) => b,
        Err(e) => {
            log::error!("Failed to set up DBus interface: {e}");
            return None;
        }
    };

    match conn.build() {
        Ok(conn) => {
            start_dbus_server(conn);
            Some((rx, wake_fd))
        }
        Err(zbus::Error::NameTaken) => {
            ipc_show();
            None
        }
        Err(e) => {
            log::error!("Failed to build DBus connection: {e}");
            None
        }
    }
}

pub fn start_dbus_server(conn: Connection) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let iter = MessageIterator::from(conn);
        for msg in iter {
            match msg {
                Ok(_) => {}
                Err(e) => {
                    log::error!("DBus error: {e}");
                    break;
                }
            }
        }
    })
}

pub fn ipc_show() -> bool {
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return false;
        }
    };
    match conn.call_method(Some(DBUS_NAME), DBUS_PATH, Some(DBUS_NAME), "Show", &()) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("D-Bus Show call failed: {e}");
            false
        }
    }
}

pub fn ipc_quit() -> bool {
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return false;
        }
    };
    match conn.call_method(Some(DBUS_NAME), DBUS_PATH, Some(DBUS_NAME), "Quit", &()) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("D-Bus Quit call failed: {e}");
            false
        }
    }
}
