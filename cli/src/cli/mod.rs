mod daemon;
mod history;
mod theme;

pub use daemon::DaemonCommands;
pub use history::HistoryCommands;
pub use theme::ThemeCommands;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(alias = "tui", about = "Launch the terminal UI (default)")]
    Ui,

    #[command(alias = "raw", about = "Output metrics as JSON for scripting")]
    Pipe {
        #[arg(
            short,
            long,
            default_value_t = 0,
            help = "Number of samples (0 = infinite)"
        )]
        samples: u32,

        #[arg(
            short,
            long,
            default_value_t = 1000,
            help = "Interval between samples in ms"
        )]
        interval: u64,

        #[arg(short, long, help = "Compact single-line JSON output")]
        compact: bool,
    },

    #[command(about = "Print system and battery debug info")]
    Debug,

    #[command(about = "Manage configuration")]
    Config {
        #[arg(long, help = "Print config file path")]
        path: bool,

        #[arg(long, help = "Reset config to defaults")]
        reset: bool,

        #[arg(short, long, help = "Open config in editor")]
        edit: bool,
    },

    #[command(alias = "themes", about = "Manage themes")]
    Theme {
        #[command(subcommand)]
        command: Option<ThemeCommands>,
    },

    #[command(about = "Control the background daemon")]
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

    #[command(about = "View and export historical data")]
    History {
        #[command(subcommand)]
        command: Option<HistoryCommands>,
    },

    #[command(about = "View daemon logs")]
    Logs {
        #[arg(short, long, default_value_t = 50, help = "Number of lines to show")]
        lines: usize,

        #[arg(short, long, help = "Follow log output")]
        follow: bool,
    },
}

#[derive(Debug, Parser)]
#[command(
    name = "jolt",
    version,
    about = "A beautiful battery and energy monitor for your terminal"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        long,
        global = true,
        help = "Set log level (error, warn, info, debug, trace)"
    )]
    pub log_level: Option<String>,
}
