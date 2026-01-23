use std::time::Duration;

use bytesize::ByteSize;
use ratatui::{layout::Rect, style::Color};

use crate::config::{DataSizeUnit, EnergyUnit, TemperatureUnit};
use crate::theme::ThemeColors;

const NOMINAL_VOLTAGE: f32 = 11.4;

/// Center a fixed-size rectangle within an area (clamped to fit).
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Center a percentage-based rectangle within an area.
pub fn centered_rect_percent(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let width = (area.width * width_percent / 100).min(area.width.saturating_sub(4));
    let height = (area.height * height_percent / 100).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Returns success/warning/danger color based on percent vs thresholds (higher is better).
pub fn color_for_percent(percent: f32, high: f32, low: f32, theme: &ThemeColors) -> Color {
    if percent > high {
        theme.success
    } else if percent > low {
        theme.warning
    } else {
        theme.danger
    }
}

/// Returns success/warning/danger color based on value vs thresholds (lower is better).
pub fn color_for_value(value: f32, low: f32, high: f32, theme: &ThemeColors) -> Color {
    if value <= low {
        theme.success
    } else if value <= high {
        theme.warning
    } else {
        theme.danger
    }
}

/// Format seconds as human-readable duration (e.g., "2h 37m", "5days 3h").
pub fn format_duration(secs: u64) -> String {
    humantime::format_duration(Duration::from_secs(secs)).to_string()
}

pub fn format_data_size(bytes: u64, unit: DataSizeUnit) -> String {
    match unit {
        DataSizeUnit::Si => ByteSize::b(bytes).display().si().to_string(),
        DataSizeUnit::Binary => ByteSize::b(bytes).display().iec().to_string(),
    }
}

pub fn format_energy(wh: f32, unit: EnergyUnit) -> String {
    match unit {
        EnergyUnit::Wh => format!("{:.1} Wh", wh),
        EnergyUnit::MAh => {
            let mah = (wh * 1000.0) / NOMINAL_VOLTAGE;
            format!("{:.0} mAh", mah)
        }
    }
}

pub fn format_energy_compact(wh: f32, unit: EnergyUnit) -> String {
    match unit {
        EnergyUnit::Wh => format!("{:.1}Wh", wh),
        EnergyUnit::MAh => {
            let mah = (wh * 1000.0) / NOMINAL_VOLTAGE;
            format!("{:.0}mAh", mah)
        }
    }
}

pub fn format_energy_ratio(current_wh: f32, max_wh: f32, unit: EnergyUnit) -> String {
    match unit {
        EnergyUnit::Wh => format!("{:.1}/{:.1}Wh", current_wh, max_wh),
        EnergyUnit::MAh => {
            let current_mah = (current_wh * 1000.0) / NOMINAL_VOLTAGE;
            let max_mah = (max_wh * 1000.0) / NOMINAL_VOLTAGE;
            format!("{:.0}/{:.0}mAh", current_mah, max_mah)
        }
    }
}

pub fn energy_unit_label(unit: EnergyUnit) -> &'static str {
    match unit {
        EnergyUnit::Wh => "Wh",
        EnergyUnit::MAh => "mAh",
    }
}

pub fn format_temperature(celsius: f32, unit: TemperatureUnit) -> String {
    match unit {
        TemperatureUnit::Celsius => format!("{:.1}°C", celsius),
        TemperatureUnit::Fahrenheit => {
            let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
            format!("{:.1}°F", fahrenheit)
        }
    }
}

pub fn format_temperature_short(celsius: f32, unit: TemperatureUnit) -> String {
    match unit {
        TemperatureUnit::Celsius => format!("{:.0}°", celsius),
        TemperatureUnit::Fahrenheit => {
            let fahrenheit = celsius * 9.0 / 5.0 + 32.0;
            format!("{:.0}°", fahrenheit)
        }
    }
}

pub fn convert_temperature(celsius: f32, unit: TemperatureUnit) -> f32 {
    match unit {
        TemperatureUnit::Celsius => celsius,
        TemperatureUnit::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
    }
}

pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Energy conversion tests (Wh to mAh using 11.4V nominal)
    #[test]
    fn test_format_energy_wh() {
        assert_eq!(format_energy(50.0, EnergyUnit::Wh), "50.0 Wh");
        assert_eq!(format_energy(72.5, EnergyUnit::Wh), "72.5 Wh");
        assert_eq!(format_energy(0.0, EnergyUnit::Wh), "0.0 Wh");
    }

    #[test]
    fn test_format_energy_mah() {
        // 50 Wh * 1000 / 11.4V = 4386 mAh
        assert_eq!(format_energy(50.0, EnergyUnit::MAh), "4386 mAh");
        // 11.4 Wh * 1000 / 11.4V = 1000 mAh
        assert_eq!(format_energy(11.4, EnergyUnit::MAh), "1000 mAh");
    }

    #[test]
    fn test_format_energy_compact() {
        assert_eq!(format_energy_compact(50.0, EnergyUnit::Wh), "50.0Wh");
        assert_eq!(format_energy_compact(50.0, EnergyUnit::MAh), "4386mAh");
    }

    #[test]
    fn test_format_energy_ratio() {
        assert_eq!(
            format_energy_ratio(50.0, 100.0, EnergyUnit::Wh),
            "50.0/100.0Wh"
        );
        // 50 Wh = 4386 mAh, 100 Wh = 8772 mAh
        assert_eq!(
            format_energy_ratio(50.0, 100.0, EnergyUnit::MAh),
            "4386/8772mAh"
        );
    }

    #[test]
    fn test_energy_unit_label() {
        assert_eq!(energy_unit_label(EnergyUnit::Wh), "Wh");
        assert_eq!(energy_unit_label(EnergyUnit::MAh), "mAh");
    }

    // Temperature conversion tests
    #[test]
    fn test_format_temperature_celsius() {
        assert_eq!(format_temperature(25.0, TemperatureUnit::Celsius), "25.0°C");
        assert_eq!(format_temperature(0.0, TemperatureUnit::Celsius), "0.0°C");
        assert_eq!(
            format_temperature(100.0, TemperatureUnit::Celsius),
            "100.0°C"
        );
    }

    #[test]
    fn test_format_temperature_fahrenheit() {
        // 0°C = 32°F
        assert_eq!(
            format_temperature(0.0, TemperatureUnit::Fahrenheit),
            "32.0°F"
        );
        // 100°C = 212°F
        assert_eq!(
            format_temperature(100.0, TemperatureUnit::Fahrenheit),
            "212.0°F"
        );
        // 25°C = 77°F
        assert_eq!(
            format_temperature(25.0, TemperatureUnit::Fahrenheit),
            "77.0°F"
        );
    }

    #[test]
    fn test_format_temperature_short() {
        assert_eq!(
            format_temperature_short(25.0, TemperatureUnit::Celsius),
            "25°"
        );
        assert_eq!(
            format_temperature_short(25.0, TemperatureUnit::Fahrenheit),
            "77°"
        );
    }

    #[test]
    fn test_convert_temperature() {
        assert_eq!(convert_temperature(0.0, TemperatureUnit::Celsius), 0.0);
        assert_eq!(convert_temperature(0.0, TemperatureUnit::Fahrenheit), 32.0);
        assert_eq!(
            convert_temperature(100.0, TemperatureUnit::Fahrenheit),
            212.0
        );
    }

    // Data size formatting tests
    #[test]
    fn test_format_data_size_si() {
        // SI uses 1000-based units (kB, MB, GB) - bytesize uses lowercase 'k'
        assert_eq!(format_data_size(1000, DataSizeUnit::Si), "1.0 kB");
        assert_eq!(format_data_size(1_000_000, DataSizeUnit::Si), "1.0 MB");
        assert_eq!(format_data_size(1_000_000_000, DataSizeUnit::Si), "1.0 GB");
    }

    #[test]
    fn test_format_data_size_binary() {
        // Binary uses 1024-based units (KiB, MiB, GiB)
        assert_eq!(format_data_size(1024, DataSizeUnit::Binary), "1.0 KiB");
        assert_eq!(format_data_size(1_048_576, DataSizeUnit::Binary), "1.0 MiB");
        assert_eq!(
            format_data_size(1_073_741_824, DataSizeUnit::Binary),
            "1.0 GiB"
        );
    }

    #[test]
    fn test_truncate_str_no_underflow_when_max_len_small() {
        assert_eq!(truncate_str("Terminal", 0), "");
        assert_eq!(truncate_str("Terminal", 1), "T");
        assert_eq!(truncate_str("Terminal", 2), "Te");
        assert_eq!(truncate_str("Terminal", 3), "Ter");
    }

    #[test]
    fn test_truncate_str_adds_ellipsis() {
        assert_eq!(truncate_str("Terminal", 7), "Term...");
        assert_eq!(truncate_str("Terminal", 4), "T...");
    }

    #[test]
    fn test_truncate_str_unchanged_when_fits() {
        assert_eq!(truncate_str("Terminal", 8), "Terminal");
        assert_eq!(truncate_str("Terminal", 10), "Terminal");
    }
}
