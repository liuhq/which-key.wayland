pub(crate) const OPAQUE_ALPHA: u8 = u8::MAX;

#[derive(Clone, Copy, Default, Debug)]
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

pub(crate) trait WkColorPixelOps: Copy + Into<f32> {
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
        let WkColor { r, g, b, .. } = val;
        cosmic_text::Color::rgb(r, g, b)
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
}
