use smithay_client_toolkit::shell::wlr_layer::Anchor;
use which_key_wayland_macros::KdlParse;

use crate::{keybind::KeyBindMap, layer::color::WkColor};

pub static SYMBOL_INDICATOR: &str = "\u{ffeb}";
pub static SYMBOL_GROUP: &str = "\u{2b}";

#[derive(Debug, Clone)]
pub struct Footer {
    pub items: Vec<(String, String)>,
}

impl Default for Footer {
    fn default() -> Self {
        Self {
            items: vec![
                ("Esc".to_string(), "Back/Quit".to_string()),
                ("Ctrl+U".to_string(), "PageUp".to_string()),
                ("Ctrl+D".to_string(), "PageDown".to_string()),
            ],
        }
    }
}

impl std::fmt::Display for Footer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (k, d)) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, "  ")?;
            }
            write!(f, "{k} {d}")?;
        }
        Ok(())
    }
}

#[derive(Debug, KdlParse)]
pub struct Margin {
    #[node(default = 0)]
    pub top: i32,
    #[node(default = 0)]
    pub right: i32,
    #[node(default = 0)]
    pub bottom: i32,
    #[node(default = 0)]
    pub left: i32,
}

#[derive(Debug, KdlParse)]
pub struct ConfigFont {
    #[node(default = 16.0)]
    pub size: f32,
    #[node(default = 20.0)]
    pub line_height: f32,
}

#[derive(Debug, KdlParse)]
pub struct ConfigColor {
    #[node(default = WkColor::rgba(255, 255, 255, 255))]
    pub fg_key: WkColor,
    #[node(default = WkColor::rgba(255, 255, 255, 255))]
    pub fg_separator: WkColor,
    #[node(default = WkColor::rgba(255, 255, 255, 255))]
    pub fg_description: WkColor,
    #[node(default = WkColor::rgba(0, 0, 0, 255))]
    pub bg: WkColor,
}

#[derive(Debug, KdlParse)]
pub struct ConfigLayout {
    #[node(default = 500)]
    pub width: u32,
    #[node(default = 10)]
    pub max_items: u32,
    #[node(default = 4)]
    pub padding: u32,
    #[node(default = Anchor::union(Anchor::BOTTOM, Anchor::RIGHT))]
    pub anchor: Anchor,
    #[node(default)]
    pub margin: Margin,
}

#[derive(Debug, KdlParse)]
pub struct Config {
    #[node(default = 2000)]
    pub timeout: u32,
    #[node(skip)]
    pub bind: KeyBindMap,
    #[node(default)]
    pub font: ConfigFont,
    #[node(default)]
    pub color: ConfigColor,
    #[node(default)]
    pub layout: ConfigLayout,
}

impl Config {
    pub(crate) fn with_padding(&self, value: u32) -> u32 {
        value + self.layout.padding * 2
    }

    pub(crate) fn without_padding(&self, value: u32) -> u32 {
        value - self.layout.padding * 2
    }
}
