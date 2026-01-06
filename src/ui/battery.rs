use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::app::App;
use crate::data::battery::ChargeState;
use crate::theme::ThemeColors;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Battery ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let info_card_height = if inner.height >= 4 { 3 } else { 0 };
    let gauge_height = inner.height.saturating_sub(info_card_height);

    if gauge_height > 0 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(gauge_height),
                Constraint::Length(info_card_height),
            ])
            .split(inner);

        render_battery_gauge(frame, chunks[0], app, theme);

        if info_card_height > 0 {
            render_battery_info_card(frame, chunks[1], app, theme);
        }
    }
}

fn render_battery_gauge(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let percent = app.battery.charge_percent();
    let ratio = (percent / 100.0).clamp(0.0, 1.0);

    let gauge_color = if percent > 50.0 {
        theme.success
    } else if percent > 20.0 {
        theme.warning
    } else {
        theme.danger
    };

    let label = format!("{:.0}%", percent);

    let gauge = ThickGauge {
        ratio,
        label,
        filled_color: gauge_color,
        border_color: theme.border,
        label_color: theme.fg,
        bg_color: theme.bg,
    };

    frame.render_widget(gauge, area);
}

struct ThickGauge {
    ratio: f32,
    label: String,
    filled_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    label_color: ratatui::style::Color,
    bg_color: ratatui::style::Color,
}

impl Widget for ThickGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height == 0 {
            return;
        }

        let (inner, has_border) = if area.height >= 3 {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.border_color))
                .style(Style::default().bg(self.bg_color));
            let inner = block.inner(area);
            block.render(area, buf);
            (inner, true)
        } else {
            (area, false)
        };

        if inner.width < 8 || inner.height == 0 {
            return;
        }

        let show_labels = inner.width >= 15 && has_border;
        let bar_start = if show_labels { inner.x + 3 } else { inner.x };
        let bar_end = if show_labels {
            inner.x + inner.width - 5
        } else {
            inner.x + inner.width
        };
        let bar_width = bar_end.saturating_sub(bar_start);

        if bar_width < 5 {
            return;
        }

        let filled_width = (bar_width as f32 * self.ratio).round() as u16;

        for y in inner.y..inner.y + inner.height {
            for x in bar_start..bar_end {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    let rel_x = x - bar_start;
                    let is_last_filled =
                        rel_x == filled_width.saturating_sub(1) && filled_width > 0;

                    if rel_x < filled_width {
                        if is_last_filled && filled_width < bar_width {
                            cell.set_char('▌');
                        } else {
                            cell.set_char('█');
                        }
                        cell.set_fg(self.filled_color);
                    } else {
                        cell.set_char(' ');
                    }
                }
            }
        }

        let label_y = inner.y + inner.height / 2;

        if show_labels {
            for (i, ch) in "0%".chars().enumerate() {
                let x = inner.x + i as u16;
                if x < bar_start {
                    if let Some(cell) = buf.cell_mut((x, label_y)) {
                        cell.set_char(ch);
                        cell.set_fg(self.border_color);
                    }
                }
            }

            for (i, ch) in "100%".chars().enumerate() {
                let x = bar_end + i as u16;
                if x < inner.x + inner.width {
                    if let Some(cell) = buf.cell_mut((x, label_y)) {
                        cell.set_char(ch);
                        cell.set_fg(self.border_color);
                    }
                }
            }
        }

        let label_with_padding = format!(" {}", self.label);
        let label_x = bar_start + filled_width;

        for (i, ch) in label_with_padding.chars().enumerate() {
            let x = label_x + i as u16;
            if x >= bar_start && x < bar_end {
                if let Some(cell) = buf.cell_mut((x, label_y)) {
                    cell.set_char(ch);
                    cell.set_fg(self.label_color);
                    cell.set_bg(self.bg_color);
                    cell.set_style(Style::default().add_modifier(Modifier::BOLD));
                }
            }
        }
    }
}

fn render_battery_info_card(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    if area.height == 0 || area.width < 20 {
        return;
    }

    let inner = area;

    let state_icon = match app.battery.state() {
        ChargeState::Charging => "⚡",
        ChargeState::Discharging => "↓",
        ChargeState::Full => "✓",
        ChargeState::NotCharging => "⏸",
        ChargeState::Unknown => "?",
    };

    let (time_label, time_value) = match app.battery.state() {
        ChargeState::Charging => (
            "Time to full:",
            app.battery
                .time_remaining_formatted()
                .unwrap_or_else(|| "Calculating...".to_string()),
        ),
        ChargeState::Discharging => (
            "Runtime:",
            app.battery
                .time_remaining_formatted()
                .unwrap_or_else(|| "Calculating...".to_string()),
        ),
        ChargeState::Full => ("Status:", "Charged".to_string()),
        ChargeState::NotCharging => ("Status:", "Not charging".to_string()),
        ChargeState::Unknown => ("Status:", "—".to_string()),
    };

    let power_text = if app.battery.is_charging() {
        app.battery.charging_watts().map(|w| {
            app.battery
                .charger_watts()
                .map_or(format!("{:.1}W", w), |c| format!("{:.1}W/{}W", w, c))
        })
    } else {
        app.battery.discharge_watts().map(|w| format!("{:.1}W", w))
    };

    let health_color = if app.battery.health_percent() >= 80.0 {
        theme.success
    } else if app.battery.health_percent() >= 50.0 {
        theme.warning
    } else {
        theme.danger
    };

    let cycles_text = app
        .battery
        .cycle_count()
        .map_or("—".to_string(), |c| c.to_string());

    let single_line = build_single_line(
        state_icon,
        app.battery.state_label(),
        time_label,
        &time_value,
        power_text.as_deref(),
        app.battery.health_percent(),
        &cycles_text,
        app.battery.max_capacity_wh(),
        app.battery.design_capacity_wh(),
        theme,
        health_color,
    );

    let single_line_width: usize = single_line.spans.iter().map(|s| s.content.len()).sum();

    if inner.width as usize >= single_line_width || inner.height < 3 {
        let centered = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner)[1];
        frame.render_widget(Paragraph::new(single_line).centered(), centered);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let row1 = Line::from(vec![
            Span::styled(
                format!("{} ", state_icon),
                Style::default().fg(theme.accent),
            ),
            Span::styled(
                app.battery.state_label(),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  │  ", Style::default().fg(theme.border)),
            Span::styled(format!("{} ", time_label), Style::default().fg(theme.muted)),
            Span::styled(&time_value, Style::default().fg(theme.fg)),
            Span::styled(
                power_text.map_or(String::new(), |p| format!("  │  {}", p)),
                Style::default().fg(theme.accent),
            ),
        ]);

        let row2 = Line::from(vec![
            Span::styled("Health: ", Style::default().fg(theme.muted)),
            Span::styled(
                format!("{:.0}%", app.battery.health_percent()),
                Style::default().fg(health_color),
            ),
            Span::styled(
                format!(
                    " ({:.1}/{:.1}Wh)",
                    app.battery.max_capacity_wh(),
                    app.battery.design_capacity_wh()
                ),
                Style::default().fg(theme.muted),
            ),
            Span::styled("  │  ", Style::default().fg(theme.border)),
            Span::styled("Cycles: ", Style::default().fg(theme.muted)),
            Span::styled(&cycles_text, Style::default().fg(theme.fg)),
        ]);

        frame.render_widget(Paragraph::new(row1).centered(), rows[0]);
        frame.render_widget(Paragraph::new(row2).centered(), rows[2]);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_single_line<'a>(
    icon: &'a str,
    state: &'a str,
    time_label: &'a str,
    time_value: &'a str,
    power: Option<&'a str>,
    health: f32,
    cycles: &'a str,
    capacity: f32,
    design_capacity: f32,
    theme: &ThemeColors,
    health_color: ratatui::style::Color,
) -> Line<'a> {
    let mut spans = vec![
        Span::styled(format!("{} ", icon), Style::default().fg(theme.accent)),
        Span::styled(
            state,
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(format!("{} ", time_label), Style::default().fg(theme.muted)),
        Span::styled(time_value, Style::default().fg(theme.fg)),
    ];

    if let Some(p) = power {
        spans.push(Span::styled(
            format!("  │  {}", p),
            Style::default().fg(theme.accent),
        ));
    }

    spans.extend([
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(
            format!("health {:.0}%", health),
            Style::default().fg(health_color),
        ),
        Span::styled(
            format!(" ({:.0}/{:.0}Wh)", capacity, design_capacity),
            Style::default().fg(theme.muted),
        ),
        Span::styled("  │  ", Style::default().fg(theme.border)),
        Span::styled(cycles, Style::default().fg(theme.fg)),
        Span::styled(" cycles", Style::default().fg(theme.muted)),
    ]);

    Line::from(spans)
}
