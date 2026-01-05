use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::config::Theme;
use crate::input::KEY_BINDINGS;

pub fn render_help(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = super::centered_rect(frame.size(), 60, 70);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .margin(1)
        .split(inner);

    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "⚡ jolt - Battery & Energy Monitor",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "Theme: {} (press 't' to change)",
                app.config.theme_mode_label()
            ),
            Style::default().fg(theme.muted),
        )]),
    ])
    .centered();

    frame.render_widget(title, chunks[0]);

    let lines: Vec<Line> = KEY_BINDINGS
        .iter()
        .map(|binding| {
            Line::from(vec![
                Span::styled(
                    format!("{:14}", binding.key),
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(binding.description, Style::default().fg(theme.fg)),
            ])
        })
        .collect();

    let help_text = Paragraph::new(lines).wrap(Wrap { trim: true });

    frame.render_widget(help_text, chunks[1]);
}

pub fn render_kill_confirm(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = super::centered_rect(frame.size(), 50, 40);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Kill Process? ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.danger))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = if let Some(process) = app.process_to_kill() {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Process: ", Style::default().fg(theme.muted)),
                Span::styled(
                    &process.name,
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("PID: ", Style::default().fg(theme.muted)),
                Span::styled(process.pid.to_string(), Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("CPU: ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{:.1}%", process.cpu_usage),
                    Style::default().fg(theme.fg),
                ),
            ]),
            Line::from(vec![
                Span::styled("Memory: ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{:.1}MB", process.memory_mb),
                    Style::default().fg(theme.fg),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "This will forcefully terminate the process.",
                Style::default().fg(theme.warning),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "[Y]",
                    Style::default()
                        .fg(theme.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Confirm  ", Style::default().fg(theme.fg)),
                Span::styled(
                    "[N]",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Cancel", Style::default().fg(theme.fg)),
            ]),
        ]
    } else {
        vec![Line::from("No process selected")]
    };

    let paragraph = Paragraph::new(content).centered();

    frame.render_widget(paragraph, inner);
}
