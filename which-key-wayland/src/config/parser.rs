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

    #[test]
    fn parse_rejects_wrong_scalar_type() {
        let error = config_parse("timeout \"slow\"").unwrap_err();
        assert!(error.to_string().contains("unexpected KDL value type"));
    }

    #[test]
    fn parse_rejects_invalid_anchor() {
        let error = config_parse("layout { anchor 5; }").unwrap_err();
        assert!(error.to_string().contains("invalid anchor value 5"));
    }

    #[test]
    fn parse_rejects_invalid_color() {
        let error = config_parse("color { fg-key \"not-a-color\"; }").unwrap_err();
        assert!(error.to_string().contains("invalid hex color"));
    }

    #[test]
    fn parse_rejects_u32_overflow() {
        let error = config_parse("layout { width 4294967296; }").unwrap_err();
        assert!(error.to_string().contains("integer overflow"));
    }

    #[test]
    fn parse_uses_kebab_case_names_and_nested_values() {
        let config = config_parse(
            "font { size 18.0; line-height 24.0; }\nlayout { margin { top 1; right 2; bottom 3; left 4; } }",
        )
        .unwrap();

        assert_eq!(config.font.size, 18.0);
        assert_eq!(config.font.line_height, 24.0);
        assert_eq!(config.layout.margin.top, 1);
        assert_eq!(config.layout.margin.right, 2);
        assert_eq!(config.layout.margin.bottom, 3);
        assert_eq!(config.layout.margin.left, 4);
    }
}
