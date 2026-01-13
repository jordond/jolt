use std::time::Duration;

use bytesize::ByteSize;
use ratatui::{layout::Rect, style::Color};

use crate::theme::ThemeColors;

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

/// Returns success/warning/danger color based on percent vs thresholds.
pub fn color_for_percent(percent: f32, high: f32, low: f32, theme: &ThemeColors) -> Color {
    if percent > high {
        theme.success
    } else if percent > low {
        theme.warning
    } else {
        theme.danger
    }
}

/// Format seconds as human-readable duration (e.g., "2h 37m", "5days 3h").
pub fn format_duration(secs: u64) -> String {
    humantime::format_duration(Duration::from_secs(secs)).to_string()
}

/// Format bytes as human-readable string (e.g., "1.5 MB", "256 KB").
pub fn format_bytes(bytes: u64) -> String {
    ByteSize::b(bytes).display().si().to_string()
}
