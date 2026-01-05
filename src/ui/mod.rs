mod battery;
mod graphs;
mod help;
mod power;
mod processes;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::{App, AppView};

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = app.config.theme();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(10),
        ])
        .split(frame.size());

    battery::render(frame, chunks[0], app, &theme);
    power::render(frame, chunks[1], app, &theme);
    processes::render(frame, chunks[2], app, &theme);
    graphs::render(frame, chunks[3], app, &theme);

    match app.view {
        AppView::Help => help::render_help(frame, app, &theme),
        AppView::KillConfirm => help::render_kill_confirm(frame, app, &theme),
        AppView::Main => {}
    }
}

pub fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
