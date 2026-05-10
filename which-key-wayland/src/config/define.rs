use std::rc::Rc;

use smithay_client_toolkit::shell::wlr_layer::Anchor;
use which_key_wayland_macros::KdlParse;

use crate::{keybind::KeyBindMap, layer::color::WkColor};

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
    pub fg: WkColor,
    #[node(default = WkColor::rgba(0, 0, 0, 255))]
    pub bg: WkColor,
}

#[derive(Debug, KdlParse)]
pub struct ConfigLayout {
    #[node(default = 400)]
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
pub struct ConfigSeparator {
    #[node(default = Rc::from(" -> "))]
    pub action: Rc<str>,
    #[node(default = Rc::from(" ++ "))]
    pub group: Rc<str>,
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
    #[node(default)]
    pub separator: ConfigSeparator,
}

impl Config {
    pub(crate) fn with_padding(&self, value: u32) -> u32 {
        value + self.layout.padding * 2
    }

    pub(crate) fn without_padding(&self, value: u32) -> u32 {
        value - self.layout.padding * 2
    }
}
