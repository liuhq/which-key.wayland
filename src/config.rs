mod bind;
pub mod parser;

use std::rc::Rc;

use smithay_client_toolkit::shell::wlr_layer::Anchor;

use crate::{keybind::KeyBindMap, layer::color::WkColor};

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
    pub max_items: u32,
    pub padding: u32,
    pub anchor: Anchor,
    pub margin: Margin,
}

#[derive(Debug)]
pub struct ConfigSeparator {
    pub action: Rc<str>,
    pub group: Rc<str>,
}

#[derive(Debug)]
pub struct Config {
    pub timeout: u32,
    pub bind: KeyBindMap,
    pub font: ConfigFont,
    pub color: ConfigColor,
    pub layout: ConfigLayout,
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
