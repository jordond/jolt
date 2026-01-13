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

use super::utils::{
    centered_rect, color_for_percent, color_for_value, convert_temperature, format_energy,
    format_temperature, format_temperature_short,
};

fn text_gauge(percent: f32, width: usize, color: Color) -> Span<'static> {
    let filled = ((percent / 100.0) * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    let gauge_str = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    Span::styled(gauge_str, Style::default().fg(color))
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let popup_width = 70;
    let popup_height = 28;
    let area = centered_rect(frame.area(), popup_width, popup_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Battery Details ")
        .borders(Borders::ALL)
        .border_style(theme.accent_style())
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
            Span::styled("Vendor:     ", theme.muted_style()),
            Span::styled(vendor, theme.fg_style()),
            Span::styled("          Model:  ", theme.muted_style()),
            Span::styled(model, theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("Serial:     ", theme.muted_style()),
            Span::styled(serial, theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("Technology: ", theme.muted_style()),
            Span::styled(technology, theme.fg_style()),
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
    let energy_unit = app.config.user_config.units.energy;

    let percent_color = color_for_percent(percent, 50.0, 20.0, theme);

    let lines = vec![
        Line::from(vec![
            Span::styled("Charge:     ", theme.muted_style()),
            Span::styled(
                format!("{:.1}%", percent),
                Style::default()
                    .fg(percent_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            text_gauge(percent, 20, percent_color),
            Span::styled(
                format!("  ({})", format_energy(energy, energy_unit)),
                theme.muted_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status:     ", theme.muted_style()),
            Span::styled(state, theme.fg_style()),
            Span::styled(
                format!(
                    "          Capacity: {}",
                    format_energy(max_capacity, energy_unit)
                ),
                theme.muted_style(),
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
    let energy_unit = app.config.user_config.units.energy;

    let health_color = color_for_percent(health, 80.0, 50.0, theme);

    let cycles_str = cycles.map_or("N/A".to_string(), |c| c.to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("Health:     ", theme.muted_style()),
            Span::styled(
                format!("{:.1}%", health),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  ({} / {})",
                    format_energy(max_capacity, energy_unit),
                    format_energy(design_capacity, energy_unit)
                ),
                theme.muted_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Cycles:     ", theme.muted_style()),
            Span::styled(&cycles_str, theme.fg_style()),
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
    let temp_unit = app.config.user_config.units.temperature;

    let temp_str = temp.map_or("N/A".to_string(), |t| format_temperature(t, temp_unit));
    let voltage_str = format!("{:.2} V", voltage as f32 / 1000.0);
    let amperage_str = format!("{} mA", amperage);
    let rate_str = format!("{:.2} W", energy_rate.abs());

    let lines = vec![
        Line::from(vec![
            Span::styled("Temp:       ", theme.muted_style()),
            Span::styled(&temp_str, theme.fg_style()),
            Span::styled("          Voltage:  ", theme.muted_style()),
            Span::styled(&voltage_str, theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("Amperage:   ", theme.muted_style()),
            Span::styled(&amperage_str, theme.fg_style()),
            Span::styled("       Power:    ", theme.muted_style()),
            Span::styled(&rate_str, theme.accent_style()),
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
            Span::styled("Today:      ", theme.muted_style()),
            Span::styled(format!("{:.0}%", min), theme.warning_style()),
            Span::styled(" - ", theme.muted_style()),
            Span::styled(format!("{:.0}%", max), theme.success_style()),
            Span::styled(" (min - max)", theme.muted_style()),
        ])
    } else {
        Line::from(vec![Span::styled(
            "Today:      N/A (macOS only)",
            theme.muted_style(),
        )])
    };

    let paragraph = Paragraph::new(vec![line]);
    frame.render_widget(paragraph, area);
}

fn render_temperature_chart(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let temp_unit = app.config.user_config.units.temperature;
    let border_color = app
        .battery
        .temperature_c()
        .map_or(theme.border, |t| color_for_value(t, 35.0, 45.0, theme));

    let block = Block::default()
        .title(" Temperature ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.dialog_bg));

    let data = app.history.temperature_values();

    if data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let (min_y_c, max_y_c) = app.history.temperature_range();
    let min_y = convert_temperature(min_y_c as f32, temp_unit) as f64;
    let max_y = convert_temperature(max_y_c as f32, temp_unit) as f64;
    let max_x = data.len().max(60) as f64;

    let converted_data: Vec<(f64, f64)> = data
        .iter()
        .map(|(x, y)| (*x, convert_temperature(*y as f32, temp_unit) as f64))
        .collect();

    let cool_threshold = convert_temperature(35.0, temp_unit) as f64;
    let hot_threshold = convert_temperature(45.0, temp_unit) as f64;

    let cool_points: Vec<(f64, f64)> = converted_data
        .iter()
        .filter(|(_, y)| *y <= cool_threshold)
        .copied()
        .collect();
    let warm_points: Vec<(f64, f64)> = converted_data
        .iter()
        .filter(|(_, y)| *y > cool_threshold && *y <= hot_threshold)
        .copied()
        .collect();
    let hot_points: Vec<(f64, f64)> = converted_data
        .iter()
        .filter(|(_, y)| *y > hot_threshold)
        .copied()
        .collect();

    let mut datasets = Vec::new();
    if !cool_points.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Scatter)
                .style(theme.success_style())
                .data(Box::leak(cool_points.into_boxed_slice())),
        );
    }
    if !warm_points.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Scatter)
                .style(theme.warning_style())
                .data(Box::leak(warm_points.into_boxed_slice())),
        );
    }
    if !hot_points.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Scatter)
                .style(theme.danger_style())
                .data(Box::leak(hot_points.into_boxed_slice())),
        );
    }

    let y_labels = vec![
        Span::styled(
            format_temperature_short(min_y_c as f32, temp_unit),
            theme.muted_style(),
        ),
        Span::styled(
            format_temperature_short(max_y_c as f32, temp_unit),
            theme.muted_style(),
        ),
    ];

    let x_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([0.0, max_x]);

    let y_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(x_axis)
        .y_axis(y_axis)
        .style(Style::default().bg(theme.dialog_bg));

    frame.render_widget(chart, area);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let line = Line::from(vec![Span::styled(
        "Press 'b' or Esc to close",
        theme.muted_style(),
    )]);

    let paragraph = Paragraph::new(vec![line]).centered();
    frame.render_widget(paragraph, area);
}
