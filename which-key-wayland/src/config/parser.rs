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
        assert_eq!(config.layout.radius, 0);
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
        assert_eq!(config.layout.radius, 8);
        assert_eq!(
            config.layout.anchor,
            Anchor::union(Anchor::BOTTOM, Anchor::RIGHT),
        );
        assert_eq!(config.layout.margin.top, 0);
        assert_eq!(config.layout.margin.right, 4);
        assert_eq!(config.layout.margin.bottom, 4);
        assert_eq!(config.layout.margin.left, 0);
    }

    #[test]
    fn parse_timeout_only() {
        let raw = "timeout 5000";
        let config = config_parse(raw).expect("should parse timeout config");
        assert_eq!(config.timeout, 5000);
    }

    #[test]
    fn parse_font_overrides() {
        let raw = "font {\n  size 24.0\n  line-height 30.0\n}";
        let config = config_parse(raw).expect("should parse font config");
        assert!((config.font.size - 24.0).abs() < f32::EPSILON);
        assert!((config.font.line_height - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_color_block() {
        let raw = "color {\n  fg-key \"#FF0000\"\n  bg \"#0000FF\"\n}";
        let config = config_parse(raw).expect("should parse color config");
        assert_eq!(config.color.fg_key, WkColor::from_hex("#FF0000").unwrap());
        assert_eq!(config.color.bg, WkColor::from_hex("#0000FF").unwrap());
    }

    #[test]
    fn parse_layout_block() {
        let raw = "layout {\n  width 600\n  max-items 20\n  padding 8\n  radius 12\n}";
        let config = config_parse(raw).expect("should parse layout config");
        assert_eq!(config.layout.width, 600);
        assert_eq!(config.layout.max_items, 20);
        assert_eq!(config.layout.padding, 8);
        assert_eq!(config.layout.radius, 12);
    }

    #[test]
    fn parse_invalid_kdl_error() {
        let raw = "invalid {";
        let result = config_parse(raw);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_anchor_error() {
        let raw = "layout {\n  anchor 99\n}";
        let result = config_parse(raw);
        assert!(result.is_err());
    }

    #[test]
    fn parse_anchor_top_right() {
        let raw = "layout {\n  anchor 1\n}";
        let config = config_parse(raw).expect("should parse anchor 1");
        assert_eq!(config.layout.anchor, Anchor::union(Anchor::TOP, Anchor::RIGHT));
    }

    #[test]
    fn parse_anchor_bottom_right() {
        let raw = "layout {\n  anchor 2\n}";
        let config = config_parse(raw).expect("should parse anchor 2");
        assert_eq!(
            config.layout.anchor,
            Anchor::union(Anchor::BOTTOM, Anchor::RIGHT)
        );
    }

    #[test]
    fn parse_anchor_bottom_left() {
        let raw = "layout {\n  anchor 3\n}";
        let config = config_parse(raw).expect("should parse anchor 3");
        assert_eq!(
            config.layout.anchor,
            Anchor::union(Anchor::BOTTOM, Anchor::LEFT)
        );
    }

    #[test]
    fn parse_anchor_top_left() {
        let raw = "layout {\n  anchor 4\n}";
        let config = config_parse(raw).expect("should parse anchor 4");
        assert_eq!(config.layout.anchor, Anchor::union(Anchor::TOP, Anchor::LEFT));
    }

    #[test]
    fn parse_margin_block() {
        let raw = "layout {\n  margin {\n    top 5\n    right 10\n    bottom 15\n    left 20\n  }\n}";
        let config = config_parse(raw).expect("should parse margin");
        assert_eq!(config.layout.margin.top, 5);
        assert_eq!(config.layout.margin.right, 10);
        assert_eq!(config.layout.margin.bottom, 15);
        assert_eq!(config.layout.margin.left, 20);
    }

    #[test]
    fn parse_timeout_zero_means_disabled() {
        let raw = "timeout 0";
        let config = config_parse(raw).expect("should parse zero timeout");
        assert_eq!(config.timeout, 0);
    }
}
