use super::{ConfigFromKdl, define::Config};
use crate::config::bind::bind_parser;

pub fn config_parse(raw: &str) -> anyhow::Result<Config> {
    let doc: kdl::KdlDocument = raw.parse()?;
    let mut config = Config::from_kdl(&doc)?;
    config.bind = bind_parser(&doc)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::color::WkColor;
    use smithay_client_toolkit::shell::wlr_layer::Anchor;

    #[test]
    fn parse_default_config() {
        let config = config_parse("").expect("should parse empty config");

        assert_eq!(config.timeout, 2000);
        assert!((config.font.size - 16.0).abs() < f32::EPSILON);
        assert!((config.font.line_height - 20.0).abs() < f32::EPSILON);
        assert_eq!(config.color.fg_key, WkColor::rgba(255, 255, 255, 255),);
        assert_eq!(config.color.fg_separator, WkColor::rgba(255, 255, 255, 255),);
        assert_eq!(
            config.color.fg_description,
            WkColor::rgba(255, 255, 255, 255),
        );
        assert_eq!(config.color.bg, WkColor::rgba(0, 0, 0, 255),);
        assert_eq!(config.layout.width, 500);
        assert_eq!(config.layout.max_items, 10);
        assert_eq!(config.layout.padding, 4);
        assert_eq!(
            config.layout.anchor,
            Anchor::union(Anchor::BOTTOM, Anchor::RIGHT),
        );
        assert_eq!(config.layout.margin.top, 0);
        assert_eq!(config.layout.margin.right, 0);
        assert_eq!(config.layout.margin.bottom, 0);
        assert_eq!(config.layout.margin.left, 0);
    }

    #[test]
    fn parse_example_config() {
        let raw = include_str!("../../../examples/config.kdl");
        let config = config_parse(raw).expect("should parse example config");

        assert_eq!(config.timeout, 0);
        assert!((config.font.size - 16.0).abs() < f32::EPSILON);
        assert!((config.font.line_height - 20.0).abs() < f32::EPSILON);
        assert_eq!(config.color.fg_key, WkColor::from_hex("#D8DEE9").unwrap(),);
        assert_eq!(
            config.color.fg_separator,
            WkColor::from_hex("#4C566A").unwrap(),
        );
        assert_eq!(
            config.color.fg_description,
            WkColor::from_hex("#88C0D0").unwrap(),
        );
        assert_eq!(config.color.bg, WkColor::from_hex("#2E3440").unwrap(),);
        assert_eq!(config.layout.width, 500);
        assert_eq!(config.layout.max_items, 10);
        assert_eq!(config.layout.padding, 8);
        assert_eq!(config.layout.margin.top, 0);
        assert_eq!(config.layout.margin.right, 4);
        assert_eq!(config.layout.margin.bottom, 4);
        assert_eq!(config.layout.margin.left, 0);
    }
}
