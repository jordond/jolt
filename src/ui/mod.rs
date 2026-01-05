mod battery;
mod config_editor;
mod graphs;
mod help;
mod power;
mod processes;
mod status_bar;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, AppView};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = app.config.theme();
    let area = frame.area();

    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(area);

    status_bar::render_title_bar(frame, outer_chunks[0], &app.system_info, &theme);
    status_bar::render_status_bar(frame, outer_chunks[2], app, &theme);

    let content_area = outer_chunks[1];
    let show_graph = app.config.user_config.show_graph;

    let (battery_height, power_height, graph_height) = if content_area.height < 28 {
        (5, 3, if show_graph { 8 } else { 0 })
    } else if content_area.height < 38 {
        (6, 4, if show_graph { 12 } else { 0 })
    } else {
        (7, 5, if show_graph { 14 } else { 0 })
    };

    let constraints = if show_graph {
        vec![
            Constraint::Length(battery_height),
            Constraint::Length(power_height),
            Constraint::Min(8),
            Constraint::Length(graph_height),
        ]
    } else {
        vec![
            Constraint::Length(battery_height),
            Constraint::Length(power_height),
            Constraint::Min(8),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(content_area);

    battery::render(frame, chunks[0], app, &theme);
    power::render(frame, chunks[1], app, &theme);
    processes::render(frame, chunks[2], app, &theme);

    if show_graph && chunks.len() > 3 {
        graphs::render(frame, chunks[3], app, &theme);
    }

    match app.view {
        AppView::Help => help::render_help(frame, app, &theme),
        AppView::About => help::render_about(frame, app, &theme),
        AppView::KillConfirm => help::render_kill_confirm(frame, app, &theme),
        AppView::Config => config_editor::render(frame, app, &theme),
        AppView::Main => {}
    }
}
