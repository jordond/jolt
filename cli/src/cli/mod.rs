mod daemon;
mod history;
mod theme;

pub use daemon::DaemonCommands;
pub use history::HistoryCommands;
pub use theme::ThemeCommands;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(alias = "tui")]
    Ui,

    #[command(alias = "raw")]
    Pipe {
        #[arg(short, long, default_value_t = 0)]
        samples: u32,

        #[arg(short, long, default_value_t = 1000)]
        interval: u64,

        #[arg(short, long)]
        compact: bool,
    },

    Debug,

    Config {
        #[arg(long)]
        path: bool,

        #[arg(long)]
        reset: bool,

        #[arg(short, long)]
        edit: bool,
    },

    #[command(alias = "themes")]
    Theme {
        #[command(subcommand)]
        command: Option<ThemeCommands>,
    },

    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

    History {
        #[command(subcommand)]
        command: Option<HistoryCommands>,
    },

    Logs {
        #[arg(short, long, default_value_t = 50)]
        lines: usize,

        #[arg(short, long)]
        follow: bool,
    },
}

#[derive(Debug, Parser)]
#[command(name = "jolt", version, verbatim_doc_comment)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, global = true)]
    pub log_level: Option<String>,
}
