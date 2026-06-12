//! Color utilities for status-based theming
//!
//! Provides status detection (Low/Medium/High) based on configurable thresholds
//! and returns appropriate colors that integrate with COSMIC theme.
//!
//! Color semantics follow system monitoring conventions:
//! - Low (Blue): Normal operation, all systems operational
//! - Medium (Orange): Warning, attention may be needed
//! - High (Red): Critical, action required
//!
//! Colors are chosen to work well in both light and dark themes.

use cosmic::iced::Color;

/// Convert a hex color code to Color
///
/// # Arguments
/// * `hex` - Hex color code in 0xRRGGBB format
///
/// # Example
/// ```
/// let blue = hex_color(0x00B4D8);
/// ```
fn hex_color(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

/// Status levels for color-coding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Low usage/normal state - displayed in blue
    Low,
    /// Medium usage/warning state - displayed in orange
    Medium,
    /// High usage/critical state - displayed in red
    High,
}

impl Status {
    /// Determine status based on value and thresholds
    ///
    /// # Arguments
    /// * `value` - The current value to evaluate
    /// * `low_max` - Maximum value for Low status (below this = Low)
    /// * `high_min` - Minimum value for High status (above this = High)
    ///
    /// Values between low_max and high_min are Medium status
    pub fn from_value(value: f32, low_max: f32, high_min: f32) -> Self {
        if value < low_max {
            Status::Low
        } else if value >= high_min {
            Status::High
        } else {
            Status::Medium
        }
    }

    /// Get color only if status is Medium or High (warning state)
    /// Returns None for Low (normal) status
    ///
    /// Use this when you want to color values only in warning states.
    pub fn warning_color(self) -> Option<Color> {
        match self {
            Status::Low => None,
            Status::Medium => Some(hex_color(0xFB8500)),
            Status::High => Some(hex_color(0xD62828)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_value_cpu() {
        // CPU thresholds: low_max=40, high_min=75
        assert_eq!(Status::from_value(30.0, 40.0, 75.0), Status::Low);
        assert_eq!(Status::from_value(40.0, 40.0, 75.0), Status::Medium);
        assert_eq!(Status::from_value(60.0, 40.0, 75.0), Status::Medium);
        assert_eq!(Status::from_value(75.0, 40.0, 75.0), Status::High);
        assert_eq!(Status::from_value(90.0, 40.0, 75.0), Status::High);
    }

    #[test]
    fn test_status_from_value_temperature() {
        // Temperature thresholds: low_max=60, high_min=80
        assert_eq!(Status::from_value(50.0, 60.0, 80.0), Status::Low);
        assert_eq!(Status::from_value(70.0, 60.0, 80.0), Status::Medium);
        assert_eq!(Status::from_value(85.0, 60.0, 80.0), Status::High);
    }
}
