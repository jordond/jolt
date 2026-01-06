use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let width = (area.width * percent_x / 100).min(area.width.saturating_sub(4));
    let height = (area.height * percent_y / 100).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let area = centered_rect(frame.area(), 85, 85);
    frame.render_widget(Clear, area);

    let status = if app.importer_loading {
        " Loading... "
    } else {
        " Import Themes "
    };

    let block = Block::default()
        .title(status)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(inner);

    render_search_bar(frame, chunks[0], app, theme);
    render_theme_list(frame, chunks[1], app, theme);
    render_footer(frame, chunks[2], app, theme);
}

fn render_search_bar(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let filter_display = if app.importer_filter.is_empty() {
        "Type to filter...".to_string()
    } else {
        app.importer_filter.clone()
    };

    let style = if app.importer_filter.is_empty() {
        Style::default().fg(theme.muted)
    } else {
        Style::default().fg(theme.fg)
    };

    let search_block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let search_inner = search_block.inner(area);
    frame.render_widget(search_block, area);

    let cursor = if app.importer_filter.is_empty() {
        ""
    } else {
        "█"
    };
    let search_text = Paragraph::new(format!("{}{}", filter_display, cursor)).style(style);
    frame.render_widget(search_text, search_inner);
}

fn render_theme_list(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let groups = app.get_filtered_importer_groups();
    let selected_idx = app.importer_index;

    let visible_height = area.height.saturating_sub(2) as usize;
    let scroll_offset = if selected_idx >= visible_height {
        selected_idx - visible_height + 1
    } else {
        0
    };

    let items: Vec<Line> = groups
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, group)| {
            let is_selected = i == selected_idx;
            let is_checked = app.importer_selected.contains(&group.name);

            let checkbox = if is_checked { "[✓] " } else { "[ ] " };

            let variant_indicator = match (&group.dark, &group.light) {
                (Some(_), Some(_)) => "◐",
                (Some(_), None) => "●",
                (None, Some(_)) => "○",
                (None, None) => "?",
            };

            let style = if is_selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else if is_checked {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.fg)
            };

            let checkbox_style = if is_selected {
                style
            } else if is_checked {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.muted)
            };

            let variant_style = if is_selected {
                style
            } else {
                match (&group.dark, &group.light) {
                    (Some(_), Some(_)) => Style::default().fg(theme.accent),
                    (Some(_), None) => Style::default().fg(theme.muted),
                    (None, Some(_)) => Style::default().fg(theme.highlight),
                    _ => Style::default().fg(theme.muted),
                }
            };

            Line::from(vec![
                Span::styled(checkbox, checkbox_style),
                Span::styled(&group.name, style),
                Span::styled(" ", style),
                Span::styled(variant_indicator, variant_style),
            ])
        })
        .collect();

    let list_block = Block::default()
        .title(format!(" {} themes ", groups.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let list_inner = list_block.inner(area);
    frame.render_widget(list_block, area);

    let list = Paragraph::new(items);
    frame.render_widget(list, list_inner);

    if groups.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(groups.len()).position(scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let selected_count = app.importer_selected.len();

    let cache_info = app
        .importer_cache_age
        .as_ref()
        .map(|s| format!("Updated {}", s))
        .unwrap_or_else(|| "Not cached".to_string());

    let left_text = if selected_count > 0 {
        format!("{} selected", selected_count)
    } else {
        cache_info
    };

    let footer = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            &left_text,
            Style::default().fg(theme.muted),
        )]),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(theme.accent)),
            Span::styled(" select  ", Style::default().fg(theme.muted)),
            Span::styled("Enter", Style::default().fg(theme.accent)),
            Span::styled(" preview  ", Style::default().fg(theme.muted)),
            Span::styled("i", Style::default().fg(theme.accent)),
            Span::styled(" import  ", Style::default().fg(theme.muted)),
            Span::styled("r", Style::default().fg(theme.accent)),
            Span::styled(" refresh  ", Style::default().fg(theme.muted)),
            Span::styled("Esc", Style::default().fg(theme.accent)),
            Span::styled(" close", Style::default().fg(theme.muted)),
        ]),
    ]);

    frame.render_widget(footer, area);
}
