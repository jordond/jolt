use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::input::keys;
use crate::settings::{self, SettingsRow, SETTINGS_LAYOUT};
use crate::theme::ThemeColors;

const CONTENT_WIDTH: u16 = 44;

fn centered_fixed_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let status_height: u16 = 4;
    let items_height = settings::row_count() as u16;
    let footer_height: u16 = 1;
    let content_height = status_height + items_height + footer_height + 6;
    let dialog_width = CONTENT_WIDTH + 8;
    let area = centered_fixed_rect(frame.area(), dialog_width, content_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let h_padding = (inner.width.saturating_sub(CONTENT_WIDTH)) / 2;
    let centered_area = Rect {
        x: inner.x + h_padding,
        y: inner.y,
        width: CONTENT_WIDTH,
        height: inner.height,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(status_height),
            Constraint::Length(1),
            Constraint::Min(items_height),
            Constraint::Length(footer_height),
        ])
        .margin(1)
        .split(centered_area);

    render_status_section(frame, chunks[0], app, theme);
    render_divider(frame, chunks[1], theme);
    render_items_section(frame, chunks[2], app, theme);
    render_footer(frame, chunks[3], theme);
}

fn render_status_section(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let history_cfg = &app.config.user_config.history;
    let background_enabled = history_cfg.background_recording;
    let (bg_status, bg_color) = if background_enabled {
        ("On", theme.success)
    } else {
        ("Off", theme.warning)
    };

    let lines = if let Some(ref status) = app.daemon_status {
        let uptime_str = format_uptime(status.uptime_secs);
        let size_str = format_bytes(status.database_size_bytes);

        const LEFT_COL_WIDTH: usize = 24;

        let bg_text = format!("Background: {}", bg_status);
        let bg_spacer = " ".repeat(LEFT_COL_WIDTH.saturating_sub(bg_text.len()));

        let samples_text = format!("Samples: {:<8}", status.sample_count);
        let samples_spacer = " ".repeat(LEFT_COL_WIDTH.saturating_sub(samples_text.len()));

        vec![
            Line::from(vec![
                Span::styled("Background: ", Style::default().fg(theme.muted)),
                Span::styled(
                    bg_status,
                    Style::default().fg(bg_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(bg_spacer),
                Span::styled("Uptime: ", Style::default().fg(theme.muted)),
                Span::styled(uptime_str, Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("Samples: ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{:<8}", status.sample_count),
                    Style::default().fg(theme.accent),
                ),
                Span::raw(samples_spacer),
                Span::styled("DB Size: ", Style::default().fg(theme.muted)),
                Span::styled(size_str, Style::default().fg(theme.fg)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Background: ", Style::default().fg(theme.muted)),
                Span::styled(
                    bg_status,
                    Style::default().fg(bg_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                "No daemon connection",
                Style::default().fg(theme.muted),
            )]),
        ]
    };

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn render_divider(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let divider = Paragraph::new(Line::from(vec![Span::styled(
        "\u{2500}".repeat(area.width as usize),
        Style::default().fg(theme.border),
    )]));
    frame.render_widget(divider, area);
}

fn render_items_section(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let items: Vec<Line> = SETTINGS_LAYOUT
        .iter()
        .enumerate()
        .map(|(i, row)| match row {
            SettingsRow::Section(label) => Line::from(vec![Span::styled(
                format!("\u{25b8} {}", label),
                Style::default()
                    .fg(theme.muted)
                    .add_modifier(Modifier::BOLD),
            )]),
            SettingsRow::Item { id, label } => {
                let value = settings::setting_value(app, *id);
                let is_selected = i == app.settings_selected_item;

                let style = if is_selected {
                    Style::default()
                        .fg(theme.selection_fg)
                        .bg(theme.selection_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.fg)
                };

                let value_style = if is_selected {
                    style.fg(theme.accent)
                } else {
                    Style::default().fg(theme.accent)
                };

                Line::from(vec![
                    Span::styled(format!("  {:<24}", label), style),
                    Span::styled(format!("{:>16}", value), value_style),
                    Span::styled("  ", style),
                ])
            }
        })
        .collect();

    let list = Paragraph::new(items);
    frame.render_widget(list, area);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("[{}]", keys::ESC),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" Close", Style::default().fg(theme.muted)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
