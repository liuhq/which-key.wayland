mod bind;
mod define;
pub mod parser;
pub mod reloader;

use std::{
    env,
    path::{Path, PathBuf},
};

pub use define::{Config, ConfigColor, Footer, SYMBOL_INDICATOR};

use crate::config::parser::config_parse;

pub trait ConfigFromKdl: Sized {
    fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>;
}

impl Config {
    const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
    const WKW_CONFIG_FILE: &str = "WKW_CONFIG_FILE";

    pub fn get_path() -> Option<PathBuf> {
        if let Ok(p) = env::var(Self::WKW_CONFIG_FILE) {
            return Some(PathBuf::from(p));
        }
        let base = env::var(Self::XDG_CONFIG_HOME)
            .ok()
            .map(PathBuf::from)
            .or_else(|| env::home_dir().map(|p| p.join(".config")))?;

        Some(base.join("which-key-wayland/config.kdl"))
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => config_parse(&raw).unwrap_or_else(|e| {
                log::error!("{e}");
                Config::default()
            }),
            Err(e) => {
                log::error!("{e}");
                Config::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        process::Command,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_PATH: AtomicUsize = AtomicUsize::new(0);

    fn test_path(label: &str) -> PathBuf {
        let n = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "which-key-config-{label}-{}-{n}.kdl",
            std::process::id()
        ))
    }

    #[test]
    fn load_reads_valid_configuration() {
        let path = test_path("valid");
        fs::write(&path, "timeout 42\nlayout { width 640; }").unwrap();

        let config = Config::load(&path);

        assert_eq!(config.timeout, 42);
        assert_eq!(config.layout.width, 640);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_invalid_configuration_returns_defaults() {
        let path = test_path("invalid");
        fs::write(&path, "timeout \"slow\"").unwrap();

        let config = Config::load(&path);

        assert_eq!(config.timeout, Config::default().timeout);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_missing_configuration_returns_defaults() {
        let path = test_path("missing");
        let _ = fs::remove_file(&path);

        let config = Config::load(&path);

        assert_eq!(config.timeout, Config::default().timeout);
        assert!(config.bind.map.is_empty());
    }

    #[test]
    fn get_path_prefers_explicit_config_file() {
        const CHILD_MARKER: &str = "WKW_TEST_EXPLICIT_PATH_CHILD";
        let expected = PathBuf::from("/tmp/which-key-explicit-config.kdl");
        if std::env::var_os(CHILD_MARKER).is_some() {
            assert_eq!(Config::get_path(), Some(expected));
            return;
        }

        let status = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("config::tests::get_path_prefers_explicit_config_file")
            .env(CHILD_MARKER, "1")
            .env(Config::WKW_CONFIG_FILE, &expected)
            .env(Config::XDG_CONFIG_HOME, "/tmp/ignored-xdg-config")
            .status()
            .unwrap();

        assert!(status.success());
    }

    #[test]
    fn get_path_uses_xdg_config_home() {
        const CHILD_MARKER: &str = "WKW_TEST_XDG_PATH_CHILD";
        let xdg_home = PathBuf::from("/tmp/which-key-xdg-config");
        let expected = xdg_home.join("which-key-wayland/config.kdl");
        if std::env::var_os(CHILD_MARKER).is_some() {
            assert_eq!(Config::get_path(), Some(expected));
            return;
        }

        let status = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("config::tests::get_path_uses_xdg_config_home")
            .env(CHILD_MARKER, "1")
            .env_remove(Config::WKW_CONFIG_FILE)
            .env(Config::XDG_CONFIG_HOME, &xdg_home)
            .status()
            .unwrap();

        assert!(status.success());
    }
}
