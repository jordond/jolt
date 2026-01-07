mod battery;
mod graphs;
mod help;
mod history;
mod power;
mod processes;
mod settings;
mod status_bar;
mod theme_importer;
mod theme_picker;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, AppView};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

struct LayoutSizes {
    battery: u16,
    power: u16,
    graph: u16,
    processes_min: u16,
}

impl LayoutSizes {
    fn calculate(content_height: u16, show_graph: bool) -> Self {
        // Height requirements: battery=10 (borders+gauge+info card), power=3, graph=8, processes=6
        const BATTERY_MIN: u16 = 10;
        const BATTERY_PREFERRED: u16 = 12;
        const POWER_MIN: u16 = 3;
        const GRAPH_MIN: u16 = 8;
        const GRAPH_PREFERRED: u16 = 10;
        const PROCESSES_MIN: u16 = 6;

        let graph_size = if show_graph { GRAPH_MIN } else { 0 };
        let graph_preferred = if show_graph { GRAPH_PREFERRED } else { 0 };
        let min_total = BATTERY_MIN + POWER_MIN + PROCESSES_MIN + graph_size;

        if content_height < min_total {
            // Compressed: prioritize processes > battery > power > graph
            let available = content_height;
            let power = POWER_MIN.min(available);
            let remaining = available.saturating_sub(power);

            let battery = BATTERY_MIN.min(remaining).max(5);
            let remaining = remaining.saturating_sub(battery);

            let graph = if show_graph && remaining > PROCESSES_MIN + 4 {
                (remaining.saturating_sub(PROCESSES_MIN)).min(GRAPH_MIN)
            } else {
                0
            };
            let remaining = remaining.saturating_sub(graph);

            Self {
                battery,
                power,
                graph,
                processes_min: remaining.max(3),
            }
        } else {
            // Normal: give extra to battery first, then graph
            let extra = content_height.saturating_sub(min_total);
            let battery_extra = (BATTERY_PREFERRED - BATTERY_MIN).min(extra);
            let battery = BATTERY_MIN + battery_extra;
            let remaining_extra = extra.saturating_sub(battery_extra);

            let graph = if show_graph {
                graph_size + (graph_preferred - graph_size).min(remaining_extra)
            } else {
                0
            };

            Self {
                battery,
                power: POWER_MIN,
                graph,
                processes_min: PROCESSES_MIN,
            }
        }
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = app.current_theme();
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

    let sizes = LayoutSizes::calculate(content_area.height, show_graph);

    let constraints = if sizes.graph > 0 {
        vec![
            Constraint::Length(sizes.battery),
            Constraint::Length(sizes.power),
            Constraint::Min(sizes.processes_min),
            Constraint::Length(sizes.graph),
        ]
    } else {
        vec![
            Constraint::Length(sizes.battery),
            Constraint::Length(sizes.power),
            Constraint::Min(sizes.processes_min),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(content_area);

    battery::render(frame, chunks[0], app, &theme);
    power::render(frame, chunks[1], app, &theme);
    processes::render(frame, chunks[2], app, &theme);

    if sizes.graph > 0 && chunks.len() > 3 {
        graphs::render(frame, chunks[3], app, &theme);
    }

    match app.view {
        AppView::Help => help::render_help(frame, app, &theme),
        AppView::About => help::render_about(frame, app, &theme),
        AppView::KillConfirm => help::render_kill_confirm(frame, app, &theme),
        AppView::ThemePicker => theme_picker::render(frame, app, &theme),
        AppView::ThemeImporter => theme_importer::render(frame, app, &theme),
        AppView::History => history::render(frame, app, &theme),
        AppView::Settings => settings::render(frame, app, &theme),
        AppView::Main => {}
    }
}
