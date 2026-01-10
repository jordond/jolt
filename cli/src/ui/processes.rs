use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::app::{App, SortColumn};
use crate::data::ProcessState;
use crate::theme::ThemeColors;

const COL_EXPAND: u16 = 6;
const COL_PID: u16 = 7;
const COL_STATUS: u16 = 1;
const COL_IMPACT: u16 = 8;
const COL_CPU: u16 = 6;
const COL_MEMORY: u16 = 8;
const COL_DISK: u16 = 9;
const COL_RUNTIME: u16 = 7;
const COL_CPUTIME: u16 = 7;
const COL_KILL: u16 = 4;
const COL_SPACING: u16 = 12;
const COL_NAME_MIN: u16 = 12;
const COL_COMMAND_MIN: u16 = 15;

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

    let fixed_width = COL_EXPAND
        + COL_PID
        + COL_STATUS
        + COL_CPU
        + COL_MEMORY
        + COL_DISK
        + COL_RUNTIME
        + COL_CPUTIME
        + COL_IMPACT
        + COL_KILL
        + COL_SPACING;
    let flex_width = inner.width.saturating_sub(fixed_width);
    let name_base = (flex_width / 6).max(COL_NAME_MIN);
    let command_base = flex_width
        .saturating_sub(flex_width / 6)
        .max(COL_COMMAND_MIN);

    // Ensure the total flexible column width does not exceed the available flex_width.
    // When space is tight, scale the Name and Command columns proportionally so that
    // name_width + command_width <= flex_width.
    let (name_width, command_width) = {
        let total_flex = name_base.saturating_add(command_base);
        if flex_width == 0 || total_flex <= flex_width {
            (name_base as usize, command_base as usize)
        } else {
            // Scale proportionally using u32 to avoid overflow during multiplication.
            let fw = flex_width as u32;
            let total = total_flex as u32;
            let name_scaled = ((u32::from(name_base) * fw) / total).max(1) as u16;
            let mut command_scaled = flex_width.saturating_sub(name_scaled);
            if command_scaled == 0 {
                // Ensure Command also gets at least 1 cell when there is space.
                command_scaled = 1;
            }
            (name_scaled as usize, command_scaled as usize)
        }
    };
    let sort_indicator = if app.sort_ascending { "▲" } else { "▼" };
    let header_cells: [String; 12] = [
        "".to_string(),
        format_header("PID", SortColumn::Pid, app.sort_column, sort_indicator),
        "S".to_string(),
        format_header(
            "Impact",
            SortColumn::Energy,
            app.sort_column,
            sort_indicator,
        ),
        format_header("Name", SortColumn::Name, app.sort_column, sort_indicator),
        "Command".to_string(),
        format_header("CPU%", SortColumn::Cpu, app.sort_column, sort_indicator),
        format_header(
            "Memory",
            SortColumn::Memory,
            app.sort_column,
            sort_indicator,
        ),
        "Disk".to_string(),
        "Runtime".to_string(),
        "CPU".to_string(),
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
            let is_selected = app.selection_mode && actual_idx == app.selected_process_index;
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

            let status_char = process.status.as_char().to_string();
            let status_style = if is_selected {
                style
            } else {
                match process.status {
                    ProcessState::Running => Style::default().fg(theme.success),
                    ProcessState::Sleeping => Style::default().fg(theme.muted),
                    ProcessState::Idle => Style::default().fg(theme.muted),
                    _ => Style::default().fg(theme.warning),
                }
            };

            let disk_io = format_disk_io(process.disk_read_bytes, process.disk_write_bytes);
            let runtime = format_duration(process.run_time_secs);
            let cpu_time = format_duration(process.total_cpu_time_secs);

            let cells = vec![
                Span::styled(format!("{}{}", indent, expand_icon), style),
                Span::styled(process.pid.to_string(), style),
                Span::styled(status_char, status_style),
                Span::styled(format!("{:.1}", process.energy_impact), style),
                Span::styled(truncate_name(display_name, name_width), style),
                Span::styled(truncate_name(&process.command_args, command_width), style),
                Span::styled(format!("{:.1}", process.cpu_usage), style),
                Span::styled(format_memory(process.memory_mb), style),
                Span::styled(disk_io, style),
                Span::styled(runtime, style),
                Span::styled(cpu_time, style),
                Span::styled(killable_indicator, killable_style),
            ];

            Row::new(cells).height(1).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(COL_EXPAND),
        Constraint::Length(COL_PID),
        Constraint::Length(COL_STATUS),
        Constraint::Length(COL_IMPACT),
        Constraint::Length(name_width as u16),
        Constraint::Length(command_width as u16),
        Constraint::Length(COL_CPU),
        Constraint::Length(COL_MEMORY),
        Constraint::Length(COL_DISK),
        Constraint::Length(COL_RUNTIME),
        Constraint::Length(COL_CPUTIME),
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

fn format_disk_io(read_bytes: u64, write_bytes: u64) -> String {
    let total = read_bytes + write_bytes;
    if total == 0 {
        "-".to_string()
    } else if total < 1024 {
        format!("{}B", total)
    } else if total < 1024 * 1024 {
        format!("{:.0}K", total as f64 / 1024.0)
    } else {
        format!("{:.1}M", total as f64 / (1024.0 * 1024.0))
    }
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if mins > 0 {
            format!("{}h{}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        if hours > 0 {
            format!("{}d{}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}

fn format_memory(mb: f64) -> String {
    if mb < 1000.0 {
        format!("{:.0}M", mb)
    } else {
        format!("{:.1}G", mb / 1024.0)
    }
}
