use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::config::Theme;

pub fn render_title_bar(frame: &mut Frame, area: Rect, theme: &Theme) {
    let version = super::VERSION;
    let title = Line::from(vec![
        Span::styled("⚡️jolt ", Style::default().fg(theme.accent)),
        Span::styled(format!("v{}", version), Style::default().fg(theme.muted)),
    ]);

    let bar = Paragraph::new(title).style(Style::default().bg(theme.bg));
    frame.render_widget(bar, area);
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let hints = [
        ("?", "help"),
        ("C", "config"),
        ("t", "theme"),
        ("g", "graph"),
        ("m", "merge"),
        ("q", "quit"),
    ];

    let mut left_spans: Vec<Span> = vec![Span::raw(" ")];
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            left_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        }
        left_spans.push(Span::styled(*key, Style::default().fg(theme.accent)));
        left_spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(theme.muted),
        ));
    }

    let refresh_str = format_refresh_rate(app.refresh_ms);
    let right_text = format!("-/+ {} ", refresh_str);

    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width = right_text.len();
    let padding = area.width as usize - left_width - right_width;

    left_spans.push(Span::raw(" ".repeat(padding.saturating_sub(1))));
    left_spans.push(Span::styled("-/+", Style::default().fg(theme.muted)));
    left_spans.push(Span::styled(
        format!(" {} ", refresh_str),
        Style::default().fg(theme.accent),
    ));

    let line = Line::from(left_spans);
    let bar = Paragraph::new(line)
        .style(Style::default().bg(theme.bg))
        .alignment(Alignment::Left);

    frame.render_widget(bar, area);
}

fn format_refresh_rate(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}
