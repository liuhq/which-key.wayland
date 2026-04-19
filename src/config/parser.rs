use kdl::{KdlDocument, KdlValue};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

use crate::{
    config::{Config, ConfigColor, ConfigFont, ConfigLayout, Margin, bind::bind_parser},
    layer::color::WkColor,
};

pub fn config_parse(raw: &str) -> anyhow::Result<Config> {
    let config: KdlDocument = raw.parse()?;

    let Some(KdlValue::Integer(timeout)) = config.get_arg("timeout") else {
        anyhow::bail!("timeout: not found or not a integer")
    };

    let font = parse_font(&config)?;
    let color = parse_color(&config)?;
    let layout = parse_layout(&config)?;
    let bind = bind_parser(&config)?;

    println!("end: run config parser");
    Ok(Config {
        timeout: *timeout as u32,
        keybinds: vec![],
        bind,
        font,
        color,
        layout,
    })
}

fn parse_font(config: &KdlDocument) -> anyhow::Result<ConfigFont> {
    let Some(font) = config.get("font").and_then(|n| n.children()) else {
        anyhow::bail!("font: not fount")
    };

    let Some(KdlValue::Float(size)) = font.get_arg("size") else {
        anyhow::bail!("font.size: not found or not a float")
    };
    let Some(KdlValue::Float(line_height)) = font.get_arg("line-height") else {
        anyhow::bail!("font.line_height: not found or not a float")
    };

    Ok(ConfigFont {
        size: *size as f32,
        line_height: *line_height as f32,
    })
}

fn parse_color(config: &KdlDocument) -> anyhow::Result<ConfigColor> {
    let Some(color) = config.get("color").and_then(|n| n.children()) else {
        anyhow::bail!("color: not fount")
    };

    let Some(KdlValue::String(fg)) = color.get_arg("fg") else {
        anyhow::bail!("color.fg: not found or not a string")
    };
    let Some(KdlValue::String(bg)) = color.get_arg("bg") else {
        anyhow::bail!("color.bg: not found or not a string")
    };

    Ok(ConfigColor {
        fg: WkColor::from_hex(fg).unwrap(),
        bg: WkColor::from_hex(bg).unwrap(),
    })
}

fn parse_layout(config: &KdlDocument) -> anyhow::Result<ConfigLayout> {
    let Some(layout) = config.get("layout").and_then(|n| n.children()) else {
        anyhow::bail!("layout: not fount")
    };

    let Some(KdlValue::Integer(width)) = layout.get_arg("width") else {
        anyhow::bail!("layout.width: not found or not a integer")
    };
    let Some(KdlValue::Integer(max_height)) = layout.get_arg("max_height") else {
        anyhow::bail!("layout.max_height: not found or not a integer")
    };
    let Some(KdlValue::Integer(padding)) = layout.get_arg("padding") else {
        anyhow::bail!("layout.padding: not found or not a integer")
    };
    let Some(KdlValue::String(anchor)) = layout.get_arg("anchor") else {
        anyhow::bail!("layout.anchor: not found or not a string")
    };

    Ok(ConfigLayout {
        width: *width as u32,
        max_height: *max_height as u32,
        padding: *padding as u32,
        anchor: Anchor::union(Anchor::RIGHT, Anchor::BOTTOM),
        margin: Margin {
            top: 0,
            right: 4,
            bottom: 4,
            left: 0,
        },
    })
}
