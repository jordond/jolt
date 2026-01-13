use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::data::{ChargeSession, SessionType};
use crate::theme::ThemeColors;

pub fn render_cycle_summary(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Battery Cycles ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref summary) = app.cycle_summary else {
        let no_data = Paragraph::new(vec![Line::from(vec![Span::styled(
            "No cycle data",
            theme.muted_style(),
        )])])
        .centered();
        frame.render_widget(no_data, inner);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let left_stats = vec![
        Line::from(vec![
            Span::styled("macOS Cycles:   ", theme.muted_style()),
            Span::styled(
                format!("{}", summary.total_cycles_macos),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Partial Cycles: ", theme.muted_style()),
            Span::styled(
                format!("{:.2}", summary.partial_cycles_calculated),
                theme.fg_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg Daily:      ", theme.muted_style()),
            Span::styled(
                format!("{:.2}/day", summary.avg_daily_cycles),
                Style::default().fg(cycle_rate_color(summary.avg_daily_cycles, theme)),
            ),
        ]),
    ];

    let right_stats = vec![
        Line::from(vec![
            Span::styled("Avg DoD:        ", theme.muted_style()),
            Span::styled(
                format!("{:.0}%", summary.avg_depth_of_discharge),
                Style::default().fg(dod_color(summary.avg_depth_of_discharge, theme)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Charges/Day:    ", theme.muted_style()),
            Span::styled(
                format!("{:.1}", summary.avg_charge_sessions_per_day),
                theme.fg_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Time >80%:      ", theme.muted_style()),
            Span::styled(
                format!("{:.0}%", summary.time_at_high_soc_percent),
                Style::default().fg(high_soc_color(summary.time_at_high_soc_percent, theme)),
            ),
        ]),
    ];

    let left_para = Paragraph::new(left_stats)
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(1)));
    frame.render_widget(left_para, chunks[0]);

    let right_para = Paragraph::new(right_stats)
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(1)));
    frame.render_widget(right_para, chunks[1]);
}

pub fn render_recent_sessions(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Recent Charge Sessions ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.recent_charge_sessions.is_empty() {
        let no_data = Paragraph::new(vec![Line::from(vec![Span::styled(
            "No sessions recorded",
            theme.muted_style(),
        )])])
        .centered();
        frame.render_widget(no_data, inner);
        return;
    }

    let header = Row::new(vec!["Type", "Start", "Duration", "Range", "Energy"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .recent_charge_sessions
        .iter()
        .rev()
        .take(5)
        .map(|s| session_to_row(s, theme))
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .column_spacing(1);

    frame.render_widget(table, inner);
}

fn session_to_row(session: &ChargeSession, theme: &ThemeColors) -> Row<'static> {
    let type_label = match session.session_type {
        SessionType::Charge => "Charge",
        SessionType::Discharge => "Discharge",
    };

    let type_color = match session.session_type {
        SessionType::Charge => theme.success,
        SessionType::Discharge => theme.warning,
    };

    let start_time = chrono::DateTime::from_timestamp(session.start_time, 0)
        .map(|dt| dt.format("%m/%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string());

    let duration = session
        .duration_secs()
        .map(format_duration)
        .unwrap_or_else(|| "-".to_string());

    let end_percent = session.end_percent.unwrap_or(session.start_percent);
    let range = format!("{}%-{}%", session.start_percent as i32, end_percent as i32);

    let energy = session
        .energy_wh
        .map(|e| format!("{:.1}Wh", e))
        .unwrap_or_else(|| "-".to_string());

    Row::new(vec![
        type_label.to_string(),
        start_time,
        duration,
        range,
        energy,
    ])
    .style(Style::default().fg(type_color))
}

fn format_duration(secs: i64) -> String {
    if secs < 0 {
        return "-".to_string();
    }
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn cycle_rate_color(rate: f32, theme: &ThemeColors) -> ratatui::style::Color {
    if rate < 0.3 {
        theme.success
    } else if rate < 0.5 {
        theme.fg
    } else if rate < 0.8 {
        theme.warning
    } else {
        theme.danger
    }
}

fn dod_color(dod: f32, theme: &ThemeColors) -> ratatui::style::Color {
    if dod < 30.0 {
        theme.success
    } else if dod < 50.0 {
        theme.fg
    } else if dod < 70.0 {
        theme.warning
    } else {
        theme.danger
    }
}

fn high_soc_color(percent: f32, theme: &ThemeColors) -> ratatui::style::Color {
    if percent < 20.0 {
        theme.success
    } else if percent < 40.0 {
        theme.fg
    } else if percent < 60.0 {
        theme.warning
    } else {
        theme.danger
    }
}
