use ratatui::{
    layout::Rect,
    style::Style,
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::app::App;
use crate::config::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = format!(" {} (press 'g' to toggle) ", app.history.metric_label());

    let block = Block::default()
        .title(title)
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
        ratatui::text::Span::raw("now"),
        ratatui::text::Span::raw(format!("-{}s", max_x as i32)),
    ];

    let y_labels = vec![
        ratatui::text::Span::styled(format!("{:.0}", min_y), Style::default().fg(theme.muted)),
        ratatui::text::Span::styled(
            format!("{:.0}", (min_y + max_y) / 2.0),
            Style::default().fg(theme.muted),
        ),
        ratatui::text::Span::styled(format!("{:.0}", max_y), Style::default().fg(theme.muted)),
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
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}
