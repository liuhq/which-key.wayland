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
        let c = config_parse("").expect("should parse empty config");

        assert_eq!(c.timeout, 2000);
        assert!((c.font.size - 16.0).abs() < f32::EPSILON);
        assert!((c.font.line_height - 20.0).abs() < f32::EPSILON);
        assert_eq!(c.color.fg_key, WkColor::rgba(216, 222, 233, 255));
        assert_eq!(c.color.fg_separator, WkColor::rgba(76, 86, 106, 255));
        assert_eq!(c.color.fg_action, WkColor::rgba(136, 192, 208, 255));
        assert_eq!(c.color.fg_group, WkColor::rgba(94, 129, 172, 255));
        assert_eq!(c.color.bg, WkColor::rgba(46, 52, 64, 255));
        assert_eq!(c.layout.width, 500);
        assert_eq!(c.layout.max_items, 10);
        assert_eq!(c.layout.padding, 4);
        assert_eq!(c.layout.radius, 0);
        assert_eq!(
            c.layout.anchor,
            Anchor::union(Anchor::BOTTOM, Anchor::RIGHT)
        );
        assert_eq!(c.layout.margin.top, 0);
        assert_eq!(c.layout.margin.right, 0);
        assert_eq!(c.layout.margin.bottom, 0);
        assert_eq!(c.layout.margin.left, 0);
    }

    #[test]
    fn parse_example_config() {
        let raw = include_str!("../../../examples/config.kdl");
        let c = config_parse(raw).expect("should parse example config");

        assert_eq!(c.timeout, 2000);
        assert!((c.font.size - 16.0).abs() < f32::EPSILON);
        assert!((c.font.line_height - 20.0).abs() < f32::EPSILON);
        assert_eq!(c.color.fg_key, WkColor::from_hex("#D8DEE9").unwrap());
        assert_eq!(c.color.fg_separator, WkColor::from_hex("#4C566A").unwrap());
        assert_eq!(c.color.fg_action, WkColor::from_hex("#88C0D0").unwrap());
        assert_eq!(c.color.fg_group, WkColor::from_hex("#5E81AC").unwrap());
        assert_eq!(c.color.bg, WkColor::from_hex("#2E3440").unwrap());
        assert_eq!(c.layout.width, 500);
        assert_eq!(c.layout.max_items, 10);
        assert_eq!(c.layout.padding, 4);
        assert_eq!(c.layout.radius, 0);
        assert_eq!(
            c.layout.anchor,
            Anchor::union(Anchor::BOTTOM, Anchor::RIGHT),
        );
        assert_eq!(c.layout.margin.top, 0);
        assert_eq!(c.layout.margin.right, 0);
        assert_eq!(c.layout.margin.bottom, 0);
        assert_eq!(c.layout.margin.left, 0);
    }
}
