use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeColors;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Power ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
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
            format!("CPU: {:.1}W", app.power.cpu_power_watts()),
            format!("GPU: {:.1}W", app.power.gpu_power_watts()),
        )
    } else {
        ("—".to_string(), "CPU: —".to_string(), "GPU: —".to_string())
    };

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
