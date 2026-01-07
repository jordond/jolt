use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::input::KEY_BINDINGS;
use crate::theme::ThemeColors;
use crate::ui::VERSION;

fn centered_fixed_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub fn render_help(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let legend_lines = 8;
    let content_height = KEY_BINDINGS.len() as u16 + legend_lines + 10;
    let content_width = 55;
    let area = centered_fixed_rect(frame.area(), content_width, content_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(KEY_BINDINGS.len() as u16 + 1),
            Constraint::Min(1),
        ])
        .margin(2)
        .split(inner);

    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "jolt - Battery & Energy Monitor",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "Theme: {} ({})",
                app.config.theme_name(),
                app.config.appearance_label()
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
                    format!("{:15}", binding.key),
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(binding.description, Style::default().fg(theme.fg)),
            ])
        })
        .collect();

    let help_text = Paragraph::new(lines);
    frame.render_widget(help_text, chunks[1]);

    let legend = vec![
        Line::from(vec![Span::styled(
            "Column Legend",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("S      ", Style::default().fg(theme.highlight)),
            Span::styled(
                "Status: R=Run S=Sleep I=Idle T=Stop Z=Zombie",
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Impact ", Style::default().fg(theme.highlight)),
            Span::styled(
                "Energy impact score (higher = more drain)",
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Disk   ", Style::default().fg(theme.highlight)),
            Span::styled(
                "Disk I/O since last refresh (read+write)",
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Run    ", Style::default().fg(theme.highlight)),
            Span::styled(
                "Process runtime (how long it's been running)",
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPU    ", Style::default().fg(theme.highlight)),
            Span::styled(
                "Accumulated CPU time consumed by process",
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    let legend_text = Paragraph::new(legend);
    frame.render_widget(legend_text, chunks[2]);
}

pub fn render_kill_confirm(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let area = centered_fixed_rect(frame.area(), 50, 14);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Kill Process? ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.danger))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let padded = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(inner)[0];

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

    frame.render_widget(paragraph, padded);
}

pub fn render_about(frame: &mut Frame, _app: &App, theme: &ThemeColors) {
    let area = centered_fixed_rect(frame.area(), 60, 16);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" About ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let padded = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(inner)[0];

    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("jolt v{}", VERSION),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "A terminal-based battery and energy monitor",
            Style::default().fg(theme.fg),
        )]),
        Line::from(vec![Span::styled(
            "for macOS Apple Silicon Macs.",
            Style::default().fg(theme.fg),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Track power consumption, battery health,",
            Style::default().fg(theme.muted),
        )]),
        Line::from(vec![Span::styled(
            "and identify energy-hungry processes.",
            Style::default().fg(theme.muted),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("GitHub: ", Style::default().fg(theme.muted)),
            Span::styled(
                "https://github.com/jordond/jolt",
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press 'A' or Esc to close",
            Style::default().fg(theme.muted),
        )]),
    ];

    let paragraph = Paragraph::new(content).centered();

    frame.render_widget(paragraph, padded);
}
