use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::config::DataSizeUnit;
use crate::theme::ThemeColors;

use super::utils::{color_for_value, format_data_size};

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let load_color = if app.system_stats.is_warmed_up() {
        color_for_value(app.system_stats.load_one(), 2.0, 4.0, theme)
    } else {
        theme.muted
    };

    let block = Block::default()
        .title(Span::styled(" System ", Style::default().fg(load_color)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(load_color))
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

    let (cpu_text, load_text, memory_text, uptime_text) = if app.system_stats.is_warmed_up() {
        let data_size_unit = app.config.user_config.units.data_size;
        let memory_text = format_memory(
            app.system_stats.memory_used_bytes(),
            app.system_stats.memory_total_bytes(),
            data_size_unit,
        );
        (
            format!("{:.0}%", app.system_stats.cpu_usage_percent()),
            app.system_stats.load_formatted(),
            memory_text,
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

    let cpu_color = if app.system_stats.is_warmed_up() {
        color_for_value(app.system_stats.cpu_usage_percent(), 60.0, 80.0, theme)
    } else {
        theme.muted
    };

    let memory_color = if app.system_stats.is_warmed_up() {
        let used = app.system_stats.memory_used_gb();
        let total = app.system_stats.memory_total_gb();
        let percent = if total > 0.0 {
            (used / total * 100.0) as f32
        } else {
            0.0
        };
        color_for_value(percent, 60.0, 80.0, theme)
    } else {
        theme.muted
    };

    let cpu = Paragraph::new(Line::from(vec![
        Span::styled("CPU: ", theme.muted_style()),
        Span::styled(
            cpu_text,
            Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let load = Paragraph::new(Line::from(vec![
        Span::styled("Load: ", theme.muted_style()),
        Span::styled(
            load_text,
            Style::default().fg(load_color).add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let memory = Paragraph::new(Line::from(vec![
        Span::styled("Mem: ", theme.muted_style()),
        Span::styled(
            memory_text,
            Style::default()
                .fg(memory_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let uptime = Paragraph::new(Line::from(vec![
        Span::styled("Uptime: ", theme.muted_style()),
        Span::styled(uptime_text, theme.fg_style().add_modifier(Modifier::BOLD)),
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
    frame.render_widget(load, v_center(chunks[1]));
    frame.render_widget(memory, v_center(chunks[2]));
    frame.render_widget(uptime, v_center(chunks[3]));
}

fn format_memory(used_bytes: u64, total_bytes: u64, unit: DataSizeUnit) -> String {
    let used = format_data_size(used_bytes, unit);
    let total = format_data_size(total_bytes, unit);
    format!("{}/{}", used, total)
}
