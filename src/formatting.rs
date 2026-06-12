//! Formatting utilities for system stats display

pub fn format_percentage(value: f32) -> String {
    format!("{:.0}%", value.clamp(0.0, 100.0))
}

pub fn format_celsius(value: f32) -> String {
    format!("{:.0}°C", value)
}

pub fn format_memory_gb(value: f32) -> String {
    format!("{:.1} GB", value)
}

/// Auto-switches to Gbps when >= 1000 Mbps
/// Always uses two decimal places for consistent width
pub fn format_network_speed(bytes_per_sec: u64) -> String {
    let mbps = bytes_per_sec as f64 / 125_000.0;

    if mbps >= 1000.0 {
        let gbps = mbps / 1000.0;
        format!("{:.2} Gbps", gbps)
    } else {
        format!("{:.2} Mbps", mbps)
    }
}

/// Format percentage with status indicator
/// Returns the formatted percentage and the status level
pub fn format_percentage_with_status(value: f32, low_max: f32, high_min: f32) -> (String, crate::color::Status) {
    let status = crate::color::Status::from_value(value.clamp(0.0, 100.0), low_max, high_min);
    (format_percentage(value), status)
}

/// Format Celsius with status indicator
/// Returns the formatted temperature and the status level
pub fn format_celsius_with_status(value: f32, low_max: f32, high_min: f32) -> (String, crate::color::Status) {
    let status = crate::color::Status::from_value(value, low_max, high_min);
    (format_celsius(value), status)
}

/// Format memory GB with status indicator based on percentage of total
/// Returns the formatted memory value and the status level
pub fn format_memory_gb_with_status(used_gb: f32, total_gb: f32, low_max: f32, high_min: f32) -> (String, crate::color::Status) {
    let usage_percent = if total_gb > 0.0 { (used_gb / total_gb) * 100.0 } else { 0.0 };
    let status = crate::color::Status::from_value(usage_percent, low_max, high_min);
    (format_memory_gb(used_gb), status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Status;

    #[test]
    fn test_format_percentage_with_status_low() {
        let (formatted, status) = format_percentage_with_status(30.0, 40.0, 75.0);
        assert_eq!(status, Status::Low);
        assert!(formatted.contains("%"));
    }

    #[test]
    fn test_format_percentage_with_status_medium() {
        let (formatted, status) = format_percentage_with_status(60.0, 40.0, 75.0);
        assert_eq!(status, Status::Medium);
        assert!(formatted.contains("%"));
    }

    #[test]
    fn test_format_percentage_with_status_high() {
        let (formatted, status) = format_percentage_with_status(85.0, 40.0, 75.0);
        assert_eq!(status, Status::High);
        assert!(formatted.contains("%"));
    }

    #[test]
    fn test_format_celsius_with_status_low() {
        let (formatted, status) = format_celsius_with_status(50.0, 60.0, 80.0);
        assert_eq!(status, Status::Low);
        assert!(formatted.contains("°C"));
    }

    #[test]
    fn test_format_celsius_with_status_medium() {
        let (formatted, status) = format_celsius_with_status(70.0, 60.0, 80.0);
        assert_eq!(status, Status::Medium);
        assert!(formatted.contains("°C"));
    }

    #[test]
    fn test_format_celsius_with_status_high() {
        let (formatted, status) = format_celsius_with_status(85.0, 60.0, 80.0);
        assert_eq!(status, Status::High);
        assert!(formatted.contains("°C"));
    }

    #[test]
    fn test_format_memory_gb_with_status_low() {
        let (formatted, status) = format_memory_gb_with_status(4.0, 16.0, 50.0, 80.0);
        assert_eq!(status, Status::Low); // 25% usage
        assert!(formatted.contains("GB"));
    }

    #[test]
    fn test_format_memory_gb_with_status_medium() {
        let (formatted, status) = format_memory_gb_with_status(10.0, 16.0, 50.0, 80.0);
        assert_eq!(status, Status::Medium); // 62.5% usage
        assert!(formatted.contains("GB"));
    }

    #[test]
    fn test_format_memory_gb_with_status_high() {
        let (formatted, status) = format_memory_gb_with_status(14.0, 16.0, 50.0, 80.0);
        assert_eq!(status, Status::High); // 87.5% usage
        assert!(formatted.contains("GB"));
    }

    #[test]
    fn test_format_memory_gb_with_status_zero_total() {
        let (formatted, status) = format_memory_gb_with_status(4.0, 0.0, 50.0, 80.0);
        assert_eq!(status, Status::Low); // 0% usage when total is 0
        assert!(formatted.contains("GB"));
    }
}
