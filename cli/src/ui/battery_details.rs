use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

fn color_for_percent(percent: f32, high: f32, low: f32, theme: &ThemeColors) -> Color {
    if percent > high {
        theme.success
    } else if percent > low {
        theme.warning
    } else {
        theme.danger
    }
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let popup_width = 70;
    let popup_height = 28;
    let area = centered_rect(frame.area(), popup_width, popup_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Battery Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let has_temp_data = app.history.has_temperature_data();
    let chart_height = if has_temp_data { 8 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(chart_height),
            Constraint::Min(1),
        ])
        .split(inner);

    render_device_info(frame, chunks[0], app, theme);
    render_charge_info(frame, chunks[1], app, theme);
    render_health_info(frame, chunks[2], app, theme);
    render_electrical_info(frame, chunks[3], app, theme);
    render_daily_soc(frame, chunks[4], app, theme);

    if has_temp_data {
        render_temperature_chart(frame, chunks[5], app, theme);
    }

    render_footer(frame, chunks[6], theme);
}

fn render_device_info(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let vendor = app.battery.vendor().unwrap_or("Unknown");
    let model = app.battery.model().unwrap_or("Unknown");
    let serial = app.battery.serial_number().unwrap_or("N/A");
    let technology = app.battery.technology().label();

    let lines = vec![
        Line::from(vec![
            Span::styled("Vendor:     ", Style::default().fg(theme.muted)),
            Span::styled(vendor, Style::default().fg(theme.fg)),
            Span::styled("          Model:  ", Style::default().fg(theme.muted)),
            Span::styled(model, Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("Serial:     ", Style::default().fg(theme.muted)),
            Span::styled(serial, Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("Technology: ", Style::default().fg(theme.muted)),
            Span::styled(technology, Style::default().fg(theme.fg)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_charge_info(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let percent = app.battery.charge_percent();
    let energy = app.battery.energy_wh();
    let max_capacity = app.battery.max_capacity_wh();
    let state = app.battery.state_label();

    let percent_color = color_for_percent(percent, 50.0, 20.0, theme);

    let gauge_width = 20;
    let filled = ((percent / 100.0) * gauge_width as f32) as usize;
    let empty = gauge_width - filled;
    let gauge_str = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    let lines = vec![
        Line::from(vec![
            Span::styled("Charge:     ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.1}%", percent),
                Style::default()
                    .fg(percent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(&gauge_str, Style::default().fg(percent_color)),
            Span::styled(
                format!("  ({:.1} Wh)", energy),
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status:     ", Style::default().fg(theme.muted)),
            Span::styled(state, Style::default().fg(theme.fg)),
            Span::styled(
                format!("          Capacity: {:.1} Wh", max_capacity),
                Style::default().fg(theme.muted),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_health_info(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let health = app.battery.health_percent();
    let cycles = app.battery.cycle_count();
    let design_capacity = app.battery.design_capacity_wh();
    let max_capacity = app.battery.max_capacity_wh();

    let health_color = color_for_percent(health, 80.0, 50.0, theme);

    let cycles_str = cycles.map_or("N/A".to_string(), |c| c.to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("Health:     ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.1}%", health),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({:.1} / {:.1} Wh)", max_capacity, design_capacity),
                Style::default().fg(theme.muted),
            ),
        ]),
        Line::from(vec![
            Span::styled("Cycles:     ", Style::default().fg(theme.muted)),
            Span::styled(&cycles_str, Style::default().fg(theme.fg)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_electrical_info(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let temp = app.battery.temperature_c();
    let voltage = app.battery.voltage_mv();
    let amperage = app.battery.amperage_ma();
    let energy_rate = app.battery.energy_rate_watts();

    let temp_str = temp.map_or("N/A".to_string(), |t| format!("{:.1}°C", t));
    let voltage_str = format!("{:.2} V", voltage as f32 / 1000.0);
    let amperage_str = format!("{} mA", amperage);
    let rate_str = format!("{:.2} W", energy_rate.abs());

    let lines = vec![
        Line::from(vec![
            Span::styled("Temp:       ", Style::default().fg(theme.muted)),
            Span::styled(&temp_str, Style::default().fg(theme.fg)),
            Span::styled("          Voltage:  ", Style::default().fg(theme.muted)),
            Span::styled(&voltage_str, Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("Amperage:   ", Style::default().fg(theme.muted)),
            Span::styled(&amperage_str, Style::default().fg(theme.fg)),
            Span::styled("       Power:    ", Style::default().fg(theme.muted)),
            Span::styled(&rate_str, Style::default().fg(theme.accent)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_daily_soc(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let min_soc = app.battery.daily_min_soc();
    let max_soc = app.battery.daily_max_soc();

    let line = if let (Some(min), Some(max)) = (min_soc, max_soc) {
        Line::from(vec![
            Span::styled("Today:      ", Style::default().fg(theme.muted)),
            Span::styled(format!("{:.0}%", min), Style::default().fg(theme.warning)),
            Span::styled(" - ", Style::default().fg(theme.muted)),
            Span::styled(format!("{:.0}%", max), Style::default().fg(theme.success)),
            Span::styled(" (min - max)", Style::default().fg(theme.muted)),
        ])
    } else {
        Line::from(vec![Span::styled(
            "Today:      N/A (macOS only)",
            Style::default().fg(theme.muted),
        )])
    };

    let paragraph = Paragraph::new(vec![line]);
    frame.render_widget(paragraph, area);
}

fn render_temperature_chart(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.dialog_bg));

    let data = app.history.temperature_values();

    if data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let (min_y, max_y) = app.history.temperature_range();
    let max_x = data.len().max(60) as f64;

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.warning))
        .data(&data);

    let y_labels = vec![
        Span::styled(format!("{:.0}°", min_y), Style::default().fg(theme.muted)),
        Span::styled(format!("{:.0}°", max_y), Style::default().fg(theme.muted)),
    ];

    let x_axis = Axis::default()
        .style(Style::default().fg(theme.muted))
        .bounds([0.0, max_x]);

    let y_axis = Axis::default()
        .style(Style::default().fg(theme.muted))
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(vec![dataset])
        .block(block)
        .x_axis(x_axis)
        .y_axis(y_axis)
        .style(Style::default().bg(theme.dialog_bg));

    frame.render_widget(chart, area);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let line = Line::from(vec![Span::styled(
        "Press 'b' or Esc to close",
        Style::default().fg(theme.muted),
    )]);

    let paragraph = Paragraph::new(vec![line]).centered();
    frame.render_widget(paragraph, area);
}
