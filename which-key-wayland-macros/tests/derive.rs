use std::rc::Rc;

use which_key_wayland_macros::KdlParse;

mod config {
    pub trait ConfigFromKdl: Sized {
        fn from_kdl(doc: &kdl::KdlDocument) -> anyhow::Result<Self>;
    }
}

use config::ConfigFromKdl;

#[derive(Debug, KdlParse)]
struct Scalars {
    unsigned: u32,
    signed: i32,
    float32: f32,
    float64: f64,
    owned: String,
    shared: Rc<str>,
}

#[derive(Debug, KdlParse)]
struct Child {
    #[node(default = 7)]
    value: u32,
}

#[derive(Debug, KdlParse)]
struct Settings {
    #[node(default = 10, rename = "item-count")]
    item_count: u32,
    #[node(skip, default = "ignored".to_string())]
    ignored: String,
    #[node(default)]
    child: Child,
}

#[test]
fn generated_parser_reads_all_general_scalar_types() {
    let document: kdl::KdlDocument = r#"
        unsigned 42
        signed -3
        float32 1.5
        float64 2.25
        owned "owned value"
        shared "shared value"
    "#
    .parse()
    .unwrap();

    let parsed = Scalars::from_kdl(&document).unwrap();

    assert_eq!(parsed.unsigned, 42);
    assert_eq!(parsed.signed, -3);
    assert_eq!(parsed.float32, 1.5);
    assert_eq!(parsed.float64, 2.25);
    assert_eq!(parsed.owned, "owned value");
    assert_eq!(&*parsed.shared, "shared value");
}

#[test]
fn generated_parser_applies_rename_skip_and_nested_defaults() {
    let document: kdl::KdlDocument = "item-count 25".parse().unwrap();

    let parsed = Settings::from_kdl(&document).unwrap();

    assert_eq!(parsed.item_count, 25);
    assert_eq!(parsed.ignored, "ignored");
    assert_eq!(parsed.child.value, 7);
}

#[test]
fn generated_default_uses_all_declared_defaults() {
    let settings = Settings::default();

    assert_eq!(settings.item_count, 10);
    assert_eq!(settings.ignored, "ignored");
    assert_eq!(settings.child.value, 7);
}

#[test]
fn generated_parser_reports_missing_required_value() {
    let document = kdl::KdlDocument::new();

    let error = Scalars::from_kdl(&document).unwrap_err();

    assert!(error.to_string().contains("unsigned: not found"));
}
