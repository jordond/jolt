use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use crate::app::App;
use crate::config::Theme;
use crate::data::processes::EnergyLevel;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    let block = Block::default()
        .title(" Processes (↑↓ navigate, Enter expand, K kill) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header_cells = ["", "PID", "Name", "CPU %", "Memory", "Energy"];
    let header = Row::new(header_cells.iter().map(|h| {
        Span::styled(
            *h,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1);

    let visible_processes = app.get_visible_processes();

    let rows: Vec<Row> = visible_processes
        .iter()
        .enumerate()
        .map(|(idx, (process, depth))| {
            let is_selected = idx == app.selected_process_index;
            let has_children = process.children.is_some();
            let is_expanded = app.expanded_groups.contains(&process.pid);

            let indent = "  ".repeat(*depth as usize);
            let expand_icon = if has_children {
                if is_expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else if *depth > 0 {
                "  └ "
            } else {
                "  "
            };

            let energy_color = match process.energy_level() {
                EnergyLevel::High => theme.danger,
                EnergyLevel::Medium => theme.warning,
                EnergyLevel::Low => theme.success,
            };

            let style = if is_selected {
                Style::default()
                    .bg(theme.selection_bg)
                    .fg(theme.selection_fg)
            } else {
                Style::default().fg(theme.fg)
            };

            let cells = vec![
                Span::styled(format!("{}{}", indent, expand_icon), style),
                Span::styled(process.pid.to_string(), style),
                Span::styled(truncate_name(&process.name, 30), style),
                Span::styled(format!("{:.1}", process.cpu_usage), style),
                Span::styled(format!("{:.1}MB", process.memory_mb), style),
                Span::styled(
                    format!("{:.1}", process.energy_impact),
                    style.fg(energy_color),
                ),
            ];

            Row::new(cells).height(1).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Min(20),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .highlight_style(
            Style::default()
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    state.select(Some(app.selected_process_index.saturating_sub(app.process_scroll_offset)));

    frame.render_stateful_widget(table, inner, &mut state);
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}
