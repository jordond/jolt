use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let themes = &app.theme_picker_themes;
    let selected = app.theme_picker_index;

    let content_height = (themes.len() as u16 + 6).min(20);
    let content_width = 45;
    let area = centered_rect(frame.area(), content_width, content_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Theme ")
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
        "Use ↑↓ to navigate, Enter to select, Esc to cancel",
        Style::default().fg(theme.muted),
    )])])
    .centered();
    frame.render_widget(header, chunks[0]);

    let visible_height = chunks[1].height as usize;
    let scroll_offset = if selected >= visible_height {
        selected - visible_height + 1
    } else {
        0
    };

    let current_theme_id = app.config.theme_id();

    let items: Vec<Line> = themes
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, theme_item)| {
            let is_selected = i == selected;
            let is_current = theme_item.id == current_theme_id;

            let prefix = if is_current { "● " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let name_style = if is_selected {
                style
            } else if !theme_item.is_builtin {
                Style::default().fg(theme.accent_secondary)
            } else {
                style
            };

            Line::from(vec![
                Span::styled(prefix, if is_selected { style } else { Style::default().fg(theme.accent) }),
                Span::styled(&theme_item.name, name_style),
            ])
        })
        .collect();

    let list = Paragraph::new(items);
    frame.render_widget(list, chunks[1]);

    let current_theme = themes.get(selected);
    let variants_info = current_theme
        .map(|t| t.variants_label())
        .unwrap_or("unknown");

    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("Variants: ", Style::default().fg(theme.muted)),
        Span::styled(variants_info, Style::default().fg(theme.accent)),
    ])])
    .centered();
    frame.render_widget(footer, chunks[2]);
}
