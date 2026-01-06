use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::history::HistoryMetric;
use crate::theme::ThemeColors;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    match app.history.current_metric {
        HistoryMetric::Split => render_split(frame, area, app, theme),
        HistoryMetric::Merged => render_merged(frame, area, app, theme),
        _ => render_single(frame, area, app, theme),
    }
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
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            current_value.unwrap_or_default(),
            Style::default().fg(theme.fg),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            avg_value.unwrap_or_default(),
            Style::default().fg(theme.muted),
        ),
        Span::styled(" (g: toggle) ", Style::default().fg(theme.muted)),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let data = app.history.current_values();

    if data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let (min_y, max_y) = app.history.value_range();
    let max_x = data.len().max(60) as f64;

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.graph_line))
        .data(&data);

    let x_labels = vec![
        Span::styled("now", Style::default().fg(theme.muted)),
        Span::styled(
            format!("-{}s", max_x as i32),
            Style::default().fg(theme.muted),
        ),
    ];

    let quarter = (max_y - min_y) / 4.0;
    let y_labels = vec![
        Span::styled(format!("{:.0}", min_y), Style::default().fg(theme.muted)),
        Span::styled(
            format!("{:.0}", min_y + quarter),
            Style::default().fg(theme.muted),
        ),
        Span::styled(
            format!("{:.0}", min_y + quarter * 2.0),
            Style::default().fg(theme.muted),
        ),
        Span::styled(
            format!("{:.0}", min_y + quarter * 3.0),
            Style::default().fg(theme.muted),
        ),
        Span::styled(format!("{:.0}", max_y), Style::default().fg(theme.muted)),
    ];

    let x_axis = Axis::default()
        .style(Style::default().fg(theme.muted))
        .bounds([0.0, max_x])
        .labels(x_labels);

    let y_axis = Axis::default()
        .style(Style::default().fg(theme.muted))
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(vec![dataset])
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
                .style(Style::default().fg(theme.accent_secondary));
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
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:.1}W", power_val),
            Style::default().fg(theme.graph_line),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            format!("{:.0}%", battery_val),
            Style::default().fg(theme.accent_secondary),
        ),
        Span::styled(" (g: toggle) ", Style::default().fg(theme.muted)),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
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

    let power_dataset = Dataset::default()
        .name("Power")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.graph_line))
        .data(&power_data);

    let battery_dataset = Dataset::default()
        .name("Battery")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.accent_secondary))
        .data(&battery_data);

    let x_labels = vec![
        Span::styled("now", Style::default().fg(theme.muted)),
        Span::styled(
            format!("-{}s", max_x as i32),
            Style::default().fg(theme.muted),
        ),
    ];

    let quarter = (max_y - min_y) / 4.0;
    let y_labels = vec![
        Span::styled(format!("{:.0}W", min_y), Style::default().fg(theme.muted)),
        Span::styled(
            format!("{:.0}W", min_y + quarter * 2.0),
            Style::default().fg(theme.muted),
        ),
        Span::styled(format!("{:.0}W", max_y), Style::default().fg(theme.muted)),
    ];

    let x_axis = Axis::default()
        .style(Style::default().fg(theme.muted))
        .bounds([0.0, max_x])
        .labels(x_labels);

    let y_axis = Axis::default()
        .style(Style::default().fg(theme.muted))
        .bounds([min_y, max_y])
        .labels(y_labels);

    let chart = Chart::new(vec![power_dataset, battery_dataset])
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
    line_color: ratatui::style::Color,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    if data.is_empty() {
        frame.render_widget(block, area);
        return;
    }

    let max_x = data.len().max(60) as f64;

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(line_color))
        .data(data);

    let y_labels = vec![
        ratatui::text::Span::styled(format!("{:.0}", min_y), Style::default().fg(theme.muted)),
        ratatui::text::Span::styled(format!("{:.0}", max_y), Style::default().fg(theme.muted)),
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
        .style(Style::default().bg(theme.bg));

    frame.render_widget(chart, area);
}
