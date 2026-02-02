//! Utility modules

pub mod color;
pub mod path;

pub use color::{Rgb, Rgba, Hsl, Hsv, named_color_to_rgb, parse_color, parse_color_alpha};
pub use path::{parse_path, expand_tilde, PathInfo};

/// Format a floating-point value for display with optional precision
///
/// If `precision` is `Some(p)`, rounds to `p` decimal places.
/// Otherwise, shows integer format for whole numbers, or trims trailing zeros.
pub fn format_display_value(value: f64, precision: Option<usize>) -> String {
    match precision {
        Some(p) => {
            let multiplier = 10_f64.powi(p as i32);
            let rounded = (value * multiplier).round() / multiplier;
            format!("{:.prec$}", rounded, prec = p)
        }
        None => {
            if value.fract() == 0.0 {
                format!("{:.0}", value)
            } else {
                format!("{}", value)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_display_value_with_precision() {
        assert_eq!(format_display_value(1.23456, Some(2)), "1.23");
        assert_eq!(format_display_value(1.23956, Some(2)), "1.24");
        assert_eq!(format_display_value(5.0, Some(2)), "5.00");
        assert_eq!(format_display_value(0.1, Some(3)), "0.100");
    }

    #[test]
    fn test_format_display_value_without_precision() {
        assert_eq!(format_display_value(42.0, None), "42");
        assert_eq!(format_display_value(1.23, None), "1.23");
        assert_eq!(format_display_value(3.10, None), "3.1");
        assert_eq!(format_display_value(3.100, None), "3.1");
        assert_eq!(format_display_value(0.5, None), "0.5");
    }
}
