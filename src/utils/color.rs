//! Color utilities for parsing and converting colors
//!
//! Provides RGB, RGBA, and HSL color types with conversions between them,
//! hex parsing, and CSS named color support.

/// RGB color (red, green, blue)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse from hex string (supports #RGB, #RRGGBB, RGB, RRGGBB)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().trim_start_matches('#');

        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self { r, g, b })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self { r, g, b })
            }
            _ => None,
        }
    }

    /// Convert to hex string (#RRGGBB)
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Convert to u32 (0xRRGGBB)
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert RGB to HSL (Hue, Saturation, Lightness)
    ///
    /// # Algorithm
    /// 1. Normalize RGB values from 0-255 to 0.0-1.0
    /// 2. Find max and min of R, G, B components
    /// 3. Calculate Lightness: L = (max + min) / 2
    /// 4. If max == min (achromatic/grayscale), H = 0, S = 0
    /// 5. Calculate Saturation based on Lightness:
    ///    - If L <= 0.5: S = (max - min) / (max + min)
    ///    - If L > 0.5:  S = (max - min) / (2 - max - min)
    /// 6. Calculate Hue based on which component is max:
    ///    - If R is max: H = (G - B) / (max - min), adjusted by +6 if G < B
    ///    - If G is max: H = (B - R) / (max - min) + 2
    ///    - If B is max: H = (R - G) / (max - min) + 4
    /// 7. Convert H to degrees (multiply by 60)
    ///
    /// # Output Ranges
    /// - Hue: 0-360 degrees
    /// - Saturation: 0-100%
    /// - Lightness: 0-100%
    pub fn to_hsl(&self) -> Hsl {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < 0.0001 {
            return Hsl::new(0.0, 0.0, l * 100.0);
        }

        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if (max - r).abs() < 0.0001 {
            let mut h = (g - b) / d;
            if g < b {
                h += 6.0;
            }
            h
        } else if (max - g).abs() < 0.0001 {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        Hsl::new(h * 60.0, s * 100.0, l * 100.0)
    }
}

/// RGBA color (red, green, blue, alpha)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse from hex string (supports #RGBA, #RRGGBBAA, or falls back to RGB)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().trim_start_matches('#');

        match hex.len() {
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()? * 17;
                Some(Self { r, g, b, a })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self { r, g, b, a })
            }
            3 | 6 => {
                let rgb = Rgb::from_hex(hex)?;
                Some(Self { r: rgb.r, g: rgb.g, b: rgb.b, a: 255 })
            }
            _ => None,
        }
    }

    /// Convert to hex string (#RRGGBBAA)
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    /// Convert to u32 (0xRRGGBBAA)
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    /// Get RGB component
    pub fn rgb(&self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }
}

/// HSL color (hue, saturation, lightness)
#[derive(Clone, Copy, Debug)]
pub struct Hsl {
    pub h: f32, // 0-360
    pub s: f32, // 0-100
    pub l: f32, // 0-100
}

impl Hsl {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 100.0),
            l: l.clamp(0.0, 100.0),
        }
    }

    /// Convert HSL to RGB
    ///
    /// # Algorithm
    /// 1. Normalize H to 0.0-1.0 (from 0-360), S and L to 0.0-1.0 (from 0-100)
    /// 2. If S == 0 (achromatic/grayscale), R = G = B = L
    /// 3. Calculate intermediate values p and q:
    ///    - If L < 0.5: q = L * (1 + S)
    ///    - If L >= 0.5: q = L + S - L * S
    ///    - p = 2 * L - q
    /// 4. Convert hue to RGB using `hue_to_rgb` helper for each channel:
    ///    - R = hue_to_rgb(p, q, H + 1/3)
    ///    - G = hue_to_rgb(p, q, H)
    ///    - B = hue_to_rgb(p, q, H - 1/3)
    ///
    /// The `hue_to_rgb` helper divides the hue circle into segments:
    /// - 0 to 1/6: Rising from p toward q
    /// - 1/6 to 1/2: Plateau at q
    /// - 1/2 to 2/3: Falling from q toward p
    /// - 2/3 to 1: Plateau at p
    ///
    /// # Input Ranges
    /// - Hue: 0-360 degrees
    /// - Saturation: 0-100%
    /// - Lightness: 0-100%
    ///
    /// # Output Ranges
    /// - R, G, B: 0-255
    pub fn to_rgb(&self) -> Rgb {
        let h = self.h / 360.0;
        let s = self.s / 100.0;
        let l = self.l / 100.0;

        if s.abs() < 0.0001 {
            let v = (l * 255.0).round() as u8;
            return Rgb::new(v, v, v);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 {
                return p + (q - p) * 6.0 * t;
            }
            if t < 1.0 / 2.0 {
                return q;
            }
            if t < 2.0 / 3.0 {
                return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
            }
            p
        }

        let r = (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0).round() as u8;
        let g = (hue_to_rgb(p, q, h) * 255.0).round() as u8;
        let b = (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0).round() as u8;

        Rgb::new(r, g, b)
    }
}

/// HSV color (hue, saturation, value/brightness)
/// This is the color model used in Photoshop-style color pickers
#[derive(Clone, Copy, Debug)]
pub struct Hsv {
    pub h: f32, // 0-360
    pub s: f32, // 0-100
    pub v: f32, // 0-100
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 100.0),
            v: v.clamp(0.0, 100.0),
        }
    }

    /// Convert HSV to RGB
    ///
    /// # Algorithm
    /// 1. Normalize H to 0.0-1.0 (from 0-360), S and V to 0.0-1.0 (from 0-100)
    /// 2. If S == 0 (achromatic/grayscale), R = G = B = V
    /// 3. Divide hue into 6 sectors (0-5) by multiplying normalized H by 6
    /// 4. Calculate intermediate values:
    ///    - i = floor(H * 6) - sector index
    ///    - f = fractional part of H * 6
    ///    - p = V * (1 - S) - minimum brightness
    ///    - q = V * (1 - S * f) - descending edge
    ///    - t = V * (1 - S * (1 - f)) - ascending edge
    /// 5. Map RGB based on sector:
    ///    - Sector 0 (red-yellow):     R=V, G=t, B=p
    ///    - Sector 1 (yellow-green):   R=q, G=V, B=p
    ///    - Sector 2 (green-cyan):     R=p, G=V, B=t
    ///    - Sector 3 (cyan-blue):      R=p, G=q, B=V
    ///    - Sector 4 (blue-magenta):   R=t, G=p, B=V
    ///    - Sector 5 (magenta-red):    R=V, G=p, B=q
    ///
    /// # Input Ranges
    /// - Hue: 0-360 degrees
    /// - Saturation: 0-100%
    /// - Value: 0-100%
    ///
    /// # Output Ranges
    /// - R, G, B: 0-255
    pub fn to_rgb(&self) -> Rgb {
        let h = self.h / 360.0;
        let s = self.s / 100.0;
        let v = self.v / 100.0;

        if s.abs() < 0.0001 {
            let val = (v * 255.0).round() as u8;
            return Rgb::new(val, val, val);
        }

        let h = h * 6.0;
        let i = h.floor() as i32;
        let f = h - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        Rgb::new(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }
}

impl Rgb {
    /// Convert RGB to HSV (Hue, Saturation, Value/Brightness)
    ///
    /// # Algorithm
    /// 1. Normalize RGB values from 0-255 to 0.0-1.0
    /// 2. Find max and min of R, G, B components
    /// 3. Value = max (the brightest component)
    /// 4. If max == min (achromatic/grayscale), H = 0, S = 0
    /// 5. Calculate Saturation: S = (max - min) / max
    /// 6. Calculate Hue based on which component is max:
    ///    - If R is max: H = (G - B) / (max - min), adjusted by +6 if G < B
    ///    - If G is max: H = (B - R) / (max - min) + 2
    ///    - If B is max: H = (R - G) / (max - min) + 4
    /// 7. Convert H to degrees (multiply by 60)
    ///
    /// # Output Ranges
    /// - Hue: 0-360 degrees
    /// - Saturation: 0-100%
    /// - Value: 0-100%
    pub fn to_hsv(&self) -> Hsv {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;

        let v = max;

        if d.abs() < 0.0001 {
            return Hsv::new(0.0, 0.0, v * 100.0);
        }

        let s = d / max;

        let h = if (max - r).abs() < 0.0001 {
            let mut h = (g - b) / d;
            if g < b {
                h += 6.0;
            }
            h
        } else if (max - g).abs() < 0.0001 {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        Hsv::new(h * 60.0, s * 100.0, v * 100.0)
    }
}

/// CSS named colors - sorted for binary search
const NAMED_COLORS: &[(&str, u32)] = &[
    ("aliceblue", 0xF0F8FF),
    ("antiquewhite", 0xFAEBD7),
    ("aqua", 0x00FFFF),
    ("aquamarine", 0x7FFFD4),
    ("azure", 0xF0FFFF),
    ("beige", 0xF5F5DC),
    ("bisque", 0xFFE4C4),
    ("black", 0x000000),
    ("blanchedalmond", 0xFFEBCD),
    ("blue", 0x0000FF),
    ("blueviolet", 0x8A2BE2),
    ("brown", 0xA52A2A),
    ("burlywood", 0xDEB887),
    ("cadetblue", 0x5F9EA0),
    ("chartreuse", 0x7FFF00),
    ("chocolate", 0xD2691E),
    ("coral", 0xFF7F50),
    ("cornflowerblue", 0x6495ED),
    ("cornsilk", 0xFFF8DC),
    ("crimson", 0xDC143C),
    ("cyan", 0x00FFFF),
    ("darkblue", 0x00008B),
    ("darkcyan", 0x008B8B),
    ("darkgoldenrod", 0xB8860B),
    ("darkgray", 0xA9A9A9),
    ("darkgreen", 0x006400),
    ("darkgrey", 0xA9A9A9),
    ("darkkhaki", 0xBDB76B),
    ("darkmagenta", 0x8B008B),
    ("darkolivegreen", 0x556B2F),
    ("darkorange", 0xFF8C00),
    ("darkorchid", 0x9932CC),
    ("darkred", 0x8B0000),
    ("darksalmon", 0xE9967A),
    ("darkseagreen", 0x8FBC8F),
    ("darkslateblue", 0x483D8B),
    ("darkslategray", 0x2F4F4F),
    ("darkslategrey", 0x2F4F4F),
    ("darkturquoise", 0x00CED1),
    ("darkviolet", 0x9400D3),
    ("deeppink", 0xFF1493),
    ("deepskyblue", 0x00BFFF),
    ("dimgray", 0x696969),
    ("dimgrey", 0x696969),
    ("dodgerblue", 0x1E90FF),
    ("firebrick", 0xB22222),
    ("floralwhite", 0xFFFAF0),
    ("forestgreen", 0x228B22),
    ("fuchsia", 0xFF00FF),
    ("gainsboro", 0xDCDCDC),
    ("ghostwhite", 0xF8F8FF),
    ("gold", 0xFFD700),
    ("goldenrod", 0xDAA520),
    ("gray", 0x808080),
    ("green", 0x008000),
    ("greenyellow", 0xADFF2F),
    ("grey", 0x808080),
    ("honeydew", 0xF0FFF0),
    ("hotpink", 0xFF69B4),
    ("indianred", 0xCD5C5C),
    ("indigo", 0x4B0082),
    ("ivory", 0xFFFFF0),
    ("khaki", 0xF0E68C),
    ("lavender", 0xE6E6FA),
    ("lavenderblush", 0xFFF0F5),
    ("lawngreen", 0x7CFC00),
    ("lemonchiffon", 0xFFFACD),
    ("lightblue", 0xADD8E6),
    ("lightcoral", 0xF08080),
    ("lightcyan", 0xE0FFFF),
    ("lightgoldenrodyellow", 0xFAFAD2),
    ("lightgray", 0xD3D3D3),
    ("lightgreen", 0x90EE90),
    ("lightgrey", 0xD3D3D3),
    ("lightpink", 0xFFB6C1),
    ("lightsalmon", 0xFFA07A),
    ("lightseagreen", 0x20B2AA),
    ("lightskyblue", 0x87CEFA),
    ("lightslategray", 0x778899),
    ("lightslategrey", 0x778899),
    ("lightsteelblue", 0xB0C4DE),
    ("lightyellow", 0xFFFFE0),
    ("lime", 0x00FF00),
    ("limegreen", 0x32CD32),
    ("linen", 0xFAF0E6),
    ("magenta", 0xFF00FF),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66CDAA),
    ("mediumblue", 0x0000CD),
    ("mediumorchid", 0xBA55D3),
    ("mediumpurple", 0x9370DB),
    ("mediumseagreen", 0x3CB371),
    ("mediumslateblue", 0x7B68EE),
    ("mediumspringgreen", 0x00FA9A),
    ("mediumturquoise", 0x48D1CC),
    ("mediumvioletred", 0xC71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xF5FFFA),
    ("mistyrose", 0xFFE4E1),
    ("moccasin", 0xFFE4B5),
    ("navajowhite", 0xFFDEAD),
    ("navy", 0x000080),
    ("oldlace", 0xFDF5E6),
    ("olive", 0x808000),
    ("olivedrab", 0x6B8E23),
    ("orange", 0xFFA500),
    ("orangered", 0xFF4500),
    ("orchid", 0xDA70D6),
    ("palegoldenrod", 0xEEE8AA),
    ("palegreen", 0x98FB98),
    ("paleturquoise", 0xAFEEEE),
    ("palevioletred", 0xDB7093),
    ("papayawhip", 0xFFEFD5),
    ("peachpuff", 0xFFDAB9),
    ("peru", 0xCD853F),
    ("pink", 0xFFC0CB),
    ("plum", 0xDDA0DD),
    ("powderblue", 0xB0E0E6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("red", 0xFF0000),
    ("rosybrown", 0xBC8F8F),
    ("royalblue", 0x4169E1),
    ("saddlebrown", 0x8B4513),
    ("salmon", 0xFA8072),
    ("sandybrown", 0xF4A460),
    ("seagreen", 0x2E8B57),
    ("seashell", 0xFFF5EE),
    ("sienna", 0xA0522D),
    ("silver", 0xC0C0C0),
    ("skyblue", 0x87CEEB),
    ("slateblue", 0x6A5ACD),
    ("slategray", 0x708090),
    ("slategrey", 0x708090),
    ("snow", 0xFFFAFA),
    ("springgreen", 0x00FF7F),
    ("steelblue", 0x4682B4),
    ("tan", 0xD2B48C),
    ("teal", 0x008080),
    ("thistle", 0xD8BFD8),
    ("tomato", 0xFF6347),
    ("turquoise", 0x40E0D0),
    ("violet", 0xEE82EE),
    ("wheat", 0xF5DEB3),
    ("white", 0xFFFFFF),
    ("whitesmoke", 0xF5F5F5),
    ("yellow", 0xFFFF00),
    ("yellowgreen", 0x9ACD32),
];

/// Look up a CSS named color
pub fn named_color_to_rgb(name: &str) -> Option<Rgb> {
    let name_lower = name.to_lowercase();
    NAMED_COLORS
        .binary_search_by_key(&name_lower.as_str(), |(n, _)| *n)
        .ok()
        .map(|i| {
            let val = NAMED_COLORS[i].1;
            Rgb::new(
                ((val >> 16) & 0xFF) as u8,
                ((val >> 8) & 0xFF) as u8,
                (val & 0xFF) as u8,
            )
        })
}

/// Parse a color string (hex or named color) to RGB
pub fn parse_color(s: &str) -> Option<Rgb> {
    Rgb::from_hex(s).or_else(|| named_color_to_rgb(s))
}

/// Parse a color string (hex or named color) to RGBA
pub fn parse_color_alpha(s: &str) -> Option<Rgba> {
    Rgba::from_hex(s).or_else(|| named_color_to_rgb(s).map(|rgb| Rgba::new(rgb.r, rgb.g, rgb.b, 255)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_from_hex_6digit() {
        assert_eq!(Rgb::from_hex("#FF0000"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(Rgb::from_hex("00FF00"), Some(Rgb::new(0, 255, 0)));
        assert_eq!(Rgb::from_hex("#3b82f6"), Some(Rgb::new(59, 130, 246)));
    }

    #[test]
    fn test_rgb_from_hex_3digit() {
        assert_eq!(Rgb::from_hex("#F00"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(Rgb::from_hex("0F0"), Some(Rgb::new(0, 255, 0)));
    }

    #[test]
    fn test_rgb_from_hex_invalid() {
        assert_eq!(Rgb::from_hex("invalid"), None);
        assert_eq!(Rgb::from_hex("#GG0000"), None);
    }

    #[test]
    fn test_rgb_to_hex() {
        assert_eq!(Rgb::new(255, 0, 0).to_hex(), "#FF0000");
        assert_eq!(Rgb::new(0, 255, 0).to_hex(), "#00FF00");
    }

    #[test]
    fn test_rgb_to_u32() {
        assert_eq!(Rgb::new(255, 0, 0).to_u32(), 0xFF0000);
        assert_eq!(Rgb::new(0, 255, 0).to_u32(), 0x00FF00);
    }

    #[test]
    fn test_rgba_from_hex_8digit() {
        assert_eq!(Rgba::from_hex("#FF000080"), Some(Rgba::new(255, 0, 0, 128)));
    }

    #[test]
    fn test_rgba_from_hex_4digit() {
        assert_eq!(Rgba::from_hex("#F008"), Some(Rgba::new(255, 0, 0, 136)));
    }

    #[test]
    fn test_rgba_from_hex_fallback() {
        assert_eq!(Rgba::from_hex("#FF0000"), Some(Rgba::new(255, 0, 0, 255)));
    }

    #[test]
    fn test_rgba_to_u32() {
        assert_eq!(Rgba::new(255, 0, 0, 128).to_u32(), 0xFF000080);
    }

    #[test]
    fn test_named_colors() {
        assert_eq!(named_color_to_rgb("red"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(named_color_to_rgb("RED"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(named_color_to_rgb("coral"), Some(Rgb::new(255, 127, 80)));
        assert_eq!(named_color_to_rgb("invalid"), None);
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("#FF0000"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(parse_color("red"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(parse_color("invalid"), None);
    }

    #[test]
    fn test_rgb_to_hsl_and_back() {
        let rgb = Rgb::new(255, 0, 0);
        let hsl = rgb.to_hsl();
        assert!((hsl.h - 0.0).abs() < 1.0);
        assert!((hsl.s - 100.0).abs() < 1.0);
        assert!((hsl.l - 50.0).abs() < 1.0);

        let back = hsl.to_rgb();
        assert_eq!(back, rgb);
    }

    #[test]
    fn test_hsl_to_rgb_and_back() {
        let hsl = Hsl::new(120.0, 100.0, 50.0); // Green
        let rgb = hsl.to_rgb();
        assert_eq!(rgb, Rgb::new(0, 255, 0));
    }

    #[test]
    fn test_hsl_normalization() {
        let hsl = Hsl::new(400.0, 150.0, -10.0);
        assert!((hsl.h - 40.0).abs() < 0.01);
        assert!((hsl.s - 100.0).abs() < 0.01);
        assert!((hsl.l - 0.0).abs() < 0.01);
    }
}
