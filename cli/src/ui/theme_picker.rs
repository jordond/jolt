use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

use super::utils::centered_rect;

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let themes = &app.theme_picker_themes;
    let selected = app.theme_picker_index;

    let content_height = (themes.len() as u16 + 8).min(24);
    let content_width = 55;
    let area = centered_rect(frame.area(), content_width, content_height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Theme ")
        .borders(Borders::ALL)
        .border_style(theme.accent_style())
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .margin(1)
        .split(inner);

    let header = Paragraph::new(Line::from(vec![Span::styled(
        "↑↓ navigate  Enter select  Esc cancel",
        theme.muted_style(),
    )]))
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
                theme.fg_style()
            };

            let name_style = if is_selected {
                style
            } else if !theme_item.is_builtin {
                theme.accent_secondary_style()
            } else {
                style
            };

            Line::from(vec![
                Span::styled(
                    prefix,
                    if is_selected {
                        style
                    } else {
                        theme.accent_style()
                    },
                ),
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

    let preview_mode = if app.preview_is_dark() {
        "dark"
    } else {
        "light"
    };

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Preview: ", theme.muted_style()),
            Span::styled(preview_mode, theme.accent_style()),
            Span::styled("  Variants: ", theme.muted_style()),
            Span::styled(variants_info, theme.accent_style()),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("a/←→", theme.accent_style()),
            Span::styled(" toggle preview  ", theme.muted_style()),
            Span::styled("i", theme.accent_style()),
            Span::styled(" import themes", theme.muted_style()),
        ]),
    ])
    .centered();
    frame.render_widget(footer, chunks[2]);
}
