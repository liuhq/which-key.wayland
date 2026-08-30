use std::os::fd::OwnedFd;
use std::sync::mpsc;

use zbus::blocking::{Connection, MessageIterator, connection::Builder};
use zbus::{self, interface};

use crate::keybind::key::Key;

pub const DBUS_NAME: &str = "com.hrtius.WhichKey";
pub const DBUS_PATH: &str = "/com/hrtius/WhichKey";

#[derive(Debug)]
pub enum DBusCommand {
    Show,
    ShowKey(Key),
    Quit,
    Reload,
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

    fn show_key(&self, key: &str) -> zbus::fdo::Result<()> {
        let key = key
            .parse()
            .map_err(|e: anyhow::Error| zbus::fdo::Error::InvalidArgs(e.to_string()))?;
        self.send(DBusCommand::ShowKey(key));
        Ok(())
    }

    fn quit(&self) -> zbus::fdo::Result<()> {
        self.send(DBusCommand::Quit);
        Ok(())
    }

    fn reload(&self) -> zbus::fdo::Result<()> {
        self.send(DBusCommand::Reload);
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

pub fn ipc_show_key(key: &str) -> bool {
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return false;
        }
    };
    match conn.call_method(Some(DBUS_NAME), DBUS_PATH, Some(DBUS_NAME), "ShowKey", &key) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("D-Bus ShowKey call failed: {e}");
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

pub fn ipc_reload() -> bool {
    let conn = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to DBus session bus: {e}");
            return false;
        }
    };
    match conn.call_method(Some(DBUS_NAME), DBUS_PATH, Some(DBUS_NAME), "Reload", &()) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("D-Bus Reload call failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_iface() -> (WhichKeyIface, mpsc::Receiver<DBusCommand>, OwnedFd) {
        let (tx, rx) = mpsc::channel();
        let wake_fd = rustix::event::eventfd(
            0,
            rustix::event::EventfdFlags::NONBLOCK | rustix::event::EventfdFlags::CLOEXEC,
        )
        .unwrap();
        let read_fd = wake_fd.try_clone().unwrap();
        (WhichKeyIface { tx, wake_fd }, rx, read_fd)
    }

    fn assert_woken(fd: &OwnedFd) {
        let mut bytes = [0; 8];
        assert_eq!(rustix::io::read(fd, &mut bytes).unwrap(), bytes.len());
        assert_eq!(u64::from_ne_bytes(bytes), 1);
    }

    #[test]
    fn show_sends_command_and_wakes_event_loop() {
        let (iface, rx, wake_fd) = test_iface();

        iface.show().unwrap();

        assert!(matches!(rx.recv().unwrap(), DBusCommand::Show));
        assert_woken(&wake_fd);
    }

    #[test]
    fn show_key_parses_key_and_sends_command() {
        let (iface, rx, wake_fd) = test_iface();

        iface.show_key("Ctrl+a").unwrap();

        let DBusCommand::ShowKey(key) = rx.recv().unwrap() else {
            panic!("expected ShowKey command");
        };
        assert_eq!(key, "Ctrl+a".parse().unwrap());
        assert_woken(&wake_fd);
    }

    #[test]
    fn invalid_show_key_does_not_send_or_wake() {
        let (iface, rx, wake_fd) = test_iface();

        assert!(iface.show_key("Ctrl++a").is_err());

        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        let mut bytes = [0; 8];
        assert_eq!(
            rustix::io::read(&wake_fd, &mut bytes),
            Err(rustix::io::Errno::AGAIN)
        );
    }

    #[test]
    fn quit_and_reload_send_their_commands() {
        let (iface, rx, wake_fd) = test_iface();

        iface.quit().unwrap();
        iface.reload().unwrap();

        assert!(matches!(rx.recv().unwrap(), DBusCommand::Quit));
        assert!(matches!(rx.recv().unwrap(), DBusCommand::Reload));
        let mut bytes = [0; 8];
        rustix::io::read(&wake_fd, &mut bytes).unwrap();
        assert_eq!(u64::from_ne_bytes(bytes), 2);
    }
}
