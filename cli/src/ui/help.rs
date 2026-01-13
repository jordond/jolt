use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::daemon::KillSignal;
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
    let legend_lines = 6;
    let content_height = KEY_BINDINGS.len() as u16 + legend_lines + 10;
    let content_width = 55;
    let area = centered_fixed_rect(frame.area(), content_width, content_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(theme.accent_style())
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
            theme.muted_style(),
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
                Span::styled(binding.description, theme.fg_style()),
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
            Span::styled("S      ", theme.highlight_style()),
            Span::styled(
                "Status: R=Run S=Sleep I=Idle T=Stop Z=Zombie",
                theme.fg_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Impact ", theme.highlight_style()),
            Span::styled(
                "Energy impact score (higher = more drain)",
                theme.fg_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Disk   ", theme.highlight_style()),
            Span::styled("Disk I/O since last refresh (read+write)", theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled("Run    ", theme.highlight_style()),
            Span::styled(
                "Process runtime (how long it's been running)",
                theme.fg_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPU    ", theme.highlight_style()),
            Span::styled(
                "CPU Time: accumulated CPU time consumed by process",
                theme.fg_style(),
            ),
        ]),
    ];

    let legend_text = Paragraph::new(legend);
    frame.render_widget(legend_text, chunks[2]);
}

pub fn render_kill_confirm(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let area = centered_fixed_rect(frame.area(), 54, 16);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Kill Process? ")
        .borders(Borders::ALL)
        .border_style(theme.danger_style())
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let padded = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .margin(1)
        .split(inner)[0];

    let content = if let Some(process) = app.process_to_kill() {
        let (graceful_style, force_style, warning_text) = match app.kill_signal {
            KillSignal::Graceful => (
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
                theme.muted_style(),
                "Process will be asked to terminate gracefully.",
            ),
            KillSignal::Force => (
                theme.muted_style(),
                Style::default()
                    .fg(theme.danger)
                    .add_modifier(Modifier::BOLD),
                "Process will be forcefully terminated immediately.",
            ),
        };

        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Process: ", theme.muted_style()),
                Span::styled(&process.name, theme.fg_style().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("PID: ", theme.muted_style()),
                Span::styled(process.pid.to_string(), theme.fg_style()),
            ]),
            Line::from(vec![
                Span::styled("CPU: ", theme.muted_style()),
                Span::styled(format!("{:.1}%", process.cpu_usage), theme.fg_style()),
            ]),
            Line::from(vec![
                Span::styled("Memory: ", theme.muted_style()),
                Span::styled(format!("{:.1}MB", process.memory_mb), theme.fg_style()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Signal: ", theme.muted_style()),
                Span::styled(" Graceful ", graceful_style),
                Span::styled(" | ", theme.muted_style()),
                Span::styled(" Force ", force_style),
                Span::styled("  [Tab]", theme.muted_style()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(warning_text, theme.warning_style())]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "[Y]",
                    Style::default()
                        .fg(theme.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Confirm  ", theme.fg_style()),
                Span::styled(
                    "[N]",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Cancel", theme.fg_style()),
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
        .border_style(theme.accent_style())
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
            theme.fg_style(),
        )]),
        Line::from(vec![Span::styled(
            "for macOS Apple Silicon Macs.",
            theme.fg_style(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Track power consumption, battery health,",
            theme.muted_style(),
        )]),
        Line::from(vec![Span::styled(
            "and identify energy-hungry processes.",
            theme.muted_style(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("GitHub: ", theme.muted_style()),
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
            theme.muted_style(),
        )]),
    ];

    let paragraph = Paragraph::new(content).centered();

    frame.render_widget(paragraph, padded);
}
