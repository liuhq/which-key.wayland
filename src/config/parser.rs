use kdl::{KdlDocument, KdlValue};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

use crate::{
    config::{Config, ConfigColor, ConfigFont, ConfigLayout, Margin, bind::bind_parser},
    layer::color::WkColor,
};

fn get_children<'a>(config: &'a KdlDocument, key: &str) -> anyhow::Result<&'a KdlDocument> {
    config
        .get(key)
        .and_then(|n| n.children())
        .ok_or_else(|| anyhow::anyhow!("{key}: not found"))
}

fn get_integer(config: &KdlDocument, key: &str, field: &str) -> anyhow::Result<i128> {
    match config.get_arg(key) {
        Some(KdlValue::Integer(val)) => Ok(*val),
        _ => anyhow::bail!("{field}: not found or not an integer"),
    }
}

fn get_float(config: &KdlDocument, key: &str, field: &str) -> anyhow::Result<f64> {
    match config.get_arg(key) {
        Some(KdlValue::Float(val)) => Ok(*val),
        _ => anyhow::bail!("{field}: not found or not a float"),
    }
}

fn get_string<'a>(config: &'a KdlDocument, key: &str, field: &str) -> anyhow::Result<&'a str> {
    match config.get_arg(key) {
        Some(KdlValue::String(val)) => Ok(val.as_str()),
        _ => anyhow::bail!("{field}: not found or not a string"),
    }
}

fn get_u32(config: &KdlDocument, key: &str, field: &str) -> anyhow::Result<u32> {
    get_integer(config, key, field)?
        .try_into()
        .map_err(|e| anyhow::anyhow!("{field} integer overflow: {e}"))
}

fn get_i32(config: &KdlDocument, key: &str, field: &str) -> anyhow::Result<i32> {
    get_integer(config, key, field)?
        .try_into()
        .map_err(|e| anyhow::anyhow!("{field} integer overflow: {e}"))
}

pub fn config_parse(raw: &str) -> anyhow::Result<Config> {
    let config: KdlDocument = raw.parse()?;

    let timeout = get_u32(&config, "timeout", "timeout")?;
    let font = parse_font(&config)?;
    let color = parse_color(&config)?;
    let layout = parse_layout(&config)?;
    let bind = bind_parser(&config)?;

    Ok(Config {
        timeout,
        keybinds: vec![],
        bind,
        font,
        color,
        layout,
    })
}

fn parse_font(config: &KdlDocument) -> anyhow::Result<ConfigFont> {
    let font = get_children(config, "font")?;

    let size = get_float(font, "size", "font.size")? as f32;
    let line_height = get_float(font, "line-height", "font.line-height")? as f32;

    Ok(ConfigFont { size, line_height })
}

fn parse_color(config: &KdlDocument) -> anyhow::Result<ConfigColor> {
    let color = get_children(config, "color")?;

    let fg = get_string(color, "fg", "color.fg")?;
    let bg = get_string(color, "bg", "color.bg")?;

    Ok(ConfigColor {
        fg: WkColor::from_hex(fg).ok_or_else(|| anyhow::anyhow!("color.fg: invalid hex color"))?,
        bg: WkColor::from_hex(bg).ok_or_else(|| anyhow::anyhow!("color.bg: invalid hex color"))?,
    })
}

fn parse_anchor(val: i128) -> Anchor {
    match val {
        1 => Anchor::union(Anchor::TOP, Anchor::RIGHT),
        2 => Anchor::union(Anchor::BOTTOM, Anchor::RIGHT),
        3 => Anchor::union(Anchor::BOTTOM, Anchor::LEFT),
        4 => Anchor::union(Anchor::TOP, Anchor::LEFT),
        _ => Anchor::union(Anchor::BOTTOM, Anchor::RIGHT), // Default
    }
}

fn parse_margin(layout: &KdlDocument) -> anyhow::Result<Margin> {
    let margin = get_children(layout, "margin")?;

    let top = get_i32(margin, "top", "layout.margin.top")?;
    let right = get_i32(margin, "right", "layout.margin.right")?;
    let bottom = get_i32(margin, "bottom", "layout.margin.bottom")?;
    let left = get_i32(margin, "left", "layout.margin.left")?;

    Ok(Margin {
        top,
        right,
        bottom,
        left,
    })
}

fn parse_layout(config: &KdlDocument) -> anyhow::Result<ConfigLayout> {
    let layout = get_children(config, "layout")?;

    let width = get_u32(layout, "width", "layout.width")?;
    let max_height = get_u32(layout, "max-height", "layout.max-height")?;
    let max_items = get_u32(layout, "max-items", "layout.max-items")?;
    let padding = get_u32(layout, "padding", "layout.padding")?;
    let anchor = parse_anchor(get_integer(layout, "anchor", "layout.anchor")?);
    let margin = parse_margin(layout)?;

    Ok(ConfigLayout {
        width,
        max_height,
        max_items,
        padding,
        anchor,
        margin,
    })
}
