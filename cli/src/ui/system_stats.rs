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
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);

    let (cpu_text, memory_text, disk_text, uptime_text) = if app.system_stats.is_warmed_up() {
        let cpu = app
            .system_stats
            .cpu_load_percent()
            .map(|v| format!("{:.0}%", v))
            .unwrap_or_else(|| "—".to_string());
        (
            cpu,
            format!("{:.0}%", app.system_stats.memory_used_percent()),
            format!("{:.0}%", app.system_stats.disk_used_percent()),
            app.system_stats.uptime_formatted(),
        )
    } else {
        (
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
        )
    };

    let percent_color = |value: f32| -> ratatui::style::Color {
        if value > 80.0 {
            theme.danger
        } else if value > 60.0 {
            theme.warning
        } else {
            theme.success
        }
    };

    let cpu_color = app
        .system_stats
        .cpu_load_percent()
        .map(percent_color)
        .unwrap_or(theme.muted);

    let memory_color = if app.system_stats.is_warmed_up() {
        percent_color(app.system_stats.memory_used_percent())
    } else {
        theme.muted
    };

    let disk_color = if app.system_stats.is_warmed_up() {
        percent_color(app.system_stats.disk_used_percent())
    } else {
        theme.muted
    };

    let cpu = Paragraph::new(Line::from(vec![
        Span::styled("CPU: ", Style::default().fg(theme.muted)),
        Span::styled(
            cpu_text,
            Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
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

    let disk = Paragraph::new(Line::from(vec![
        Span::styled("Disk: ", Style::default().fg(theme.muted)),
        Span::styled(
            disk_text,
            Style::default().fg(disk_color).add_modifier(Modifier::BOLD),
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

    frame.render_widget(cpu, v_center(chunks[0]));
    frame.render_widget(memory, v_center(chunks[1]));
    frame.render_widget(disk, v_center(chunks[2]));
    frame.render_widget(uptime, v_center(chunks[3]));
}
