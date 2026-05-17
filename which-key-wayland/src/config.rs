mod bind;
mod define;
pub mod parser;

use std::{env, path::PathBuf};

pub use define::{Config, Footer, SYMBOL_INDICATOR};

use crate::config::parser::config_parse;

pub trait ConfigFromKdl: Sized {
    fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>;
}

impl Config {
    const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
    const WKW_CONFIG_FILE: &str = "WKW_CONFIG_FILE";

    pub fn config_path() -> Option<String> {
        if let Ok(p) = env::var(Self::WKW_CONFIG_FILE) {
            return Some(p);
        }
        let base = env::var(Self::XDG_CONFIG_HOME)
            .ok()
            .map(PathBuf::from)
            .or_else(|| env::home_dir().map(|p| p.join(".config")))?;

        Some(
            base.join("which-key-wayland/config.kdl")
                .to_string_lossy()
                .into_owned(),
        )
    }

    pub fn init() -> Self {
        let Some(path) = Self::config_path() else {
            return Config::default();
        };
        match std::fs::read_to_string(&path) {
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
