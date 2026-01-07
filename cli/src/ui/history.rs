use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Clear, Dataset, GraphType, Paragraph, Row, Sparkline, Table,
    },
    Frame,
};

use crate::app::{App, HistoryPeriod};
use crate::input::keys;
use crate::theme::ThemeColors;

fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let popup_width = area.width * width_percent / 100;
    let popup_height = area.height * height_percent / 100;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    Rect::new(x, y, popup_width, popup_height)
}

pub fn render(frame: &mut Frame, app: &App, theme: &ThemeColors) {
    let area = centered_rect(frame.area(), 90, 85);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" History ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.dialog_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if !app.daemon_connected {
        render_no_daemon(frame, inner, theme);
        return;
    }

    if app.history_loading {
        render_loading(frame, inner, theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .margin(1)
        .split(inner);

    render_period_tabs(frame, chunks[0], app, theme);
    render_power_chart(frame, chunks[1], app, theme);
    render_stats_and_processes(frame, chunks[2], app, theme);
    render_footer(frame, chunks[3], theme);
}

fn render_no_daemon(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let message = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Daemon not running",
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Start the daemon to collect history data:",
            Style::default().fg(theme.muted),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  jolt daemon start",
            Style::default().fg(theme.accent),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Or install as a service:",
            Style::default().fg(theme.muted),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  jolt daemon install",
            Style::default().fg(theme.accent),
        )]),
    ])
    .centered();
    frame.render_widget(message, area);
}

fn render_loading(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let message = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Loading history data...",
            Style::default().fg(theme.muted),
        )]),
    ])
    .centered();
    frame.render_widget(message, area);
}

fn render_period_tabs(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let periods = [
        HistoryPeriod::Today,
        HistoryPeriod::Week,
        HistoryPeriod::Month,
        HistoryPeriod::All,
    ];

    let tabs: Vec<Span> = periods
        .iter()
        .flat_map(|&p| {
            let is_selected = p == app.history_period;
            let style = if is_selected {
                Style::default()
                    .fg(theme.selection_fg)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };
            vec![
                Span::styled(format!(" {} ", p.label()), style),
                Span::raw("  "),
            ]
        })
        .collect();

    let tab_line = Line::from(tabs);
    let tabs_para = Paragraph::new(vec![Line::from(""), tab_line]).centered();
    frame.render_widget(tabs_para, area);
}

fn render_power_chart(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Power Usage ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.history_daily_stats.is_empty() && app.history_hourly_stats.is_empty() {
        let no_data = Paragraph::new(vec![Line::from(vec![Span::styled(
            "No data for this period",
            Style::default().fg(theme.muted),
        )])])
        .centered();
        frame.render_widget(no_data, inner);
        return;
    }

    let data_points: Vec<(f64, f64)> = if app.history_period == HistoryPeriod::Today {
        app.history_hourly_stats
            .iter()
            .enumerate()
            .map(|(i, h)| (i as f64, h.avg_power as f64))
            .collect()
    } else {
        app.history_daily_stats
            .iter()
            .enumerate()
            .map(|(i, d)| (i as f64, d.avg_power as f64))
            .collect()
    };

    if data_points.is_empty() {
        let no_data = Paragraph::new(vec![Line::from(vec![Span::styled(
            "No data for this period",
            Style::default().fg(theme.muted),
        )])])
        .centered();
        frame.render_widget(no_data, inner);
        return;
    }

    let max_power = data_points
        .iter()
        .map(|(_, p)| *p)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.accent))
        .data(&data_points);

    let x_label = if app.history_period == HistoryPeriod::Today {
        "Hour"
    } else {
        "Day"
    };

    let chart = Chart::new(vec![dataset])
        .x_axis(
            Axis::default()
                .title(Span::styled(x_label, Style::default().fg(theme.muted)))
                .style(Style::default().fg(theme.border))
                .bounds([0.0, data_points.len() as f64]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Watts", Style::default().fg(theme.muted)))
                .style(Style::default().fg(theme.border))
                .bounds([0.0, max_power * 1.1])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.0}", max_power / 2.0)),
                    Span::raw(format!("{:.0}", max_power)),
                ]),
        );

    frame.render_widget(chart, inner);
}

fn render_stats_and_processes(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(area);

    render_sparklines(frame, vertical[0], app, theme);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(vertical[1]);

    render_summary_stats(frame, horizontal[0], app, theme);
    render_top_processes(frame, horizontal[1], app, theme);
}

fn render_sparklines(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let power_data: Vec<u64> = if app.history_period == HistoryPeriod::Today {
        app.history_hourly_stats
            .iter()
            .map(|h| (h.avg_power * 10.0) as u64)
            .collect()
    } else {
        app.history_daily_stats
            .iter()
            .map(|d| (d.avg_power * 10.0) as u64)
            .collect()
    };

    let battery_data: Vec<u64> = if app.history_period == HistoryPeriod::Today {
        app.history_hourly_stats
            .iter()
            .map(|h| h.avg_battery as u64)
            .collect()
    } else {
        app.history_daily_stats
            .iter()
            .map(|d| (d.total_energy_wh * 10.0) as u64)
            .collect()
    };

    let power_block = Block::default()
        .title(" Avg Power (W) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let power_sparkline = Sparkline::default()
        .block(power_block)
        .data(&power_data)
        .max(power_data.iter().copied().max().unwrap_or(100).max(100))
        .style(Style::default().fg(theme.accent));

    frame.render_widget(power_sparkline, chunks[0]);

    let energy_label = if app.history_period == HistoryPeriod::Today {
        " Battery % "
    } else {
        " Energy (Wh) "
    };

    let energy_block = Block::default()
        .title(energy_label)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let energy_sparkline = Sparkline::default()
        .block(energy_block)
        .data(&battery_data)
        .max(battery_data.iter().copied().max().unwrap_or(100).max(100))
        .style(Style::default().fg(theme.success));

    frame.render_widget(energy_sparkline, chunks[1]);
}

fn render_summary_stats(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Summary ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.history_daily_stats.is_empty() {
        let no_data = Paragraph::new(vec![Line::from(vec![Span::styled(
            "No data",
            Style::default().fg(theme.muted),
        )])])
        .centered();
        frame.render_widget(no_data, inner);
        return;
    }

    let total_energy: f32 = app
        .history_daily_stats
        .iter()
        .map(|d| d.total_energy_wh)
        .sum();
    let avg_power: f32 = app
        .history_daily_stats
        .iter()
        .map(|d| d.avg_power)
        .sum::<f32>()
        / app.history_daily_stats.len() as f32;
    let max_power = app
        .history_daily_stats
        .iter()
        .map(|d| d.max_power)
        .fold(0.0_f32, f32::max);
    let total_screen_hours: f32 = app
        .history_daily_stats
        .iter()
        .map(|d| d.screen_on_hours)
        .sum();
    let total_charging: f32 = app
        .history_daily_stats
        .iter()
        .map(|d| d.charging_hours)
        .sum();

    let stats = vec![
        Line::from(vec![
            Span::styled("Total Energy:  ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.1} Wh", total_energy),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("Avg Power:     ", Style::default().fg(theme.muted)),
            Span::styled(format!("{:.1} W", avg_power), Style::default().fg(theme.fg)),
        ]),
        Line::from(vec![
            Span::styled("Max Power:     ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.1} W", max_power),
                Style::default().fg(theme.warning),
            ),
        ]),
        Line::from(vec![
            Span::styled("Screen On:     ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.1} hrs", total_screen_hours),
                Style::default().fg(theme.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("Charging:      ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.1} hrs", total_charging),
                Style::default().fg(theme.success),
            ),
        ]),
        Line::from(vec![
            Span::styled("Days:          ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{}", app.history_daily_stats.len()),
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    let para = Paragraph::new(stats)
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(1)));
    frame.render_widget(para, inner);
}

const HISTORY_COL_AVG_W: u16 = 7;
const HISTORY_COL_TOTAL_WH: u16 = 9;
const HISTORY_COL_CPU: u16 = 5;
const HISTORY_COL_SPACING: u16 = 3;
const HISTORY_COL_NAME_MIN: u16 = 18;

fn render_top_processes(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Top Power Consumers ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.history_top_processes.is_empty() {
        let no_data = Paragraph::new(vec![Line::from(vec![Span::styled(
            "No process data",
            Style::default().fg(theme.muted),
        )])])
        .centered();
        frame.render_widget(no_data, inner);
        return;
    }

    let fixed_width =
        HISTORY_COL_AVG_W + HISTORY_COL_TOTAL_WH + HISTORY_COL_CPU + HISTORY_COL_SPACING;
    let name_width = inner
        .width
        .saturating_sub(fixed_width)
        .max(HISTORY_COL_NAME_MIN) as usize;

    let header = Row::new(vec!["Process", "Avg W", "Total Wh", "CPU%"])
        .style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let max_power = app
        .history_top_processes
        .iter()
        .map(|p| p.avg_power)
        .fold(0.0_f32, f32::max);

    let rows: Vec<Row> = app
        .history_top_processes
        .iter()
        .take(8)
        .map(|p| {
            let power_color = power_level_color(p.avg_power, max_power, theme);

            Row::new(vec![
                truncate_name(&p.process_name, name_width),
                format!("{:.1}", p.avg_power),
                format!("{:.1}", p.total_energy_wh),
                format!("{:.0}", p.avg_cpu),
            ])
            .style(Style::default().fg(power_color))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(HISTORY_COL_NAME_MIN),
            Constraint::Length(HISTORY_COL_AVG_W),
            Constraint::Length(HISTORY_COL_TOTAL_WH),
            Constraint::Length(HISTORY_COL_CPU),
        ],
    )
    .header(header)
    .column_spacing(1);

    frame.render_widget(table, inner);
}

fn power_level_color(power: f32, max_power: f32, theme: &ThemeColors) -> ratatui::style::Color {
    const EPS: f32 = 1e-6;
    let effective_max = if max_power.abs() < EPS {
        EPS
    } else {
        max_power
    };
    let ratio = power / effective_max;
    if ratio >= 0.7 {
        theme.danger
    } else if ratio >= 0.4 {
        theme.warning
    } else {
        theme.success
    }
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &ThemeColors) {
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            format!("[{}/{}]", keys::PERIOD_PREV, keys::PERIOD_NEXT),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" Period  ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("[{}]", keys::SETTINGS),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" Settings  ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("[{}]", keys::ESC),
            Style::default().fg(theme.accent),
        ),
        Span::styled(" Close", Style::default().fg(theme.muted)),
    ])])
    .centered();
    frame.render_widget(footer, area);
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}
