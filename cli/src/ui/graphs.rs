use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::history::HistoryMetric;
use crate::theme::ThemeColors;

const MIN_WIDTH_FOR_SIDE_BY_SIDE: u16 = 80;
const BATTERY_WARNING_THRESHOLD: f64 = 20.0;

fn horizontal_line_points(y_value: f64, max_x: f64) -> Vec<(f64, f64)> {
    vec![(0.0, y_value), (max_x, y_value)]
}

fn x_axis_time_labels(data_len: usize, theme: &ThemeColors) -> Vec<Span<'static>> {
    let max_x = data_len.max(60);
    vec![
        Span::styled("now", theme.muted_style()),
        Span::styled(format!("-{}s", max_x / 2), theme.muted_style()),
        Span::styled(format!("-{}s", max_x), theme.muted_style()),
    ]
}

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let has_temp_data = app.history.has_temperature_data();

    if has_temp_data {
        if area.width >= MIN_WIDTH_FOR_SIDE_BY_SIDE {
            render_with_temp_side_by_side(frame, area, app, theme);
        } else {
            render_with_temp_stacked(frame, area, app, theme);
        }
    } else {
        match app.history.current_metric {
            HistoryMetric::Split => render_split(frame, area, app, theme),
            HistoryMetric::Merged => render_merged(frame, area, app, theme),
            _ => render_single(frame, area, app, theme),
        }
    }
}

fn render_with_temp_side_by_side(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    match app.history.current_metric {
        HistoryMetric::Split => render_split(frame, chunks[0], app, theme),
        HistoryMetric::Merged => render_merged(frame, chunks[0], app, theme),
        _ => render_single(frame, chunks[0], app, theme),
    }

    render_temperature_chart(frame, chunks[1], app, theme);
}

fn render_with_temp_stacked(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    match app.history.current_metric {
        HistoryMetric::Split => render_split(frame, chunks[0], app, theme),
        HistoryMetric::Merged => render_merged(frame, chunks[0], app, theme),
        _ => render_single(frame, chunks[0], app, theme),
    }

    render_temperature_chart(frame, chunks[1], app, theme);
}

fn render_temperature_chart(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let temp_data = app.history.temperature_values();
    let current_temp = app.history.latest_temperature();

    let title_line = Line::from(vec![
        Span::styled(" Temp ", theme.warning_style().add_modifier(Modifier::BOLD)),
        Span::styled(
            current_temp.map_or("--".to_string(), |t| format!("{:.1}°C", t)),
            theme.fg_style(),
        ),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(Style::default().bg(theme.bg));

    if temp_data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let (min_y, max_y) = app.history.temperature_range();
    let max_x = temp_data.len().max(60) as f64;

    let mut datasets = Vec::new();

    let grid_color = Color::Rgb(60, 60, 60);
    let mid_y = (min_y + max_y) / 2.0;
    let grid_data: Vec<(f64, f64)> = horizontal_line_points(mid_y, max_x);
    datasets.push(
        Dataset::default()
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(grid_color))
            .data(Box::leak(grid_data.into_boxed_slice())),
    );

    datasets.push(
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(theme.warning_style())
            .data(&temp_data),
    );

    let x_labels = x_axis_time_labels(temp_data.len(), theme);

    let y_labels = vec![
        Span::styled(format!("{:.0}°", min_y), theme.muted_style()),
        Span::styled(format!("{:.0}°", mid_y), theme.muted_style()),
        Span::styled(format!("{:.0}°", max_y), theme.muted_style()),
    ];

    let x_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([0.0, max_x])
        .labels(x_labels);

    let y_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(x_axis)
        .y_axis(y_axis)
        .style(Style::default().bg(theme.bg));

    frame.render_widget(chart, area);
}

fn render_single(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let is_battery = app.history.current_metric == HistoryMetric::Battery;

    let current_value = if is_battery {
        app.history
            .points
            .back()
            .map(|p| format!("{:.0}%", p.battery_percent))
    } else {
        app.history
            .points
            .back()
            .map(|p| format!("{:.1}W", p.power_watts))
    };

    let avg_value = if is_battery {
        let sum: f32 = app.history.points.iter().map(|p| p.battery_percent).sum();
        if !app.history.points.is_empty() {
            Some(format!(
                "avg: {:.0}%",
                sum / app.history.points.len() as f32
            ))
        } else {
            None
        }
    } else {
        let sum: f32 = app.history.points.iter().map(|p| p.power_watts).sum();
        if !app.history.points.is_empty() {
            Some(format!(
                "avg: {:.1}W",
                sum / app.history.points.len() as f32
            ))
        } else {
            None
        }
    };

    let title_line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.history.metric_label()),
            theme.accent_style().add_modifier(Modifier::BOLD),
        ),
        Span::styled(current_value.unwrap_or_default(), theme.fg_style()),
        Span::styled(" ", Style::default()),
        Span::styled(avg_value.unwrap_or_default(), theme.muted_style()),
        Span::styled(" (g: toggle) ", theme.muted_style()),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(Style::default().bg(theme.bg));

    let data = app.history.current_values();

    if data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let (min_y, max_y) = app.history.value_range();
    let max_x = data.len().max(60) as f64;

    let mut datasets = Vec::new();

    let quarter = (max_y - min_y) / 4.0;
    let grid_color = Color::Rgb(60, 60, 60);
    for i in 1..4 {
        let grid_y = min_y + quarter * i as f64;
        let grid_data: Vec<(f64, f64)> = horizontal_line_points(grid_y, max_x);
        datasets.push(
            Dataset::default()
                .marker(Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(grid_color))
                .data(Box::leak(grid_data.into_boxed_slice())),
        );
    }

    if is_battery && min_y < BATTERY_WARNING_THRESHOLD && max_y > BATTERY_WARNING_THRESHOLD {
        let threshold_data: Vec<(f64, f64)> =
            horizontal_line_points(BATTERY_WARNING_THRESHOLD, max_x);
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(theme.danger_style())
                .data(Box::leak(threshold_data.into_boxed_slice())),
        );
    }

    datasets.push(
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(theme.graph_style())
            .data(&data),
    );

    let x_labels = x_axis_time_labels(data.len(), theme);

    let y_labels = vec![
        Span::styled(format!("{:.0}", min_y), theme.muted_style()),
        Span::styled(format!("{:.0}", min_y + quarter), theme.muted_style()),
        Span::styled(format!("{:.0}", min_y + quarter * 2.0), theme.muted_style()),
        Span::styled(format!("{:.0}", min_y + quarter * 3.0), theme.muted_style()),
        Span::styled(format!("{:.0}", max_y), theme.muted_style()),
    ];

    let x_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([0.0, max_x])
        .labels(x_labels);

    let y_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(x_axis)
        .y_axis(y_axis)
        .style(Style::default().bg(theme.bg));

    frame.render_widget(chart, area);

    if is_battery && !app.history.battery_changes.is_empty() {
        render_battery_markers(frame, area, app, theme, max_x, min_y, max_y);
    }
}

fn render_battery_markers(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &ThemeColors,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) {
    let inner = Rect::new(
        area.x + 8,
        area.y + 1,
        area.width.saturating_sub(10),
        area.height.saturating_sub(3),
    );

    for change in &app.history.battery_changes {
        let x_ratio = change.index as f64 / max_x;
        let y_ratio = (change.value as f64 - min_y) / (max_y - min_y);

        let x = inner.x + (x_ratio * inner.width as f64) as u16;
        let y = inner.y + inner.height - (y_ratio * inner.height as f64) as u16;

        if x < inner.x + inner.width && y >= inner.y && y < inner.y + inner.height {
            let marker = Paragraph::new(format!("{:.0}", change.value))
                .style(theme.accent_secondary_style());
            let marker_area = Rect::new(x.saturating_sub(1), y.saturating_sub(1), 4, 1);
            frame.render_widget(marker, marker_area);
        }
    }
}

fn render_merged(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let power_val = app
        .history
        .points
        .back()
        .map(|p| p.power_watts)
        .unwrap_or(0.0);
    let battery_val = app
        .history
        .points
        .back()
        .map(|p| p.battery_percent)
        .unwrap_or(0.0);

    let title_line = Line::from(vec![
        Span::styled(
            " Combined ",
            theme.accent_style().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{:.1}W", power_val), theme.graph_style()),
        Span::styled(" │ ", theme.border_style()),
        Span::styled(
            format!("{:.0}%", battery_val),
            theme.accent_secondary_style(),
        ),
        Span::styled(" (g: toggle) ", theme.muted_style()),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(Style::default().bg(theme.bg));

    let power_data = app.history.power_values();
    let battery_data: Vec<(f64, f64)> = app
        .history
        .battery_values()
        .iter()
        .map(|(x, y)| (*x, y / 100.0 * app.history.power_range().1))
        .collect();

    if power_data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let (min_y, max_y) = app.history.power_range();
    let max_x = power_data.len().max(60) as f64;

    let mut datasets = Vec::new();

    let quarter = (max_y - min_y) / 4.0;
    let grid_color = Color::Rgb(60, 60, 60);
    for i in 1..3 {
        let grid_y = min_y + quarter * (i * 2) as f64;
        let grid_data: Vec<(f64, f64)> = horizontal_line_points(grid_y, max_x);
        datasets.push(
            Dataset::default()
                .marker(Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(grid_color))
                .data(Box::leak(grid_data.into_boxed_slice())),
        );
    }

    datasets.push(
        Dataset::default()
            .name("Power")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(theme.graph_style())
            .data(&power_data),
    );

    datasets.push(
        Dataset::default()
            .name("Battery")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(theme.accent_secondary_style())
            .data(&battery_data),
    );

    let x_labels = x_axis_time_labels(power_data.len(), theme);

    let y_labels = vec![
        Span::styled(format!("{:.0}W", min_y), theme.muted_style()),
        Span::styled(
            format!("{:.0}W", min_y + quarter * 2.0),
            theme.muted_style(),
        ),
        Span::styled(format!("{:.0}W", max_y), theme.muted_style()),
    ];

    let x_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([0.0, max_x])
        .labels(x_labels);

    let y_axis = Axis::default()
        .style(theme.muted_style())
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(x_axis)
        .y_axis(y_axis)
        .style(Style::default().bg(theme.bg));

    frame.render_widget(chart, area);
}

fn render_split(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_mini_chart(
        frame,
        chunks[0],
        " Power (W) ",
        &app.history.power_values(),
        app.history.power_range(),
        theme,
        theme.graph_line,
    );

    render_mini_chart(
        frame,
        chunks[1],
        " Battery % ",
        &app.history.battery_values(),
        (0.0, 100.0),
        theme,
        theme.accent_secondary,
    );
}

fn render_mini_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    data: &[(f64, f64)],
    (min_y, max_y): (f64, f64),
    theme: &ThemeColors,
    line_color: Color,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .style(Style::default().bg(theme.bg));

    if data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let max_x = data.len().max(60) as f64;

    let mut datasets = Vec::new();

    let mid_y = (min_y + max_y) / 2.0;
    let grid_color = Color::Rgb(60, 60, 60);
    let grid_data: Vec<(f64, f64)> = horizontal_line_points(mid_y, max_x);
    datasets.push(
        Dataset::default()
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(grid_color))
            .data(Box::leak(grid_data.into_boxed_slice())),
    );

    datasets.push(
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(line_color))
            .data(data),
    );

    let y_labels = vec![
        Span::styled(format!("{:.0}", min_y), theme.muted_style()),
        Span::styled(format!("{:.0}", mid_y), theme.muted_style()),
        Span::styled(format!("{:.0}", max_y), theme.muted_style()),
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
        .style(Style::default().bg(theme.bg));

    frame.render_widget(chart, area);
}
