pub const OPAQUE_ALPHA: u8 = u8::MAX;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct WkColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl WkColor {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Support: #RGB, #RRGGBB, #RRGGBBAA (`#` is also optional)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix("#").unwrap_or(hex);
        let parse_byte = |s: &str| u8::from_str_radix(s, 16).ok();

        match hex.chars().count() {
            3 => Some(Self {
                r: parse_byte(&hex[0..1].repeat(2))?,
                g: parse_byte(&hex[1..2].repeat(2))?,
                b: parse_byte(&hex[2..3].repeat(2))?,
                a: OPAQUE_ALPHA,
            }),
            6 => Some(Self {
                r: parse_byte(&hex[0..2])?,
                g: parse_byte(&hex[2..4])?,
                b: parse_byte(&hex[4..6])?,
                a: OPAQUE_ALPHA,
            }),
            8 => Some(Self {
                r: parse_byte(&hex[0..2])?,
                g: parse_byte(&hex[2..4])?,
                b: parse_byte(&hex[4..6])?,
                a: parse_byte(&hex[6..8])?,
            }),
            _ => None,
        }
    }
}

pub trait WkColorPixelOps: Copy + Into<f32> {
    fn normalize_alpha(self) -> f32 {
        self.into() / 255.0
    }

    fn blend_to(self, destination: u8, alpha: f32) -> u8 {
        (self.into() * alpha + destination as f32 * (1.0 - alpha)).round() as u8
    }
}

impl WkColorPixelOps for u8 {}

impl From<WkColor> for tiny_skia::Color {
    fn from(val: WkColor) -> Self {
        let WkColor { r, g, b, a } = val;
        tiny_skia::Color::from_rgba8(r, g, b, a)
    }
}

impl From<WkColor> for cosmic_text::Color {
    fn from(val: WkColor) -> Self {
        let WkColor { r, g, b, a } = val;
        cosmic_text::Color::rgba(r, g, b, a)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hex_with_hash() {
        let c = WkColor::from_hex("#FF6611").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xFF, 0x66, 0x11, 0xFF));
    }

    #[test]
    fn hex_without_hash() {
        let c = WkColor::from_hex("FF6611").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xFF, 0x66, 0x11, 0xFF));
    }

    #[test]
    fn hex_3char_with_hash() {
        let c = WkColor::from_hex("#F61").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xFF, 0x66, 0x11, 0xFF));
    }

    #[test]
    fn hex_3char_without_hash() {
        let c = WkColor::from_hex("F61").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xFF, 0x66, 0x11, 0xFF));
    }

    #[test]
    fn hex_8char_with_hash() {
        let c = WkColor::from_hex("#FF661188").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xFF, 0x66, 0x11, 0x88));
    }

    #[test]
    fn hex_8char_without_hash() {
        let c = WkColor::from_hex("FF661188").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xFF, 0x66, 0x11, 0x88));
    }

    #[test]
    fn hex_invalid_length() {
        assert!(WkColor::from_hex("FF").is_none());
        assert!(WkColor::from_hex("FF661").is_none());
        assert!(WkColor::from_hex("FF661122AAFF").is_none());
        assert!(WkColor::from_hex("").is_none());
    }

    #[test]
    fn hex_invalid_chars() {
        assert!(WkColor::from_hex("#GGGGGG").is_none());
        assert!(WkColor::from_hex("#ZZZZZZ").is_none());
        assert!(WkColor::from_hex("#00000G").is_none());
    }

    #[test]
    fn hex_lowercase() {
        let c = WkColor::from_hex("#aabbcc").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xAA, 0xBB, 0xCC, 0xFF));
    }

    #[test]
    fn hex_mixed_case() {
        let c = WkColor::from_hex("#aAbbCc").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (0xAA, 0xBB, 0xCC, 0xFF));
    }

    #[test]
    fn hex_black_and_white() {
        let black = WkColor::from_hex("#000000").unwrap();
        assert_eq!((black.r, black.g, black.b, black.a), (0, 0, 0, 255));

        let white = WkColor::from_hex("#FFFFFF").unwrap();
        assert_eq!((white.r, white.g, white.b, white.a), (255, 255, 255, 255));
    }

    #[test]
    fn rgb_constructor() {
        let c = WkColor::rgb(10, 20, 30);
        assert_eq!((c.r, c.g, c.b, c.a), (10, 20, 30, 255));
    }

    #[test]
    fn rgba_constructor() {
        let c = WkColor::rgba(10, 20, 30, 128);
        assert_eq!((c.r, c.g, c.b, c.a), (10, 20, 30, 128));
    }

    #[test]
    fn normalize_alpha() {
        assert!((0u8.normalize_alpha() - 0.0).abs() < f32::EPSILON);
        assert!((255u8.normalize_alpha() - 1.0).abs() < f32::EPSILON);
        assert!((128u8.normalize_alpha() - (128.0 / 255.0)).abs() < 0.001);
    }

    #[test]
    fn blend_to_opaque_fg_on_black() {
        let result = 255u8.blend_to(0, 1.0);
        assert_eq!(result, 255);
    }

    #[test]
    fn blend_to_transparent_fg_on_black() {
        let result = 255u8.blend_to(0, 0.0);
        assert_eq!(result, 0);
    }

    #[test]
    fn blend_to_half_alpha() {
        let result = 200u8.blend_to(100, 0.5);
        assert_eq!(result, 150);
    }

    #[test]
    fn default_color_is_black_transparent() {
        let c = WkColor::default();
        assert_eq!((c.r, c.g, c.b, c.a), (0, 0, 0, 0));
    }
}
