use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" System ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

    let (load_text, memory_text, uptime_text) = if app.system_stats.is_warmed_up() {
        (
            format!("{:.2}", app.system_stats.load_one()),
            app.system_stats.memory_formatted(),
            app.system_stats.uptime_formatted(),
        )
    } else {
        ("—".to_string(), "—".to_string(), "—".to_string())
    };

    let load_color = if app.system_stats.is_warmed_up() {
        let load = app.system_stats.load_one();
        if load > 4.0 {
            theme.danger
        } else if load > 2.0 {
            theme.warning
        } else {
            theme.success
        }
    } else {
        theme.muted
    };

    let memory_color = if app.system_stats.is_warmed_up() {
        let used = app.system_stats.memory_used_gb();
        let total = app.system_stats.memory_total_gb();
        let percent = if total > 0.0 {
            used / total * 100.0
        } else {
            0.0
        };
        if percent > 80.0 {
            theme.danger
        } else if percent > 60.0 {
            theme.warning
        } else {
            theme.success
        }
    } else {
        theme.muted
    };

    let load = Paragraph::new(Line::from(vec![
        Span::styled("Load: ", Style::default().fg(theme.muted)),
        Span::styled(
            load_text,
            Style::default().fg(load_color).add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let memory = Paragraph::new(Line::from(vec![
        Span::styled("Mem: ", Style::default().fg(theme.muted)),
        Span::styled(
            memory_text,
            Style::default()
                .fg(memory_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let uptime = Paragraph::new(Line::from(vec![
        Span::styled("Up: ", Style::default().fg(theme.muted)),
        Span::styled(
            uptime_text,
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

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

    frame.render_widget(load, v_center(chunks[0]));
    frame.render_widget(memory, v_center(chunks[1]));
    frame.render_widget(uptime, v_center(chunks[2]));
}
