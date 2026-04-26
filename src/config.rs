mod bind;
pub mod parser;

use std::collections::HashMap;

use smithay_client_toolkit::shell::wlr_layer::Anchor;

use crate::{keybind::KeyBindMap, layer::color::WkColor};

#[derive(Debug)]
pub struct WkEntry {
    pub prefix: String,
    pub separator: String,
    pub description: String,
}

impl WkEntry {
    pub fn new(prefix: String, description: String) -> Self {
        Self {
            prefix,
            separator: String::from(" -> "),
            description,
        }
    }
}

impl std::fmt::Display for WkEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.prefix, self.separator, self.description)
    }
}

#[derive(Debug)]
pub struct Margin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug)]
pub struct ConfigFont {
    pub size: f32,
    pub line_height: f32,
}

#[derive(Debug)]
pub struct ConfigColor {
    pub fg: WkColor,
    pub bg: WkColor,
}

#[derive(Debug)]
pub struct ConfigLayout {
    pub width: u32,
    pub max_height: u32,
    pub max_items: u32,
    pub padding: u32,
    pub anchor: Anchor,
    pub margin: Margin,
}

#[derive(Debug)]
pub struct Config {
    pub timeout: u32,
    pub keybinds: Vec<WkEntry>,
    pub bind: KeyBindMap,
    pub font: ConfigFont,
    pub color: ConfigColor,
    pub layout: ConfigLayout,
}

impl Config {
    pub fn mock() -> Self {
        Self {
            keybinds: vec![
                WkEntry::new(
                    "A".to_string(),
                    "The description's text is wrapped when the line is too long to fit"
                        .to_string(),
                ),
                WkEntry::new(
                    "BCDEFG".to_string(),
                    "The description's text is wrapped".to_string(),
                ),
                WkEntry::new("C".to_string(), "The description's text".to_string()),
                WkEntry::new(
                    "D".to_string(),
                    "The description's text is wrapped when the line is too long to fit"
                        .to_string(),
                ),
            ],
            bind: HashMap::new(),
            timeout: 1000,
            font: ConfigFont {
                size: 16.0,
                line_height: 20.0,
            },
            color: ConfigColor {
                fg: WkColor::rgb(255, 255, 255),
                bg: WkColor::rgba(0, 0, 0, 255),
            },
            layout: ConfigLayout {
                width: 360,
                max_height: 720,
                max_items: 10,
                padding: 16,
                anchor: Anchor::union(Anchor::RIGHT, Anchor::BOTTOM),
                margin: Margin {
                    top: 0,
                    right: 4,
                    bottom: 4,
                    left: 0,
                },
            },
        }
    }

    pub(crate) fn with_padding(&self, value: u32) -> u32 {
        value + self.layout.padding * 2
    }

    pub(crate) fn without_padding(&self, value: u32) -> u32 {
        value - self.layout.padding * 2
    }
}
