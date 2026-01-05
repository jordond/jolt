mod app;
mod config;
mod data;
mod input;
mod ui;

use std::io;
use std::time::Duration;

use app::App;
use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "jolt")]
#[command(
    author,
    version,
    about = "Beautiful battery & energy monitor for macOS"
)]
struct Args {
    #[arg(short, long, default_value = "1000")]
    refresh_ms: u64,

    #[arg(short, long, default_value = "auto")]
    theme: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, args);
    restore_terminal(&mut terminal)?;

    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, args: Args) -> Result<()> {
    let theme_mode = match args.theme.as_str() {
        "dark" => config::ThemeMode::Dark,
        "light" => config::ThemeMode::Light,
        _ => config::ThemeMode::Auto,
    };

    let mut app = App::new(theme_mode)?;
    let tick_rate = Duration::from_millis(args.refresh_ms);

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = input::handle_key(&app, key);
                    if !app.handle_action(action) {
                        break;
                    }
                }
            }
        }

        app.tick()?;
    }

    Ok(())
}
