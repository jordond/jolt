use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::data::SystemInfo;
use crate::input::keys;
use crate::theme::ThemeColors;

use super::utils::truncate_str;

pub fn render_title_bar(
    frame: &mut Frame,
    area: Rect,
    system_info: &SystemInfo,
    theme: &ThemeColors,
) {
    let version = super::VERSION;

    let left_spans = vec![
        Span::styled("⚡️jolt ", theme.accent_style()),
        Span::styled(format!("v{}", version), theme.muted_style()),
    ];

    let right_text = format!(
        "{} · macOS {} · {} ",
        system_info.chip,
        system_info.os_version,
        system_info.cores_display()
    );

    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width = right_text.chars().count();
    let padding = (area.width as usize).saturating_sub(left_width + right_width);

    let mut spans = left_spans;
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(right_text, theme.muted_style()));

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.bg));
    frame.render_widget(bar, area);
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let theme_name = app.config.theme_name();
    let theme_display = truncate_str(theme_name, 12);

    let appearance = app.config.appearance_label().to_lowercase();

    let theme_hint = format!("theme ({})", theme_display);
    let appearance_hint = format!("appearance ({})", appearance);

    let left_hints: Vec<(&str, String)> = vec![
        (keys::HELP, "help".to_string()),
        (keys::THEME, theme_hint),
        (keys::APPEARANCE, appearance_hint),
        (keys::QUIT, "quit".to_string()),
    ];

    let mut left_spans: Vec<Span> = vec![Span::raw(" ")];
    for (i, (key, desc)) in left_hints.iter().enumerate() {
        if i > 0 {
            left_spans.push(Span::styled(" │ ", theme.border_style()));
        }
        left_spans.push(Span::styled(*key, theme.accent_style()));
        left_spans.push(Span::styled(format!(" {}", desc), theme.muted_style()));
    }

    let background_recording = app.config.user_config.history.background_recording;

    let mut right_spans: Vec<Span> = Vec::new();

    if app.is_reconnecting() {
        right_spans.push(Span::styled("⟳ reconnecting ", theme.warning_style()));
        right_spans.push(Span::styled("│ ", theme.border_style()));
    } else if app.is_data_stale() {
        right_spans.push(Span::styled("⚠ stale ", theme.warning_style()));
        right_spans.push(Span::styled("│ ", theme.border_style()));
    }

    if background_recording {
        right_spans.extend(vec![
            Span::styled("background: ", theme.muted_style()),
            Span::styled("on", theme.success_style()),
            Span::styled(" │ ", theme.border_style()),
        ]);
    }

    let refresh_display = if app.refresh_ms >= 1000 {
        format!("{:.1}s", app.refresh_ms as f64 / 1000.0)
    } else {
        format!("{}ms", app.refresh_ms)
    };
    right_spans.extend(vec![
        Span::styled("refresh: ", theme.muted_style()),
        Span::styled(refresh_display, theme.fg_style()),
        Span::styled(" │ ", theme.border_style()),
    ]);

    right_spans.extend(vec![
        Span::styled(keys::HISTORY, theme.accent_style()),
        Span::styled(" history ", theme.muted_style()),
        Span::styled(keys::SETTINGS, theme.accent_style()),
        Span::styled(" settings ", theme.muted_style()),
    ]);

    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width: usize = right_spans.iter().map(|s| s.width()).sum();
    let padding = (area.width as usize).saturating_sub(left_width + right_width);

    left_spans.push(Span::raw(" ".repeat(padding)));
    left_spans.extend(right_spans);

    let line = Line::from(left_spans);
    let bar = Paragraph::new(line)
        .style(Style::default().bg(theme.bg))
        .alignment(Alignment::Left);

    frame.render_widget(bar, area);
}
