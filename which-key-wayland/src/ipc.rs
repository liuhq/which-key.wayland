use std::os::fd::OwnedFd;
use std::sync::mpsc;

use zbus::blocking::{Connection, MessageIterator};
use zbus::fdo::RequestNameFlags;
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
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return None;
        }
    };

    match conn.request_name_with_flags(DBUS_NAME, RequestNameFlags::DoNotQueue.into()) {
        Ok(_) => {
            let (tx, rx) = mpsc::channel();

            let wake_fd = match rustix::event::eventfd(
                0,
                rustix::event::EventfdFlags::NONBLOCK
                    | rustix::event::EventfdFlags::CLOEXEC,
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

            start_dbus_server(conn, tx, wake_fd_dbus);
            Some((rx, wake_fd))
        }
        Err(zbus::Error::NameTaken) => {
            let _ = conn.call_method(
                Some(DBUS_NAME),
                DBUS_PATH,
                Some(DBUS_NAME),
                "Show",
                &(),
            );
            None
        }
        Err(e) => {
            log::error!("Failed to request DBus name: {e}");
            None
        }
    }
}

pub fn start_dbus_server(
    conn: Connection,
    tx: mpsc::Sender<DBusCommand>,
    wake_fd: OwnedFd,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let iface = WhichKeyIface { tx, wake_fd };
        if let Err(e) = conn.object_server().at(DBUS_PATH, iface) {
            log::error!("Failed to register DBus interface: {e}");
            return;
        }

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

pub fn ipc_show() {
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return;
        }
    };
    let _ = conn.call_method(Some(DBUS_NAME), DBUS_PATH, Some(DBUS_NAME), "Show", &());
}

pub fn ipc_quit() {
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return;
        }
    };
    let _ = conn.call_method(Some(DBUS_NAME), DBUS_PATH, Some(DBUS_NAME), "Quit", &());
}
