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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::Duration};

    fn unique_path(label: &str) -> PathBuf {
        let pid = std::process::id();
        PathBuf::from(format!("/tmp/which-key-reloader-{}-{}.kdl", label, pid))
    }

    #[test]
    fn init_mtime_existing_file_detected() {
        let path = unique_path("existing");
        fs::write(&path, "timeout 1000").unwrap();
        let mut cr = ConfigReloader::init_mtime(path.clone());
        assert!(
            cr.has_changed_by_mtime(),
            "existing file should be detected"
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn init_mtime_nonexistent_file_returns_none() {
        let path = unique_path("nonexistent1");
        let _ = fs::remove_file(&path);
        let cr = ConfigReloader::init_mtime(path);
        assert!(
            cr.try_read_mtime().is_none(),
            "non-existent file should yield None"
        );
    }

    #[test]
    fn init_mtime_nonexistent_file_has_changed_is_false() {
        let path = unique_path("nonexistent2");
        let _ = fs::remove_file(&path);
        let mut cr = ConfigReloader::init_mtime(path);
        assert!(
            !cr.has_changed_by_mtime(),
            "non-existent file should return false"
        );
    }

    #[test]
    fn try_read_mtime_existing_file_returns_some() {
        let path = unique_path("try_read_some");
        fs::write(&path, "timeout 1000").unwrap();
        let cr = ConfigReloader::init_mtime(path.clone());
        assert!(
            cr.try_read_mtime().is_some(),
            "should read mtime from existing file"
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn try_read_mtime_nonexistent_file_returns_none() {
        let path = unique_path("try_read_none");
        let _ = fs::remove_file(&path);
        let cr = ConfigReloader::init_mtime(path);
        assert!(cr.try_read_mtime().is_none());
    }

    #[test]
    fn try_read_mtime_reflects_file_change() {
        let path = unique_path("changed");
        fs::write(&path, "timeout 1000").unwrap();
        let cr = ConfigReloader::init_mtime(path.clone());
        let mtime1 = cr.try_read_mtime().unwrap();

        std::thread::sleep(Duration::from_millis(10));
        fs::write(&path, "timeout 2000").unwrap();

        let mtime2 = cr.try_read_mtime().unwrap();
        assert!(mtime2 > mtime1, "mtime should increase after file write");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn has_changed_by_mtime_false_after_file_deleted() {
        let path = unique_path("deleted");
        fs::write(&path, "timeout 1000").unwrap();
        let mut cr = ConfigReloader::init_mtime(path.clone());
        assert!(
            cr.has_changed_by_mtime(),
            "existing file should return true"
        );

        let mtime_before = cr.try_read_mtime();
        assert!(mtime_before.is_some());

        fs::remove_file(&path).unwrap();
        assert!(
            !cr.has_changed_by_mtime(),
            "deleted file should return false"
        );
        assert!(
            cr.try_read_mtime().is_none(),
            "mtime should be None after deletion"
        );
    }
}
