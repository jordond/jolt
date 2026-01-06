use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::input::keys;
use crate::theme::ThemeColors;

fn centered_fixed_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let area = centered_fixed_rect(frame.area(), 50, 22);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" History Recording ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .margin(1)
        .split(inner);

    let history_cfg = &app.config.user_config.history;
    let interval_str = format!("{}s", history_cfg.sample_interval_secs);
    let retention_str = format!(
        "{}d raw, {}d hourly",
        history_cfg.retention_raw_days, history_cfg.retention_hourly_days
    );

    if let Some(ref status) = app.daemon_status {
        let uptime_str = format_uptime(status.uptime_secs);
        let size_str = format_bytes(status.database_size_bytes);
        let last_sample = status
            .last_sample_time
            .map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            })
            .unwrap_or_else(|| "Never".to_string());

        let lines = vec![
            Line::from(vec![
                Span::styled("Status:        ", Style::default().fg(theme.muted)),
                Span::styled(
                    "Recording",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Version:       ", Style::default().fg(theme.muted)),
                Span::styled(&status.version, Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("Uptime:        ", Style::default().fg(theme.muted)),
                Span::styled(uptime_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Sample Count:  ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{}", status.sample_count),
                    Style::default().fg(theme.accent),
                ),
            ]),
            Line::from(vec![
                Span::styled("Last Sample:   ", Style::default().fg(theme.muted)),
                Span::styled(last_sample, Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("Database Size: ", Style::default().fg(theme.muted)),
                Span::styled(size_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Interval:      ", Style::default().fg(theme.muted)),
                Span::styled(&interval_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("Retention:     ", Style::default().fg(theme.muted)),
                Span::styled(&retention_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Run on login: jolt daemon install",
                Style::default().fg(theme.muted),
            )]),
        ];

        let para = Paragraph::new(lines);
        frame.render_widget(para, chunks[0]);
    } else {
        let lines = vec![
            Line::from(vec![
                Span::styled("Status:        ", Style::default().fg(theme.muted)),
                Span::styled(
                    "Off",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Recording tracks your battery and power usage",
                Style::default().fg(theme.fg),
            )]),
            Line::from(vec![Span::styled(
                "over time, helping you:",
                Style::default().fg(theme.fg),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  - View historical power trends",
                Style::default().fg(theme.muted),
            )]),
            Line::from(vec![Span::styled(
                "  - Identify power-hungry applications",
                Style::default().fg(theme.muted),
            )]),
            Line::from(vec![Span::styled(
                "  - Track battery health over time",
                Style::default().fg(theme.muted),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Interval:      ", Style::default().fg(theme.muted)),
                Span::styled(&interval_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("Retention:     ", Style::default().fg(theme.muted)),
                Span::styled(&retention_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Run on login: jolt daemon install",
                Style::default().fg(theme.muted),
            )]),
        ];

        let para = Paragraph::new(lines);
        frame.render_widget(para, chunks[0]);
    }

    let daemon_running = app.daemon_status.is_some();
    let action_key = if daemon_running {
        keys::DAEMON_STOP
    } else {
        keys::DAEMON_START
    };
    let action_label = if daemon_running {
        "Stop"
    } else {
        "Start Recording"
    };

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!("[{}]", action_key),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                format!(" {}  ", action_label),
                Style::default().fg(theme.muted),
            ),
            Span::styled(
                format!("[{}]", keys::HISTORY),
                Style::default().fg(theme.accent),
            ),
            Span::styled(" History  ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("[{}]", keys::HISTORY_CONFIG),
                Style::default().fg(theme.accent),
            ),
            Span::styled(" Config", Style::default().fg(theme.muted)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("[{}]", keys::ESC),
                Style::default().fg(theme.accent),
            ),
            Span::styled(" Close", Style::default().fg(theme.muted)),
        ]),
    ])
    .centered();
    frame.render_widget(footer, chunks[1]);
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
