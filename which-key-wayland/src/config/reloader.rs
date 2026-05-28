use std::{mem::MaybeUninit, path::PathBuf, time::SystemTime};

pub enum ConfigReloader {
    Mtime {
        path: PathBuf,
        last_mtime: Option<SystemTime>,
    },
    Inotify {
        inotify_fd: rustix::fd::OwnedFd,
        wd: i32,
        buffer: Vec<MaybeUninit<u8>>,
    },
}

impl ConfigReloader {
    pub fn init_mtime(path: PathBuf) -> Self {
        let last_mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        ConfigReloader::Mtime { path, last_mtime }
    }

    pub fn try_read_mtime(&self) -> Option<SystemTime> {
        let ConfigReloader::Mtime { path, .. } = self else {
            return None;
        };

        std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
    }

    pub fn sync_mtime(&mut self, mtime: Option<SystemTime>) {
        let ConfigReloader::Mtime { last_mtime, .. } = self else {
            return;
        };

        if mtime != *last_mtime {
            *last_mtime = mtime;
        }
    }

    pub fn has_changed_by_mtime(&mut self) -> bool {
        let mtime = self.try_read_mtime();

        if mtime.is_some() {
            self.sync_mtime(mtime);
            true
        } else {
            false
        }
    }
}
