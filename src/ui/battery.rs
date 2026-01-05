use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::config::Theme;
use crate::data::battery::ChargeState;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(" Battery ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    render_battery_gauge(frame, chunks[0], app, theme);
    render_battery_info(frame, chunks[1], app, theme);
}

fn render_battery_gauge(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let percent = app.battery.charge_percent();
    let ratio = (percent / 100.0).clamp(0.0, 1.0);

    let gauge_color = if percent > 50.0 {
        theme.success
    } else if percent > 20.0 {
        theme.warning
    } else {
        theme.danger
    };

    let label = format!("{:.0}%", percent);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color).bg(theme.muted))
        .ratio(ratio as f64)
        .label(Span::styled(
            label,
            Style::default()
                .fg(theme.fg)
                .add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(gauge, area);
}

fn render_battery_info(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let state_icon = match app.battery.state() {
        ChargeState::Charging => "⚡",
        ChargeState::Discharging => "🔋",
        ChargeState::Full => "✓",
        ChargeState::NotCharging => "⏸",
        ChargeState::Unknown => "?",
    };

    let state_text = format!("{} {}", state_icon, app.battery.state_label());
    let time_text = app
        .battery
        .time_remaining_formatted()
        .map(|t| {
            if app.battery.is_charging() {
                format!("Full in {}", t)
            } else {
                format!("{} remaining", t)
            }
        })
        .unwrap_or_else(|| "Calculating...".to_string());

    let health_text = format!(
        "Health: {:.0}%  Cycles: {}",
        app.battery.health_percent(),
        app.battery.cycle_count().map_or("N/A".to_string(), |c| c.to_string())
    );

    let left = Paragraph::new(Line::from(vec![Span::styled(
        state_text,
        Style::default().fg(theme.accent),
    )]));

    let center = Paragraph::new(Line::from(vec![Span::styled(
        time_text,
        Style::default().fg(theme.fg),
    )]))
    .centered();

    let right = Paragraph::new(Line::from(vec![Span::styled(
        health_text,
        Style::default().fg(theme.muted),
    )]))
    .right_aligned();

    frame.render_widget(left, chunks[0]);
    frame.render_widget(center, chunks[1]);
    frame.render_widget(right, chunks[2]);
}
