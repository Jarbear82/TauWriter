// Color utility functions for parsing and formatting hex colors.
// Supports #RGB, #RGBA, #RRGGBB, #RRGGBBAA formats (LSP Color model).

use lsp_types::Color;

/// Parse a hex color string into an LSP [`Color`].
///
/// Accepts: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA` (with or without leading `#`).
pub fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim().strip_prefix('#')?;
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? as f32 / 15.0;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? as f32 / 15.0;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? as f32 / 15.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: 1.0,
            })
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? as f32 / 15.0;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? as f32 / 15.0;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? as f32 / 15.0;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? as f32 / 15.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: a,
            })
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: 1.0,
            })
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
            let a = u8::from_str_radix(&s[6..8], 16).ok()? as f32 / 255.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: a,
            })
        }
        _ => None,
    }
}

/// Format an LSP [`Color`] into a HubGS-compatible hex color string.
///
/// Returns `#RRGGBB` for opaque colors, `#RRGGBBAA` otherwise.
pub fn format_hex_color(color: Color) -> String {
    let r = (color.red * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (color.green * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (color.blue * 255.0).round().clamp(0.0, 255.0) as u8;
    let a = (color.alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    if a == 255 {
        format!("\"#{:02x}{:02x}{:02x}\"", r, g, b)
    } else {
        format!("\"#{:02x}{:02x}{:02x}{:02x}\"", r, g, b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_3_digit() {
        let c = parse_hex_color("#f00").unwrap();
        assert!((c.red - 1.0).abs() < f32::EPSILON);
        assert!((c.green - 0.0).abs() < f32::EPSILON);
        assert!((c.blue - 0.0).abs() < f32::EPSILON);
        assert!((c.alpha - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_hex_4_digit() {
        let c = parse_hex_color("#f00f").unwrap();
        assert!((c.red - 1.0).abs() < f32::EPSILON);
        assert!((c.blue - 0.0).abs() < f32::EPSILON);
        assert!((c.alpha - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_hex_6_digit() {
        let c = parse_hex_color("#ff0000").unwrap();
        assert!((c.red - 1.0).abs() < f32::EPSILON);
        assert!((c.green - 0.0).abs() < f32::EPSILON);
        assert!((c.blue - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_hex_8_digit() {
        let c = parse_hex_color("#ff000080").unwrap();
        assert!((c.red - 1.0).abs() < f32::EPSILON);
        assert!((c.alpha - 0.502).abs() < 0.01);
    }

    #[test]
    fn parse_hex_no_hash() {
        assert!(parse_hex_color("f00").is_none()); // requires # prefix
    }

    #[test]
    fn parse_hex_invalid_len() {
        assert!(parse_hex_color("#12345").is_none()); // not a valid length
        assert!(parse_hex_color("#xyz").is_none());
    }

    #[test]
    fn format_hex_opaque() {
        let color = Color {
            red: 1.0,
            green: 0.5,
            blue: 0.0,
            alpha: 1.0,
        };
        assert_eq!(format_hex_color(color), "\"#ff8000\"");
    }

    #[test]
    fn format_hex_transparent() {
        let color = Color {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.5,
        };
        assert_eq!(format_hex_color(color), "\"#ff000080\"");
    }
}
