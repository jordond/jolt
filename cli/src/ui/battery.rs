use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::battery::ChargeState;
use crate::theme::ThemeColors;

fn percent_to_color(
    percent: f32,
    high_threshold: f32,
    low_threshold: f32,
    theme: &ThemeColors,
) -> Color {
    if percent > high_threshold {
        theme.success
    } else if percent > low_threshold {
        theme.warning
    } else {
        theme.danger
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Battery ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let info_card_height = if inner.height >= 4 { 4 } else { 0 };
    let gauge_height = inner.height.saturating_sub(info_card_height);

    if gauge_height > 0 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(gauge_height),
                Constraint::Length(info_card_height),
            ])
            .split(inner);

        render_battery_gauge(frame, chunks[0], app, theme);

        if info_card_height > 0 {
            render_battery_info_card(frame, chunks[1], app, theme);
        }
    }
}

fn render_battery_gauge(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let percent = app.battery.charge_percent();
    let gauge_color = percent_to_color(percent, 50.0, 20.0, theme);
    let unfilled_color = darken_color(theme.border, 0.6);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color).bg(unfilled_color))
        .ratio((percent / 100.0).clamp(0.0, 1.0) as f64)
        .label(format!("{:.0}%", percent))
        .use_unicode(true);

    frame.render_widget(gauge, area);
}

fn darken_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        _ => Color::Rgb(40, 40, 45),
    }
}

fn render_battery_info_card(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    if area.height == 0 || area.width < 20 {
        return;
    }

    let inner = area;

    let state_icon = match app.battery.state() {
        ChargeState::Charging => "⚡",
        ChargeState::Discharging => "↓",
        ChargeState::Full => "✓",
        ChargeState::NotCharging => "⏸",
        ChargeState::Unknown => "?",
    };

    let (time_label, time_value) = match app.battery.state() {
        ChargeState::Charging => (
            "Time to full:",
            app.battery
                .time_remaining_formatted()
                .unwrap_or_else(|| "Calculating...".to_string()),
        ),
        ChargeState::Discharging => (
            "Runtime:",
            app.battery
                .time_remaining_formatted()
                .unwrap_or_else(|| "Calculating...".to_string()),
        ),
        ChargeState::Full => ("Status:", "Charged".to_string()),
        ChargeState::NotCharging => ("Status:", "Not charging".to_string()),
        ChargeState::Unknown => ("Status:", "—".to_string()),
    };

    let forecast_info: Option<(String, bool)> = if app.battery.state() == ChargeState::Discharging {
        Some(match app.forecast.formatted() {
            Some(f) => {
                let source_hint = match app.forecast.source() {
                    crate::data::ForecastSource::Daemon => "d",
                    crate::data::ForecastSource::Session => "s",
                    crate::data::ForecastSource::None => "",
                };
                let text = if source_hint.is_empty() {
                    f
                } else {
                    format!("{} ({})", f, source_hint)
                };
                (text, true)
            }
            None => ("Calculating...".to_string(), false),
        })
    } else {
        None
    };

    let power_text = if app.battery.is_charging() {
        app.battery.charging_watts().map(|w| {
            app.battery
                .charger_watts()
                .map_or(format!("{:.1}W", w), |c| format!("{:.1}W/{}W", w, c))
        })
    } else {
        app.battery.discharge_watts().map(|w| format!("{:.1}W", w))
    };

    let health_color = percent_to_color(app.battery.health_percent(), 79.0, 49.0, theme);

    let cycles_text = app
        .battery
        .cycle_count()
        .map_or("—".to_string(), |c| c.to_string());

    let single_line = build_single_line(
        state_icon,
        app.battery.state_label(),
        time_label,
        &time_value,
        forecast_info.as_ref().map(|(t, v)| (t.as_str(), *v)),
        power_text.as_deref(),
        app.battery.health_percent(),
        &cycles_text,
        app.battery.max_capacity_wh(),
        app.battery.design_capacity_wh(),
        theme,
        health_color,
    );

    let single_line_width: usize = single_line.spans.iter().map(|s| s.content.len()).sum();

    if inner.width as usize >= single_line_width || inner.height < 3 {
        let centered = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner)[1];
        frame.render_widget(Paragraph::new(single_line).centered(), centered);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let mut row1_spans = vec![
            Span::styled(
                format!("{} ", state_icon),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                app.battery.state_label(),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  │  ", Style::default().fg(theme.border)),
            Span::styled(format!("{} ", time_label), Style::default().fg(theme.muted)),
            Span::styled(&time_value, Style::default().fg(theme.fg)),
        ];

        if let Some((ref text, has_value)) = forecast_info {
            row1_spans.push(Span::styled("  │  ", Style::default().fg(theme.border)));
            row1_spans.push(Span::styled("Forecast: ", Style::default().fg(theme.muted)));
            let color = if has_value {
                theme.success
            } else {
                theme.muted
            };
            row1_spans.push(Span::styled(text, Style::default().fg(color)));
        }

        row1_spans.push(Span::styled(
            power_text.map_or(String::new(), |p| format!("  │  {}", p)),
            Style::default().fg(theme.accent),
        ));

        let row1 = Line::from(row1_spans);

        let mut row2_spans = vec![
            Span::styled("Health: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.0}%", app.battery.health_percent()),
                Style::default().fg(health_color),
            ),
            Span::styled(
                format!(
                    " ({:.1}/{:.1}Wh)",
                    app.battery.max_capacity_wh(),
                    app.battery.design_capacity_wh()
                ),
                Style::default().fg(theme.muted),
            ),
            Span::styled("  │  ", Style::default().fg(theme.border)),
            Span::styled("Cycles: ", Style::default().fg(theme.muted)),
            Span::styled(&cycles_text, Style::default().fg(theme.fg)),
        ];

        if let Some(temp) = app.battery.temperature_c() {
            row2_spans.push(Span::styled("  │  ", Style::default().fg(theme.border)));
            row2_spans.push(Span::styled(
                format!("{:.1}°C", temp),
                Style::default().fg(theme.warning),
            ));
        }

        row2_spans.push(Span::styled("  │  ", Style::default().fg(theme.border)));
        row2_spans.push(Span::styled(
            format!("{:.1}Wh", app.battery.energy_wh()),
            Style::default().fg(theme.fg),
        ));

        if let (Some(min), Some(max)) = (app.battery.daily_min_soc(), app.battery.daily_max_soc()) {
            row2_spans.push(Span::styled("  │  ", Style::default().fg(theme.border)));
            row2_spans.push(Span::styled(
                format!("{:.0}-{:.0}%", min, max),
                Style::default().fg(theme.muted),
            ));
        }

        let row2 = Line::from(row2_spans);

        frame.render_widget(Paragraph::new(row1).centered(), rows[0]);
        frame.render_widget(Paragraph::new(row2).centered(), rows[2]);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_single_line<'a>(
    icon: &'a str,
    state: &'a str,
    time_label: &'a str,
    time_value: &'a str,
    forecast: Option<(&'a str, bool)>,
    power: Option<&'a str>,
    health: f32,
    cycles: &'a str,
    capacity: f32,
    design_capacity: f32,
    theme: &ThemeColors,
    health_color: ratatui::style::Color,
) -> Line<'a> {
    let mut spans = vec![
        Span::styled(format!("{} ", icon), Style::default().fg(theme.accent)),
        Span::styled(
            state,
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(format!("{} ", time_label), Style::default().fg(theme.muted)),
        Span::styled(time_value, Style::default().fg(theme.fg)),
    ];

    if let Some((text, has_value)) = forecast {
        spans.push(Span::styled("  │  ", Style::default().fg(theme.border)));
        spans.push(Span::styled("Forecast: ", Style::default().fg(theme.muted)));
        let color = if has_value {
            theme.success
        } else {
            theme.muted
        };
        spans.push(Span::styled(text, Style::default().fg(color)));
    }

    if let Some(p) = power {
        spans.push(Span::styled(
            format!("  │  {}", p),
            Style::default().fg(theme.accent),
        ));
    }

    spans.extend([
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(
            format!("health {:.0}%", health),
            Style::default().fg(health_color),
        ),
        Span::styled(
            format!(" ({:.0}/{:.0}Wh)", capacity, design_capacity),
            Style::default().fg(theme.muted),
        ),
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(cycles, Style::default().fg(theme.fg)),
        Span::styled(" cycles", Style::default().fg(theme.muted)),
    ]);

    Line::from(spans)
}
