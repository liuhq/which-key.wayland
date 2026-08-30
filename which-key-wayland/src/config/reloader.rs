use std::{
    mem::MaybeUninit,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::ffi::OsStrExt,
    },
    path::{Path, PathBuf},
    time::SystemTime,
};

use rustix::fs::inotify;

pub enum ConfigReloader {
    Mtime {
        path: PathBuf,
        last_mtime: Option<SystemTime>,
    },
    Inotify {
        path: PathBuf,
        file_name: Vec<u8>,
        inotify_fd: rustix::fd::OwnedFd,
        wd: i32,
        buffer: Vec<MaybeUninit<u8>>,
    },
}

impl ConfigReloader {
    pub fn init(path: PathBuf) -> Self {
        match Self::try_init_inotify(path.clone()) {
            Ok(reloader) => {
                log::debug!("using inotify to watch config: {}", path.display());
                reloader
            }
            Err(e) => {
                log::warn!(
                    "failed to initialize inotify for {} ({e}); falling back to mtime checks",
                    path.display()
                );
                Self::init_mtime(path)
            }
        }
    }

    fn try_init_inotify(path: PathBuf) -> anyhow::Result<Self> {
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("config path has no file name"))?
            .as_bytes()
            .to_vec();
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let inotify_fd =
            inotify::init(inotify::CreateFlags::NONBLOCK | inotify::CreateFlags::CLOEXEC)?;
        let wd = inotify::add_watch(
            &inotify_fd,
            parent,
            inotify::WatchFlags::CLOSE_WRITE | inotify::WatchFlags::MOVED_TO,
        )?;

        Ok(Self::Inotify {
            path,
            file_name,
            inotify_fd,
            wd,
            buffer: vec![MaybeUninit::uninit(); 4096],
        })
    }

    pub fn init_mtime(path: PathBuf) -> Self {
        let last_mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());
        ConfigReloader::Mtime { path, last_mtime }
    }

    pub fn path(&self) -> &Path {
        match self {
            ConfigReloader::Mtime { path, .. } => path,
            ConfigReloader::Inotify { path, .. } => path,
        }
    }

    pub fn inotify_fd(&self) -> Option<BorrowedFd<'_>> {
        match self {
            ConfigReloader::Mtime { .. } => None,
            ConfigReloader::Inotify { inotify_fd, .. } => Some(inotify_fd.as_fd()),
        }
    }

    pub fn consume_inotify_events(&mut self) -> bool {
        let ConfigReloader::Inotify {
            file_name,
            inotify_fd,
            wd,
            buffer,
            ..
        } = self
        else {
            return false;
        };

        let mut changed = false;
        let mut reader = inotify::Reader::new(&*inotify_fd, buffer);
        loop {
            match reader.next() {
                Ok(event) => {
                    if event.events().contains(inotify::ReadFlags::QUEUE_OVERFLOW) {
                        changed = true;
                        continue;
                    }

                    let is_config = event.wd() == *wd
                        && event
                            .file_name()
                            .is_some_and(|name| name.to_bytes() == file_name);
                    let is_complete_write = event
                        .events()
                        .intersects(inotify::ReadFlags::CLOSE_WRITE | inotify::ReadFlags::MOVED_TO);
                    changed |= is_config && is_complete_write;
                }
                Err(rustix::io::Errno::AGAIN) => break,
                Err(e) => {
                    log::error!("failed to read inotify events: {e}");
                    break;
                }
            }
        }
        changed
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

        *last_mtime = mtime;
    }

    pub fn has_changed_by_mtime(&mut self) -> bool {
        let mtime = self.try_read_mtime();
        let ConfigReloader::Mtime { last_mtime, .. } = self else {
            return false;
        };

        if mtime.is_some() && mtime != *last_mtime {
            *last_mtime = mtime;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, thread, time::Duration};

    fn unique_dir(label: &str) -> PathBuf {
        let pid = std::process::id();
        PathBuf::from(format!("/tmp/which-key-reloader-{label}-{pid}"))
    }

    fn wait_for_change(cr: &mut ConfigReloader) -> bool {
        for _ in 0..100 {
            if cr.consume_inotify_events() {
                return true;
            }
            thread::sleep(Duration::from_millis(5));
        }
        false
    }

    #[test]
    fn mtime_only_reports_actual_changes() {
        let dir = unique_dir("mtime-change");
        let path = dir.join("config.kdl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        fs::write(&path, "timeout 1000").unwrap();

        let mut cr = ConfigReloader::init_mtime(path.clone());
        assert!(!cr.has_changed_by_mtime());
        thread::sleep(Duration::from_millis(10));
        fs::write(&path, "timeout 2000").unwrap();
        assert!(cr.has_changed_by_mtime());
        assert!(!cr.has_changed_by_mtime());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mtime_missing_file_does_not_change() {
        let dir = unique_dir("mtime-missing");
        let path = dir.join("config.kdl");
        let _ = fs::remove_dir_all(&dir);
        let mut cr = ConfigReloader::init_mtime(path);
        assert!(cr.try_read_mtime().is_none());
        assert!(!cr.has_changed_by_mtime());
    }

    #[test]
    fn mtime_detects_file_created_after_initialization() {
        let dir = unique_dir("mtime-create");
        let path = dir.join("config.kdl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        let mut cr = ConfigReloader::init_mtime(path.clone());

        fs::write(&path, "timeout 1000").unwrap();

        assert!(cr.has_changed_by_mtime());
        assert!(!cr.has_changed_by_mtime());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sync_mtime_suppresses_an_already_observed_change() {
        let dir = unique_dir("mtime-sync");
        let path = dir.join("config.kdl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        fs::write(&path, "timeout 1000").unwrap();
        let mut cr = ConfigReloader::init_mtime(path.clone());
        thread::sleep(Duration::from_millis(10));
        fs::write(&path, "timeout 2000").unwrap();

        let mtime = cr.try_read_mtime();
        cr.sync_mtime(mtime);

        assert!(!cr.has_changed_by_mtime());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mtime_reloader_has_no_inotify_fd_or_events() {
        let mut cr = ConfigReloader::init_mtime(PathBuf::from("config.kdl"));

        assert!(cr.inotify_fd().is_none());
        assert!(!cr.consume_inotify_events());
        assert_eq!(cr.path(), Path::new("config.kdl"));
    }

    #[test]
    fn invalid_watch_falls_back_to_mtime() {
        let dir = unique_dir("fallback");
        let _ = fs::remove_dir_all(&dir);
        let cr = ConfigReloader::init(dir.join("config.kdl"));
        assert!(matches!(cr, ConfigReloader::Mtime { .. }));
    }

    #[test]
    fn inotify_detects_direct_write_and_ignores_other_files() {
        let dir = unique_dir("inotify-write");
        let path = dir.join("config.kdl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        fs::write(&path, "timeout 1000").unwrap();
        let mut cr = ConfigReloader::try_init_inotify(path.clone()).unwrap();

        fs::write(dir.join("other.kdl"), "timeout 2000").unwrap();
        thread::sleep(Duration::from_millis(10));
        assert!(!cr.consume_inotify_events());

        fs::write(&path, "timeout 3000").unwrap();
        assert!(wait_for_change(&mut cr));
        assert!(!cr.consume_inotify_events());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn inotify_detects_atomic_replace() {
        let dir = unique_dir("inotify-rename");
        let path = dir.join("config.kdl");
        let temp = dir.join("config.kdl.tmp");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        fs::write(&path, "timeout 1000").unwrap();
        let mut cr = ConfigReloader::try_init_inotify(path.clone()).unwrap();

        fs::write(&temp, "timeout 2000").unwrap();
        fs::rename(&temp, &path).unwrap();
        assert!(wait_for_change(&mut cr));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn inotify_detects_initial_file_creation() {
        let dir = unique_dir("inotify-create");
        let path = dir.join("config.kdl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        let mut cr = ConfigReloader::try_init_inotify(path.clone()).unwrap();

        fs::write(&path, "timeout 1000").unwrap();
        assert!(wait_for_change(&mut cr));

        fs::remove_dir_all(&dir).unwrap();
    }
}
