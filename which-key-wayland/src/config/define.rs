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
    #[node(default = WkColor::rgba(216, 222, 233, 255))]
    pub fg_key: WkColor,
    #[node(default = WkColor::rgba(76, 86, 106, 255))]
    pub fg_separator: WkColor,
    #[node(default = WkColor::rgba(136, 192, 208, 255))]
    pub fg_action: WkColor,
    #[node(default = WkColor::rgba(94, 129, 172, 255))]
    pub fg_group: WkColor,
    #[node(default = WkColor::rgba(46, 52, 64, 255))]
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
    #[node(default = 0)]
    pub radius: u32,
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
    pub fn with_padding(&self, value: u32) -> u32 {
        value + self.layout.padding * 2
    }

    pub fn without_padding(&self, value: u32) -> u32 {
        value - self.layout.padding * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_padding_adds_double_padding() {
        let config = Config::default();
        assert_eq!(config.with_padding(100), 100 + 4 * 2);
        assert_eq!(config.with_padding(0), 8);
    }

    #[test]
    fn without_padding_subtracts_double_padding() {
        let config = Config::default();
        assert_eq!(config.without_padding(108), 100);
        assert_eq!(config.without_padding(8), 0);
    }

    #[test]
    fn footer_default_items() {
        let footer = Footer::default();
        assert_eq!(footer.items.len(), 3);
        assert_eq!(
            footer.items[0],
            ("Esc".to_string(), "Back/Quit".to_string())
        );
        assert_eq!(
            footer.items[1],
            ("Ctrl+U".to_string(), "PageUp".to_string())
        );
        assert_eq!(
            footer.items[2],
            ("Ctrl+D".to_string(), "PageDown".to_string())
        );
    }

    #[test]
    fn footer_display() {
        let footer = Footer::default();
        let display = footer.to_string();
        assert_eq!(display, "Esc Back/Quit  Ctrl+U PageUp  Ctrl+D PageDown");
    }

    #[test]
    fn footer_display_empty() {
        let footer = Footer { items: Vec::new() };
        assert_eq!(footer.to_string(), "");
    }

    #[test]
    fn footer_display_single() {
        let footer = Footer {
            items: vec![("F1".to_string(), "Help".to_string())],
        };
        assert_eq!(footer.to_string(), "F1 Help");
    }

    #[test]
    fn margin_default() {
        let m = Margin::default();
        assert_eq!(m.top, 0);
        assert_eq!(m.right, 0);
        assert_eq!(m.bottom, 0);
        assert_eq!(m.left, 0);
    }

    #[test]
    fn config_font_default() {
        let f = ConfigFont::default();
        assert!((f.size - 16.0).abs() < f32::EPSILON);
        assert!((f.line_height - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn config_color_default() {
        let c = ConfigColor::default();
        assert_eq!(c.fg_key, WkColor::rgba(216, 222, 233, 255));
        assert_eq!(c.fg_separator, WkColor::rgba(76, 86, 106, 255));
        assert_eq!(c.fg_action, WkColor::rgba(136, 192, 208, 255));
        assert_eq!(c.fg_group, WkColor::rgba(94, 129, 172, 255));
        assert_eq!(c.bg, WkColor::rgba(46, 52, 64, 255));
    }

    #[test]
    fn config_layout_default() {
        let l = ConfigLayout::default();
        assert_eq!(l.width, 500);
        assert_eq!(l.max_items, 10);
        assert_eq!(l.padding, 4);
        assert_eq!(l.radius, 0);
        assert_eq!(l.anchor, Anchor::union(Anchor::BOTTOM, Anchor::RIGHT));
    }

    #[test]
    fn config_timeout_default() {
        let c = Config::default();
        assert_eq!(c.timeout, 2000);
    }
}
