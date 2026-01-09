//! TUI (Terminal User Interface) runtime loop.
//!
//! This module contains the terminal setup, event loop, and teardown
//! logic for running the TUI application.

use std::io;
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tracing::{debug, trace};

use crate::config::UserConfig;
use crate::input;
use crate::ui;

use super::App;

/// Sets up the terminal for TUI mode.
///
/// Enables raw mode and switches to the alternate screen buffer.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its original state.
///
/// Disables raw mode and returns to the main screen buffer.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Entry point for running the TUI application.
///
/// Sets up the terminal, runs the main event loop, and restores the terminal
/// when finished. This is the main entry point called from the CLI handler.
pub fn run_tui(user_config: UserConfig) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_tui_loop(&mut terminal, user_config);
    restore_terminal(&mut terminal)?;
    result
}

/// The main TUI event loop.
///
/// This function:
/// - Creates the App instance
/// - Renders frames at the configured refresh rate
/// - Handles keyboard and resize events
/// - Processes user actions
/// - Runs until the user quits
fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    user_config: UserConfig,
) -> Result<()> {
    let mut app = App::new(user_config)?;
    let mut needs_redraw = true;
    let mut last_tick = std::time::Instant::now();
    let mut tick_count: u64 = 0;
    let mut wait_log_counter: u64 = 0;

    loop {
        let tick_rate = Duration::from_millis(app.refresh_ms);
        let elapsed = last_tick.elapsed();

        let data_changed = if elapsed >= tick_rate {
            tick_count += 1;
            let tick_start = std::time::Instant::now();
            last_tick = std::time::Instant::now();
            let result = app.tick()?;
            let tick_duration = tick_start.elapsed();
            debug!(
                tick_count,
                elapsed_ms = elapsed.as_millis() as u64,
                tick_rate_ms = app.refresh_ms,
                data_changed = result,
                tick_duration_ms = tick_duration.as_millis() as u64,
                using_daemon = app.using_daemon_data,
                "TUI tick completed"
            );
            wait_log_counter = 0;
            result
        } else {
            // Only log occasionally to avoid spam (~every 500ms at 10ms poll rate)
            wait_log_counter += 1;
            if wait_log_counter.is_multiple_of(50) {
                trace!(
                    elapsed_ms = elapsed.as_millis() as u64,
                    tick_rate_ms = app.refresh_ms,
                    wait_iterations = wait_log_counter,
                    "TUI waiting for next tick"
                );
            }
            false
        };
        needs_redraw = needs_redraw || data_changed;

        if needs_redraw {
            terminal.draw(|frame| ui::render(frame, &mut app))?;
            needs_redraw = false;
        }

        let poll_timeout = if app.using_daemon_data {
            Duration::from_millis(10)
        } else {
            tick_rate.saturating_sub(elapsed)
        };

        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let action = input::handle_key(&app, key);
                    if !app.handle_action(action) {
                        break;
                    }
                    needs_redraw = true;
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }
    }

    app.cleanup();
    Ok(())
}
