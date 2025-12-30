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
pub fn format_network_speed(bytes_per_sec: u64) -> String {
    let mbps = bytes_per_sec as f64 / 125_000.0;

    if mbps >= 1000.0 {
        let gbps = mbps / 1000.0;
        format!("{:.2} Gbps", gbps)
    } else {
        format!("{:.1} Mbps", mbps)
    }
}
