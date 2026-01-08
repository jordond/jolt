use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::power::PowerMode;
use crate::theme::ThemeColors;

const GAUGE_WIDTH: usize = 8;
const GAUGE_FILLED: char = '█';
const GAUGE_EMPTY: char = '░';

fn render_mini_gauge(percent: f32, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "{}{}",
        GAUGE_FILLED.to_string().repeat(filled),
        GAUGE_EMPTY.to_string().repeat(empty)
    )
}

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Power ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let show_display = app.config.user_config.show_display_power;

    let chunks = if show_display {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner)
    };

    let (total_power, cpu_power, gpu_power) = if app.power.is_warmed_up() {
        (
            format!("{:.1}W", app.power.total_power_watts()),
            format!("CPU: {:.1}W", app.power.cpu_power_watts()),
            format!("GPU: {:.1}W", app.power.gpu_power_watts()),
        )
    } else {
        ("—".to_string(), "CPU: —".to_string(), "GPU: —".to_string())
    };

    let mode_icon = match app.power.power_mode() {
        PowerMode::LowPower => "🐢",
        PowerMode::HighPerformance => "🚀",
        PowerMode::Automatic => "⚙️",
        PowerMode::Unknown => "",
    };
    let mode_text = format!("{} {}", mode_icon, app.power.power_mode_label());

    let power_color = if !app.power.is_warmed_up() {
        theme.muted
    } else if app.power.total_power_watts() > 15.0 {
        theme.danger
    } else if app.power.total_power_watts() > 8.0 {
        theme.warning
    } else {
        theme.success
    };

    let total = Paragraph::new(Line::from(vec![
        Span::styled("Total: ", Style::default().fg(theme.muted)),
        Span::styled(
            total_power,
            Style::default()
                .fg(power_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let cpu = Paragraph::new(Line::from(vec![Span::styled(
        cpu_power,
        Style::default().fg(theme.accent),
    )]))
    .centered();

    let gpu = Paragraph::new(Line::from(vec![Span::styled(
        gpu_power,
        Style::default().fg(theme.accent_secondary),
    )]))
    .centered();

    let mode = Paragraph::new(Line::from(vec![Span::styled(
        mode_text,
        Style::default().fg(theme.fg),
    )]))
    .centered();

    let v_center = |chunk: Rect| {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(chunk)[1]
    };

    frame.render_widget(total, v_center(chunks[0]));
    frame.render_widget(cpu, v_center(chunks[1]));
    frame.render_widget(gpu, v_center(chunks[2]));

    if show_display {
        let brightness = app.display.brightness_percent();
        let gauge = render_mini_gauge(brightness, GAUGE_WIDTH);

        let brightness_color = if brightness > 80.0 {
            theme.warning
        } else if brightness > 50.0 {
            theme.fg
        } else {
            theme.success
        };

        let display = Paragraph::new(Line::from(vec![
            Span::styled("☀ ", Style::default().fg(theme.muted)),
            Span::styled(gauge, Style::default().fg(brightness_color)),
            Span::styled(
                format!(" {:.0}%", brightness),
                Style::default().fg(brightness_color),
            ),
        ]))
        .centered();

        frame.render_widget(display, v_center(chunks[3]));
        frame.render_widget(mode, v_center(chunks[4]));
    } else {
        frame.render_widget(mode, v_center(chunks[3]));
    }
}
