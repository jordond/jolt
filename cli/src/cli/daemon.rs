use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum DaemonCommands {
    Start {
        #[arg(short, long)]
        foreground: bool,
    },

    Stop,

    #[cfg(target_os = "macos")]
    Status,

    #[cfg(target_os = "macos")]
    Install {
        #[arg(short, long)]
        force: bool,
    },

    #[cfg(target_os = "macos")]
    Uninstall,
}
