use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::input::keys;
use crate::theme::ThemeColors;

fn centered_fixed_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let content_height = App::HISTORY_CONFIG_ITEMS.len() as u16 + 8;
    let content_width = 50;
    let area = centered_fixed_rect(frame.area(), content_width, content_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" History Config ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);

    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        "Use \u{2191}\u{2193} to select, \u{2190}\u{2192} or Enter to change",
        Style::default().fg(theme.muted),
    )])])
    .centered();
    frame.render_widget(header, chunks[0]);

    let items: Vec<Line> = App::HISTORY_CONFIG_ITEMS
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let value = app.history_config_item_value(i);
            let is_selected = i == app.history_config_selected_item;

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
                Span::styled(format!("  {:<24}", name), style),
                Span::styled(format!("{:>10}", value), value_style),
                Span::styled("  ", style),
            ])
        })
        .collect();

    let list = Paragraph::new(items);
    frame.render_widget(list, chunks[1]);

    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            format!("[{}]", keys::ESC),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" Close", Style::default().fg(theme.muted)),
    ])])
    .centered();
    frame.render_widget(footer, chunks[2]);
}
