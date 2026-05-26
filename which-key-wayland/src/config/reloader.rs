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
    pub fn has_changed(&mut self) -> bool {
        match self {
            ConfigReloader::Mtime { path, last_mtime } => {
                let Ok(meta) = std::fs::metadata(path) else {
                    return false;
                };

                let Ok(mtime) = meta.modified() else {
                    return false;
                };

                let mtime = Some(mtime);

                if mtime != *last_mtime {
                    *last_mtime = mtime;
                    true
                } else {
                    false
                }
            }
            ConfigReloader::Inotify {
                inotify_fd,
                wd,
                buffer,
            } => false,
        }
    }
}
