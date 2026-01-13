use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Padding, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::battery::ChargeState;
use crate::data::power::PowerMode;
use crate::theme::ThemeColors;

use super::utils::color_for_percent;

/// Returns the icon for the given power mode.
fn power_mode_icon(mode: PowerMode) -> &'static str {
    match mode {
        PowerMode::LowPower => "🐢",
        PowerMode::HighPerformance => "🚀",
        PowerMode::Automatic => "⚙️",
        PowerMode::Unknown => "",
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Battery ")
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let spacer_height = 1;
    let info_card_height = if inner.height >= 5 { 3 } else { 0 };
    let gauge_height = inner
        .height
        .saturating_sub(spacer_height + info_card_height);

    if gauge_height > 0 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(gauge_height),
                Constraint::Length(spacer_height),
                Constraint::Length(info_card_height),
            ])
            .split(inner);

        render_battery_gauge(frame, chunks[0], app, theme);

        if info_card_height > 0 {
            render_battery_info_card(frame, chunks[2], app, theme);
        }
    }
}

fn render_battery_gauge(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let percent = app.battery.charge_percent();
    let gauge_color = color_for_percent(percent, 50.0, 20.0, theme);
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

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    let v_center = |chunk: Rect| {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(chunk)[1]
    };

    let left_block = Block::default()
        .title(" Status ")
        .title_alignment(Alignment::Right)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.bg));
    let left_inner = left_block.inner(chunks[0]);
    frame.render_widget(left_block, chunks[0]);

    let right_block = Block::default()
        .title(" Health ")
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.bg));
    let right_inner = right_block.inner(chunks[1]);
    frame.render_widget(right_block, chunks[1]);

    let state_icon = match app.battery.state() {
        ChargeState::Charging => "⚡",
        ChargeState::Discharging => "↓",
        ChargeState::Full => "✓",
        ChargeState::NotCharging => "⏸",
        ChargeState::Unknown => "?",
    };

    let (time_label, time_value) = match app.battery.state() {
        ChargeState::Charging => (
            "Full in",
            app.battery
                .time_remaining_formatted()
                .unwrap_or_else(|| "—".to_string()),
        ),
        ChargeState::Discharging => (
            "Runtime",
            app.battery
                .time_remaining_formatted()
                .unwrap_or_else(|| "—".to_string()),
        ),
        ChargeState::Full => ("", "Charged".to_string()),
        ChargeState::NotCharging => ("", "Not charging".to_string()),
        ChargeState::Unknown => ("", "—".to_string()),
    };

    let mut left_spans = vec![
        Span::styled(format!("{} ", state_icon), theme.accent_style()),
        Span::styled(
            app.battery.state_label(),
            theme.fg_style().add_modifier(Modifier::BOLD),
        ),
    ];

    if !time_label.is_empty() {
        left_spans.push(Span::styled("  ", Style::default()));
        left_spans.push(Span::styled(
            format!("{}: ", time_label),
            theme.muted_style(),
        ));
        left_spans.push(Span::styled(&time_value, theme.fg_style()));
    }

    if app.battery.state() == ChargeState::Discharging {
        if let Some(forecast) = app.forecast.formatted() {
            left_spans.push(Span::styled("  ", Style::default()));
            left_spans.push(Span::styled("Forecast: ", theme.muted_style()));
            left_spans.push(Span::styled(forecast, theme.success_style()));
        }
    }

    if let Some(power) = if app.battery.is_charging() {
        app.battery.charging_watts().map(|w| {
            app.battery
                .charger_watts()
                .map_or(format!("{:.1}W", w), |c| format!("{:.1}W/{}W", w, c))
        })
    } else {
        app.battery.discharge_watts().map(|w| format!("{:.1}W", w))
    } {
        left_spans.push(Span::styled("  ", Style::default()));
        left_spans.push(Span::styled("Power: ", theme.muted_style()));
        left_spans.push(Span::styled(power, theme.accent_style()));
    }

    let left = Paragraph::new(Line::from(left_spans)).alignment(Alignment::Right);
    frame.render_widget(left, v_center(left_inner));

    let health_color = color_for_percent(app.battery.health_percent(), 79.0, 49.0, theme);
    let cycles_text = app
        .battery
        .cycle_count()
        .map_or("—".to_string(), |c| c.to_string());

    let mut right_spans = vec![
        Span::styled("Health: ", theme.muted_style()),
        Span::styled(
            format!("{:.0}%", app.battery.health_percent()),
            Style::default()
                .fg(health_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled("Cycles: ", theme.muted_style()),
        Span::styled(cycles_text, theme.fg_style()),
    ];

    if let Some(temp) = app.battery.temperature_c() {
        right_spans.push(Span::styled("  ", Style::default()));
        right_spans.push(Span::styled("Temp: ", theme.muted_style()));
        right_spans.push(Span::styled(
            format!("{:.1}°C", temp),
            theme.warning_style(),
        ));
    }

    right_spans.push(Span::styled("  ", Style::default()));
    right_spans.push(Span::styled("Energy: ", theme.muted_style()));
    right_spans.push(Span::styled(
        format!("{:.1}Wh", app.battery.energy_wh()),
        theme.fg_style(),
    ));

    if app.power.power_mode() != PowerMode::Unknown {
        let mode_icon = power_mode_icon(app.power.power_mode());
        right_spans.push(Span::styled("  ", Style::default()));
        right_spans.push(Span::styled("Mode: ", theme.muted_style()));
        right_spans.push(Span::styled(
            format!("{} {}", mode_icon, app.power.power_mode_label()),
            theme.fg_style(),
        ));
    }

    let right = Paragraph::new(Line::from(right_spans)).alignment(Alignment::Left);
    frame.render_widget(right, v_center(right_inner));
}
