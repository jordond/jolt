use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum DaemonCommands {
    Start {
        #[arg(short, long)]
        foreground: bool,
    },

    Stop,

    Status,

    Install {
        #[arg(short, long)]
        force: bool,
    },

    Uninstall,
}
