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
