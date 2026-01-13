use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

use super::utils::color_for_value;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let power_color = if app.power.is_warmed_up() {
        color_for_value(app.power.total_power_watts(), 8.0, 15.0, theme)
    } else {
        theme.muted
    };

    let block = Block::default()
        .title(Span::styled(" Power ", Style::default().fg(power_color)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(power_color))
        .style(Style::default().bg(theme.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(inner);

    let (total_power, cpu_power, gpu_power) = if app.power.is_warmed_up() {
        (
            format!("{:.1}W", app.power.total_power_watts()),
            format!("{:.1}W", app.power.cpu_power_watts()),
            format!("{:.1}W", app.power.gpu_power_watts()),
        )
    } else {
        ("—".to_string(), "—".to_string(), "—".to_string())
    };

    let total = Paragraph::new(Line::from(vec![
        Span::styled("Total: ", theme.muted_style()),
        Span::styled(
            total_power,
            Style::default()
                .fg(power_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .centered();

    let cpu = Paragraph::new(Line::from(vec![
        Span::styled("CPU: ", theme.muted_style()),
        Span::styled(cpu_power, theme.accent_style()),
    ]))
    .centered();

    let gpu = Paragraph::new(Line::from(vec![
        Span::styled("GPU: ", theme.muted_style()),
        Span::styled(gpu_power, theme.accent_secondary_style()),
    ]))
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
}
