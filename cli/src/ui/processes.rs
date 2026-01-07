use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::app::{App, SortColumn};
use crate::theme::ThemeColors;

const COL_EXPAND: u16 = 6;
const COL_PID: u16 = 8;
const COL_CPU: u16 = 8;
const COL_MEMORY: u16 = 10;
const COL_IMPACT: u16 = 8;
const COL_KILL: u16 = 4;
const COL_SPACING: u16 = 6;
const COL_NAME_MIN: u16 = 20;

fn energy_gradient_color(energy: f32, theme: &ThemeColors) -> Color {
    let (low_r, low_g, low_b) = extract_rgb(theme.success);
    let (mid_r, mid_g, mid_b) = extract_rgb(theme.warning);
    let (high_r, high_g, high_b) = extract_rgb(theme.danger);

    let t = (energy / 50.0).clamp(0.0, 1.0);

    let (r, g, b) = if t < 0.3 {
        let t2 = t / 0.3;
        (
            lerp(low_r, mid_r, t2),
            lerp(low_g, mid_g, t2),
            lerp(low_b, mid_b, t2),
        )
    } else {
        let t2 = (t - 0.3) / 0.7;
        (
            lerp(mid_r, high_r, t2),
            lerp(mid_g, high_g, t2),
            lerp(mid_b, high_b, t2),
        )
    };

    Color::Rgb(r, g, b)
}

fn extract_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

pub fn render(frame: &mut Frame, area: Rect, app: &mut App, theme: &ThemeColors) {
    let title = if app.selection_mode {
        " Processes [SELECTION MODE - Esc to exit] "
    } else if app.merge_mode {
        " Processes [MERGED] "
    } else {
        " Processes "
    };

    let border_color = if app.selection_mode {
        theme.accent
    } else {
        theme.border
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let fixed_width =
        COL_EXPAND + COL_PID + COL_CPU + COL_MEMORY + COL_IMPACT + COL_KILL + COL_SPACING;
    let name_width = inner.width.saturating_sub(fixed_width).max(COL_NAME_MIN) as usize;

    let sort_indicator = if app.sort_ascending { "▲" } else { "▼" };
    let header_cells: [String; 7] = [
        "".to_string(),
        format_header("PID", SortColumn::Pid, app.sort_column, sort_indicator),
        format_header("Name", SortColumn::Name, app.sort_column, sort_indicator),
        format_header("CPU %", SortColumn::Cpu, app.sort_column, sort_indicator),
        format_header(
            "Memory",
            SortColumn::Memory,
            app.sort_column,
            sort_indicator,
        ),
        format_header(
            "Impact",
            SortColumn::Energy,
            app.sort_column,
            sort_indicator,
        ),
        "Kill".to_string(),
    ];
    let header = Row::new(header_cells.iter().map(|h| {
        Span::styled(
            h.as_str(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
    }))
    .height(1);

    let all_processes = app.get_visible_processes();
    let max_visible = (inner.height.saturating_sub(1)) as usize;
    let visible_processes: Vec<_> = all_processes
        .iter()
        .skip(app.process_scroll_offset)
        .take(max_visible)
        .collect();

    let rows: Vec<Row> = visible_processes
        .iter()
        .enumerate()
        .map(|(idx, (process, depth))| {
            let actual_idx = idx + app.process_scroll_offset;
            let is_selected = actual_idx == app.selected_process_index;
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

            let energy_color = energy_gradient_color(process.energy_impact, theme);

            let style = if is_selected {
                Style::default()
                    .bg(theme.selection_bg)
                    .fg(theme.selection_fg)
            } else {
                Style::default().fg(energy_color)
            };

            let killable_indicator = if process.is_killable { "✓" } else { "✗" };
            let killable_style = if is_selected {
                style
            } else if process.is_killable {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.muted)
            };

            let display_name = if *depth > 0 {
                &process.command
            } else {
                &process.name
            };

            let cells = vec![
                Span::styled(format!("{}{}", indent, expand_icon), style),
                Span::styled(process.pid.to_string(), style),
                Span::styled(truncate_name(display_name, name_width), style),
                Span::styled(format!("{:.1}", process.cpu_usage), style),
                Span::styled(format!("{:.1}MB", process.memory_mb), style),
                Span::styled(format!("{:.1}", process.energy_impact), style),
                Span::styled(killable_indicator, killable_style),
            ];

            Row::new(cells).height(1).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(COL_EXPAND),
        Constraint::Length(COL_PID),
        Constraint::Min(COL_NAME_MIN),
        Constraint::Length(COL_CPU),
        Constraint::Length(COL_MEMORY),
        Constraint::Length(COL_IMPACT),
        Constraint::Length(COL_KILL),
    ];

    let table = Table::new(rows, widths).header(header);

    frame.render_widget(table, inner);
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}

fn format_header(name: &str, col: SortColumn, current: SortColumn, indicator: &str) -> String {
    if col == current {
        format!("{} {}", name, indicator)
    } else {
        name.to_string()
    }
}
