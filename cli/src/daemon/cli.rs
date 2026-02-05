use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "jolt-daemon",
    version,
    about = "Jolt battery monitor daemon"
)]
pub struct DaemonCli {
    #[command(subcommand)]
    pub command: Option<DaemonCommands>,

    #[arg(
        long,
        global = true,
        help = "Set log level (error, warn, info, debug, trace)"
    )]
    pub log_level: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum DaemonCommands {
    Start {
        #[arg(short, long)]
        foreground: bool,
    },

    Stop,

    Status,

    #[cfg(target_os = "macos")]
    Install {
        #[arg(short, long)]
        force: bool,
    },

    #[cfg(target_os = "macos")]
    Uninstall,
}
