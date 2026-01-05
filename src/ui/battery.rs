use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::app::App;
use crate::config::Theme;
use crate::data::battery::ChargeState;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let block = Block::default()
        .title(" Battery ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(inner);

    render_battery_gauge(frame, chunks[0], app, theme);
    render_battery_info(frame, chunks[1], app, theme);
}

fn render_battery_gauge(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
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
        if area.height < 3 || area.width < 10 {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.border_color))
            .style(Style::default().bg(self.bg_color));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 8 || inner.height == 0 {
            return;
        }

        let bar_start = inner.x + 3;
        let bar_end = inner.x + inner.width - 5;
        let bar_width = bar_end.saturating_sub(bar_start);

        if bar_width < 5 {
            return;
        }

        let filled_width = (bar_width as f32 * self.ratio).round() as u16;

        for y in inner.y..inner.y + inner.height {
            for x in bar_start..bar_end {
                let cell = buf.cell_mut((x, y)).unwrap();
                let rel_x = x - bar_start;
                let is_last_filled = rel_x == filled_width.saturating_sub(1) && filled_width > 0;

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

        let label_y = inner.y + inner.height / 2;

        for (i, ch) in "0%".chars().enumerate() {
            let x = inner.x + i as u16;
            if x < bar_start {
                let cell = buf.cell_mut((x, label_y)).unwrap();
                cell.set_char(ch);
                cell.set_fg(self.border_color);
            }
        }

        let end_label = "100%";
        for (i, ch) in end_label.chars().enumerate() {
            let x = bar_end + i as u16;
            if x < inner.x + inner.width {
                let cell = buf.cell_mut((x, label_y)).unwrap();
                cell.set_char(ch);
                cell.set_fg(self.border_color);
            }
        }

        let label_with_padding = format!(" {}", self.label);
        let label_x = bar_start + filled_width;

        for (i, ch) in label_with_padding.chars().enumerate() {
            let x = label_x + i as u16;
            if x >= bar_start && x < bar_end {
                let cell = buf.cell_mut((x, label_y)).unwrap();
                cell.set_char(ch);
                cell.set_fg(self.label_color);
                cell.set_bg(self.bg_color);
                cell.set_style(Style::default().add_modifier(Modifier::BOLD));
            }
        }
    }
}

fn render_battery_info(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let state_icon = match app.battery.state() {
        ChargeState::Charging => "⚡",
        ChargeState::Discharging => "🔋",
        ChargeState::Full => "✓",
        ChargeState::NotCharging => "⏸",
        ChargeState::Unknown => "?",
    };

    let power_info = if app.battery.is_charging() {
        if let Some(watts) = app.battery.charging_watts() {
            let charger = app
                .battery
                .charger_watts()
                .map_or(String::new(), |w| format!("/{}W", w));
            format!(" ({:.1}W{})", watts, charger)
        } else {
            String::new()
        }
    } else if let Some(watts) = app.battery.discharge_watts() {
        format!(" ({:.1}W)", watts)
    } else {
        String::new()
    };

    let state_text = format!("{} {}{}", state_icon, app.battery.state_label(), power_info);
    let time_text = app
        .battery
        .time_remaining_formatted()
        .map(|t| {
            if app.battery.is_charging() {
                format!("Full in {}", t)
            } else {
                format!("{} remaining", t)
            }
        })
        .unwrap_or_else(|| {
            if app.battery.is_charging() {
                "Calculating...".to_string()
            } else {
                "".to_string()
            }
        });

    let health_text = format!(
        "{:.1}Wh  Health: {:.0}%  Cycles: {}",
        app.battery.max_capacity_wh(),
        app.battery.health_percent(),
        app.battery
            .cycle_count()
            .map_or("N/A".to_string(), |c| c.to_string())
    );

    let left = Paragraph::new(Line::from(vec![Span::styled(
        state_text,
        Style::default().fg(theme.accent),
    )]));

    let center = Paragraph::new(Line::from(vec![Span::styled(
        time_text,
        Style::default().fg(theme.fg),
    )]))
    .centered();

    let right = Paragraph::new(Line::from(vec![Span::styled(
        health_text,
        Style::default().fg(theme.muted),
    )]))
    .right_aligned();

    frame.render_widget(left, chunks[0]);
    frame.render_widget(center, chunks[1]);
    frame.render_widget(right, chunks[2]);
}
